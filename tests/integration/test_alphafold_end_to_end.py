"""Combined PostgreSQL + MinIO crawler flow using a fixture-backed AFDB response."""

import hashlib
import time
import uuid
from pathlib import Path
from typing import cast
from unittest.mock import MagicMock
from urllib.error import URLError
from urllib.request import urlopen

import pytest
from alembic import command
from alembic.config import Config
from pydantic import SecretStr
from sqlalchemy import create_engine, func, select
from sqlalchemy.engine import make_url
from sqlalchemy.orm import Session
from testcontainers.core.container import DockerContainer  # type: ignore[import-untyped]
from testcontainers.postgres import PostgresContainer  # type: ignore[import-untyped]

from protein3d.config import Settings, get_settings
from protein3d.ingestion.alphafold import (
    AlphaFoldClient,
    AlphaFoldModel,
    DownloadedCoordinate,
    parse_prediction,
)
from protein3d.ingestion.alphafold_crawler import run_configured_crawl
from protein3d.ingestion.minio_storage import MinioObjectStore
from protein3d.metadata.models import (
    Artifact,
    DatasetRun,
    DatasetStatus,
    Job,
    JobStatus,
    Protein,
    ProteinIdMapping,
    Structure,
)


@pytest.mark.integration
def test_fixture_crawl_persists_verified_minio_artifact_and_resumes(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    with PostgresContainer("postgres:17.6-alpine", driver="psycopg") as postgres:
        database_url = make_url(postgres.get_connection_url())
        monkeypatch.setenv("PROTEIN3D_DATABASE_HOST", str(database_url.host))
        monkeypatch.setenv("PROTEIN3D_DATABASE_PORT", str(database_url.port))
        monkeypatch.setenv("PROTEIN3D_DATABASE_NAME", str(database_url.database))
        monkeypatch.setenv("PROTEIN3D_DATABASE_USER", str(database_url.username))
        monkeypatch.setenv("PROTEIN3D_DATABASE_PASSWORD", str(database_url.password))
        get_settings.cache_clear()
        command.upgrade(Config("alembic.ini"), "head")
        engine = create_engine(postgres.get_connection_url(), pool_pre_ping=True)
        try:
            with Session(engine, expire_on_commit=False) as session:
                protein = Protein(organism_id="9606", canonical_accession="Q9Y261")
                session.add(protein)
                session.flush()
                session.add(
                    ProteinIdMapping(
                        protein_id=protein.protein_id,
                        source="uniprot",
                        source_id="Q9Y261",
                        mapping_status="resolved",
                    )
                )
                session.commit()

                username = "protein3dtest"
                password = "protein3d-test-secret"
                container = (
                    DockerContainer("soldevelo/minio:2026.7.17")
                    .with_exposed_ports(9000)
                    .with_env("MINIO_ROOT_USER", username)
                    .with_env("MINIO_ROOT_PASSWORD", password)
                )
                with container as minio:
                    host = minio.get_container_host_ip()
                    port = int(minio.get_exposed_port(9000))
                    health_url = f"http://{host}:{port}/minio/health/ready"
                    deadline = time.monotonic() + 60
                    while True:
                        try:
                            with urlopen(health_url, timeout=2) as response:
                                if response.status == 200:
                                    break
                        except (OSError, URLError):
                            if time.monotonic() >= deadline:
                                raise TimeoutError(
                                    "MinIO did not become ready within 60 seconds"
                                ) from None
                            time.sleep(1)

                    object_store = MinioObjectStore(
                        endpoint=f"{host}:{port}",
                        bucket="protein3d-test",
                        access_key=username,
                        secret_key=password,
                        timeout_seconds=10,
                    )
                    object_store.ensure_bucket()

                    payload = b"data_AF-Q9Y261-F1\n#\n"
                    model: AlphaFoldModel = parse_prediction(
                        "Q9Y261",
                        [
                            {
                                "modelEntityId": "AF-Q9Y261-F1",
                                "cifUrl": "https://alphafold.ebi.ac.uk/files/model.cif",
                                "latestVersion": 6,
                                "sequence": "A" * 20,
                            }
                        ],
                    )[0]
                    coordinate = DownloadedCoordinate(
                        source_url=model.cif_url,
                        retrieved_at="2026-11-20T00:00:00+00:00",
                        object_key=(f"alphafold/{protein.protein_id}/AF-Q9Y261-F1/6/model.cif"),
                        content_type="chemical/x-cif",
                        size_bytes=len(payload),
                        sha256=hashlib.sha256(payload).hexdigest(),
                    )
                    client = MagicMock()
                    client.get_predictions.return_value = [model]

                    def download_fixture(
                        _model: AlphaFoldModel,
                        _protein_id: str,
                        destination: Path,
                        **_kwargs: object,
                    ) -> DownloadedCoordinate:
                        destination.write_bytes(payload)
                        return coordinate

                    client.download_cif_to_path.side_effect = download_fixture
                    settings = Settings(
                        object_storage_endpoint=f"{host}:{port}",
                        object_storage_bucket="protein3d-test",
                        object_storage_access_key=username,
                        object_storage_secret_key=SecretStr(password),
                        object_storage_prefix="artifacts",
                        external_request_timeout_seconds=10,
                        external_max_retries=0,
                        alphafold_taxonomy_id=9606,
                        alphafold_sample_limit=1,
                        alphafold_max_coordinate_bytes=1024 * 1024,
                    )

                    class TrackingObjectStore:
                        def __init__(self, delegate: MinioObjectStore) -> None:
                            self.delegate = delegate
                            self.put_calls: list[str] = []
                            self.fail_next_put = False

                        def ensure_bucket(self) -> None:
                            self.delegate.ensure_bucket()

                        def head_object(self, object_key: str) -> int | None:
                            return self.delegate.head_object(object_key)

                        def get_file(self, object_key: str, destination: Path) -> tuple[int, str]:
                            return self.delegate.get_file(object_key, destination)

                        def put_file(
                            self,
                            object_key: str,
                            file_path: Path,
                            content_type: str,
                            sha256: str,
                        ) -> None:
                            self.put_calls.append(object_key)
                            if self.fail_next_put:
                                self.fail_next_put = False
                                raise RuntimeError("simulated MinIO upload interruption")
                            self.delegate.put_file(object_key, file_path, content_type, sha256)

                    tracking_store = TrackingObjectStore(object_store)

                    monkeypatch.setattr(
                        "protein3d.ingestion.alphafold_crawler.AlphaFoldClient",
                        lambda **_kwargs: cast(AlphaFoldClient, client),
                    )
                    monkeypatch.setattr(
                        "protein3d.ingestion.alphafold_crawler.MinioObjectStore",
                        lambda **_kwargs: cast(MinioObjectStore, tracking_store),
                    )

                    tracking_store.fail_next_put = True
                    failed = run_configured_crawl(
                        session,
                        settings,
                        tmp_path / "failed.jsonl",
                        sample_limit=1,
                    )
                    assert failed.failed == 1
                    assert failed.retryable_failed == 1
                    assert failed.permanent_failed == 0
                    assert session.scalar(select(func.count()).select_from(Structure)) == 0
                    assert session.scalar(select(func.count()).select_from(Artifact)) == 0
                    failed_run = session.get(DatasetRun, uuid.UUID(failed.dataset_run_id or ""))
                    failed_job = session.get(Job, uuid.UUID(failed.job_id or ""))
                    assert failed_run is not None
                    assert failed_run.status == DatasetStatus.FAILED.value
                    assert failed_job is not None
                    assert failed_job.status == JobStatus.FAILED.value
                    tracking_store.put_calls.clear()

                    first = run_configured_crawl(
                        session,
                        settings,
                        tmp_path / "first.jsonl",
                        sample_limit=1,
                    )
                    assert first.downloaded == 1
                    assert first.failed == 0
                    assert len(tracking_store.put_calls) == 1

                    tracking_store.put_calls.clear()
                    second = run_configured_crawl(
                        session,
                        settings,
                        tmp_path / "second.jsonl",
                        sample_limit=1,
                    )
                    assert second.downloaded == 1
                    assert second.failed == 0
                    assert tracking_store.put_calls == []
                    assert session.scalar(select(func.count()).select_from(Structure)) == 1
                    assert session.scalar(select(func.count()).select_from(Artifact)) == 1
                    assert session.scalar(select(func.count()).select_from(DatasetRun)) == 3
                    assert session.scalar(select(func.count()).select_from(Job)) == 3

                    runs = session.scalars(select(DatasetRun).order_by(DatasetRun.started_at)).all()
                    assert runs[0].status == DatasetStatus.FAILED.value
                    assert all(
                        run.status == DatasetStatus.COMPLETED.value for run in runs[1:]
                    )
                    assert runs[-1].provenance["downloaded"] == 1
                    jobs = session.scalars(select(Job).order_by(Job.started_at)).all()
                    assert jobs[0].status == JobStatus.FAILED.value
                    assert all(
                        job.status == JobStatus.SUCCEEDED.value for job in jobs[1:]
                    )

                    structure = session.scalar(select(Structure))
                    assert structure is not None
                    assert structure.metadata_json["protein_id"] == str(protein.protein_id)
                    assert structure.storage_uri == (
                        f"s3://protein3d-test/artifacts/{coordinate.object_key}"
                    )
                    artifact = session.scalar(select(Artifact))
                    assert artifact is not None
                    assert artifact.checksum == coordinate.sha256
                    assert artifact.dataset_run_id == runs[-1].dataset_run_id
                    assert tracking_store.head_object(f"artifacts/{coordinate.object_key}") == len(
                        payload
                    )
        finally:
            engine.dispose()
            get_settings.cache_clear()
