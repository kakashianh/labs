"""Validated environment-backed application settings."""

# fmt: off

from functools import lru_cache
from typing import Literal
from urllib.parse import quote

from pydantic import AliasChoices, Field, SecretStr, model_validator
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        env_prefix="PROTEIN3D_",
        extra="ignore",
        case_sensitive=False,
        populate_by_name=True,
    )

    app_env: Literal["local", "test", "staging", "production"] = "local"
    log_level: str = "INFO"
    database_host: str = "127.0.0.1"
    database_port: int = Field(
        default=5432,
        ge=1,
        le=65535,
        validation_alias=AliasChoices("PROTEIN3D_DATABASE_PORT", "POSTGRES_PORT"),
    )
    database_name: str = Field(
        default="protein3d",
        validation_alias=AliasChoices("PROTEIN3D_DATABASE_NAME", "POSTGRES_DB"),
    )
    database_user: str = Field(
        default="protein3d",
        validation_alias=AliasChoices("PROTEIN3D_DATABASE_USER", "POSTGRES_USER"),
    )
    database_password: SecretStr = Field(
        default=SecretStr("change_me_local_postgres"),
        validation_alias=AliasChoices(
            "PROTEIN3D_DATABASE_PASSWORD", "POSTGRES_PASSWORD"
        ),
    )
    database_pool_size: int = Field(default=5, ge=1, le=50)
    database_max_overflow: int = Field(default=10, ge=0, le=100)
    database_connect_timeout_seconds: int = Field(default=5, ge=1, le=120)

    object_storage_endpoint: str = "127.0.0.1:9000"
    object_storage_bucket: str = "protein3d-artifacts"
    object_storage_access_key: str = Field(
        default="protein3d",
        validation_alias=AliasChoices(
            "PROTEIN3D_OBJECT_STORAGE_ACCESS_KEY", "MINIO_ROOT_USER"
        ),
    )
    object_storage_secret_key: SecretStr = Field(
        default=SecretStr("change_me_local_minio"),
        validation_alias=AliasChoices(
            "PROTEIN3D_OBJECT_STORAGE_SECRET_KEY", "MINIO_ROOT_PASSWORD"
        ),
    )
    object_storage_secure: bool = False
    object_storage_prefix: str = "artifacts"

    graph_enabled: bool = False
    graph_endpoint: str | None = None
    vector_enabled: bool = False
    vector_endpoint: str | None = None

    external_request_timeout_seconds: float = Field(default=20.0, gt=0, le=300)
    external_max_retries: int = Field(default=3, ge=0, le=10)
    alphafold_api_base_url: str = "https://alphafold.ebi.ac.uk/api"
    alphafold_taxonomy_id: int = Field(default=9606, ge=1)
    alphafold_sample_limit: int = Field(default=10, ge=1, le=10_000)
    alphafold_max_coordinate_bytes: int = Field(default=100_000_000, ge=1024)
    processing_batch_size: int = Field(default=500, ge=1, le=100_000)
    processing_concurrency: int = Field(default=2, ge=1, le=64)

    @model_validator(mode="after")
    def validate_optional_services(self) -> "Settings":
        if self.graph_enabled and not self.graph_endpoint:
            raise ValueError(
                "PROTEIN3D_GRAPH_ENDPOINT is required when graph is enabled"
            )
        if self.vector_enabled and not self.vector_endpoint:
            raise ValueError(
                "PROTEIN3D_VECTOR_ENDPOINT is required when vector is enabled"
            )
        return self

    @property
    def database_url(self) -> str:
        username = quote(self.database_user, safe="")
        password = quote(self.database_password.get_secret_value(), safe="")
        return (
            f"postgresql+psycopg://{username}:{password}@"
            f"{self.database_host}:{self.database_port}/{self.database_name}"
            f"?connect_timeout={self.database_connect_timeout_seconds}"
        )

    def sanitized(self) -> dict[str, object]:
        """Return diagnostics without credential values."""
        return {
            "app_env": self.app_env,
            "log_level": self.log_level,
            "database_host": self.database_host,
            "database_port": self.database_port,
            "database_name": self.database_name,
            "database_user": self.database_user,
            "database_password": "***",
            "object_storage_endpoint": self.object_storage_endpoint,
            "object_storage_bucket": self.object_storage_bucket,
            "object_storage_access_key": self.object_storage_access_key,
            "object_storage_secret_key": "***",
            "object_storage_secure": self.object_storage_secure,
            "graph_enabled": self.graph_enabled,
            "vector_enabled": self.vector_enabled,
            "alphafold_api_base_url": self.alphafold_api_base_url,
            "alphafold_taxonomy_id": self.alphafold_taxonomy_id,
            "alphafold_sample_limit": self.alphafold_sample_limit,
            "alphafold_max_coordinate_bytes": self.alphafold_max_coordinate_bytes,
            "processing_batch_size": self.processing_batch_size,
            "processing_concurrency": self.processing_concurrency,
        }


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    return Settings()
# fmt: on
