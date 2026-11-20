"""Bounded, resumable AlphaFold DB cohort crawler."""

from __future__ import annotations

import json
import uuid
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any
from urllib.parse import urlsplit

from sqlalchemy import func, select
from sqlalchemy.orm import Session

from protein3d.config import Settings
from protein3d.ingestion.alphafold import (
    AFDB_SOURCE,
    AlphaFoldClient,
    AlphaFoldError,
    AlphaFoldNotFound,
    AlphaFoldPermanentError,
    AlphaFoldRetryableError,
    manifest_record,
)
from protein3d.ingestion.alphafold_repository import persist_alpha_fold_artifacts
from protein3d.ingestion.minio_storage import MinioObjectStore
from protein3d.metadata.models import (
    DatasetRun,
    DatasetStatus,
    Job,
    JobStatus,
    Protein,
    ProteinIdMapping,
)
from protein3d.metadata.storage import S3ArtifactStorage


@dataclass(frozen=True, slots=True)
class ProteinCandidate:
    """A PP-12 protein UUID and its resolved UniProt accession."""

    protein_id: uuid.UUID
    accession: str
    sequence_length: int | None = None


@dataclass(frozen=True, slots=True)
class CrawlSummary:
    discovered: int
    downloaded: int
    no_model: int
    unsupported: int
    failed: int
    retryable_failed: int
    permanent_failed: int
    manifest_path: str
    dataset_run_id: str | None = None
    job_id: str | None = None


def select_candidates(
    session: Session, taxonomy_id: int, sample_limit: int
) -> list[ProteinCandidate]:
    """Select a deterministic, bounded cohort with resolved UniProt mappings."""
    if sample_limit < 1 or sample_limit > 10_000:
        raise ValueError("sample_limit must be between 1 and 10000")
    rows = session.execute(
        select(
            Protein.protein_id,
            func.min(ProteinIdMapping.source_id),
            func.length(Protein.sequence),
        )
        .join(ProteinIdMapping, ProteinIdMapping.protein_id == Protein.protein_id)
        .where(
            Protein.organism_id == str(taxonomy_id),
            ProteinIdMapping.source == "uniprot",
            ProteinIdMapping.mapping_status == "resolved",
        )
        .group_by(Protein.protein_id)
        .order_by(Protein.protein_id)
        .limit(sample_limit)
    ).all()
    return [
        ProteinCandidate(protein_id=row[0], accession=row[1], sequence_length=row[2])
        for row in rows
    ]


