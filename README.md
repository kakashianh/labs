# labs

Repository workspace for the user's software and research projects.

## Project tracks

- **3D-map-medical** — 3D urban GIS with spatial epidemic simulation.
- **Protein3D-PPI** — protein 3D structure and protein-protein interaction analysis.

## Protein3D-PPI

A reproducible Python workspace for protein 3D structure and protein-protein interaction (PPI) analysis. The initial repository setup is intentionally minimal; domain modules and scientific dependencies are introduced by their owning project tasks.

### Development environment

#### Supported Python version

The project currently targets **CPython 3.13.x** (`>=3.13,<3.14`). This baseline was selected against the planned scientific stack:

- PyTorch 2.15 supports Python 3.11–3.15.
- Current PySpark 4.3 documentation requires Python 3.11 or newer.
- Biopython 1.88 supports Python 3.10–3.14 (and has tested the 3.15 release candidate).

Python 3.13 is within the documented support ranges of these key libraries, while retaining a longer maintenance window than Python 3.11. Re-check compatibility when selecting exact dependency versions in the relevant pipeline/ML tasks.

#### Prerequisites

- CPython 3.13
- [uv](https://docs.astral.sh/uv/)
- GNU Make (optional; commands can be run directly)

#### First-time setup

```bash
uv sync --locked
```

The `uv.lock` file is committed to keep the project dependency graph reproducible. PP-9 intentionally declares no scientific/runtime dependencies yet; those are added with the domain tasks that own them.

#### Common commands

```bash
make setup
make lint
make format-check
make type-check
make test
make smoke
make check
```

The quality tools are invoked at explicit versions by the Makefile and CI. Unit tests are isolated from infrastructure-dependent integration tests using pytest markers. Integration tests can be expanded by PP-10 and subsequent infrastructure/pipeline tasks.

#### Run the CLI

```bash
uv run --locked protein3d --help
uv run --locked protein3d --version
```

#### Configuration and secrets

Copy `.env.example` to `.env` for local-only settings. Never commit `.env`, credentials, access tokens, or private connection strings. PP-12 owns the canonical typed application settings and PP-11/PP-12 will add infrastructure-specific variables as needed.

#### Data and model artifacts

Do not commit downloaded protein structures, large source datasets, model weights, embeddings, or generated pipeline outputs. Keep small synthetic test fixtures in version control; use the storage locations and lifecycle conventions defined by downstream tasks for larger artifacts.


### Local infrastructure (PP-11)

Docker Compose provides PostgreSQL and MinIO by default. Start optional graph and vector services only when needed:

```bash
cp .env.example .env
make infra-up
make infra-up-graph   # optional JanusGraph + Cassandra + Elasticsearch
make infra-up-vector  # optional Milvus + etcd
make infra-status
make infra-down
```

See [Local infrastructure guide](docs/infrastructure/docker-compose.md) for service profiles, ports, readiness checks, data persistence, troubleshooting, and the explicitly destructive `make infra-reset` command. Never commit the local `.env` file or its credentials.

### Project layout

```text
.
├── .github/workflows/ci.yml
├── configs/
├── data/
├── docs/
├── scripts/
├── src/protein3d/
├── tests/
│   ├── integration/
│   └── unit/
├── .env.example
├── .python-version
├── Makefile
├── pyproject.toml
└── uv.lock
```

### Compatibility references

- [Python version status](https://devguide.python.org/versions/)
- [PyTorch release compatibility matrix](https://github.com/pytorch/pytorch/blob/main/RELEASE.md)
- [PySpark installation and supported Python versions](https://spark.apache.org/docs/latest/api/python/getting_started/install.html)
- [Biopython release notes](https://github.com/biopython/biopython/blob/master/NEWS.rst)
