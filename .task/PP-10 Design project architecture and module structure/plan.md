# PP-10 — Design project architecture and module structure

## 1. Objective

Define the system architecture, module boundaries, dependency rules, shared data contracts, and architecture decision records (ADRs) for Protein3D-PPI. This task produces an implementation-ready reference for downstream tasks without implementing their services, pipelines, database schema, ML workflows, REST endpoints, or UI.

## 2. Scope and design context

PP-10 owns the **logical architecture** and **cross-module contracts**. It does not own infrastructure deployment, physical database migrations, ETL implementation, graph/vector import, model training, endpoint implementation, or dashboard implementation.

**Upstream dependency:** PP-9 establishes repository layout, Python version, tooling, and development conventions.

**Technology baseline to document and validate against downstream plans**
- Python application and shared domain layer.
- PySpark for batch ingestion and large-scale processing.
- Biopython/Biotite for protein structure parsing and feature extraction.
- PostgreSQL for relational metadata.
- JanusGraph for protein interaction graph storage.
- Milvus for vector search.
- MinIO/local-compatible object storage for large artifacts such as PDB/mmCIF, Parquet, and model files.
- FastAPI for the backend API.

Document the role and boundary of each technology. Do not add dependencies or provision services in PP-10.

**Related plans:** [PP-9](../PP-9%20Initialize%20repository%20and%20Python%20development%20environment/plan.md) · [PP-11](../PP-11%20Configure%20Docker%20Compose%20infrastructure/plan.md) · [PP-12](../PP-12%20Design%20metadata%20database%20schema%20and%20application%20configuration/plan.md) · [PP-17](../PP-17%20Build%20Spark%20ingestion%20pipeline/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-26](../PP-26%20Design%20JanusGraph%20protein%20interaction%20schema/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md) · [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md) · [PP-39](../PP-39%20Implement%20protein%20search%20and%20detail%20REST%20APIs/plan.md) · [PP-41](../PP-41%20Build%20protein%203D%20and%20PPI%20visualization%20dashboard/plan.md).

## 3. Architecture principles

1. **Domain-first:** core protein, interaction, annotation, and prediction concepts must not depend on infrastructure frameworks.
2. **Dependency direction:** infrastructure and delivery layers depend inward on domain contracts; domain code must not import Spark, FastAPI, JanusGraph, Milvus, or storage SDKs.
3. **Ports and adapters:** define stable interfaces at boundaries; technology-specific adapters implement those interfaces in their owning tasks.
4. **Single ownership:** every physical schema, service, pipeline, and API endpoint has one owning task. Shared logical contracts are owned here.
5. **Traceability and reproducibility:** shared records carry source/version/provenance information where applicable.
6. **Explicit data semantics:** missing, unknown, and negative values must not be conflated; confidence scores must not be confused with prediction probabilities.
7. **Artifact separation:** large binary or columnar artifacts are stored outside relational metadata tables; metadata records reference their URI and checksum.

## 4. Step-by-step implementation and to-do

### Step 1 — Map the end-to-end system and data flow

Describe the lifecycle from external biological data sources to user-facing search and visualization.

**To-do**
- [x] Create a system context/component diagram showing external sources, ingestion, processing, structure analysis, graph/vector stores, ML, API, and UI.
- [x] Create a data-flow diagram from raw source data through normalized datasets, derived features, embeddings/predictions, and online queries.
- [x] Separate batch/offline processing paths from online request paths.
- [x] Identify the owner, input, output, and persistence boundary of each major component.
- [x] Document principal data and artifact formats at each boundary.

### Step 2 — Define module structure and dependency rules

Specify the intended package architecture as a design target. Do not create empty packages or implement modules in this task.

**Target package structure**
~~~text
src/protein3d/
├── domain/
│   ├── protein/
│   ├── interaction/
│   ├── annotation/
│   └── prediction/
├── ingestion/
│   ├── sources/
│   ├── adapters/
│   └── manifests/
├── processing/
│   ├── spark/
│   ├── normalization/
│   └── quality/
├── structures/
│   ├── parsers/
│   ├── coordinates/
│   └── features/
├── graph/
│   ├── repositories/
│   └── services/
├── embeddings/
│   ├── encoders/
│   └── repositories/
├── ml/
│   ├── datasets/
│   ├── models/
│   └── evaluation/
├── api/
│   ├── routes/
│   ├── schemas/
│   └── dependencies/
└── shared/
    ├── config/
    ├── logging/
    └── errors/
~~~

