# PP-27 — Import protein vertices and PPI edges

## 1. Objective

Load curated protein and interaction datasets into JanusGraph with idempotency and reconciliation.

## 2. Scope and dependencies

PP-26 approved graph schema; PP-20 versioned Parquet datasets; PP-11 graph runtime.

Related plans: [PP-12](../PP-12%20Design%20metadata%20database%20schema%20and%20application%20configuration/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-26](../PP-26%20Design%20JanusGraph%20protein%20interaction%20schema/plan.md) · [PP-28](../PP-28%20Implement%20graph%20query%20and%20neighborhood%20services/plan.md) · [PP-29](../PP-29%20Implement%20PPI%20network%20analytics%20and%20community%20detection/plan.md) · [PP-36](../PP-36%20Implement%20combined%20feature%20and%20graph-based%20prediction/plan.md) · [PP-40](../PP-40%20Implement%20PPI%20exploration%20and%20similarity%20search%20APIs/plan.md). Keep graph identifiers, schema version, query semantics, and vector feature versions consistent across these tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Design import mapping and run protocol

Map Parquet columns to graph properties and define batch size, checkpoint, and rerun behavior.

**To-do**
- [ ] Map project-owned `protein_id` UUID to vertex key and store UniProt accession as an indexed external property
- [ ] Map PPI edge key, UUID endpoints, confidence, and source provenance properties
- [ ] Define batch size, retry, and checkpoint strategy
- [ ] Record source dataset version and graph load run ID

### Step 2 — Implement vertex upsert

Load proteins before edges and make repeated imports safe.

**To-do**
- [ ] Read protein master in bounded batches
- [ ] Upsert by canonical accession using schema index
- [ ] Track created/updated/skipped/failed counts
- [ ] Quarantine invalid records and preserve failure details

### Step 3 — Implement edge upsert

Load valid interactions after endpoint vertices exist.

**To-do**
- [ ] Verify both endpoint vertices before edge creation
- [ ] Upsert by stable edge identity and documented direction policy
- [ ] Preserve confidence and evidence/source metadata
- [ ] Checkpoint completed batches and support resume

### Step 4 — Reconcile graph against Parquet source

Compare counts, endpoints, scores, and representative traversals.

**To-do**
- [ ] Compare protein and edge counts with PP-20 manifests
- [ ] Check missing vertices, duplicate edges, and invalid references
- [ ] Test representative neighbor traversals
- [ ] Publish load report and graph dataset version

## 4. Deliverables

- Versioned graph schema/service/import implementation for this ticket.
- Reproducible load/query examples and automated tests.
- Documentation of IDs, properties, query limits, and operational behavior.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Project-owned `protein_id` UUID remains the stable internal identity; UniProt accession is retained as an external property.
- [ ] Imports/queries are bounded, observable, and safe to rerun where applicable.
- [ ] Graph data reconciles with the source dataset version.
- [ ] Unit and integration tests pass against a representative graph.

## 6. Testing strategy

- **Unit:** mapping, traversal construction, pagination, validation, and error handling.
- **Integration:** JanusGraph schema, import, and query operations.
- **Reconciliation:** vertex/edge counts, endpoint integrity, duplicate checks, and source lineage.
- **Performance:** measure batch throughput and query latency at representative graph size.

## 7. Cross-task handoff

- **Upstream:** PP-26 approved graph schema; PP-20 versioned Parquet datasets; PP-11 graph runtime.
- **Consumers / coordination:** PP-20, PP-26, PP-28, PP-29, PP-36, PP-40
- **Contract rule:** publish graph schema version, key semantics, traversal contract, load/query metrics, and limitations. Coordinate schema changes with every consumer.

## 8. Completion notes

Record graph backend/version, source dataset version, load counts, query examples, test results, and deviations.
