# PP-40 — Implement PPI exploration and similarity search APIs

## 1. Objective

Expose graph neighborhood and vector similarity capabilities through stable REST endpoints.

## 2. Scope and dependencies

PP-28 graph query service; PP-31 Milvus index; PP-32 similarity service; PP-20 canonical metadata.

Related plans: [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-28](../PP-28%20Implement%20graph%20query%20and%20neighborhood%20services/plan.md) · [PP-31](../PP-31%20Configure%20Milvus%20collections%20and%20vector%20indexing/plan.md) · [PP-32](../PP-32%20Implement%20protein%20similarity%20search/plan.md) · [PP-39](../PP-39%20Implement%20protein%20search%20and%20detail%20REST%20APIs/plan.md) · [PP-41](../PP-41%20Build%20protein%203D%20and%20PPI%20visualization%20dashboard/plan.md) · [PP-42](../PP-42%20Integrate%20prediction%20results%20and%20perform%20end-to-end%20testing/plan.md). These plans define the data/API handoffs; preserve their identifiers, versions, and error semantics.

## 3. Step-by-step implementation and to-do

### Step 1 — Define graph and similarity endpoints

Specify neighborhood depth, confidence filters, top-k, modality/model version, and response contracts.

**To-do**
- [ ] Define PPI neighbor endpoint and bounded depth semantics
- [ ] Define similarity endpoint by accession or embedding query
- [ ] Define top-k and confidence filter limits
- [ ] Define score/distance interpretation and provenance fields

### Step 2 — Implement service orchestration

Compose graph/vector services and enrich results with canonical protein metadata.

**To-do**
- [ ] Call PP-28 graph service for neighborhood results
- [ ] Call PP-32 similarity service for vector results
- [ ] Resolve IDs and names through protein master
- [ ] Preserve edge confidence and embedding/model version

### Step 3 — Add safety and resilience

Protect APIs from expensive traversals and unavailable backends.

**To-do**
- [ ] Enforce depth, top-k, and request timeout limits
- [ ] Handle graph/vector backend outage separately
- [ ] Add pagination and response-size limits
- [ ] Add structured logs and latency/error metrics

### Step 4 — Test API contracts and handoff

Validate endpoint behavior with known graph and vector fixtures.

**To-do**
- [ ] Add unit tests for request validation and response mapping
- [ ] Add integration tests against JanusGraph and Milvus
- [ ] Verify stable ordering, IDs, scores, and empty-result behavior
- [ ] Publish OpenAPI docs and examples for PP-41/PP-42

## 4. Deliverables

- Working analysis/API capability and documented contract.
- Reproducible outputs, representative examples, and version/provenance metadata.
- Automated tests, operational guidance, and known limitations.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Inputs, outputs, score semantics, and version metadata are documented.
- [ ] Invalid, missing, and unavailable data/backends are handled explicitly.
- [ ] Resource limits and observability are in place where applicable.
- [ ] Unit and integration tests pass on representative fixtures.

## 6. Testing strategy

- **Unit:** business rules, validation, mapping, and error handling.
- **Integration:** relevant database, graph, vector, model, and REST boundaries.
- **Quality:** reproducibility, referential integrity, metric correctness, and provenance.
- **Performance:** query latency, response size, and bounded resource use.

## 7. Cross-task handoff

- **Upstream:** PP-28 graph query service; PP-31 Milvus index; PP-32 similarity service; PP-20 canonical metadata.
- **Consumers / coordination:** PP-20, PP-28, PP-31, PP-32, PP-39, PP-41, PP-42
- **Contract rule:** publish API/schema versions, sample requests/responses, evaluation or quality results, and limitations. Coordinate breaking changes with all linked plans.

## 8. Completion notes

Record data/model/backend versions, commands, test results, sample outputs, and deviations.