**To-do**
- [x] Document each package's responsibility, public interface, and ownership boundary.
- [x] Define dependency direction and prohibited imports, especially domain-to-infrastructure dependencies.
- [x] Identify ports/interfaces and the adapters that will implement them.
- [x] Distinguish shared domain contracts from pipeline-, storage-, API-, and UI-specific DTOs.
- [x] Document the approach to avoiding circular imports and cross-layer coupling.
- [x] Map package/module ownership to downstream PP tasks.

### Step 3 — Specify shared identifiers and data contracts

Define logical contracts and semantics only. Physical PostgreSQL DDL/ORM models belong to PP-12; Parquet implementation belongs to PP-20; JanusGraph schema belongs to PP-26; embedding model/vector implementation belongs to PP-30; API request/response implementation belongs to PP-39.

**Minimum contracts**

| Contract | Required logical fields / semantics |
|---|---|
| Protein | protein_id, canonical uniprot_accession, sequence, organism_id; stable identity and sequence provenance |
| Protein ID mapping | source, source_id, protein_id, mapping_status; preserve source identifiers and merged/deprecated mapping outcomes |
| PPI edge | source_protein_id, target_protein_id, confidence, evidence, source_version; interaction direction and duplicate handling |
| GO annotation | protein_id, go_id, evidence_code, source_version; distinguish absent annotation from an explicit negative assertion |
| Structure metadata | protein_id, structure_id, format, chain_id, artifact_uri; source/provenance and checksum where available |
| Feature vector | protein_id, feature_type, dimension, feature_version; value representation and provenance |
| Embedding | protein_id, embedding_type, dimension, model_name, model_version, artifact_or_vector_ref; model-specific details belong to PP-30 |
| Prediction | protein_id, go_id, score, model_version, dataset_version; score semantics and prediction provenance |

**Contract rules to settle**
- [x] Use a canonical UniProt accession policy for cross-system protein identity while preserving original source IDs and mapping status.
- [x] Define behavior for merged, deprecated, unresolved, and one-to-many external identifiers.
- [x] Treat biological PPI as undirected unless a source explicitly represents directionality; define canonical endpoint ordering or an equivalent deduplication rule.
- [x] Keep interaction confidence, evidence strength, and ML prediction probability as distinct concepts.
- [x] Define nullability and distinguish missing/unknown/not-applicable from negative or zero values.
- [x] Define schema, source dataset, feature, embedding, and model version/provenance conventions.
- [x] Define units, coordinate conventions, and reference-frame expectations for structure-derived values where relevant.
- [x] Store large PDB/mmCIF, Parquet, embedding/model artifacts in object storage or their designated stores; metadata contracts carry references and checksums rather than large payloads.

### Step 4 — Define component ownership and downstream handoff

Document the boundary between this architecture task and implementation tasks.

| Area | Primary ownership |
|---|---|
| Repository conventions and developer tooling | PP-9 |
| Logical architecture, module boundaries, shared contracts, ADRs | PP-10 |
| Compose services, networks, volumes, profiles, health checks | PP-11 |
| PostgreSQL physical schema, migrations, typed application configuration | PP-12 |
| Source acquisition and ingestion stages | PP-13–PP-17 |
| Protein master and normalized PPI datasets | PP-18–PP-20 |
| Structure parsing and derived structural features | PP-21–PP-25 |
| JanusGraph schema and graph operations | PP-26–PP-29 |
| Embedding generation and vector indexing/search integration | PP-30–PP-33 |
| ML datasets, models, training, and evaluation | PP-34–PP-38 |
| REST API contracts and implementation | PP-39–PP-40 |
| Dashboard and visualization | PP-41–PP-42 |

**To-do**
- [ ] Record input/output contracts consumed or produced by PP-11, PP-12, PP-17, PP-20, PP-26, PP-30, PP-34, PP-39, and PP-41.
- [x] State explicitly which downstream task owns each physical schema, service, pipeline, and endpoint.
- [x] Ensure PP-10 does not duplicate PP-11 infrastructure configuration or PP-12 database implementation.
- [x] Ensure model-specific embedding details remain in PP-30 and endpoint-level schemas remain in PP-39.
- [ ] Identify conflicting or missing downstream assumptions and record required follow-up instead of silently redefining contracts.

### Step 5 — Publish architecture documentation and decision records

**To-do**
- [x] Publish the architecture entry point and system overview.
- [x] Publish data-flow and component diagrams.
- [x] Publish module structure and dependency rules.
- [x] Publish one contract document for each shared record defined in Step 3.
- [x] Record architecture style, protein identity, and data-storage boundary decisions as ADRs.
- [ ] Add links from relevant downstream plans to the exact architecture/contract documents.

