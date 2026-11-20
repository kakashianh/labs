"""Retry and transport-failure tests for the AlphaFold API client."""

import json
from email.message import Message
from io import BytesIO
from unittest.mock import patch
from urllib.error import HTTPError

import pytest

from protein3d.ingestion.alphafold import (
    AlphaFoldClient,
    AlphaFoldError,
    AlphaFoldPermanentError,
    AlphaFoldRateLimit,
)

RECORD = {
    "modelEntityId": "AF-Q9Y261-F1",
    "cifUrl": "https://alphafold.ebi.ac.uk/files/AF-Q9Y261-F1-model_v6.cif",
    "sequenceStart": 1,
    "sequenceEnd": 100,
    "sequence": "A" * 100,
    "latestVersion": 6,
}


@pytest.mark.unit
def test_client_retries_rate_limit_using_retry_after_header() -> None:
    url = "https://alphafold.ebi.ac.uk/api/prediction/Q9Y261"
    headers = Message()
    headers["Retry-After"] = "0"
    rate_limit = HTTPError(url, 429, "Too Many Requests", headers, None)
    response = BytesIO(json.dumps([RECORD]).encode())
    with (
        patch("protein3d.ingestion.alphafold.urlopen", side_effect=[rate_limit, response]),
        patch("protein3d.ingestion.alphafold.time.sleep") as sleep,
    ):
        models = AlphaFoldClient(max_retries=1).get_predictions("Q9Y261")
    assert len(models) == 1
    sleep.assert_called_once_with(0.0)


@pytest.mark.unit
def test_client_retries_transient_server_error() -> None:
    url = "https://alphafold.ebi.ac.uk/api/prediction/Q9Y261"
    server_error = HTTPError(url, 503, "Service Unavailable", Message(), None)
    response = BytesIO(json.dumps([RECORD]).encode())
    with (
        patch("protein3d.ingestion.alphafold.urlopen", side_effect=[server_error, response]),
        patch("protein3d.ingestion.alphafold.time.sleep") as sleep,
    ):
        models = AlphaFoldClient(max_retries=1).get_predictions("Q9Y261")
    assert len(models) == 1
    sleep.assert_called_once_with(1.0)


@pytest.mark.unit
def test_client_classifies_timeout_and_malformed_json() -> None:
    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        side_effect=TimeoutError("request timed out"),
    ):
        with pytest.raises(AlphaFoldError, match="failed after retries"):
            AlphaFoldClient(max_retries=0).get_predictions("Q9Y261")

    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        return_value=BytesIO(b"{not-json"),
    ):
        with pytest.raises(AlphaFoldError, match="malformed JSON"):
            AlphaFoldClient(max_retries=0).get_predictions("Q9Y261")


@pytest.mark.unit
def test_exhausted_rate_limit_is_retryable() -> None:
    url = "https://alphafold.ebi.ac.uk/api/prediction/Q9Y261"
    error = HTTPError(url, 429, "Too Many Requests", Message(), None)
    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        side_effect=error,
    ):
        with pytest.raises(AlphaFoldRateLimit):
            AlphaFoldClient(max_retries=0).get_predictions("Q9Y261")


@pytest.mark.unit
def test_permanent_client_http_error_is_not_retried() -> None:
    url = "https://alphafold.ebi.ac.uk/api/prediction/Q9Y261"
    error = HTTPError(url, 401, "Unauthorized", Message(), None)
    with patch(
        "protein3d.ingestion.alphafold.urlopen",
        side_effect=error,
    ) as request:
        with pytest.raises(AlphaFoldPermanentError):
            AlphaFoldClient(max_retries=3).get_predictions("Q9Y261")
    request.assert_called_once()
