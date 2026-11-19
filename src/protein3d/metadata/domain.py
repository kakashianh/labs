"""Domain guards for identity and lifecycle invariants."""
from protein3d.metadata.models import MappingStatus

_DATASET_TRANSITIONS = {"pending": {"running", "cancelled"}, "running": {"completed", "failed", "cancelled"}, "completed": set(), "failed": set(), "cancelled": set()}
_JOB_TRANSITIONS = {"queued": {"running", "cancelled"}, "running": {"succeeded", "failed", "cancelled"}, "succeeded": set(), "failed": {"queued"}, "cancelled": set()}

def validate_mapping_target(status: str, has_protein_id: bool) -> None:
    if status not in {item.value for item in MappingStatus}:
        raise ValueError(f"Unsupported protein mapping status: {status}")
    if status in {"resolved", "merged"} and not has_protein_id:
        raise ValueError("Resolved and merged mappings require an internal protein UUID")
    if status in {"unresolved", "ambiguous"} and has_protein_id:
        raise ValueError("Unresolved and ambiguous mappings cannot reference an internal protein UUID")

def validate_dataset_transition(current: str, target: str) -> None:
    if target not in _DATASET_TRANSITIONS.get(current, set()):
        raise ValueError(f"Invalid dataset run transition: {current} -> {target}")

def validate_job_transition(current: str, target: str) -> None:
    if target not in _JOB_TRANSITIONS.get(current, set()):
        raise ValueError(f"Invalid job transition: {current} -> {target}")

def validate_artifact_metadata(size_bytes: int | None, checksum: str | None, algorithm: str | None) -> None:
    if size_bytes is not None and size_bytes < 0:
        raise ValueError("Artifact size_bytes cannot be negative")
    if (checksum is None) != (algorithm is None):
        raise ValueError("Checksum and checksum_algorithm must be provided together")