**Expected documentation layout**
~~~text
docs/
└── architecture/
    ├── README.md
    ├── system-overview.md
    ├── data-flow.md
    ├── module-structure.md
    ├── dependency-rules.md
    ├── data-contracts/
    │   ├── protein.md
    │   ├── protein-id-mapping.md
    │   ├── ppi-edge.md
    │   ├── go-annotation.md
    │   ├── structure-metadata.md
    │   ├── feature-vector.md
    │   ├── embedding.md
    │   └── prediction.md
    └── decisions/
        ├── ADR-001-architecture-style.md
        ├── ADR-002-protein-identity.md
        └── ADR-003-data-storage-boundaries.md
~~~

### Step 6 — Validate architecture consistency

Validation in PP-10 is documentation and contract review, not implementation of downstream services.

**To-do**
- [x] Review diagrams, package ownership, and dependency rules for consistency.
- [x] Check every shared contract for field definitions, types, nullability, units where applicable, versioning, and provenance.
- [ ] Cross-check contracts against PP-11, PP-12, PP-17, PP-20, PP-26, PP-30, PP-34, PP-39, and PP-41.
- [x] Add representative JSON examples or fixtures for shared records where useful.
- [ ] Document unresolved decisions, assumptions, and follow-up tasks.
- [ ] Verify all architecture documentation links from the repository.

## 5. Deliverables

- docs/architecture/README.md as the architecture entry point.
- System context/component and end-to-end data-flow diagrams.
- Target package/module structure and explicit dependency rules.
- Versioned logical data contracts for protein, ID mapping, PPI, GO annotation, structure metadata, feature vector, embedding, and prediction.
- ADRs documenting major architecture decisions and trade-offs.
- Downstream ownership/contract mapping and validation notes.

No production pipeline, API endpoint, database migration, graph schema, vector index, model, or UI feature is implemented in PP-10.

## 6. Acceptance criteria

- [ ] System overview clearly explains major components, data flow, and batch versus online paths.
- [ ] Every module has a documented responsibility, owner, and dependency boundary.
- [ ] Domain concepts do not depend on infrastructure-specific frameworks or SDKs.
- [ ] All minimum shared contracts and their semantics are documented, including identifiers, nullability, versioning, and provenance.
- [ ] Protein identity, PPI direction/deduplication, score semantics, missing-versus-negative behavior, and artifact storage rules are explicit.
- [ ] ADRs capture decisions and relevant trade-offs.
- [ ] Downstream ownership is unambiguous and relevant plans link to the contracts they consume.
- [ ] Documentation is internally consistent, links resolve, and representative contract examples are valid.
- [ ] No credentials, generated large datasets, or out-of-scope implementation are committed.

## 7. Testing and verification strategy

- **Documentation:** verify Markdown structure, internal links, diagram rendering, and consistency between diagrams and written ownership.
- **Contract examples:** validate representative JSON examples against documented field/type/nullability rules; fixtures are illustrative and do not establish a new physical schema.
- **Architecture review:** manually trace dependency direction and ensure domain contracts remain technology-independent.
- **Downstream consistency:** compare contract definitions with referenced task plans and record required follow-up.
- **Regression:** review changes to shared identifiers/contracts against existing downstream assumptions before accepting updates.

## 8. Cross-task links

- **Upstream:** PP-9 provides repository and development conventions.
- **Infrastructure and persistence:** PP-11 owns local service topology; PP-12 owns PostgreSQL schema and application configuration.
- **Data and domain processing:** PP-13–PP-25 own source ingestion, normalized datasets, and structure processing.
- **Graph and vector search:** PP-26–PP-33 own graph and embedding implementations.
- **ML and delivery:** PP-34–PP-42 own prediction workflows, API implementation, and dashboard.
- **Change rule:** PP-10 is the logical contract owner. Any change to a shared identifier or contract must update this task's documentation and all affected consumer plans in the same coordinated change set. Physical implementation remains with the owning downstream task.

## 9. Completion notes

- Created the architecture entry point, system overview, batch/online data-flow diagrams, target module layout, and dependency rules under docs/architecture/.
- Added eight logical contract documents: protein, protein ID mapping, PPI edge, GO annotation, structure metadata, feature vector, embedding, and prediction.
- Added ADR-001 (architecture style), ADR-002 (protein identity), and ADR-003 (data/storage boundaries).
- Verified the architecture, contract, and ADR directory contents on the feature branch.
- Parsed the representative Protein and PPI JSON examples successfully. Other contracts are specified as field/semantic tables and do not yet have JSON fixtures.
- Downstream task plans have not yet been edited to add direct links to the new contracts. Full downstream contract-by-contract validation and repository-wide Markdown/diagram rendering checks remain follow-up verification.
- No application code, dependencies, database migrations, service configuration, or large datasets were added.
