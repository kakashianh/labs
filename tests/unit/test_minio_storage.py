"""Unit tests for MinIO upload signing and path validation."""

from unittest.mock import MagicMock, patch

import pytest

from protein3d.ingestion.minio_storage import MinioObjectStore


@pytest.mark.unit
def test_minio_put_uses_signed_request() -> None:
    response = MagicMock()
    response.status = 200
    response.__enter__.return_value = response
    with patch("protein3d.ingestion.minio_storage.urlopen", return_value=response) as mocked:
        MinioObjectStore(
            endpoint="localhost:9000",
            bucket="protein3d-artifacts",
            access_key="test-user",
            secret_key="test-secret",
        ).put_object("alphafold/uuid/model.cif", b"data_test", "chemical/x-cif")
    request = mocked.call_args.args[0]
    assert request.method == "PUT"
    assert request.headers["Authorization"].startswith("AWS4-HMAC-SHA256")
    assert request.headers["X-amz-content-sha256"]


@pytest.mark.unit
@pytest.mark.parametrize("key", ["/absolute.cif", "../escape.cif", "a//b.cif", "a/./b.cif"])
def test_minio_rejects_unsafe_object_key(key: str) -> None:
    with pytest.raises(ValueError):
        MinioObjectStore(
            endpoint="localhost:9000",
            bucket="protein3d-artifacts",
            access_key="user",
            secret_key="secret",
        ).put_object(key, b"data_test", "chemical/x-cif")


@pytest.mark.unit
def test_minio_rejects_empty_payload() -> None:
    with pytest.raises(ValueError):
        MinioObjectStore(
            endpoint="localhost:9000",
            bucket="protein3d-artifacts",
            access_key="user",
            secret_key="secret",
        ).put_object("a.cif", b"", "chemical/x-cif")
