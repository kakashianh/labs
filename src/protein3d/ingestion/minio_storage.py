"""Streaming S3-compatible MinIO object operations using AWS Signature V4."""

from __future__ import annotations

import hashlib
import hmac
import http.client
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from urllib.parse import quote, urlsplit


@dataclass(frozen=True, slots=True)
class MinioObjectStore:
    endpoint: str
    bucket: str
    access_key: str
    secret_key: str
    secure: bool = False
    region: str = "us-east-1"
    timeout_seconds: float = 30.0

    def _connection(self) -> tuple[http.client.HTTPConnection, str]:
        raw = self.endpoint if "://" in self.endpoint else f"{'https' if self.secure else 'http'}://{self.endpoint}"
        parsed = urlsplit(raw)
        if not parsed.hostname or parsed.path not in ("", "/"):
            raise ValueError("MinIO endpoint must be a host with optional port")
        host = parsed.netloc
        connection_type = http.client.HTTPSConnection if parsed.scheme == "https" else http.client.HTTPConnection
        return connection_type(host, timeout=self.timeout_seconds), host

    def _signed_headers(
        self, method: str, path: str, host: str, payload_hash: str, content_type: str | None
    ) -> dict[str, str]:
        now = datetime.now(UTC)
        amz_date = now.strftime("%Y%m%dT%H%M%SZ")
        date_stamp = now.strftime("%Y%m%d")
        headers = {
            "host": host,
            "x-amz-content-sha256": payload_hash,
            "x-amz-date": amz_date,
        }
        if content_type:
            headers["content-type"] = content_type
        signed_names = sorted(headers)
        canonical_headers = "".join(f"{name}:{headers[name]}\n" for name in signed_names)
        signed_headers = ";".join(signed_names)
        canonical_request = "\n".join(
            [method, path, "", canonical_headers, signed_headers, payload_hash]
        )
        scope = f"{date_stamp}/{self.region}/s3/aws4_request"
        string_to_sign = "\n".join(
            [
                "AWS4-HMAC-SHA256",
                amz_date,
                scope,
                hashlib.sha256(canonical_request.encode()).hexdigest(),
            ]
        )
        date_key = _sign(("AWS4" + self.secret_key).encode(), date_stamp)
        region_key = _sign(date_key, self.region)
        service_key = _sign(region_key, "s3")
        signing_key = _sign(service_key, "aws4_request")
        signature = hmac.new(signing_key, string_to_sign.encode(), hashlib.sha256).hexdigest()
        result = {
            "Host": host,
            "x-amz-content-sha256": payload_hash,
            "x-amz-date": amz_date,
            "Authorization": (
                f"AWS4-HMAC-SHA256 Credential={self.access_key}/{scope}, "
                f"SignedHeaders={signed_headers}, Signature={signature}"
            ),
        }
        if content_type:
            result["Content-Type"] = content_type
        return result

    def _object_path(self, object_key: str) -> str:
        if object_key.startswith("/") or any(
            part in {"", ".", ".."} for part in object_key.split("/")
        ):
            raise ValueError("Object key must be a safe relative path")
        return f"/{quote(self.bucket, safe='')}/{quote(object_key, safe='/')}"

    def ensure_bucket(self) -> None:
        """Create the bucket only when a signed HEAD confirms it is absent."""
        connection, host = self._connection()
        path = f"/{quote(self.bucket, safe='')}"
        empty_hash = hashlib.sha256(b"").hexdigest()
        try:
            connection.request(
                "HEAD", path, headers=self._signed_headers("HEAD", path, host, empty_hash, None)
            )
            response = connection.getresponse()
            response.read()
            if response.status in (200, 204):
                return
            if response.status != 404:
                raise RuntimeError(f"MinIO bucket check returned HTTP {response.status}")
            connection.close()
            connection, host = self._connection()
            connection.request(
                "PUT", path, body=b"",
                headers=self._signed_headers("PUT", path, host, empty_hash, None),
            )
            response = connection.getresponse()
            response.read()
            if response.status not in (200, 201, 204):
                raise RuntimeError(f"MinIO bucket creation returned HTTP {response.status}")
        finally:
            connection.close()

    def put_file(
        self,
        object_key: str,
        file_path: Path,
        content_type: str,
        sha256: str,
        chunk_size: int = 1024 * 1024,
    ) -> None:
        """Upload a file in bounded chunks without loading it into memory."""
        size = file_path.stat().st_size
        if size <= 0:
            raise ValueError("Cannot upload an empty object")
        path = self._object_path(object_key)
        connection, host = self._connection()
        try:
            headers = self._signed_headers(
                "PUT", path, host, sha256, content_type
            )
            headers["Content-Length"] = str(size)
            connection.putrequest("PUT", path, skip_accept_encoding=True)
            for name, value in headers.items():
                connection.putheader(name, value)
            connection.endheaders()
            with file_path.open("rb") as source:
                while chunk := source.read(chunk_size):
                    connection.send(chunk)
            response = connection.getresponse()
            response.read()
            if response.status not in (200, 201, 204):
                raise RuntimeError(f"MinIO object upload returned HTTP {response.status}")
        finally:
            connection.close()

    def head_object(self, object_key: str) -> int:
        """Return stored object size or raise when the object is unavailable."""
        path = self._object_path(object_key)
        connection, host = self._connection()
        try:
            empty_hash = hashlib.sha256(b"").hexdigest()
            connection.request(
                "HEAD",
                path,
                headers=self._signed_headers("HEAD", path, host, empty_hash, None),
            )
            response = connection.getresponse()
            response.read()
            if response.status not in (200, 204):
                raise RuntimeError(f"MinIO object HEAD returned HTTP {response.status}")
            return int(response.getheader("Content-Length", "0"))
        finally:
            connection.close()


    def get_file(self, object_key: str, destination: Path) -> tuple[int, str]:
        """Stream an object back to disk and return its size and SHA-256."""
        path = self._object_path(object_key)
        connection, host = self._connection()
        digest = hashlib.sha256()
        size = 0
        try:
            connection.request(
                "GET",
                path,
                headers=self._signed_headers(
                    "GET", path, host, hashlib.sha256(b"").hexdigest(), None
                ),
            )
            response = connection.getresponse()
            if response.status != 200:
                response.read()
                raise RuntimeError(f"MinIO object GET returned HTTP {response.status}")
            destination.parent.mkdir(parents=True, exist_ok=True)
            with destination.open("wb") as output:
                while chunk := response.read(1024 * 1024):
                    output.write(chunk)
                    digest.update(chunk)
                    size += len(chunk)
            return size, digest.hexdigest()
        finally:
            connection.close()


def _sign(key: bytes, value: str) -> bytes:
    return hmac.new(key, value.encode(), hashlib.sha256).digest()
