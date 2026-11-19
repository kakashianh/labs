import pytest
from pydantic import SecretStr, ValidationError

from protein3d.config import Settings


@pytest.mark.unit
def test_settings_defaults_keep_optional_services_disabled() -> None:
    settings = Settings()
    assert settings.graph_enabled is False
    assert settings.vector_enabled is False
    assert settings.database_port == 5432


@pytest.mark.unit
def test_enabled_optional_service_requires_endpoint() -> None:
    with pytest.raises(ValidationError):
        Settings(graph_enabled=True)


@pytest.mark.unit
def test_sanitized_settings_redact_credentials() -> None:
    settings = Settings(
        database_password=SecretStr("super-secret"),
        object_storage_secret_key=SecretStr("minio-secret"),
    )
    sanitized = settings.sanitized()
    assert sanitized["database_password"] == "***"
    assert sanitized["object_storage_secret_key"] == "***"
    assert "super-secret" not in str(sanitized)
    assert "minio-secret" not in str(sanitized)


@pytest.mark.unit
def test_invalid_database_port_is_rejected() -> None:
    with pytest.raises(ValidationError):
        Settings(database_port=70000)
