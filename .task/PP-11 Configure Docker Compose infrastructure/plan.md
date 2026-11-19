# PP-11 — Configure Docker Compose infrastructure

## 1. Objective

Provide a reproducible local infrastructure environment for Protein3D-PPI development and integration testing. Keep the default startup lightweight while allowing graph and vector services to be enabled independently.

## 2. Scope and design context

PP-11 owns local container orchestration, service configuration, persistent volumes, health/readiness checks, operational commands, and infrastructure documentation.

**In scope**
- Docker Compose topology, service profiles, networks, ports, volumes, health checks, and restart policies.
- Local non-secret environment templates and safe secret handling.
- Reproducible startup, shutdown, logs, reset, and troubleshooting workflows.
- Compose validation and lightweight infrastructure smoke checks.

**Out of scope**
- PostgreSQL application schema, ORM models, and migrations (PP-12).
- Data ingestion, Spark processing, and production data loading (PP-13–PP-25).
- JanusGraph logical schema, graph import, and graph query implementation (PP-26–PP-29).
- Embedding generation, Milvus collection schema, vector ingestion, and search implementation (PP-30–PP-33).
- Production deployment/orchestration, Kubernetes, and cloud infrastructure.

**Upstream dependencies:** PP-9 for repository tooling; PP-10 for logical architecture and storage boundaries; PP-12 for application configuration coordination.

**Architecture references**
- [PP-10 architecture entry point](../../docs/architecture/README.md)
- [System overview](../../docs/architecture/system-overview.md)
- [Dependency rules](../../docs/architecture/dependency-rules.md)
- [ADR-003 — Data storage boundaries](../../docs/architecture/decisions/ADR-003-data-storage-boundaries.md)
- [Structure metadata contract](../../docs/architecture/data-contracts/structure-metadata.md)
- [Embedding contract](../../docs/architecture/data-contracts/embedding.md)

**Related plans:** [PP-9](../PP-9%20Initialize%20repository%20and%20Python%20development%20environment/plan.md) · [PP-10](../PP-10%20Design%20project%20architecture%20and%20module%20structure/plan.md) · [PP-12](../PP-12%20Design%20metadata%20database%20schema%20and%20application%20configuration/plan.md) · [PP-13](../PP-13%20Implement%20AlphaFold%20structure%20data%20crawler/plan.md) · [PP-16](../PP-16%20Implement%20ingestion%20validation%20and%20raw%20data%20management/plan.md) · [PP-26](../PP-26%20Design%20JanusGraph%20protein%20interaction%20schema/plan.md) · [PP-31](../PP-31%20Configure%20Milvus%20collections%20and%20vector%20indexing/plan.md) · [PP-39](../PP-39%20Implement%20protein%20search%20and%20detail%20REST%20APIs/plan.md) · [PP-42](../PP-42%20Integrate%20prediction%20results%20and%20perform%20end-to-end%20testing/plan.md).

## 3. Proposed local topology

Use Compose profiles to avoid running every infrastructure service on every developer machine.

| Profile | Services | Purpose | Startup |
|---|---|---|---|
| Default (no profile) | PostgreSQL, MinIO | Core metadata and object/artifact storage | Default |
| `graph` | JanusGraph, Cassandra, Elasticsearch | PPI graph development and integration tests | Explicit opt-in |
| `vector` | Milvus standalone, etcd | Embedding/vector search development and integration tests | Explicit opt-in; reuses core MinIO |

### Service design decisions

- **PostgreSQL:** core relational metadata service. PP-12 owns database schema and migrations; PP-11 only provides the local service, connection settings, health check, and persistent volume.
- **MinIO:** core S3-compatible object storage for local raw structure files and derived artifacts. Use separate buckets or prefixes for project-owned artifacts and Milvus storage; do not store large artifacts in PostgreSQL rows.
- **JanusGraph:** enable only with the `graph` profile. Use an explicitly selected and version-compatible storage/index backend. The initial local topology should use Cassandra for JanusGraph storage and Elasticsearch for mixed-index support; validate the selected JanusGraph, Cassandra, and Elasticsearch versions and configuration together before implementation.
- **Milvus:** enable only with the `vector` profile in standalone mode, with etcd for metadata and the core MinIO service for object storage. Configure Milvus to use a dedicated bucket/prefix so it does not collide with application artifacts.
- **Spark:** do not run a permanent Spark cluster in this task. Batch jobs can run locally through the project environment or as explicitly invoked one-off jobs when a downstream task requires it.
- **Application/API containers:** not part of PP-11 unless a later task establishes a concrete need and an explicit ownership boundary.

