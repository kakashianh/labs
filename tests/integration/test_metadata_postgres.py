"""PostgreSQL migration and persistence integration tests."""

# fmt: off

import pytest
from alembic import command
from alembic.config import Config
from sqlalchemy import create_engine, inspect, select
from sqlalchemy.engine import make_url
from sqlalchemy.exc import IntegrityError
from sqlalchemy.orm import Session
from testcontainers.postgres import PostgresContainer  # type: ignore[import-untyped]

from protein3d.config import get_settings
from protein3d.metadata.models import (
    Artifact,
    DatasetRun,
    Job,
    Protein,
    ProteinIdMapping,
    Structure,
    StructureProteinChain,
)


@pytest.mark.integration
def test_postgres_migrations_and_metadata_constraints(
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
        engine = create_engine(get_settings().database_url)
        try:
            expected = {
                "proteins",
                "protein_id_mappings",
                "structures",
                "structure_protein_chains",
                "dataset_runs",
                "jobs",
                "artifacts",
            }
            assert expected <= set(inspect(engine).get_table_names())

            with Session(engine) as session:
                protein = Protein(canonical_accession="P12345")
                structure = Structure(
                    source="alphafold",
                    source_structure_id="AF-P12345-F1",
                    structure_format="cif",
                    storage_uri=(
                        "s3://protein3d-artifacts/structures/AF-P12345-F1.cif"
                    ),
                )
                dataset = DatasetRun(
                    dataset_name="protein-master",
                    dataset_type="protein",
                    source="uniprot",
                    source_version="2026_01",
                )
                session.add_all([protein, structure, dataset])
                session.flush()

                session.add_all(
                    [
                        ProteinIdMapping(
                            source="uniprot",
                            source_id="OLD-ID",
                            source_version="2026_01",
                            mapping_status="unresolved",
                        ),
                        ProteinIdMapping(
                            source="uniprot",
                            source_id="AMBIGUOUS-ID",
                            source_version="2026_01",
                            mapping_status="ambiguous",
                        ),
                        ProteinIdMapping(
                            protein_id=protein.protein_id,
                            source="uniprot",
                            source_id="P12345",
                            source_version="2026_01",
                            mapping_status="resolved",
                        ),
                        ProteinIdMapping(
                            protein_id=protein.protein_id,
                            source="uniprot",
                            source_id="MERGED-ID",
                            source_version="2026_01",
                            mapping_status="merged",
                        ),
                        ProteinIdMapping(
                            source="uniprot",
                            source_id="RETIRED-ID",
                            source_version="2026_01",
                            mapping_status="deprecated",
                        ),
                        StructureProteinChain(
                            structure=structure,
                            protein=protein,
                            chain_id="A",
                        ),
                    ]
                )
                job = Job(
                    dataset_run=dataset,
                    job_type="structure-index",
                    idempotency_key="index:AF-P12345-F1",
                )
                session.add(job)
                session.flush()
                session.add(
                    Artifact(
                        dataset_run_id=dataset.dataset_run_id,
                        job_id=job.job_id,
                        storage_backend="s3",
                        uri="s3://protein3d-artifacts/test/structure.cif",
                        artifact_format="cif",
                        size_bytes=128,
                        checksum="abcd",
                        checksum_algorithm="sha256",
                    )
                )
                session.commit()

                mappings = session.scalars(select(ProteinIdMapping)).all()
                assert len(mappings) == 5
                assert sum(mapping.protein_id is None for mapping in mappings) == 3
                assert len(structure.chains) == 1
                assert len(job.artifacts) == 1

                session.add(
                    ProteinIdMapping(
                        protein_id=protein.protein_id,
                        source="uniprot",
                        source_id="INVALID-AMBIGUOUS",
                        source_version="2026_01",
                        mapping_status="ambiguous",
                    )
                )
                with pytest.raises(IntegrityError):
                    session.commit()
                session.rollback()

            command.downgrade(cfg, "base")
            command.upgrade(cfg, "head")
        finally:
            engine.dispose()
            get_settings.cache_clear()
# fmt: on
