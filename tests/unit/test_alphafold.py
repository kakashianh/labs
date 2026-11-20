"""Unit tests for the AlphaFold API adapter and manifest contract."""

import hashlib
import json
from email.message import Message
from io import BytesIO
from pathlib import Path
from unittest.mock import patch
from urllib.error import HTTPError, URLError

import pytest

from protein3d.ingestion.alphafold import (
    AlphaFoldClient,
    AlphaFoldContractError,
    AlphaFoldError,
    AlphaFoldNotFound,
    _retry_delay,
    manifest_record,
    parse_prediction,
)

RECORD = {
    "modelEntityId": "AF-Q9Y261-F1",
    "cifUrl": "https://alphafold.ebi.ac.uk/files/AF-Q9Y261-F1-model_v6.cif",
    "sequenceStart": 1,
    "sequenceEnd": 100,
    "sequence": "A" * 100,
    "latestVersion": 6,
    "isUniProtReviewed": True,
    "isUniProtReferenceProteome": False,
    "globalMetricValue": 92.4,
}


@pytest.mark.unit
def test_parse_current_afdb_fields_without_legacy_aliases() -> None:
    models = parse_prediction("Q9Y261", [RECORD])
    assert len(models) == 1
    model = models[0]
    assert model.model_entity_id == "AF-Q9Y261-F1"
    assert model.sequence_start == 1
    assert model.sequence_end == 100
    assert model.sequence_length == 100
    assert model.model_version == "6"
    assert model.is_uniprot_reviewed is True
    assert "globalMetricValue" in model.confidence_metadata


@pytest.mark.unit
def test_parse_preserves_distinct_isoform_model_records() -> None:
    second = {**RECORD, "modelEntityId": "AF-Q9Y261-2-F1"}
    models = parse_prediction("Q9Y261", [RECORD, second])
    assert [item.model_entity_id for item in models] == [
        "AF-Q9Y261-F1",
        "AF-Q9Y261-2-F1",
    ]


@pytest.mark.unit
@pytest.mark.parametrize("payload", [None, {}, ["invalid"]])
def test_parse_rejects_invalid_payload(payload: object) -> None:
    with pytest.raises(AlphaFoldContractError):
        parse_prediction("Q9Y261", payload)


@pytest.mark.unit
def test_parse_rejects_non_https_coordinate_url() -> None:
    with pytest.raises(AlphaFoldContractError):
        parse_prediction("Q9Y261", [{**RECORD, "cifUrl": "http://example.com/model.cif"}])


@pytest.mark.unit
def test_parse_does_not_depend_on_removed_legacy_fields() -> None:
    legacy_only = {
        "entryId": "AF-Q9Y261-F1",
        "paeImageUrl": "https://example.com/pae.png",
    }
    with pytest.raises(AlphaFoldContractError):
        parse_prediction("Q9Y261", [legacy_only])


@pytest.mark.unit
def test_client_encodes_accession_and_parses_response() -> None:
    response = BytesIO(json.dumps([RECORD]).encode())
    with patch("protein3d.ingestion.alphafold.urlopen", return_value=response) as mocked:
        result = AlphaFoldClient(max_retries=0).get_predictions("Q9Y261")
    assert result[0].accession == "Q9Y261"
    assert "Q9Y261" in mocked.call_args.args[0].full_url


@pytest.mark.unit
def test_client_raises_not_found_without_retry() -> None:
    error = HTTPError(
        "https://alphafold.ebi.ac.uk/api/prediction/Q9Y261",
        404,
        "Not Found",
        Message(),
        None,
    )
    with patch("protein3d.ingestion.alphafold.urlopen", side_effect=error):
        with pytest.raises(AlphaFoldNotFound):
            AlphaFoldClient(max_retries=0).get_predictions("Q9Y261")


@pytest.mark.unit
def test_accession_validation_rejects_path_injection() -> None:
    with pytest.raises(ValueError):
        AlphaFoldClient().get_predictions("../Q9Y261")


@pytest.mark.unit
def test_manifest_keeps_internal_uuid_and_external_accession_separate() -> None:
    model = parse_prediction("Q9Y261", [RECORD])[0]
    result = manifest_record(
        "7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81",
        model,
        None,
        "discovered",
    )
    assert result["protein_id"] == "7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81"
    assert result["accession"] == "Q9Y261"
    assert result["model_entity_id"] == "AF-Q9Y261-F1"


