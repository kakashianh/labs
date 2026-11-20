"""Unit tests for bounded AlphaFold crawler outcome handling."""

import hashlib
import json
import uuid
from pathlib import Path
from typing import cast
from unittest.mock import MagicMock

import pytest
from sqlalchemy.orm import Session

from protein3d.ingestion.alphafold import (
    AlphaFoldClient,
    AlphaFoldContractError,
    AlphaFoldRateLimit,
    DownloadedCoordinate,
    parse_prediction,
)
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


@pytest.mark.unit
def test_crawler_reuses_existing_minio_object_when_checksum_matches(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    payload = b"data_AF-Q9Y261-F1\n#\n"
    checksum = hashlib.sha256(payload).hexdigest()
    model = parse_prediction(
        "Q9Y261",
        [
            {
                "modelEntityId": "AF-Q9Y261-F1",
                "cifUrl": "https://alphafold.ebi.ac.uk/files/model.cif",
                "latestVersion": 6,
            }
        ],
    )[0]
    coordinate = DownloadedCoordinate(
        source_url=model.cif_url,
        retrieved_at="2026-11-20T00:00:00+00:00",
        object_key="alphafold/7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81/AF-Q9Y261-F1/6/model.cif",
        content_type="chemical/x-cif",
        size_bytes=len(payload),
        sha256=checksum,
    )
    client = MagicMock()
    client.get_predictions.return_value = [model]

    def write_download(
        _model: object, _protein_id: str, destination: Path, **_kwargs: object
    ) -> DownloadedCoordinate:
        destination.write_bytes(payload)
        return coordinate

    client.download_cif_to_path.side_effect = write_download
    object_store = MagicMock()
    object_store.head_object.return_value = len(payload)

    def write_readback(_key: str, destination: Path) -> tuple[int, str]:
        destination.write_bytes(payload)
        return len(payload), checksum

    object_store.get_file.side_effect = write_readback
    monkeypatch.setattr(
        "protein3d.ingestion.alphafold_crawler.persist_alpha_fold_artifacts",
        MagicMock(),
    )
    summary = crawl_candidates(
        [
            ProteinCandidate(
                protein_id=uuid.UUID("7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81"),
                accession="Q9Y261",
            )
        ],
        session=cast(Session, MagicMock()),
        client=cast(AlphaFoldClient, client),
        object_store=cast(MinioObjectStore, object_store),
        artifact_storage=S3ArtifactStorage("protein3d-artifacts"),
        manifest_path=tmp_path / "resume.jsonl",
    )
    assert summary.downloaded == 1
    assert summary.failed == 0
    object_store.put_file.assert_not_called()
    object_store.get_file.assert_called_once()
    record = json.loads((tmp_path / "resume.jsonl").read_text().splitlines()[0])
    assert record["storage_action"] == "reused"


@pytest.mark.unit
def test_crawler_manifest_distinguishes_retryable_and_permanent_api_failures(
    tmp_path: Path,
) -> None:
    client = MagicMock()
    client.get_predictions.side_effect = [
        AlphaFoldRateLimit("AFDB 429"),
        AlphaFoldContractError("AFDB response contract changed"),
    ]
    candidates = [
        ProteinCandidate(
            protein_id=uuid.UUID("7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81"),
            accession="Q9Y261",
        ),
        ProteinCandidate(
            protein_id=uuid.UUID("a8e3f9a8-cd21-4e1f-b4f6-7a6d5c4b3a21"),
            accession="P12345",
        ),
    ]
    manifest = tmp_path / "failures.jsonl"
    summary = crawl_candidates(
        candidates,
        session=cast(Session, MagicMock()),
        client=cast(AlphaFoldClient, client),
        object_store=cast(MinioObjectStore, MagicMock()),
        artifact_storage=S3ArtifactStorage("protein3d-artifacts"),
        manifest_path=manifest,
    )
    records = [json.loads(line) for line in manifest.read_text().splitlines()]
    assert summary.failed == 2
    assert summary.retryable_failed == 1
    assert summary.permanent_failed == 1
    assert [record["status"] for record in records] == [
        "retryable_failure",
        "permanent_failure",
    ]
