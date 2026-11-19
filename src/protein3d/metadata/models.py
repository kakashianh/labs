"""SQLAlchemy metadata models owned by PP-12."""

from __future__ import annotations

import enum
import uuid
from datetime import UTC, datetime
from typing import Any

from sqlalchemy import (
    BigInteger,
    Boolean,
    CheckConstraint,
    DateTime,
    ForeignKey,
    Index,
    Integer,
    JSON,
    String,
    Text,
    UniqueConstraint,
    Uuid,
    func,
)
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship


def utc_now() -> datetime:
    return datetime.now(UTC)


class Base(DeclarativeBase):
    """Base for all relational metadata models."""


class MappingStatus(str, enum.Enum):
    RESOLVED = "resolved"
    MERGED = "merged"
    DEPRECATED = "deprecated"
    UNRESOLVED = "unresolved"
    AMBIGUOUS = "ambiguous"


class DatasetStatus(str, enum.Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class JobStatus(str, enum.Enum):
    QUEUED = "queued"
    RUNNING = "running"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    CANCELLED = "cancelled"


class Protein(Base):
    __tablename__ = "proteins"
    __table_args__ = (Index("ix_proteins_canonical_accession", "canonical_accession"),)

    protein_id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid.uuid4)
    canonical_accession: Mapped[str | None] = mapped_column(String(64), nullable=True)
    sequence: Mapped[str | None] = mapped_column(Text, nullable=True)
    organism_id: Mapped[str | None] = mapped_column(String(64), nullable=True)
    provenance: Mapped[dict[str, Any]] = mapped_column(JSON, default=dict, nullable=False)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    mappings: Mapped[list[ProteinIdMapping]] = relationship(back_populates="protein")
    chain_associations: Mapped[list[StructureProteinChain]] = relationship(back_populates="protein")


class ProteinIdMapping(Base):
    __tablename__ = "protein_id_mappings"
    __table_args__ = (
        UniqueConstraint("source", "source_id", "source_version", name="uq_protein_mapping_source_key"),
        CheckConstraint(
            "mapping_status IN ('resolved','merged','deprecated','unresolved','ambiguous')",
            name="ck_protein_mapping_status",
        ),
        CheckConstraint(
            "(mapping_status IN ('resolved','merged') AND protein_id IS NOT NULL) OR "
            "(mapping_status IN ('deprecated','unresolved','ambiguous'))",
            name="ck_protein_mapping_target",
        ),
        Index("ix_protein_mappings_protein_id", "protein_id"),
    )

    mapping_id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid.uuid4)
    protein_id: Mapped[uuid.UUID | None] = mapped_column(ForeignKey("proteins.protein_id", ondelete="RESTRICT"), nullable=True)
    source: Mapped[str] = mapped_column(String(80), nullable=False)
    source_id: Mapped[str] = mapped_column(String(255), nullable=False)
    source_version: Mapped[str] = mapped_column(String(100), default="unspecified", nullable=False)
    mapping_status: Mapped[str] = mapped_column(String(20), nullable=False)
    mapping_evidence: Mapped[str | None] = mapped_column(Text, nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    protein: Mapped[Protein | None] = relationship(back_populates="mappings")


class Structure(Base):
    __tablename__ = "structures"
    __table_args__ = (UniqueConstraint("source", "source_structure_id", "source_version", name="uq_structure_source_key"),)

    structure_id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid.uuid4)
    source: Mapped[str] = mapped_column(String(80), nullable=False)
    source_structure_id: Mapped[str] = mapped_column(String(255), nullable=False)
    source_version: Mapped[str | None] = mapped_column(String(100), nullable=True)
    structure_format: Mapped[str] = mapped_column(String(24), nullable=False)
    storage_uri: Mapped[str] = mapped_column(Text, nullable=False)
    resolution_angstrom: Mapped[float | None] = mapped_column(nullable=True)
    metadata_json: Mapped[dict[str, Any]] = mapped_column(JSON, default=dict, nullable=False)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    chains: Mapped[list[StructureProteinChain]] = relationship(back_populates="structure")


