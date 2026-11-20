"""AlphaFold Database API client and safe coordinate downloader.

The adapter intentionally consumes current AFDB field names and keeps source
identifiers separate from the project's internal protein UUID.
"""

from __future__ import annotations

import hashlib
import json
import logging
import re
import time
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path, PurePosixPath
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlparse
from urllib.request import Request, urlopen

LOGGER = logging.getLogger(__name__)
AFDB_API_BASE = "https://alphafold.ebi.ac.uk/api"
AFDB_SOURCE = "alphafold_db"
USER_AGENT = "protein3d-ppi/0.1 (+https://github.com/kakashianh/labs)"


class AlphaFoldError(Exception):
    """Base class for AlphaFold retrieval errors."""


class AlphaFoldNotFound(AlphaFoldError):
    """The API confirmed that no prediction exists for the accession."""


class AlphaFoldRateLimit(AlphaFoldError):
    """The API rate-limited the request after retry attempts."""


class AlphaFoldContractError(AlphaFoldError):
    """The upstream response does not match the supported API contract."""


@dataclass(frozen=True, slots=True)
class AlphaFoldModel:
    accession: str
    model_entity_id: str
    sequence_start: int | None
    sequence_end: int | None
    sequence_length: int | None
    cif_url: str
    model_version: str | None
    is_uniprot_reviewed: bool | None
    is_uniprot_reference_proteome: bool | None
    confidence_metadata: dict[str, Any]
    source_metadata: dict[str, Any]


@dataclass(frozen=True, slots=True)
class DownloadedCoordinate:
    source_url: str
    retrieved_at: str
    object_key: str
    content_type: str
    size_bytes: int
    sha256: str
    format: str = "mmcif"


def _optional_int(value: Any) -> int | None:
    return value if isinstance(value, int) and not isinstance(value, bool) else None


def _optional_bool(value: Any) -> bool | None:
    return value if isinstance(value, bool) else None


def parse_prediction(accession: str, payload: Any) -> list[AlphaFoldModel]:
    """Parse current AFDB prediction records; preserve distinct model entities."""
    if not isinstance(payload, list):
        raise AlphaFoldContractError("AFDB prediction response must be a JSON list")
    models: list[AlphaFoldModel] = []
    for item in payload:
        if not isinstance(item, dict):
            raise AlphaFoldContractError("AFDB prediction record must be an object")
        model_id = item.get("modelEntityId")
        cif_url = item.get("cifUrl")
        if not isinstance(model_id, str) or not model_id.strip():
            raise AlphaFoldContractError("AFDB record is missing modelEntityId")
        if not isinstance(cif_url, str) or not _is_safe_https_url(cif_url):
            raise AlphaFoldContractError("AFDB record has no valid HTTPS cifUrl")
        sequence = item.get("sequence")
        confidence = {
            key: item[key]
            for key in ("globalMetricValue", "globalMetricName", "confidenceCategory")
            if key in item
        }
        models.append(
            AlphaFoldModel(
                accession=accession,
                model_entity_id=model_id,
                sequence_start=_optional_int(item.get("sequenceStart")),
                sequence_end=_optional_int(item.get("sequenceEnd")),
                sequence_length=len(sequence) if isinstance(sequence, str) else None,
                cif_url=cif_url,
                model_version=(
                    str(item["latestVersion"]) if item.get("latestVersion") is not None else None
                ),
                is_uniprot_reviewed=_optional_bool(item.get("isUniProtReviewed")),
                is_uniprot_reference_proteome=_optional_bool(
                    item.get("isUniProtReferenceProteome")
                ),
                confidence_metadata=confidence,
                source_metadata=item,
            )
        )
    return models


def _is_safe_https_url(value: str) -> bool:
    parsed = urlparse(value)
    return parsed.scheme == "https" and bool(parsed.hostname) and not parsed.username


