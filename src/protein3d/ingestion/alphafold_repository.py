"""Persistence helpers for AlphaFold structure and artifact metadata."""

from __future__ import annotations

import uuid
from typing import Any

from sqlalchemy import select
from sqlalchemy.orm import Session

from protein3d.ingestion.alphafold import AlphaFoldModel, DownloadedCoordinate
from protein3d.metadata.models import Artifact, ProteinIdMapping, Structure


def resolve_uniprot_protein_id(session: Session, accession: str) -> uuid.UUID | None:
    """Return an internal UUID only for a resolved UniProt mapping."""
    mapping = session.scalar(
        select(ProteinIdMapping).where(
            ProteinIdMapping.source == "uniprot",
            ProteinIdMapping.source_id == accession,
            ProteinIdMapping.mapping_status == "resolved",
        )
    )
    return mapping.protein_id if mapping is not None else None


def persist_alpha_fold_artifacts(
    session: Session,
    protein_id: uuid.UUID,
    model: AlphaFoldModel,
    coordinate: DownloadedCoordinate,
    artifact_uri: str,
    *,
    dataset_run_id: uuid.UUID | None = None,
    job_id: uuid.UUID | None = None,
) -> tuple[Structure, Artifact]:
    """Upsert logical structure metadata and its immutable checksum artifact.

    PP-12 has no direct structures.protein_id foreign key. The internal UUID is
    therefore retained in metadata_json until PP-21 creates chain associations.
    The caller owns the transaction and must call this only after MinIO upload.
    """
    storage_uri = artifact_uri
    source_version = model.model_version or "unversioned"
    structure = session.scalar(
        select(Structure).where(
            Structure.source == "alphafold_db",
            Structure.source_structure_id == model.model_entity_id,
            Structure.source_version == source_version,
        )
    )
    metadata: dict[str, Any] = {
        **model.source_metadata,
        "protein_id": str(protein_id),
        "accession": model.accession,
        "sequence_start": model.sequence_start,
        "sequence_end": model.sequence_end,
        "sequence_length": model.sequence_length,
        "confidence": model.confidence_metadata,
        "source_url": coordinate.source_url,
        "retrieved_at": coordinate.retrieved_at,
        "sha256": coordinate.sha256,
    }
    if structure is None:
        structure = Structure(
            source="alphafold_db",
            source_structure_id=model.model_entity_id,
            source_version=source_version,
            structure_format="mmcif",
            storage_uri=storage_uri,
            metadata_json=metadata,
        )
        session.add(structure)
    else:
        structure.storage_uri = storage_uri
        structure.metadata_json = metadata

    artifact = session.scalar(select(Artifact).where(Artifact.uri == storage_uri))
    if artifact is None:
        artifact = Artifact(
            dataset_run_id=dataset_run_id,
            job_id=job_id,
            storage_backend="minio",
            uri=storage_uri,
            artifact_format="mmcif",
            content_type=coordinate.content_type,
            size_bytes=coordinate.size_bytes,
            checksum=coordinate.sha256,
            checksum_algorithm="sha256",
            source_version=source_version,
        )
        session.add(artifact)
    else:
        artifact.size_bytes = coordinate.size_bytes
        artifact.checksum = coordinate.sha256
        artifact.checksum_algorithm = "sha256"
        artifact.content_type = coordinate.content_type
        artifact.source_version = source_version
        artifact.dataset_run_id = dataset_run_id
        artifact.job_id = job_id
    return structure, artifact
