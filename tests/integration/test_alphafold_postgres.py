"""PostgreSQL integration coverage for the AlphaFold crawler persistence boundary."""

# ruff: noqa: I001

import uuid

import pytest
from alembic import command
from alembic.config import Config
from sqlalchemy import create_engine, func, select
from sqlalchemy.engine import make_url
from sqlalchemy.orm import Session
from testcontainers.postgres import PostgresContainer  # type: ignore[import-untyped]

from protein3d.config import get_settings
from protein3d.ingestion.alphafold import DownloadedCoordinate, parse_prediction
from protein3d.ingestion.alphafold_crawler import select_candidates
from protein3d.ingestion.alphafold_repository import persist_alpha_fold_artifacts
from protein3d.metadata.models import (
    Artifact,
    DatasetRun,
    Job,
    Protein,
    ProteinIdMapping,
    Structure,
)


RECORD = {
    "modelEntityId": "AF-Q9Y261-F1",
    "cifUrl": "https://alphafold.ebi.ac.uk/files/AF-Q9Y261-F1-model_v6.cif",
    "sequenceStart": 1,
    "sequenceEnd": 100,
    "sequence": "A" * 100,
    "latestVersion": 6,
}


@pytest.mark.integration
def test_crawler_candidate_selection_and_metadata_upsert(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    with PostgresContainer("postgres:17.6-alpine", driver="psycopg") as postgres:
        url = make_url(postgres.get_connection_url())
        monkeypatch.setenv("PROTEIN3D_DATABASE_HOST", str(url.host))
        monkeypatch.setenv("PROTEIN3D_DATABASE_PORT", str(url.port))
        monkeypatch.setenv("PROTEIN3D_DATABASE_NAME", str(url.database))
        monkeypatch.setenv("PROTEIN3D_DATABASE_USER", str(url.username))
        monkeypatch.setenv("PROTEIN3D_DATABASE_PASSWORD", str(url.password))
        get_settings.cache_clear()

        cfg = Config("alembic.ini")
        command.upgrade(cfg, "head")
        engine = create_engine(postgres.get_connection_url(), pool_pre_ping=True)
        try:
            with Session(engine, expire_on_commit=False) as session:
                target = Protein(organism_id="9606", canonical_accession="Q9Y261")
                mouse = Protein(organism_id="10090", canonical_accession="P12345")
                session.add_all([target, mouse])
                session.flush()
                session.add_all(
                    [
                        ProteinIdMapping(
                            protein_id=target.protein_id,
                            source="uniprot",
                            source_id="Q9Y261",
                            mapping_status="resolved",
                        ),
                        ProteinIdMapping(
                            protein_id=target.protein_id,
                            source="uniprot",
                            source_id="A0A000",
                            mapping_status="resolved",
                        ),
                        ProteinIdMapping(
                            source="uniprot",
                            source_id="UNRESOLVED",
                            mapping_status="unresolved",
                        ),
                        ProteinIdMapping(
                            protein_id=mouse.protein_id,
                            source="uniprot",
                            source_id="P12345",
                            mapping_status="resolved",
                        ),
                    ]
                )
                session.commit()

                candidates = select_candidates(session, taxonomy_id=9606, sample_limit=10)
                assert len(candidates) == 1
                assert candidates[0].protein_id == target.protein_id
                assert candidates[0].accession == "A0A000"
                with pytest.raises(ValueError, match="sample_limit"):
                    select_candidates(session, 9606, 0)

                model = parse_prediction("A0A000", [RECORD])[0]
                coordinate = DownloadedCoordinate(
                    source_url=model.cif_url,
                    retrieved_at="2026-11-20T00:00:00+00:00",
                    object_key=f"alphafold/{target.protein_id}/AF-Q9Y261-F1/6/model.cif",
                    content_type="chemical/x-cif",
                    size_bytes=42,
                    sha256="a" * 64,
                )
                artifact_uri = f"s3://protein3d-artifacts/artifacts/{coordinate.object_key}"
                dataset_run = DatasetRun(
                    dataset_name="alphafold-test",
                    dataset_type="structure_crawl",
                    source="alphafold_db",
                )
                session.add(dataset_run)
                session.flush()
                job = Job(
                    dataset_run_id=dataset_run.dataset_run_id,
                    job_type="alphafold_structure_crawl",
                    idempotency_key=f"alphafold-test:{uuid.uuid4()}",
                )
                session.add(job)
                session.flush()
                run_id = dataset_run.dataset_run_id
                job_id = job.job_id

                persist_alpha_fold_artifacts(
                    session,
                    target.protein_id,
                    model,
                    coordinate,
                    artifact_uri,
                    dataset_run_id=run_id,
                    job_id=job_id,
                )
                session.commit()
                persist_alpha_fold_artifacts(
                    session,
                    target.protein_id,
                    model,
                    coordinate,
                    artifact_uri,
                    dataset_run_id=run_id,
                    job_id=job_id,
                )
                session.commit()

                assert session.scalar(select(func.count()).select_from(Structure)) == 1
                assert session.scalar(select(func.count()).select_from(Artifact)) == 1
                structure = session.scalar(select(Structure))
                assert structure is not None
                assert structure.storage_uri == artifact_uri
                assert structure.metadata_json["protein_id"] == str(target.protein_id)
                artifact = session.scalar(select(Artifact))
                assert artifact is not None
                assert artifact.checksum == "a" * 64
                assert artifact.dataset_run_id == run_id
                assert artifact.job_id == job_id
        finally:
            engine.dispose()
            get_settings.cache_clear()