### Resource and exposure policy

- Keep the default profile suitable for a developer laptop; document measured or vendor-recommended resource requirements for optional profiles.
- Treat graph and vector profiles as resource-intensive. Document approximate CPU/RAM/disk expectations after validating the chosen image versions; do not invent hard guarantees.
- Bind published development ports to loopback by default (`127.0.0.1`) unless a documented Codespaces/dev-container use case requires another binding.
- Do not require a GPU for the local infrastructure baseline.
- Pin explicit image versions; avoid floating `latest` tags. Record compatibility-sensitive version choices in the infrastructure documentation.

## 4. Step-by-step implementation and to-do

### Step 1 — Finalize service topology and compatibility

Translate the logical architecture into an explicit, minimal local service topology.

**To-do**
- [ ] Confirm core services: PostgreSQL and MinIO.
- [ ] Confirm the `graph` profile: JanusGraph + Cassandra + Elasticsearch; record the selected compatible versions and required JanusGraph backend/index configuration.
- [ ] Confirm the `vector` profile: Milvus standalone + etcd, reusing MinIO with a dedicated bucket/prefix.
- [ ] Record each service's purpose, dependency, internal service DNS name, host port, health/readiness mechanism, and persistent data location.
- [ ] Document expected CPU, memory, and disk requirements for core, graph, and vector profiles based on selected image documentation and local validation.
- [ ] Define the default profile behavior and ensure optional services do not start unless explicitly requested.
- [ ] Confirm that no permanent Spark cluster or application/API container is needed in PP-11.

### Step 2 — Implement Compose and service configuration

Create reproducible Compose configuration with pinned images and isolated persistence.

**To-do**
- [ ] Add or update `compose.yaml` with explicit image versions (no `latest` tags).
- [ ] Configure PostgreSQL and MinIO as default core services.
- [ ] Add graph and vector services under their respective Compose profiles.
- [ ] Configure service DNS-based internal connectivity; avoid hardcoding container IP addresses.
- [ ] Configure named volumes for persistent database, graph, vector metadata, and object data.
- [ ] Configure health checks and `depends_on: condition: service_healthy` where supported and appropriate; do not treat container startup order as application readiness.
- [ ] Configure restart policies suitable for local development.
- [ ] Bind host ports to loopback by default and document any intentional exception.
- [ ] Configure Milvus to use the MinIO service with a dedicated bucket/prefix and configure required bucket initialization safely.
- [ ] Keep service credentials and secret values out of Compose source; use environment interpolation and local ignored env files.
- [ ] Pin compatible versions for JanusGraph, Cassandra, Elasticsearch, Milvus, etcd, PostgreSQL, and MinIO.

### Step 3 — Environment, lifecycle commands, and safe operations

Make the environment straightforward to start, inspect, stop, and reset.

**To-do**
- [ ] Provide a complete `.env.example` containing variable names and safe placeholders only.
- [ ] Ensure real `.env` files are ignored by Git and never required to be pasted into chat or committed.
- [ ] Add or update Makefile targets for core startup, graph startup, vector startup, shutdown, logs, status, and configuration validation.
- [ ] Document equivalent raw Docker Compose commands for every target.
- [ ] Document how to start optional profiles without accidentally enabling all heavyweight services.
- [ ] Provide an explicit destructive data-reset command; make its volume deletion behavior clear and never run it as part of ordinary shutdown.
- [ ] Document backup/restore expectations and the fact that deleting named volumes permanently removes local service data.
- [ ] Document port conflicts, common startup failures, and how to inspect service logs.

### Step 4 — Validate service readiness and integration

Verify that the documented environment works from a clean checkout and survives normal restarts.

**To-do**
- [ ] Validate Compose syntax and fully rendered configuration with `docker compose config`.
- [ ] Start the default profile from a clean environment and wait for healthy PostgreSQL and MinIO services.
- [ ] Verify PostgreSQL accepts a connection using the configured local credentials without printing secrets.
- [ ] Verify MinIO readiness and create/list/read/delete a disposable test object in a test bucket.
- [ ] Start the graph profile and verify JanusGraph readiness plus connectivity to its configured storage and index backends.
- [ ] Start the vector profile and verify Milvus readiness, etcd connectivity, and MinIO access.
- [ ] Verify named-volume data persists across container restart.
- [ ] Verify stopping the stack does not delete persisted data.
- [ ] Verify optional profiles can be started independently and do not expose unrelated services.
- [ ] Record actual startup results, resource observations, and any limitations.

