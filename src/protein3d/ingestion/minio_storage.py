"""Minimal S3-compatible MinIO object writer using AWS Signature V4."""

from __future__ import annotations

import hashlib
import hmac
from dataclasses import dataclass
from datetime import UTC, datetime
from urllib.error import HTTPError, URLError
from urllib.parse import quote
from urllib.request import Request, urlopen


@dataclass(frozen=True, slots=True)
class MinioObjectStore:
    endpoint: str
    bucket: str
    access_key: str
    secret_key: str
    secure: bool = False
    region: str = "us-east-1"
    timeout_seconds: float = 30.0

    def put_object(self, object_key: str, body: bytes, content_type: str) -> None:
        """Upload a bounded payload to MinIO; raises if the PUT is not successful."""
        if not body:
            raise ValueError("Cannot upload an empty object")
        if object_key.startswith("/") or any(part in {"", ".", ".."} for part in object_key.split("/")):
            raise ValueError("Object key must be a safe relative path")
        scheme = "https" if self.secure else "http"
        host = self.endpoint.removeprefix("http://").removeprefix("https://").rstrip("/")
        path = f"/{quote(self.bucket, safe='')}/{quote(object_key, safe='/')}"
        url = f"{scheme}://{host}{path}"
        now = datetime.now(UTC)
        amz_date = now.strftime("%Y%m%dT%H%M%SZ")
        date_stamp = now.strftime("%Y%m%d")
        payload_hash = hashlib.sha256(body).hexdigest()
        canonical_headers = (
            f"host:{host}\n"
            f"x-amz-content-sha256:{payload_hash}\n"
            f"x-amz-date:{amz_date}\n"
            f"content-type:{content_type}\n"
        )
        signed_headers = "content-type;host;x-amz-content-sha256;x-amz-date"
        canonical_request = "\n".join(
            ["PUT", path, "", canonical_headers, signed_headers, payload_hash]
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
        signature = hmac.new(
            signing_key, string_to_sign.encode(), hashlib.sha256
        ).hexdigest()
        authorization = (
            f"AWS4-HMAC-SHA256 Credential={self.access_key}/{scope}, "
            f"SignedHeaders={signed_headers}, Signature={signature}"
        )
        request = Request(
            url,
            data=body,
            method="PUT",
            headers={
                "Authorization": authorization,
                "Content-Type": content_type,
                "Host": host,
                "x-amz-content-sha256": payload_hash,
                "x-amz-date": amz_date,
            },
        )
        try:
            with urlopen(request, timeout=self.timeout_seconds) as response:
                if response.status not in (200, 201, 204):
                    raise RuntimeError(f"MinIO returned HTTP {response.status}")
        except (HTTPError, URLError, TimeoutError) as exc:
            raise RuntimeError("MinIO object upload failed") from exc


def _sign(key: bytes, value: str) -> bytes:
    return hmac.new(key, value.encode(), hashlib.sha256).digest()
