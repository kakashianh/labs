"""Small-cohort AlphaFold crawler orchestration primitives."""

from __future__ import annotations

import json
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from sqlalchemy.orm import Session

from protein3d.ingestion.alphafold import (
    AFDB_SOURCE,
    AlphaFoldClient,
    AlphaFoldError,
    AlphaFoldNotFound,
    manifest_record,
)
from protein3d.ingestion.alphafold_repository import persist_alpha_fold_artifacts
from protein3d.ingestion.minio_storage import MinioObjectStore
from protein3d.metadata.storage import S3ArtifactStorage


@dataclass(frozen=True, slots=True)
class ProteinCandidate:
    """A PP-12 protein UUID and its resolved UniProt accession."""

    protein_id: uuid.UUID
    accession: str


@dataclass(frozen=True, slots=True)
class CrawlSummary:
    discovered: int
    downloaded: int
    no_model: int
    failed: int
    manifest_path: str


def crawl_candidates(
    candidates: list[ProteinCandidate],
    *,
    session: Session,
    client: AlphaFoldClient,
    object_store: MinioObjectStore,
    artifact_storage: S3ArtifactStorage,
    manifest_path: Path,
    max_coordinate_bytes: int = 100_000_000,
) -> CrawlSummary:
    """Download a bounded cohort and persist successful artifacts.

    The caller controls candidate selection from PP-12 and owns the database
    transaction. The manifest is written as JSONL so interrupted runs retain
    completed outcomes. Each successful artifact is persisted only after PUT.
    """
    discovered = downloaded = no_model = failed = 0
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    with manifest_path.open("a", encoding="utf-8") as manifest:
        for candidate in candidates:
            try:
                models = client.get_predictions(candidate.accession)
            except AlphaFoldNotFound:
                no_model += 1
                manifest.write(
                    json.dumps(
                        {
                            "protein_id": str(candidate.protein_id),
                            "source": AFDB_SOURCE,
                            "accession": candidate.accession,
                            "status": "no_model",
                            "reason": "AFDB API returned 404",
                        }
                    )
                    + "\n"
                )
                manifest.flush()
                continue
            except AlphaFoldError as exc:
                failed += 1
                manifest.write(
                    json.dumps(
                        {
                            "protein_id": str(candidate.protein_id),
                            "source": AFDB_SOURCE,
                            "accession": candidate.accession,
                            "status": "failed",
                            "reason": str(exc),
                        }
                    )
                    + "\n"
                )
                manifest.flush()
                continue

            if not models:
                no_model += 1
                manifest.write(
                    json.dumps(
                        {
                            "protein_id": str(candidate.protein_id),
                            "source": AFDB_SOURCE,
                            "accession": candidate.accession,
                            "status": "no_model",
                            "reason": "AFDB returned an empty prediction list",
                        }
                    )
                    + "\n"
                )
                manifest.flush()
                continue

            for model in models:
                discovered += 1
                try:
                    body, coordinate = client.download_cif(
                        model, str(candidate.protein_id), max_bytes=max_coordinate_bytes
                    )
                    object_store.put_object(
                        coordinate.object_key, body, coordinate.content_type
                    )
                    artifact_uri = artifact_storage.uri_for(coordinate.object_key)
                    persist_alpha_fold_artifacts(
                        session,
                        candidate.protein_id,
                        model,
                        coordinate,
                        artifact_uri,
                    )
                    record: dict[str, Any] = manifest_record(
                        str(candidate.protein_id), model, coordinate, "downloaded"
                    )
                    downloaded += 1
                except Exception as exc:
                    failed += 1
                    record = manifest_record(
                        str(candidate.protein_id),
                        model,
                        None,
                        "failed",
                        reason=str(exc),
                    )
                manifest.write(json.dumps(record, sort_keys=True) + "\n")
                manifest.flush()
    return CrawlSummary(
        discovered=discovered,
        downloaded=downloaded,
        no_model=no_model,
        failed=failed,
        manifest_path=str(manifest_path),
    )
