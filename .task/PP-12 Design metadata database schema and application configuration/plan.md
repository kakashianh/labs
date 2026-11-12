# PP-12 — Design metadata database schema and application configuration

## 1. Objective

Define persistent metadata and configuration contracts for protein records, source provenance, jobs, and generated artifacts.

## 2. Scope and design context

Keep large raw structures, Parquet files, and embeddings out of relational metadata tables; store references and lifecycle metadata instead.

**Upstream dependencies:** PP-10 defines schemas; PP-11 provides local database services.

**Related plans:** `PP-13–PP-16` · `PP-17–PP-20` · `PP-39–PP-42`. Use the links below to keep contracts aligned; update this plan if an upstream interface changes.

## 3. Step-by-step implementation and to-do

### Step 1 — Identify metadata entities and lifecycle

Model protein identity, source cross-references, structure records, dataset runs, processing status, and artifact locations.

**To-do**
- [ ] Define entity relationships and ownership
- [ ] Separate canonical protein identity from source-specific identifiers
- [ ] Define dataset/run status and timestamps
- [ ] Store artifact URI/checksum rather than large binary payloads

### Step 2 — Design schema and migrations

Create normalized tables, constraints, indexes, and migration strategy.

**To-do**
- [ ] Define DDL/ORM models and primary/foreign keys
- [ ] Add uniqueness constraints for canonical IDs and source mappings
- [ ] Add indexes for common protein and job lookups
- [ ] Define migration and rollback conventions

### Step 3 — Define configuration model

Centralize typed configuration for databases, external APIs, storage, Spark, graph/vector stores, and model paths.

**To-do**
- [ ] Define environment-specific config profiles
- [ ] Validate required settings at startup
- [ ] Define timeout, retry, batch-size, and concurrency defaults
- [ ] Ensure secrets are injected through environment/secret management

### Step 4 — Implement schema smoke checks

Apply migrations and test key constraints using local infrastructure.

**To-do**
- [ ] Apply migrations against Compose database
- [ ] Test duplicate and invalid reference rejection
- [ ] Test config loading and missing-secret failure behavior
- [ ] Document schema and configuration changes

## 4. Deliverables

- Implementation/configuration and documentation within this ticket's scope.
- Versioned interfaces and assumptions needed by dependent tasks.
- Automated tests and reproducible verification instructions.

## 5. Acceptance criteria

- [ ] All step to-dos are completed and reviewed.
- [ ] Contracts, configuration, and operational assumptions are documented.
- [ ] Failure cases are explicit; no credentials or generated large datasets are committed.
- [ ] Tests pass from the documented local environment.
- [ ] Downstream plans and interfaces remain consistent.

## 6. Testing strategy

- **Unit:** validate core rules, configuration, schemas, and edge cases.
- **Integration:** verify the relevant infrastructure or module boundary.
- **Regression:** preserve stable IDs and contracts with fixed fixtures.
- **Manual:** follow the documented clean-checkout workflow.

## 7. Cross-task links

- **Upstream:** PP-10 defines schemas; PP-11 provides local database services.
- **Downstream / coordination:** PP-13–PP-16, PP-17–PP-20, PP-39–PP-42
- **Rule:** consume upstream outputs as defined; do not silently redefine shared identifiers, schemas, or configuration. If a contract must change, update the owning plan and all affected consumers in the same change set.

## 8. Completion notes

Record implementation decisions, commands and test results, artifacts produced, data/source versions, and deviations from this plan.
