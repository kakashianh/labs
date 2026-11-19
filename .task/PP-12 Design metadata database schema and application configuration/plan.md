# PP-12 — Design metadata database schema and application configuration

## 1. Objective

Implement the PostgreSQL metadata persistence foundation and typed application configuration for protein identity, external identifier mappings, structure-chain associations, dataset/job lifecycle, and artifact metadata.

This task establishes stable contracts for downstream ingestion, processing, search, and prediction tasks. It does not implement those downstream pipelines or APIs.

## 2. Scope and design context

**Upstream dependencies**
- PP-10 defines architecture, ownership, and data contracts.
- PP-11 provides local PostgreSQL and MinIO services through Docker Compose.

**Related plans:** [PP-10](../PP-10%20Design%20project%20architecture%20and%20module%20structure/plan.md) · [PP-11](../PP-11%20Configure%20Docker%20Compose%20infrastructure/plan.md) · [PP-13](../PP-13%20Implement%20AlphaFold%20structure%20data%20crawler/plan.md) · [PP-16](../PP-16%20Implement%20ingestion%20validation%20and%20raw%20data%20management/plan.md) · [PP-17](../PP-17%20Build%20Spark%20ingestion%20pipeline/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-39](../PP-39%20Implement%20protein%20search%20and%20detail%20REST%20APIs/plan.md) · [PP-42](../PP-42%20Integrate%20prediction%20results%20and%20perform%20end-to-end%20testing/plan.md).

### 2.1 Ownership boundaries

PP-12 owns:
- PostgreSQL metadata schema and migration history.
- Protein master identity and external identifier mapping.
- Structure metadata and protein-chain associations.
- Dataset run and processing job metadata.
- Artifact metadata and storage URI conventions.
- Typed application settings for configured services.

Out of scope:
- JanusGraph PPI graph schema and graph operations.
- Milvus collection, index, and vector lifecycle.
- Spark ingestion/feature pipelines and crawler implementation.
- Model training/inference implementation and full REST API endpoints.
- Production secrets management or deployment orchestration.

### 2.2 Technology decisions

- PostgreSQL is the source of truth for relational metadata.
- Use SQLAlchemy 2.x for persistence models and Alembic for versioned migrations.
- Use Pydantic Settings for typed environment-backed configuration.
- Use UUID primary keys for project-owned internal entities.
- Store large structure files, Parquet datasets, model files, and embeddings outside PostgreSQL. PostgreSQL stores metadata and references only.
- MinIO is the local object-storage target supplied by PP-11. Keep storage backend and URI explicit so another backend can be configured later.

## 3. Step-by-step implementation and to-do

### Step 1 — Define metadata domain model and identity rules

Define entities, relationships, ownership, and lifecycle semantics before implementing persistence.

**Required entities**
- `proteins`: project-owned canonical protein records.
- `protein_id_mappings`: external identifiers and their mapping/resolution state.
- `structures`: source structure metadata and object location.
- `structure_protein_chains`: association between a structure, a protein, and its chain identifier.
- `dataset_runs`: source/dataset version, processing lifecycle, and provenance.
- `jobs`: individual processing task lifecycle and execution metadata.
- `artifacts`: metadata for files produced or consumed by a dataset/job.

PPI edges remain owned by JanusGraph; embeddings remain owned by Milvus. Do not create duplicate relational tables for either.

**Identity contract**
- `protein_id` is an opaque, immutable UUID generated/owned by this project.
- UniProt accession and identifiers from other providers are external keys stored in `protein_id_mappings`; never derive the internal primary key from an accession.
- A mapping must identify its namespace/source and source identifier. Include source release/version where it affects interpretation.
- Preserve mapping states such as `resolved`, `merged`, `deprecated`, `unresolved`, and `ambiguous`; unresolved or ambiguous identifiers must not be silently treated as absent proteins or negative biological evidence.
- Reconcile PP-10 examples that could be read as accession-derived internal IDs (for example, `protein:Q9Y261`) with this opaque-UUID rule. Update the owning PP-10 contract/example in the same change set if it is genuinely contradictory; otherwise document the interpretation here and link the exact contract.

**To-do**
- [x] Define entity relationship diagram and ownership notes.
- [x] Define field-level data dictionary and nullability.
- [x] Confirm UUID internal identity and external mapping semantics against PP-10; corrected the Protein contract example to use an opaque UUID.
- [x] Define mapping status transitions and uniqueness scope (source + source ID + source version).
- [x] Define dataset/job lifecycle states and allowed transitions.
- [x] Define structure-to-protein chain cardinality and association semantics.

### Step 2 — Design normalized PostgreSQL schema

Implement relational models with explicit constraints and indexes.

