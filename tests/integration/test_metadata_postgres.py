"""PostgreSQL migration and persistence integration tests."""
from alembic import command
from alembic.config import Config
import pytest
from sqlalchemy import create_engine, inspect, select
from sqlalchemy.engine import make_url
from sqlalchemy.exc import IntegrityError
from sqlalchemy.orm import Session
from testcontainers.postgres import PostgresContainer
from protein3d.config import get_settings
from protein3d.metadata.models import Protein, ProteinIdMapping

@pytest.mark.integration
def test_postgres_migrations_and_identity_constraints(monkeypatch: pytest.MonkeyPatch) -> None:
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
            table_names = set(inspect(engine).get_table_names())
            expected = {"proteins", "protein_id_mappings", "structures", "structure_protein_chains", "dataset_runs", "jobs", "artifacts"}
            assert expected <= table_names
            with Session(engine) as session:
                protein = Protein(canonical_accession="P12345")
                session.add(protein)
                session.flush()
                session.add(ProteinIdMapping(source="uniprot", source_id="OLD-ID", source_version="2026_01", mapping_status="unresolved"))
                session.add(ProteinIdMapping(protein_id=protein.protein_id, source="uniprot", source_id="P12345", source_version="2026_01", mapping_status="resolved"))
                session.commit()
                mappings = session.scalars(select(ProteinIdMapping)).all()
                assert len(mappings) == 2
                assert sum(mapping.protein_id is None for mapping in mappings) == 1
                session.add(ProteinIdMapping(protein_id=protein.protein_id, source="uniprot", source_id="BAD", source_version="2026_01", mapping_status="ambiguous"))
                with pytest.raises(IntegrityError):
                    session.commit()
                session.rollback()
            command.downgrade(cfg, "base")
            command.upgrade(cfg, "head")
        finally:
            engine.dispose()
            get_settings.cache_clear()