@pytest.mark.unit
@pytest.mark.parametrize(
    ("header", "attempt", "expected"),
    [("5", 0, 5.0), ("invalid", 2, 4.0), (None, 1, 2.0)],
)
def test_retry_delay_is_bounded(header: str | None, attempt: int, expected: float) -> None:
    assert _retry_delay(header, attempt) == expected


class FakeCifResponse(BytesIO):
    headers = {"Content-Type": "chemical/x-cif; charset=utf-8"}

    def __enter__(self) -> "FakeCifResponse":
        return self

    def __exit__(self, *args: object) -> None:
        self.close()


@pytest.mark.unit
def test_download_cif_streams_to_path_and_calculates_checksum(tmp_path: Path) -> None:
    payload = b"data_AF-Q9Y261-F1\n#\n"
    model = parse_prediction("Q9Y261", [RECORD])[0]
    destination = tmp_path / "model.cif"
    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        return_value=FakeCifResponse(payload),
    ):
        result = AlphaFoldClient(max_retries=0).download_cif_to_path(
            model,
            "7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81",
            destination,
            max_bytes=1024,
            chunk_size=4,
        )
    assert destination.read_bytes() == payload
    assert result.size_bytes == len(payload)
    assert result.sha256 == hashlib.sha256(payload).hexdigest()
    assert result.content_type == "chemical/x-cif"
    assert result.object_key.endswith("/6/model.cif")


@pytest.mark.unit
def test_download_cif_removes_partial_file_when_size_limit_exceeded(
    tmp_path: Path,
) -> None:
    model = parse_prediction("Q9Y261", [RECORD])[0]
    destination = tmp_path / "too-large.cif"
    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        return_value=FakeCifResponse(b"data_example\n" + b"x" * 100),
    ):
        with pytest.raises(AlphaFoldError, match="size limit"):
            AlphaFoldClient(max_retries=0).download_cif_to_path(
                model,
                "7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81",
                destination,
                max_bytes=20,
                chunk_size=8,
            )
    assert not destination.exists()


@pytest.mark.unit
def test_retry_delay_accepts_http_date() -> None:
    assert _retry_delay("Wed, 21 Oct 2015 07:28:00 GMT", 0) == 0.0


@pytest.mark.unit
def test_download_cif_rejects_html_content_type(tmp_path: Path) -> None:
    response = FakeCifResponse(b"data_example\n#\n")
    response.headers = {"Content-Type": "text/html"}
    destination = tmp_path / "html.cif"
    model = parse_prediction("Q9Y261", [RECORD])[0]
    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        return_value=response,
    ):
        with pytest.raises(AlphaFoldContractError, match="content type"):
            AlphaFoldClient(max_retries=0).download_cif_to_path(
                model,
                "7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81",
                destination,
            )
    assert not destination.exists()


@pytest.mark.unit
def test_download_cif_rejects_non_cif_payload(tmp_path: Path) -> None:
    destination = tmp_path / "corrupt.cif"
    model = parse_prediction("Q9Y261", [RECORD])[0]
    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        return_value=FakeCifResponse(b"<html>not a CIF file</html>"),
    ):
        with pytest.raises(AlphaFoldContractError, match="mmCIF"):
            AlphaFoldClient(max_retries=0).download_cif_to_path(
                model,
                "7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81",
                destination,
            )
    assert not destination.exists()



@pytest.mark.unit
def test_client_classifies_connection_failure() -> None:
    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        side_effect=URLError("connection refused"),
    ):
        with pytest.raises(AlphaFoldError, match="failed after retries"):
            AlphaFoldClient(max_retries=0).get_predictions("Q9Y261")


@pytest.mark.unit
def test_download_cif_rejects_empty_coordinate_response(tmp_path: Path) -> None:
    destination = tmp_path / "empty.cif"
    model = parse_prediction("Q9Y261", [RECORD])[0]
    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        return_value=FakeCifResponse(b""),
    ):
        with pytest.raises(AlphaFoldContractError, match="mmCIF"):
            AlphaFoldClient(max_retries=0).download_cif_to_path(
                model,
                "7e5c2f48-1f1a-4b22-9b13-2d7e43d66a81",
                destination,
            )
    assert not destination.exists()