### Step 5 — Documentation and CI validation

Publish a reliable handoff for developers and downstream implementation tasks.

**To-do**
- [ ] Document architecture, profile selection, services, ports, volumes, dependencies, and resource expectations.
- [ ] Document setup prerequisites and clean-checkout startup instructions.
- [ ] Document health/readiness checks, lifecycle commands, troubleshooting, and destructive reset behavior.
- [ ] Add a CI job or workflow step to run Compose configuration validation without requiring all heavyweight services to start.
- [ ] Add lightweight automated checks for environment templates and Compose configuration where practical.
- [ ] Link PP-12, PP-26–PP-29, and PP-31–PP-33 to the infrastructure service names, connection endpoints, and readiness assumptions they consume.
- [ ] Record version compatibility decisions and known limitations.

## 5. Deliverables

- `compose.yaml` with a lightweight default profile and opt-in graph/vector profiles.
- Version-pinned service definitions, health checks, isolated network, and named volumes.
- Safe `.env.example` and documented secret injection conventions.
- Makefile lifecycle and validation commands.
- Infrastructure documentation covering topology, ports, resource expectations, readiness, persistence, reset, and troubleshooting.
- CI validation for Compose configuration.
- Verified local startup/connectivity results for each supported profile.

## 6. Acceptance criteria

- [ ] Default `docker compose up -d` starts only PostgreSQL and MinIO.
- [ ] Graph services start only when the `graph` profile is explicitly enabled.
- [ ] Vector services start only when the `vector` profile is explicitly enabled.
- [ ] All images use explicit, compatible versions; no floating `latest` tags are used.
- [ ] Services use a dedicated network, stable service DNS names, persistent named volumes, and meaningful health checks.
- [ ] Host ports are loopback-bound by default, with any exception documented.
- [ ] No real credentials, access keys, or secret values are committed.
- [ ] PostgreSQL and MinIO connectivity checks pass; graph/vector readiness checks pass when those profiles are enabled.
- [ ] Persistent data survives container restart, and ordinary shutdown does not remove volumes.
- [ ] Destructive reset behavior is explicit and documented.
- [ ] Compose configuration validation passes in CI.
- [ ] Downstream plans can identify service endpoints, profile requirements, and readiness assumptions without redefining logical contracts.

## 7. Testing and verification strategy

- **Static configuration:** run `docker compose config` for the default and each optional profile; validate required variables and reject unresolved placeholders where appropriate.
- **Core integration:** start PostgreSQL and MinIO; verify health, connection, disposable object operations, and persistence across restart.
- **Graph integration:** start the graph profile; verify JanusGraph server readiness and connectivity to Cassandra and Elasticsearch using the selected compatible versions.
- **Vector integration:** start the vector profile; verify etcd, Milvus standalone, and MinIO connectivity.
- **Lifecycle regression:** verify stop/start preserves named volumes and that destructive reset is separate and explicit.
- **CI:** validate Compose rendering and safe configuration without making every pull request download and start all heavyweight services.
- **Manual:** execute the documented clean-checkout workflow and record resource usage and troubleshooting notes.

## 8. Cross-task links and ownership

- **Upstream:** PP-9 provides project tooling; PP-10 owns logical architecture and shared contracts.
- **PostgreSQL:** PP-12 owns relational schema, migrations, and typed application configuration; PP-11 owns only the local PostgreSQL service.
- **Graph:** PP-26 owns graph schema; PP-27–PP-29 own graph import and operations; PP-11 supplies the local JanusGraph and its selected backend/index services.
- **Vector:** PP-30 owns embedding generation; PP-31 owns Milvus collection/index configuration; PP-32–PP-33 own vector search and feature integration; PP-11 supplies local Milvus dependencies and object storage connectivity.
- **Ingestion/API/E2E:** PP-13–PP-25 and PP-39–PP-42 consume the documented service endpoints and readiness assumptions as needed.
- **Contract rule:** PP-10 remains the logical contract owner. PP-11 must not redefine protein identity, PPI semantics, embedding dimensions, or application data schemas. Any required logical contract change must be coordinated with PP-10 and all affected consumers.

## 9. Completion notes

Record implementation decisions, exact image versions, compatibility references, commands and test results, resource observations, artifacts produced, and deviations from this plan.
