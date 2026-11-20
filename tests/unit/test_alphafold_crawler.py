"""Unit tests for bounded AlphaFold crawler outcome handling."""

import json
import uuid
from pathlib import Path
from typing import cast
from unittest.mock import MagicMock

import pytest
from sqlalchemy.orm import Session

from protein3d.ingestion.alphafold_crawler import (
    ProteinCandidate,
    crawl_candidates,
)
from protein3d.ingestion.minio_storage import MinioObjectStore
from protein3d.metadata.storage import S3ArtifactStorage


@pytest.mark.unit
def test_long_protein_is_reported_unsupported_without_api_request(
    tmp_path: Path,
) -> None:
    client = MagicMock()
    object_store = MagicMock()
    manifest = tmp_path / "alphafold.jsonl"
    candidate = ProteinCandidate(
        protein_id=uuid.UUID("7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81"),
        accession="Q9Y261",
        sequence_length=2801,
    )
    summary = crawl_candidates(
        [candidate],
        session=cast(Session, MagicMock()),
        client=client,
        object_store=cast(MinioObjectStore, object_store),
        artifact_storage=S3ArtifactStorage("protein3d-artifacts"),
        manifest_path=manifest,
    )
    assert summary.unsupported == 1
    assert summary.no_model == 0
    assert summary.failed == 0
    client.get_predictions.assert_not_called()
    object_store.put_file.assert_not_called()
    record = json.loads(manifest.read_text(encoding="utf-8").splitlines()[0])
    assert record["status"] == "unsupported"
    assert record["sequence_length"] == 2801
    assert "outside PP-13 scope" in record["reason"]
