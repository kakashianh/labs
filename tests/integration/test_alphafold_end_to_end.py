"""Combined PostgreSQL + MinIO crawler flow using a fixture-backed AFDB response."""

import hashlib
import time
from pathlib import Path
from typing import cast
from urllib.error import URLError
from urllib.request import urlopen
from unittest.mock import MagicMock

import pytest
from alembic import command
from alembic.config import Config
from sqlalchemy import create_engine, func, select
from sqlalchemy.engine import make_url
from sqlalchemy.orm import Session
from testcontainers.core.container import DockerContainer  # type: ignore[import-untyped]
from testcontainers.postgres import PostgresContainer  # type: ignore[import-untyped]

from protein3d.config import get_settings
from protein3d.ingestion.alphafold import (
    AlphaFoldClient,
    AlphaFoldModel,
    DownloadedCoordinate,
    parse_prediction,
)
from protein3d.ingestion.alphafold_crawler import ProteinCandidate, crawl_candidates
from protein3d.ingestion.minio_storage import MinioObjectStore
from protein3d.metadata.models import Artifact, Protein, ProteinIdMapping, Structure
from protein3d.metadata.storage import S3ArtifactStorage


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
                        object_key=(
                            f"alphafold/{protein.protein_id}/AF-Q9Y261-F1/6/model.cif"
                        ),
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
                    storage = S3ArtifactStorage(
                        bucket="protein3d-test",
                        prefix="artifacts",
                    )
                    candidate = ProteinCandidate(
                        protein_id=protein.protein_id,
                        accession="Q9Y261",
                        sequence_length=20,
                    )
                    original_put_file = object_store.put_file
                    put_calls: list[str] = []

                    def tracked_put_file(
                        object_key: str,
                        file_path: Path,
                        content_type: str,
                        sha256: str,
                    ) -> None:
                        put_calls.append(object_key)
                        original_put_file(object_key, file_path, content_type, sha256)

                    object.__setattr__(object_store, "put_file", tracked_put_file)

                    first = crawl_candidates(
                        [candidate],
                        session=session,
                        client=cast(AlphaFoldClient, client),
                        object_store=object_store,
                        artifact_storage=storage,
                        manifest_path=tmp_path / "first.jsonl",
                    )
                    assert first.downloaded == 1
                    assert first.failed == 0
                    assert len(put_calls) == 1

                    put_calls.clear()
                    second = crawl_candidates(
                        [candidate],
                        session=session,
                        client=cast(AlphaFoldClient, client),
                        object_store=object_store,
                        artifact_storage=storage,
                        manifest_path=tmp_path / "second.jsonl",
                    )
                    assert second.downloaded == 1
                    assert second.failed == 0
                    assert put_calls == []
                    assert session.scalar(select(func.count()).select_from(Structure)) == 1
                    assert session.scalar(select(func.count()).select_from(Artifact)) == 1
                    structure = session.scalar(select(Structure))
                    assert structure is not None
                    assert structure.metadata_json["protein_id"] == str(protein.protein_id)
                    assert structure.storage_uri == (
                        f"s3://protein3d-test/artifacts/{coordinate.object_key}"
                    )
                    artifact = session.scalar(select(Artifact))
                    assert artifact is not None
                    assert artifact.checksum == coordinate.sha256
                    assert object_store.head_object(
                        f"artifacts/{coordinate.object_key}"
                    ) == len(payload)
        finally:
            engine.dispose()
            get_settings.cache_clear()
