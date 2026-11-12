# PP-26 — Design JanusGraph protein interaction schema

## 1. Objective

Design a graph schema that represents canonical proteins and evidence-backed PPI relationships consistently.

## 2. Scope and dependencies

PP-10 shared entity contracts; PP-11 graph infrastructure; PP-20 protein/PPI Parquet datasets.

Related plans: [PP-10](../PP-10%20Design%20project%20architecture%20and%20module%20structure/plan.md) · [PP-11](../PP-11%20Configure%20Docker%20Compose%20infrastructure/plan.md) · [PP-19](../PP-19%20Clean%20and%20filter%20protein%20interaction%20datasets/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-27](../PP-27%20Import%20protein%20vertices%20and%20PPI%20edges/plan.md) · [PP-28](../PP-28%20Implement%20graph%20query%20and%20neighborhood%20services/plan.md) · [PP-29](../PP-29%20Implement%20PPI%20network%20analytics%20and%20community%20detection/plan.md) · [PP-36](../PP-36%20Implement%20combined%20feature%20and%20graph-based%20prediction/plan.md) · [PP-40](../PP-40%20Implement%20PPI%20exploration%20and%20similarity%20search%20APIs/plan.md). Keep graph identifiers, schema version, query semantics, and vector feature versions consistent across these tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Model graph entities and relationships

Define Protein vertices and INTERACTS edges with canonical identifiers and biological provenance.

**To-do**
- [ ] Define vertex label and canonical accession property
- [ ] Define edge label, confidence score, evidence, source release, and direction policy
- [ ] Decide whether self-interactions are retained
- [ ] Define property cardinality and nullability

### Step 2 — Define indexes and constraints

Design unique and mixed indexes based on expected access patterns and backend support.

**To-do**
- [ ] Define unique index for canonical protein accession
- [ ] Define indexes for confidence, source, and selected protein properties
- [ ] Define schema creation order and consistency settings
- [ ] Document backend-specific limitations

### Step 3 — Define graph query contract

Specify traversal patterns for neighbors, confidence filtering, degree, and community analytics.

**To-do**
- [ ] Define request/response shapes for one-hop and k-hop neighbors
- [ ] Define pagination and traversal limits
- [ ] Define stable graph IDs versus canonical protein IDs
- [ ] Document query timeout and safety limits

### Step 4 — Validate schema against data and workloads

Use a small PP-20 sample to verify the schema supports graph loading and intended traversals.

**To-do**
- [ ] Create schema in local JanusGraph
- [ ] Load a small synthetic/sample graph
- [ ] Test unique constraints and representative traversals
- [ ] Publish schema version and migration notes for PP-27–PP-29

## 4. Deliverables

- Versioned graph schema/service/import implementation for this ticket.
- Reproducible load/query examples and automated tests.
- Documentation of IDs, properties, query limits, and operational behavior.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Canonical protein accession remains the stable cross-system identifier.
- [ ] Imports/queries are bounded, observable, and safe to rerun where applicable.
- [ ] Graph data reconciles with the source dataset version.
- [ ] Unit and integration tests pass against a representative graph.

## 6. Testing strategy

- **Unit:** mapping, traversal construction, pagination, validation, and error handling.
- **Integration:** JanusGraph schema, import, and query operations.
- **Reconciliation:** vertex/edge counts, endpoint integrity, duplicate checks, and source lineage.
- **Performance:** measure batch throughput and query latency at representative graph size.

## 7. Cross-task handoff

- **Upstream:** PP-10 shared entity contracts; PP-11 graph infrastructure; PP-20 protein/PPI Parquet datasets.
- **Consumers / coordination:** PP-10, PP-11, PP-19, PP-20, PP-27, PP-28, PP-29, PP-36, PP-40
- **Contract rule:** publish graph schema version, key semantics, traversal contract, load/query metrics, and limitations. Coordinate schema changes with every consumer.

## 8. Completion notes

Record graph backend/version, source dataset version, load counts, query examples, test results, and deviations.
