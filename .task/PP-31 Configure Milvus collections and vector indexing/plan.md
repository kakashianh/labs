# PP-31 — Configure Milvus collections and vector indexing

## 1. Objective

Create versioned Milvus collections and indexes that store protein embeddings with metadata filters.

## 2. Scope and dependencies

PP-30 embedding schema and generated vectors; PP-11 Milvus runtime; PP-10 identifier conventions.

Related plans: [PP-11](../PP-11%20Configure%20Docker%20Compose%20infrastructure/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md) · [PP-32](../PP-32%20Implement%20protein%20similarity%20search/plan.md) · [PP-33](../PP-33%20Integrate%20structure%2C%20sequence%20and%20PPI%20features/plan.md) · [PP-39](../PP-39%20Implement%20protein%20search%20and%20detail%20REST%20APIs/plan.md) · [PP-40](../PP-40%20Implement%20PPI%20exploration%20and%20similarity%20search%20APIs/plan.md). These links define the expected producer/consumer relationships for this feature.

## 3. Step-by-step implementation and to-do

### Step 1 — Design collection and partition strategy

Choose collection separation by modality/model version and define metadata fields for filtering.

**To-do**
- [ ] Define collection naming/version policy
- [ ] Define vector dimension, metric type, and scalar metadata fields
- [ ] Decide partitioning strategy by model/modality/species if justified
- [ ] Define index type and search parameter defaults

### Step 2 — Implement collection lifecycle

Create collections, indexes, and safe migration/rebuild procedures.

**To-do**
- [ ] Add idempotent collection bootstrap script
- [ ] Configure index build parameters for expected corpus size
- [ ] Add readiness and schema compatibility checks
- [ ] Document drop/rebuild behavior and backup implications

### Step 3 — Implement embedding ingestion

Bulk-load PP-30 vectors with canonical IDs and provenance metadata.

**To-do**
- [ ] Validate vector dimensions before insert
- [ ] Upsert using stable protein/modality/version keys
- [ ] Batch inserts and track checkpoint/progress
- [ ] Reconcile inserted counts with source manifest

### Step 4 — Verify search readiness

Run representative searches and tune recall/latency trade-offs.

**To-do**
- [ ] Test vector search for known proteins
- [ ] Verify metadata filters and version isolation
- [ ] Measure latency and recall on a fixed evaluation sample
- [ ] Publish collection/index configuration for PP-32 and PP-33

## 4. Deliverables

- Tested implementation and versioned data/service contract.
- Reproducible sample run and representative fixtures.
- Configuration, provenance, monitoring, and operational documentation.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Identifiers and model/index versions are traceable.
- [ ] Input validation, bounded resource use, and failure behavior are implemented.
- [ ] Quality and performance are measured on a representative sample.
- [ ] Unit and integration tests pass and consumers can use the documented contract.

## 6. Testing strategy

- **Unit:** algorithms, transformations, schema checks, score semantics, and edge cases.
- **Integration:** graph/vector infrastructure and metadata lookups.
- **Quality:** reproducibility, coverage, dimensionality, ranking correctness, and provenance.
- **Performance:** batch throughput, memory, index build time, and query latency as applicable.

## 7. Cross-task handoff

- **Upstream:** PP-30 embedding schema and generated vectors; PP-11 Milvus runtime; PP-10 identifier conventions.
- **Consumers / coordination:** PP-11, PP-20, PP-30, PP-32, PP-33, PP-39, PP-40
- **Contract rule:** publish schemas, versions, parameters, evaluation results, and limitations. Do not change embedding dimensions, score semantics, or graph feature definitions without coordinating with linked consumers.

## 8. Completion notes

Record model/data/index versions, parameters, artifact locations, quality/performance results, and deviations.