class AlphaFoldClient:
    """Small stdlib HTTP client with bounded retries and Retry-After support."""

    def __init__(
        self,
        base_url: str = AFDB_API_BASE,
        timeout_seconds: float = 20.0,
        max_retries: int = 3,
        user_agent: str = USER_AGENT,
    ) -> None:
        if timeout_seconds <= 0 or max_retries < 0:
            raise ValueError("timeout must be positive and max_retries non-negative")
        self.base_url = base_url.rstrip("/")
        self.timeout_seconds = timeout_seconds
        self.max_retries = max_retries
        self.user_agent = user_agent

    def get_predictions(self, accession: str) -> list[AlphaFoldModel]:
        if not re.fullmatch(r"[A-Za-z0-9_.-]{1,64}", accession):
            raise ValueError("Invalid UniProt accession")
        url = f"{self.base_url}/prediction/{quote(accession, safe='')}"
        payload = self._get_json(url)
        return parse_prediction(accession, payload)

    def _get_json(self, url: str) -> Any:
        for attempt in range(self.max_retries + 1):
            request = Request(
                url,
                headers={"Accept": "application/json", "User-Agent": self.user_agent},
            )
            try:
                with urlopen(request, timeout=self.timeout_seconds) as response:
                    return json.loads(response.read().decode("utf-8"))
            except HTTPError as exc:
                if exc.code == 404:
                    raise AlphaFoldNotFound(url) from exc
                if exc.code == 429:
                    if attempt >= self.max_retries:
                        raise AlphaFoldRateLimit(url) from exc
                    delay = _retry_delay(exc.headers.get("Retry-After"), attempt)
                elif exc.code >= 500 and attempt < self.max_retries:
                    delay = _retry_delay(None, attempt)
                else:
                    raise AlphaFoldError(f"AFDB HTTP {exc.code}") from exc
            except (URLError, TimeoutError, json.JSONDecodeError) as exc:
                if attempt >= self.max_retries:
                    raise AlphaFoldError("AFDB request failed after retries") from exc
                delay = _retry_delay(None, attempt)
            time.sleep(delay)
        raise AlphaFoldError("AFDB request exhausted retries")

    def download_cif(
        self,
        model: AlphaFoldModel,
        protein_id: str,
        max_bytes: int = 100_000_000,
    ) -> tuple[bytes, DownloadedCoordinate]:
        """Download a CIF payload with size bound and checksum.

        Object-store upload is deliberately performed by the caller so that a
        database row is never marked successful before the upload completes.
        """
        if not _is_safe_https_url(model.cif_url):
            raise AlphaFoldContractError("Coordinate URL must use HTTPS")
        if not re.fullmatch(r"[0-9a-fA-F-]{36}", protein_id):
            raise ValueError("protein_id must be an internal UUID")
        request = Request(model.cif_url, headers={"User-Agent": self.user_agent})
        try:
            with urlopen(request, timeout=self.timeout_seconds) as response:
                content_type = response.headers.get("Content-Type", "application/octet-stream")
                chunks: list[bytes] = []
                size = 0
                while chunk := response.read(1024 * 1024):
                    size += len(chunk)
                    if size > max_bytes:
                        raise AlphaFoldError("Coordinate file exceeds configured size limit")
                    chunks.append(chunk)
        except (HTTPError, URLError, TimeoutError) as exc:
            raise AlphaFoldError("Coordinate download failed") from exc
        body = b"".join(chunks)
        if not body or b"data_" not in body[:4096].lower():
            raise AlphaFoldContractError("Downloaded payload is empty or not recognizable mmCIF")
        checksum = hashlib.sha256(body).hexdigest()
        version = model.model_version or "unversioned"
        safe_model = re.sub(r"[^A-Za-z0-9_.-]", "_", model.model_entity_id)
        object_key = str(PurePosixPath("alphafold", protein_id, safe_model, version, "model.cif"))
        metadata = DownloadedCoordinate(
            source_url=model.cif_url,
            retrieved_at=datetime.now(UTC).isoformat(),
            object_key=object_key,
            content_type=content_type.split(";", maxsplit=1)[0],
            size_bytes=len(body),
            sha256=checksum,
        )
        return body, metadata


    def download_cif_to_path(
        self,
        model: AlphaFoldModel,
        protein_id: str,
        destination: Path,
        max_bytes: int = 100_000_000,
        chunk_size: int = 1024 * 1024,
    ) -> DownloadedCoordinate:
        """Stream a CIF response to disk while enforcing size and hashing bytes."""
        if not _is_safe_https_url(model.cif_url):
            raise AlphaFoldContractError("Coordinate URL must use HTTPS")
        if not re.fullmatch(r"[0-9a-fA-F-]{36}", protein_id):
            raise ValueError("protein_id must be an internal UUID")
        if max_bytes < 1 or chunk_size < 1:
            raise ValueError("max_bytes and chunk_size must be positive")

        request = Request(model.cif_url, headers={"User-Agent": self.user_agent})
        digest = hashlib.sha256()
        size = 0
        prefix = bytearray()
        try:
            with urlopen(request, timeout=self.timeout_seconds) as response:
                content_type = response.headers.get("Content-Type", "application/octet-stream")
                with destination.open("wb") as output:
                    while chunk := response.read(chunk_size):
                        size += len(chunk)
                        if size > max_bytes:
                            raise AlphaFoldError("Coordinate file exceeds configured size limit")
                        if len(prefix) < 4096:
                            prefix.extend(chunk[: 4096 - len(prefix)])
                        digest.update(chunk)
                        output.write(chunk)
        except (HTTPError, URLError, TimeoutError, OSError) as exc:
            destination.unlink(missing_ok=True)
            raise AlphaFoldError("Coordinate download failed") from exc

        if not size or b"data_" not in bytes(prefix).lower():
            destination.unlink(missing_ok=True)
            raise AlphaFoldContractError("Downloaded payload is empty or not recognizable mmCIF")

        version = model.model_version or "unversioned"
        safe_model = re.sub(r"[^A-Za-z0-9_.-]", "_", model.model_entity_id)
        object_key = str(PurePosixPath("alphafold", protein_id, safe_model, version, "model.cif"))
        return DownloadedCoordinate(
            source_url=model.cif_url,
            retrieved_at=datetime.now(UTC).isoformat(),
            object_key=object_key,
            content_type=content_type.split(";", maxsplit=1)[0],
            size_bytes=size,
            sha256=digest.hexdigest(),
        )


def _retry_delay(retry_after: str | None, attempt: int) -> float:
    if retry_after:
        try:
            return min(max(float(retry_after), 0.0), 60.0)
        except ValueError:
            pass
    return min(2.0**attempt, 30.0)


def manifest_record(
    protein_id: str,
    model: AlphaFoldModel,
    coordinate: DownloadedCoordinate | None,
    status: str,
    reason: str | None = None,
) -> dict[str, Any]:
    """Serialize one model outcome for the PP-16 ingestion manifest."""
    return {
        "protein_id": protein_id,
        "source": AFDB_SOURCE,
        "accession": model.accession,
        "model_entity_id": model.model_entity_id,
        "source_version": model.model_version,
        "sequence_start": model.sequence_start,
        "sequence_end": model.sequence_end,
        "sequence_length": model.sequence_length,
        "format": "mmcif",
        "status": status,
        "reason": reason,
        "source_url": model.cif_url,
        "artifact": asdict(coordinate) if coordinate else None,
    }
