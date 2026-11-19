import pytest
from protein3d.metadata.domain import (
    validate_artifact_metadata,
    validate_dataset_transition,
    validate_job_transition,
    validate_mapping_target,
)

@pytest.mark.unit
def test_mapping_status_requires_internal_target_only_when_resolved() -> None:
    validate_mapping_target("resolved", True)
    validate_mapping_target("merged", True)
    validate_mapping_target("unresolved", False)
    validate_mapping_target("ambiguous", False)
    with pytest.raises(ValueError):
        validate_mapping_target("unresolved", True)

@pytest.mark.unit
def test_dataset_lifecycle_transitions_are_guarded() -> None:
    validate_dataset_transition("pending", "running")
    validate_dataset_transition("running", "completed")
    with pytest.raises(ValueError):
        validate_dataset_transition("completed", "running")

@pytest.mark.unit
def test_job_retry_transition_is_explicit() -> None:
    validate_job_transition("failed", "queued")
    with pytest.raises(ValueError):
        validate_job_transition("succeeded", "queued")

@pytest.mark.unit
def test_artifact_metadata_validation() -> None:
    validate_artifact_metadata(1024, "abcd", "sha256")
    with pytest.raises(ValueError):
        validate_artifact_metadata(-1, None, None)
    with pytest.raises(ValueError):
        validate_artifact_metadata(1, "abcd", None)
