"""Unit tests for the AlphaFold API adapter and manifest contract."""

import json
from email.message import Message
from io import BytesIO
from unittest.mock import patch
from urllib.error import HTTPError

import pytest

from protein3d.ingestion.alphafold import (
    AlphaFoldClient,
    AlphaFoldContractError,
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