**Schema requirements**
- `proteins`: UUID primary key, canonical metadata, timestamps, and only fields owned by the project.
- `protein_id_mappings`: FK to `proteins` when resolved, source namespace, external ID, optional source version, mapping status, provenance, and timestamps. Support unresolved/ambiguous records without fabricating a protein FK.
- `structures`: stable internal UUID, source/provider identifier, source version, structure format, sequence/quality metadata where available, and storage reference.
- `structure_protein_chains`: FKs to structure and protein, chain identifier, optional residue/range metadata, and uniqueness appropriate to a structure-chain association.
- `dataset_runs`: dataset name/type, source and source version, run status, start/end timestamps, and provenance.
- `jobs`: job type, status, retry count, idempotency key where applicable, timestamps, error summary, and relation to its dataset run.
- `artifacts`: artifact URI, storage backend, format/content type, size in bytes, checksum and checksum algorithm, source version, creation time, and optional owning job/dataset relation.

**Lifecycle conventions**
- Dataset run states: `pending`, `running`, `completed`, `failed`, `cancelled`.
- Job states: `queued`, `running`, `succeeded`, `failed`, `cancelled`.
- Record UTC timestamps consistently.
- Define retry/resume and idempotency behavior; retries must not create duplicate logical outputs.
- Preserve failure summaries and provenance without storing secrets or unbounded raw logs in metadata rows.

**Constraints and indexes**
- Add primary/foreign keys, check constraints, and uniqueness constraints for source mappings and structure-chain associations.
- Define whether uniqueness is global or scoped by source namespace/version; document the decision.
- Add indexes for canonical protein lookup, external-ID lookup, structure lookup, dataset/job status and chronology, and artifact ownership.
- Use explicit delete/update behavior; avoid cascading deletion of provenance or artifacts without a documented reason.

**To-do**
- [x] Implement SQLAlchemy 2.x models and PostgreSQL types.
- [x] Add required constraints, indexes, and relationship definitions.
- [x] Add deterministic integration fixtures for resolved, merged, deprecated, unresolved, and ambiguous mappings.
- [x] Document lifecycle transitions, uniqueness rules, and deletion semantics.

### Step 3 — Implement versioned migrations

Use Alembic as the sole schema migration mechanism.

**To-do**
- [x] Initialize Alembic and connect it to the application settings/model metadata.
- [x] Create the initial migration for all PP-12-owned tables, indexes, and constraints.
- [x] Ensure migrations work on an empty PostgreSQL database (Testcontainers integration test).
- [x] Document upgrade, downgrade, and revision-generation commands.
- [x] Test migration upgrade and downgrade/re-upgrade behavior against an isolated PostgreSQL container.
- [x] Ensure application startup does not silently mutate the schema outside migrations; Alembic is the only schema mutation path.

### Step 4 — Implement typed application configuration

Centralize environment-backed settings using Pydantic Settings.

**Configuration groups**
- Application: environment, log level, service name.
- PostgreSQL: host, port, database, username, password, connection options/pool sizing.
- MinIO/object storage: endpoint, bucket, access key, secret key, secure/TLS flag, and object prefix.
- Optional graph/vector services: connection settings only; these must not be required for a PostgreSQL-only application startup.
- External data sources: base URLs, request timeout, retry policy, and user-agent where applicable.
- Processing defaults: batch size and concurrency, with conservative documented defaults.

**Security and validation**
- Load local development values from environment and an ignored local `.env`; commit only `.env.example`.
- Never hardcode credentials or print secret values in logs/errors.
- Validate required settings at startup and fail with actionable, redacted messages.
- Keep optional service settings optional unless the corresponding feature is enabled.
- Define safe defaults for connection/request timeouts and bounded retries; avoid infinite retry loops.

**To-do**
- [x] Implement Pydantic Settings classes and environment variable naming convention.
- [x] Add validated defaults and conditional requirements for optional services.
- [x] Add a sanitized settings representation for diagnostics.
- [x] Update `.env.example` without real credentials.
- [x] Test valid config, missing optional-service endpoint, invalid values, and secret redaction.

### Step 5 — Integrate PostgreSQL and MinIO configuration

Connect the application persistence layer to PP-11 infrastructure without taking ownership of Compose topology.

**To-do**
- [x] Build SQLAlchemy engine/session lifecycle from typed settings.
- [x] Add a lightweight PostgreSQL connectivity/readiness check.
- [x] Define MinIO bucket and object-prefix conventions consistent with PP-11's dedicated storage configuration.
- [x] Define artifact URI format and metadata-only storage boundary; object bytes remain outside PostgreSQL.
- [x] Verify the metadata persistence layer/readiness check with PostgreSQL only; graph/vector remain disabled by default.
- [x] Document local startup prerequisites and expected service endpoints without assuming Docker runtime validation has occurred.

### Step 6 — Automated tests and schema verification

**Unit tests**
- [x] Test mapping-state validation and identity invariants.
- [x] Test configuration parsing, defaults, conditional requirements, and secret redaction.
- [x] Test lifecycle transition validation and artifact metadata validation.