class StructureProteinChain(Base):
    __tablename__ = "structure_protein_chains"
    __table_args__ = (
        UniqueConstraint("structure_id", "chain_id", name="uq_structure_chain"),
        Index("ix_structure_chain_protein_id", "protein_id"),
    )

    association_id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid.uuid4)
    structure_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("structures.structure_id", ondelete="RESTRICT"), nullable=False)
    protein_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("proteins.protein_id", ondelete="RESTRICT"), nullable=False)
    chain_id: Mapped[str] = mapped_column(String(16), nullable=False)
    residue_start: Mapped[int | None] = mapped_column(Integer, nullable=True)
    residue_end: Mapped[int | None] = mapped_column(Integer, nullable=True)

    structure: Mapped[Structure] = relationship(back_populates="chains")
    protein: Mapped[Protein] = relationship(back_populates="chain_associations")


class DatasetRun(Base):
    __tablename__ = "dataset_runs"
    __table_args__ = (
        CheckConstraint("status IN ('pending','running','completed','failed','cancelled')", name="ck_dataset_run_status"),
        Index("ix_dataset_runs_status_created", "status", "created_at"),
    )

    dataset_run_id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid.uuid4)
    dataset_name: Mapped[str] = mapped_column(String(120), nullable=False)
    dataset_type: Mapped[str] = mapped_column(String(80), nullable=False)
    source: Mapped[str] = mapped_column(String(120), nullable=False)
    source_version: Mapped[str | None] = mapped_column(String(100), nullable=True)
    status: Mapped[str] = mapped_column(String(20), default=DatasetStatus.PENDING.value, nullable=False)
    provenance: Mapped[dict[str, Any]] = mapped_column(JSON, default=dict, nullable=False)
    started_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    finished_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    jobs: Mapped[list[Job]] = relationship(back_populates="dataset_run")


class Job(Base):
    __tablename__ = "jobs"
    __table_args__ = (
        CheckConstraint("status IN ('queued','running','succeeded','failed','cancelled')", name="ck_job_status"),
        CheckConstraint("retry_count >= 0", name="ck_job_retry_count"),
        UniqueConstraint("idempotency_key", name="uq_job_idempotency_key"),
        Index("ix_jobs_status_created", "status", "created_at"),
    )

    job_id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid.uuid4)
    dataset_run_id: Mapped[uuid.UUID | None] = mapped_column(ForeignKey("dataset_runs.dataset_run_id", ondelete="RESTRICT"), nullable=True)
    job_type: Mapped[str] = mapped_column(String(100), nullable=False)
    status: Mapped[str] = mapped_column(String(20), default=JobStatus.QUEUED.value, nullable=False)
    retry_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    idempotency_key: Mapped[str | None] = mapped_column(String(255), nullable=True)
    error_summary: Mapped[str | None] = mapped_column(Text, nullable=True)
    parameters: Mapped[dict[str, Any]] = mapped_column(JSON, default=dict, nullable=False)
    started_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    finished_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    dataset_run: Mapped[DatasetRun | None] = relationship(back_populates="jobs")
    artifacts: Mapped[list[Artifact]] = relationship(back_populates="job")


class Artifact(Base):
    __tablename__ = "artifacts"
    __table_args__ = (
        CheckConstraint("size_bytes IS NULL OR size_bytes >= 0", name="ck_artifact_size_nonnegative"),
        Index("ix_artifacts_dataset_run_id", "dataset_run_id"),
        Index("ix_artifacts_job_id", "job_id"),
    )

    artifact_id: Mapped[uuid.UUID] = mapped_column(Uuid, primary_key=True, default=uuid.uuid4)
    dataset_run_id: Mapped[uuid.UUID | None] = mapped_column(ForeignKey("dataset_runs.dataset_run_id", ondelete="RESTRICT"), nullable=True)
    job_id: Mapped[uuid.UUID | None] = mapped_column(ForeignKey("jobs.job_id", ondelete="RESTRICT"), nullable=True)
    storage_backend: Mapped[str] = mapped_column(String(40), nullable=False)
    uri: Mapped[str] = mapped_column(Text, nullable=False, unique=True)
    artifact_format: Mapped[str] = mapped_column(String(40), nullable=False)
    content_type: Mapped[str | None] = mapped_column(String(120), nullable=True)
    size_bytes: Mapped[int | None] = mapped_column(BigInteger, nullable=True)
    checksum: Mapped[str | None] = mapped_column(String(256), nullable=True)
    checksum_algorithm: Mapped[str | None] = mapped_column(String(24), nullable=True)
    source_version: Mapped[str | None] = mapped_column(String(100), nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    job: Mapped[Job | None] = relationship(back_populates="artifacts")
