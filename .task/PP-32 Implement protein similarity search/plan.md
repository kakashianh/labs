# PP-32 — Implement protein similarity search

## 1. Objective

Provide a reusable similarity-search service over indexed protein embeddings with explainable result metadata.

## 2. Scope and dependencies

PP-30 embeddings; PP-31 Milvus collections/indexes; PP-20 canonical protein metadata.

Related plans: [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md) · [PP-31](../PP-31%20Configure%20Milvus%20collections%20and%20vector%20indexing/plan.md) · [PP-33](../PP-33%20Integrate%20structure%2C%20sequence%20and%20PPI%20features/plan.md) · [PP-39](../PP-39%20Implement%20protein%20search%20and%20detail%20REST%20APIs/plan.md) · [PP-40](../PP-40%20Implement%20PPI%20exploration%20and%20similarity%20search%20APIs/plan.md). These links define the expected producer/consumer relationships for this feature.

## 3. Step-by-step implementation and to-do

### Step 1 — Define similarity query contract

Specify query by accession or vector, modality/model version, top-k, filters, and score semantics.

**To-do**
- [ ] Define typed request and response models
- [ ] Specify distance metric and score conversion/ordering
- [ ] Define top-k bounds and metadata filter allowlist
- [ ] Define missing embedding and collection-version behavior

### Step 2 — Implement search repository

Wrap Milvus search and map hits to canonical protein metadata.

**To-do**
- [ ] Resolve query accession to embedding vector
- [ ] Execute vector search with selected collection/index
- [ ] Exclude self-match when requested
- [ ] Return accession, score/distance, modality, and model version

### Step 3 — Add validation and operational safeguards

Protect the service from invalid vectors, excessive k, and unavailable indexes.

**To-do**
- [ ] Validate vector dimension and finite values
- [ ] Enforce top-k and timeout limits
- [ ] Handle empty result and backend errors explicitly
- [ ] Add structured logs and latency/result-count metrics

### Step 4 — Evaluate and document behavior

Test known similarity cases and publish API-ready examples.

**To-do**
- [ ] Build a small query benchmark set
- [ ] Verify stable ranking for fixed index/model version
- [ ] Measure latency and result quality
- [ ] Document score interpretation and handoff to PP-33, PP-39, and PP-40

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

- **Upstream:** PP-30 embeddings; PP-31 Milvus collections/indexes; PP-20 canonical protein metadata.
- **Consumers / coordination:** PP-20, PP-30, PP-31, PP-33, PP-39, PP-40
- **Contract rule:** publish schemas, versions, parameters, evaluation results, and limitations. Do not change embedding dimensions, score semantics, or graph feature definitions without coordinating with linked consumers.

## 8. Completion notes

Record model/data/index versions, parameters, artifact locations, quality/performance results, and deviations.