**PostgreSQL integration tests**
- [x] Use Testcontainers to provision an isolated PostgreSQL instance.
- [x] Apply Alembic migrations against a clean database.
- [x] Verify primary/foreign keys, uniqueness, check constraints, and indexes.
- [x] Verify unresolved/ambiguous mappings can be represented without inventing a protein record.
- [x] Verify structure-chain associations support multiple proteins/chains correctly.
- [x] Verify dataset/job/artifact relationships and idempotency constraints.
- [x] Verify migration upgrade and downgrade/re-upgrade behavior where supported.

**CI and reproducibility**
- [x] Run formatting/lint/type checks and the full unit test suite.
- [x] Run PostgreSQL integration tests in CI with deterministic cleanup.
- [x] Ensure tests do not depend on a developer's local database or MinIO data.
- [x] Record exact commands and expected outcomes.

### Step 7 — Documentation and downstream handoff

**To-do**
- [x] Add ER diagram and concise schema/data dictionary documentation.
- [x] Document identity/mapping semantics and lifecycle states.
- [x] Document environment variables, local setup, migration, and test commands.
- [x] Document artifact URI/bucket/prefix convention and ownership boundary.
- [x] Review downstream PP-13–PP-20 and PP-39–PP-42 plans; aligned UUID/accession semantics and repaired affected cross-task links without implementing downstream scope.
- [x] Record decisions, commands, test results, known limitations, and deferred runtime checks in completion notes.

## 4. Deliverables

- SQLAlchemy 2.x metadata models for all PP-12-owned entities.
- Alembic initial migration and documented migration workflow.
- Pydantic Settings configuration with safe defaults and validation.
- PostgreSQL integration tests and configuration/domain unit tests.
- ER diagram, data dictionary, identity/lifecycle documentation, and local setup guide.
- Explicit downstream interface/ownership notes for dependent plans.

## 5. Acceptance criteria

- [ ] Internal protein identity is an opaque project-owned UUID; external accessions are mappings only.
- [ ] Resolved, merged, deprecated, unresolved, and ambiguous identifier states are represented without conflation.
- [ ] Structure/protein/chain relationships support the required many-to-many cardinality.
- [ ] Dataset runs, jobs, and artifacts have documented lifecycle/provenance semantics.
- [ ] Large structures, Parquet files, and embeddings are not stored as relational binary payloads.
- [ ] PostgreSQL schema is reproducible from versioned Alembic migrations on a clean database.
- [ ] Typed configuration validates required values, keeps optional services optional, and redacts secrets.
- [ ] Unit and PostgreSQL integration tests pass in CI.
- [ ] Documentation and downstream contracts are consistent with PP-10 and PP-11.
- [ ] No real credentials, generated large datasets, or local runtime state are committed.

## 6. Testing strategy

- **Unit:** domain invariants, mapping states, lifecycle transitions, artifact metadata, settings validation, and secret redaction.
- **Integration:** clean PostgreSQL provision, Alembic migrations, constraints, relationships, and persistence behavior.
- **Regression:** fixed fixtures prove stable UUID identity and external mapping semantics.
- **CI:** deterministic lint/type/unit/integration checks; no dependency on developer-local services.
- **Manual:** follow clean-checkout setup and migration instructions. Validate MinIO connectivity when Docker runtime is available and record the result separately from CI configuration checks.

## 7. Cross-task links

- **Upstream:** PP-10 owns shared architecture/data contracts; PP-11 owns Compose infrastructure and service topology.
- **Downstream:** PP-13–PP-16 consume protein identity, external mappings, structure metadata, and artifact references; PP-17–PP-20 consume dataset/run and artifact contracts; PP-39–PP-42 consume stable protein identifiers and metadata interfaces.
- **Rule:** consume upstream outputs as defined. Do not silently redefine shared identifiers, schemas, storage ownership, or configuration. If a shared contract must change, update the owning plan and affected consumers in the same change set.

## 8. Completion notes

Implementation status (2026-11-19): schema models, initial migration, typed settings, session/readiness helpers, unit/integration test cases, and schema documentation have been added on `feat/pp-12-metadata-schema-config`.

Validation completed in GitHub Actions run [44479313105](https://github.com/kakashianh/labs/actions/runs/44479313105): uv lock consistency, Ruff lint/format, Mypy, unit tests, PostgreSQL Testcontainers integration tests (including clean migration upgrade/downgrade/re-upgrade, constraints, indexes, identity mappings, multi-chain associations, jobs/artifacts), CLI smoke checks, and Docker Compose profile validation all passed. The refreshed `uv.lock` is committed. Downstream PP-13–PP-20, PP-26/PP-27, PP-30, PP-34, PP-39, and PP-42 plans were reviewed and aligned with opaque internal UUID semantics.

Deferred manual check: live MinIO connectivity/bucket creation is not part of the PostgreSQL integration suite; verify it when running the local PP-11 Docker Compose environment.

Record:
- Implemented schema/models and migration revision.
- Configuration variables and defaults.
- Commands executed and test/CI results.
- Artifacts/docs produced.
- Runtime checks actually performed (distinguish CI config validation from live service connectivity).
- Deviations, unresolved decisions, and downstream follow-up work.
