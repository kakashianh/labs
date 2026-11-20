"""Live MinIO integration test for bucket creation, upload, and readback."""

import hashlib
import socket
import time
from pathlib import Path
from urllib.error import URLError
from urllib.request import urlopen

import pytest
from testcontainers.core.container import DockerContainer  # type: ignore[import-untyped]

from protein3d.ingestion.minio_storage import MinioObjectStore


@pytest.mark.integration
def test_minio_bucket_upload_head_and_streaming_readback(tmp_path: Path) -> None:
    username = "protein3dtest"
    password = "protein3d-test-secret"
    image = "soldevelo/minio:2026.7.17"
    container = (
        DockerContainer(image)
        .with_exposed_ports(9000)
        .with_env("MINIO_ROOT_USER", username)
        .with_env("MINIO_ROOT_PASSWORD", password)
    )
    with container as minio:
        host = minio.get_container_host_ip()
        port = int(minio.get_exposed_port(9000))

        deadline = time.monotonic() + 60
        health_url = f"http://{host}:{port}/minio/health/ready"
        while True:
            try:
                with urlopen(health_url, timeout=2) as response:
                    if response.status == 200:
                        break
            except (OSError, URLError):
                if time.monotonic() >= deadline:
                    raise TimeoutError(
                        "MinIO did not become ready within 60 seconds"
                    ) from None
                time.sleep(1)

        store = MinioObjectStore(
            endpoint=f"{host}:{port}",
            bucket="protein3d-test",
            access_key=username,
            secret_key=password,
            timeout_seconds=10,
        )
        store.ensure_bucket()

        payload = b"data_AF-Q9Y261-F1\n#\n" + b"ATOM " * 10000
        source = tmp_path / "model.cif"
        source.write_bytes(payload)
        checksum = hashlib.sha256(payload).hexdigest()
        key = "artifacts/alphafold/test/model.cif"
        store.put_file(key, source, "chemical/x-cif", checksum)

        assert store.head_object(key) == len(payload)
        readback = tmp_path / "readback.cif"
        readback_size, readback_checksum = store.get_file(key, readback)
        assert readback_size == len(payload)
        assert readback_checksum == checksum
        assert readback.read_bytes() == payload
