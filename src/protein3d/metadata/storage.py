"""Storage URI contract; object bytes are handled outside PostgreSQL."""
from dataclasses import dataclass
from pathlib import PurePosixPath
from typing import Protocol

class ArtifactStorage(Protocol):
    def uri_for(self, object_key: str) -> str:
        """Return a stable URI for an object key."""

@dataclass(frozen=True, slots=True)
class S3ArtifactStorage:
    bucket: str
    prefix: str = "artifacts"

    def __post_init__(self) -> None:
        if not self.bucket or "/" in self.bucket:
            raise ValueError("S3 bucket must be a non-empty bucket name")
        if self.prefix.startswith("/"):
            raise ValueError("Artifact prefix must be a relative object prefix")

    def uri_for(self, object_key: str) -> str:
        key = PurePosixPath(object_key)
        if not object_key or key.is_absolute() or any(part in {"", ".", ".."} for part in key.parts):
            raise ValueError("Artifact object key must be a safe relative path")
        prefix = self.prefix.strip("/")
        path = f"{prefix}/{key.as_posix()}" if prefix else key.as_posix()
        return f"s3://{self.bucket}/{path}"
