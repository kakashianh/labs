"""Unit tests for streaming MinIO requests and object-key validation."""

from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from protein3d.ingestion.minio_storage import MinioObjectStore


@pytest.mark.unit
def test_minio_put_file_streams_and_signs_request(tmp_path: Path) -> None:
    payload = b"data_test\n# mmCIF sample\n"
    source = tmp_path / "sample.cif"
    source.write_bytes(payload)
    connection = MagicMock()
    response = MagicMock()
    response.status = 200
    connection.getresponse.return_value = response
    with patch(
        "protein3d.ingestion.minio_storage.http.client.HTTPConnection",
        return_value=connection,
    ):
        MinioObjectStore(
            endpoint="localhost:9000",
            bucket="protein3d-artifacts",
            access_key="test-user",
            secret_key="test-secret",
        ).put_file(
            "artifacts/alphafold/uuid/model.cif",
            source,
            "chemical/x-cif",
            "b2d8e5e5",
        )
    connection.putrequest.assert_called_once_with(
        "PUT",
        "/protein3d-artifacts/artifacts/alphafold/uuid/model.cif",
        skip_accept_encoding=True,
    )
    assert any(call.args == (payload,) for call in connection.send.call_args_list)
    headers = {call.args[0]: call.args[1] for call in connection.putheader.call_args_list}
    assert headers["Authorization"].startswith("AWS4-HMAC-SHA256")
    assert headers["Content-Length"] == str(len(payload))
    connection.close.assert_called_once()


@pytest.mark.unit
@pytest.mark.parametrize(
    "key", ["/absolute.cif", "../escape.cif", "a//b.cif", "a/./b.cif"]
)
def test_minio_rejects_unsafe_object_key(key: str) -> None:
    store = MinioObjectStore(
        endpoint="localhost:9000",
        bucket="protein3d-artifacts",
        access_key="user",
        secret_key="secret",
    )
    with pytest.raises(ValueError):
        store._object_path(key)


@pytest.mark.unit
def test_minio_rejects_empty_file(tmp_path: Path) -> None:
    source = tmp_path / "empty.cif"
    source.write_bytes(b"")
    store = MinioObjectStore(
        endpoint="localhost:9000",
        bucket="protein3d-artifacts",
        access_key="user",
        secret_key="secret",
    )
    with pytest.raises(ValueError):
        store.put_file("empty.cif", source, "chemical/x-cif", "e3b0c442")


@pytest.mark.unit
def test_signed_head_object_request_returns_content_length() -> None:
    connection = MagicMock()
    response = MagicMock()
    response.status = 200
    response.getheader.return_value = "1234"
    connection.getresponse.return_value = response
    with patch(
        "protein3d.ingestion.minio_storage.http.client.HTTPConnection",
        return_value=connection,
    ):
        size = MinioObjectStore(
            endpoint="localhost:9000",
            bucket="protein3d-artifacts",
            access_key="user",
            secret_key="secret",
        ).head_object("artifacts/model.cif")
    assert size == 1234
    connection.request.assert_called_once()
    assert connection.request.call_args.args[0] == "HEAD"