def crawl_candidates(
    candidates: list[ProteinCandidate],
    *,
    session: Session,
    client: AlphaFoldClient,
    object_store: MinioObjectStore,
    artifact_storage: S3ArtifactStorage,
    manifest_path: Path,
    max_coordinate_bytes: int = 100_000_000,
    dataset_run_id: uuid.UUID | None = None,
    job_id: uuid.UUID | None = None,
) -> CrawlSummary:
    """Stream each model to disk/MinIO and commit metadata per completed model.

    The deterministic object key and unique PP-12 structure/artifact keys make
    retries idempotent. Each successful object is HEAD-verified before metadata
    is committed. The manifest is append-only JSONL for crash recovery.
    """
    discovered = downloaded = no_model = unsupported = 0
    failed = retryable_failed = permanent_failed = 0
    manifest_path.parent.mkdir(parents=True, exist_ok=True)

    with TemporaryDirectory(prefix="protein3d-afdb-") as temp_dir:
        temp_root = Path(temp_dir)
        with manifest_path.open("a", encoding="utf-8") as manifest:
            for candidate in candidates:
                if candidate.sequence_length is not None and candidate.sequence_length > 2700:
                    unsupported += 1
                    _write_manifest(
                        manifest,
                        {
                            "protein_id": str(candidate.protein_id),
                            "source": AFDB_SOURCE,
                            "accession": candidate.accession,
                            "sequence_length": candidate.sequence_length,
                            "status": "unsupported",
                            "reason": (
                                "Protein exceeds the individual API length limit; "
                                "proteome archive fragments are outside PP-13 scope"
                            ),
                        },
                    )
                    continue
                try:
                    models = client.get_predictions(candidate.accession)
                except AlphaFoldNotFound:
                    no_model += 1
                    _write_manifest(
                        manifest,
                        {
                            "protein_id": str(candidate.protein_id),
                            "source": AFDB_SOURCE,
                            "accession": candidate.accession,
                            "status": "no_model",
                            "reason": "AFDB API returned 404",
                        },
                    )
                    continue
                except AlphaFoldRetryableError as exc:
                    failed += 1
                    retryable_failed += 1
                    _write_manifest(
                        manifest,
                        {
                            "protein_id": str(candidate.protein_id),
                            "source": AFDB_SOURCE,
                            "accession": candidate.accession,
                            "status": "retryable_failure",
                            "reason": str(exc),
                        },
                    )
                    continue
                except AlphaFoldError as exc:
                    failed += 1
                    permanent_failed += 1
                    _write_manifest(
                        manifest,
                        {
                            "protein_id": str(candidate.protein_id),
                            "source": AFDB_SOURCE,
                            "accession": candidate.accession,
                            "status": "permanent_failure",
                            "reason": str(exc),
                        },
                    )
                    continue

                if not models:
                    no_model += 1
                    _write_manifest(
                        manifest,
                        {
                            "protein_id": str(candidate.protein_id),
                            "source": AFDB_SOURCE,
                            "accession": candidate.accession,
                            "status": "no_model",
                            "reason": "AFDB returned an empty prediction list",
                        },
                    )
                    continue

                for model in models:
                    discovered += 1
                    local_path = temp_root / f"{uuid.uuid4().hex}.cif"
                    try:
                        coordinate = client.download_cif_to_path(
                            model,
                            str(candidate.protein_id),
                            local_path,
                            max_bytes=max_coordinate_bytes,
                        )
                        artifact_uri = artifact_storage.uri_for(coordinate.object_key)
                        parsed_uri = urlsplit(artifact_uri)
                        object_key = parsed_uri.path.lstrip("/")
                        if (
                            parsed_uri.scheme != "s3"
                            or parsed_uri.netloc != artifact_storage.bucket
                        ):
                            raise ValueError("Artifact storage returned an unexpected S3 URI")
                        readback_path = temp_root / f"{uuid.uuid4().hex}.readback.cif"
                        existing_size = object_store.head_object(object_key)
                        reused = False
                        if existing_size == coordinate.size_bytes:
                            readback_size, readback_sha256 = object_store.get_file(
                                object_key, readback_path
                            )
                            reused = (
                                readback_size == coordinate.size_bytes
                                and readback_sha256 == coordinate.sha256
                            )
                        if not reused:
                            object_store.put_file(
                                object_key,
                                local_path,
                                coordinate.content_type,
                                coordinate.sha256,
                            )
                            stored_size = object_store.head_object(object_key)
                            if stored_size != coordinate.size_bytes:
                                raise RuntimeError(
                                    f"MinIO HEAD size mismatch: expected "
                                    f"{coordinate.size_bytes}, got {stored_size}"
                                )
                            readback_size, readback_sha256 = object_store.get_file(
                                object_key, readback_path
                            )
                            if (
                                readback_size != coordinate.size_bytes
                                or readback_sha256 != coordinate.sha256
                            ):
                                raise RuntimeError(
                                    "MinIO readback checksum/size verification failed"
                                )
                        readback_path.unlink(missing_ok=True)
                        persist_alpha_fold_artifacts(
                            session,
                            candidate.protein_id,
                            model,
                            coordinate,
                            artifact_uri,
                            dataset_run_id=dataset_run_id,
                            job_id=job_id,
                        )
                        session.commit()
                        record: dict[str, Any] = manifest_record(
                            str(candidate.protein_id), model, coordinate, "downloaded"
                        )
                        record["storage_action"] = "reused" if reused else "uploaded"
                        downloaded += 1
                    except AlphaFoldRetryableError as exc:
                        session.rollback()
                        failed += 1
                        retryable_failed += 1
                        record = manifest_record(
                            str(candidate.protein_id),
                            model,
                            None,
                            "retryable_failure",
                            reason=f"{type(exc).__name__}: {exc}",
                        )
                    except (AlphaFoldPermanentError, ValueError) as exc:
                        session.rollback()
                        failed += 1
                        permanent_failed += 1
                        record = manifest_record(
                            str(candidate.protein_id),
                            model,
                            None,
                            "permanent_failure",
                            reason=f"{type(exc).__name__}: {exc}",
                        )
                    except Exception as exc:
                        session.rollback()
                        failed += 1
                        retryable_failed += 1
                        record = manifest_record(
                            str(candidate.protein_id),
                            model,
                            None,
                            "retryable_failure",
                            reason=f"{type(exc).__name__}: {exc}",
                        )
                    finally:
                        local_path.unlink(missing_ok=True)
                    _write_manifest(manifest, record)

    return CrawlSummary(
        discovered=discovered,
        downloaded=downloaded,
        no_model=no_model,
        unsupported=unsupported,
        failed=failed,
        retryable_failed=retryable_failed,
        permanent_failed=permanent_failed,
        manifest_path=str(manifest_path),
        dataset_run_id=str(dataset_run_id) if dataset_run_id else None,
        job_id=str(job_id) if job_id else None,
    )


