"""Create PP-12 relational metadata schema."""

# fmt: off

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

revision = "0001_initial_metadata"
down_revision = None
branch_labels = None
depends_on = None


def upgrade() -> None:
    op.create_table(
        "proteins",
        sa.Column("protein_id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("canonical_accession", sa.String(64)),
        sa.Column("sequence", sa.Text()),
        sa.Column("organism_id", sa.String(64)),
        sa.Column(
            "provenance",
            postgresql.JSONB(),
            nullable=False,
            server_default=sa.text("'{}'::jsonb"),
        ),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            nullable=False,
            server_default=sa.func.now(),
        ),
        sa.Column(
            "updated_at",
            sa.DateTime(timezone=True),
            nullable=False,
            server_default=sa.func.now(),
        ),
    )
    op.create_index(
        "ix_proteins_canonical_accession", "proteins", ["canonical_accession"]
    )

    op.create_table(
        "protein_id_mappings",
        sa.Column("mapping_id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column(
            "protein_id",
            postgresql.UUID(as_uuid=True),
            sa.ForeignKey("proteins.protein_id", ondelete="RESTRICT"),
        ),
        sa.Column("source", sa.String(80), nullable=False),
        sa.Column("source_id", sa.String(255), nullable=False),
        sa.Column(
            "source_version",
            sa.String(100),
            nullable=False,
            server_default="unspecified",
        ),
        sa.Column("mapping_status", sa.String(20), nullable=False),
        sa.Column("mapping_evidence", sa.Text()),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            nullable=False,
            server_default=sa.func.now(),
        ),
        sa.CheckConstraint(
            "mapping_status IN ('resolved','merged','deprecated','unresolved','ambiguous')",
            name="ck_protein_mapping_status",
        ),
        sa.CheckConstraint(
            "(mapping_status IN ('resolved','merged') AND protein_id IS NOT NULL) "
            "OR (mapping_status = 'deprecated') "
            "OR (mapping_status IN ('unresolved','ambiguous') AND protein_id IS NULL)",
            name="ck_protein_mapping_target",
        ),
        sa.UniqueConstraint(
            "source", "source_id", "source_version", name="uq_protein_mapping_source_key"
        ),
    )
    op.create_index(
        "ix_protein_mappings_protein_id", "protein_id_mappings", ["protein_id"]
    )

    op.create_table(
        "structures",
        sa.Column("structure_id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("source", sa.String(80), nullable=False),
        sa.Column("source_structure_id", sa.String(255), nullable=False),
        sa.Column("source_version", sa.String(100)),
        sa.Column("structure_format", sa.String(24), nullable=False),
        sa.Column("storage_uri", sa.Text(), nullable=False),
        sa.Column("resolution_angstrom", sa.Float()),
        sa.Column(
            "metadata_json",
            postgresql.JSONB(),
            nullable=False,
            server_default=sa.text("'{}'::jsonb"),
        ),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            nullable=False,
            server_default=sa.func.now(),
        ),
        sa.UniqueConstraint(
            "source",
            "source_structure_id",
            "source_version",
            name="uq_structure_source_key",
        ),
    )

    op.create_table(
        "structure_protein_chains",
        sa.Column("association_id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column(
            "structure_id",
            postgresql.UUID(as_uuid=True),
            sa.ForeignKey("structures.structure_id", ondelete="RESTRICT"),
            nullable=False,
        ),
        sa.Column(
            "protein_id",
            postgresql.UUID(as_uuid=True),
            sa.ForeignKey("proteins.protein_id", ondelete="RESTRICT"),
            nullable=False,
        ),
        sa.Column("chain_id", sa.String(16), nullable=False),
        sa.Column("residue_start", sa.Integer()),
        sa.Column("residue_end", sa.Integer()),
        sa.UniqueConstraint("structure_id", "chain_id", name="uq_structure_chain"),
    )
    op.create_index(
        "ix_structure_chain_protein_id",
        "structure_protein_chains",
        ["protein_id"],
    )

    op.create_table(
        "dataset_runs",
        sa.Column("dataset_run_id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("dataset_name", sa.String(120), nullable=False),
        sa.Column("dataset_type", sa.String(80), nullable=False),
        sa.Column("source", sa.String(120), nullable=False),
        sa.Column("source_version", sa.String(100)),
        sa.Column("status", sa.String(20), nullable=False, server_default="pending"),
        sa.Column(
            "provenance",
            postgresql.JSONB(),
            nullable=False,
            server_default=sa.text("'{}'::jsonb"),
        ),
        sa.Column("started_at", sa.DateTime(timezone=True)),
        sa.Column("finished_at", sa.DateTime(timezone=True)),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            nullable=False,
            server_default=sa.func.now(),
        ),
        sa.CheckConstraint(
            "status IN ('pending','running','completed','failed','cancelled')",
            name="ck_dataset_run_status",
        ),
    )
    op.create_index(
        "ix_dataset_runs_status_created", "dataset_runs", ["status", "created_at"]
    )

    op.create_table(
        "jobs",
        sa.Column("job_id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column(
            "dataset_run_id",
            postgresql.UUID(as_uuid=True),
            sa.ForeignKey("dataset_runs.dataset_run_id", ondelete="RESTRICT"),
        ),
        sa.Column("job_type", sa.String(100), nullable=False),
        sa.Column("status", sa.String(20), nullable=False, server_default="queued"),
        sa.Column("retry_count", sa.Integer(), nullable=False, server_default="0"),
        sa.Column("idempotency_key", sa.String(255)),
        sa.Column("error_summary", sa.Text()),
        sa.Column(
            "parameters",
            postgresql.JSONB(),
            nullable=False,
            server_default=sa.text("'{}'::jsonb"),
        ),
        sa.Column("started_at", sa.DateTime(timezone=True)),
        sa.Column("finished_at", sa.DateTime(timezone=True)),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            nullable=False,
            server_default=sa.func.now(),
        ),
        sa.CheckConstraint(
            "status IN ('queued','running','succeeded','failed','cancelled')",
            name="ck_job_status",
        ),
        sa.CheckConstraint("retry_count >= 0", name="ck_job_retry_count"),
        sa.UniqueConstraint("idempotency_key", name="uq_job_idempotency_key"),
    )
    op.create_index("ix_jobs_status_created", "jobs", ["status", "created_at"])

    op.create_table(
        "artifacts",
        sa.Column("artifact_id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column(
            "dataset_run_id",
            postgresql.UUID(as_uuid=True),
            sa.ForeignKey("dataset_runs.dataset_run_id", ondelete="RESTRICT"),
        ),
        sa.Column(
            "job_id",
            postgresql.UUID(as_uuid=True),
            sa.ForeignKey("jobs.job_id", ondelete="RESTRICT"),
        ),
        sa.Column("storage_backend", sa.String(40), nullable=False),
        sa.Column("uri", sa.Text(), nullable=False, unique=True),
        sa.Column("artifact_format", sa.String(40), nullable=False),
        sa.Column("content_type", sa.String(120)),
        sa.Column("size_bytes", sa.BigInteger()),
        sa.Column("checksum", sa.String(256)),
        sa.Column("checksum_algorithm", sa.String(24)),
        sa.Column("source_version", sa.String(100)),
        sa.Column(
            "created_at",
            sa.DateTime(timezone=True),
            nullable=False,
            server_default=sa.func.now(),
        ),
        sa.CheckConstraint(
            "size_bytes IS NULL OR size_bytes >= 0",
            name="ck_artifact_size_nonnegative",
        ),
    )
    op.create_index(
        "ix_artifacts_dataset_run_id", "artifacts", ["dataset_run_id"]
    )
    op.create_index("ix_artifacts_job_id", "artifacts", ["job_id"])


def downgrade() -> None:
    op.drop_table("artifacts")
    op.drop_table("jobs")
    op.drop_table("dataset_runs")
    op.drop_table("structure_protein_chains")
    op.drop_table("structures")
    op.drop_table("protein_id_mappings")
    op.drop_table("proteins")
# fmt: on
