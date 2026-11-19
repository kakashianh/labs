import pytest

from protein3d.metadata.storage import S3ArtifactStorage


@pytest.mark.unit
def test_artifact_storage_builds_stable_s3_uri() -> None:
    storage = S3ArtifactStorage(bucket="protein3d-artifacts", prefix="artifacts")
    assert (
        storage.uri_for("structures/AF-P12345-F1.cif")
        == "s3://protein3d-artifacts/artifacts/structures/AF-P12345-F1.cif"
    )


@pytest.mark.unit
def test_artifact_storage_rejects_path_traversal() -> None:
    storage = S3ArtifactStorage(bucket="protein3d-artifacts")
    with pytest.raises(ValueError):
        storage.uri_for("../private/secret.pdb")