def run_configured_crawl(
    session: Session,
    settings: Settings,
    manifest_path: Path,
    sample_limit: int | None = None,
) -> CrawlSummary:
    """Run one auditable dataset/job execution using PP-12 configuration."""
    limit = sample_limit if sample_limit is not None else settings.alphafold_sample_limit
    if limit < 1 or limit > 10_000:
        raise ValueError("sample_limit must be between 1 and 10000")
    candidates = select_candidates(session, settings.alphafold_taxonomy_id, limit)
    run = DatasetRun(
        dataset_name="alphafold_structures",
        dataset_type="structure_crawl",
        source=AFDB_SOURCE,
        source_version="api-current",
        status=DatasetStatus.RUNNING.value,
        started_at=datetime.now(UTC),
        provenance={
            "taxonomy_id": settings.alphafold_taxonomy_id,
            "sample_limit": limit,
            "candidate_count": len(candidates),
            "manifest_path": str(manifest_path),
        },
    )
    session.add(run)
    session.flush()
    job = Job(
        dataset_run_id=run.dataset_run_id,
        job_type="alphafold_structure_crawl",
        status=JobStatus.RUNNING.value,
        idempotency_key=f"alphafold-crawl:{run.dataset_run_id}",
        parameters={
            "taxonomy_id": settings.alphafold_taxonomy_id,
            "sample_limit": limit,
        },
        started_at=run.started_at,
    )
    session.add(job)
    session.commit()

    client = AlphaFoldClient(
        base_url=settings.alphafold_api_base_url,
        timeout_seconds=settings.external_request_timeout_seconds,
        max_retries=settings.external_max_retries,
    )
    object_store = MinioObjectStore(
        endpoint=settings.object_storage_endpoint,
        bucket=settings.object_storage_bucket,
        access_key=settings.object_storage_access_key,
        secret_key=settings.object_storage_secret_key.get_secret_value(),
        secure=settings.object_storage_secure,
        timeout_seconds=settings.external_request_timeout_seconds,
    )
    artifact_storage = S3ArtifactStorage(
        bucket=settings.object_storage_bucket,
        prefix=settings.object_storage_prefix,
    )

    try:
        object_store.ensure_bucket()
        summary = crawl_candidates(
            candidates,
            session=session,
            client=client,
            object_store=object_store,
            artifact_storage=artifact_storage,
            manifest_path=manifest_path,
            max_coordinate_bytes=settings.alphafold_max_coordinate_bytes,
            dataset_run_id=run.dataset_run_id,
            job_id=job.job_id,
        )
        finished = datetime.now(UTC)
        final_status = (
            DatasetStatus.FAILED.value if summary.failed else DatasetStatus.COMPLETED.value
        )
        run.status = final_status
        run.finished_at = finished
        run.provenance = {
            **run.provenance,
            "discovered": summary.discovered,
            "downloaded": summary.downloaded,
            "no_model": summary.no_model,
            "unsupported": summary.unsupported,
            "failed": summary.failed,
            "retryable_failed": summary.retryable_failed,
            "permanent_failed": summary.permanent_failed,
        }
        job.status = JobStatus.FAILED.value if summary.failed else JobStatus.SUCCEEDED.value
        job.error_summary = (
            f"{summary.retryable_failed} retryable and "
            f"{summary.permanent_failed} permanent model failure(s)"
            if summary.failed
            else None
        )
        job.finished_at = finished
        session.commit()
        return CrawlSummary(
            discovered=summary.discovered,
            downloaded=summary.downloaded,
            no_model=summary.no_model,
            unsupported=summary.unsupported,
            failed=summary.failed,
            retryable_failed=summary.retryable_failed,
            permanent_failed=summary.permanent_failed,
            manifest_path=summary.manifest_path,
            dataset_run_id=str(run.dataset_run_id),
            job_id=str(job.job_id),
        )
    except Exception as exc:
        session.rollback()
        failed_run = session.get(DatasetRun, run.dataset_run_id)
        failed_job = session.get(Job, job.job_id)
        if failed_run is not None:
            failed_run.status = DatasetStatus.FAILED.value
            failed_run.finished_at = datetime.now(UTC)
        if failed_job is not None:
            failed_job.status = JobStatus.FAILED.value
            failed_job.error_summary = f"{type(exc).__name__}: {exc}"
            failed_job.finished_at = datetime.now(UTC)
        session.commit()
        raise


def _write_manifest(manifest: Any, record: dict[str, Any]) -> None:
    manifest.write(json.dumps(record, sort_keys=True) + "\n")
    manifest.flush()
