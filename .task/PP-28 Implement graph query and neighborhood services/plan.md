# PP-28 — Implement graph query and neighborhood services

## 1. Objective

Expose reusable graph operations for protein neighborhood retrieval and downstream graph feature/model workflows.

## 2. Scope and dependencies

PP-26 graph schema; PP-27 imported graph; PP-10 API/service boundary definitions.

Related plans: [PP-26](../PP-26%20Design%20JanusGraph%20protein%20interaction%20schema/plan.md) · [PP-27](../PP-27%20Import%20protein%20vertices%20and%20PPI%20edges/plan.md) · [PP-29](../PP-29%20Implement%20PPI%20network%20analytics%20and%20community%20detection/plan.md) · [PP-36](../PP-36%20Implement%20combined%20feature%20and%20graph-based%20prediction/plan.md) · [PP-39](../PP-39%20Implement%20protein%20search%20and%20detail%20REST%20APIs/plan.md) · [PP-40](../PP-40%20Implement%20PPI%20exploration%20and%20similarity%20search%20APIs/plan.md). Keep graph identifiers, schema version, query semantics, and vector feature versions consistent across these tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Define query service contract

Specify lookup by accession, neighbor depth, confidence cutoff, pagination, and response limits.

**To-do**
- [ ] Define typed request/response models
- [ ] Define one-hop and bounded k-hop semantics
- [ ] Define score filtering and deterministic ordering
- [ ] Define not-found, timeout, and partial-result behavior

### Step 2 — Implement repository and traversal layer

Separate JanusGraph traversal code from service/business logic.

**To-do**
- [ ] Implement accession-to-vertex lookup
- [ ] Implement one-hop neighbor query with edge properties
- [ ] Implement bounded multi-hop traversal with cycle protection
- [ ] Add pagination/limit guards and query timeouts

### Step 3 — Add resilience and observability

Make graph access failures explicit and diagnosable.

**To-do**
- [ ] Add connection lifecycle and retry policy for transient errors
- [ ] Add structured logs and query latency/count metrics
- [ ] Avoid logging sensitive connection strings
- [ ] Define graceful behavior when graph backend is unavailable

### Step 4 — Test and publish service interface

Verify graph service output against known fixture and downstream expectations.

**To-do**
- [ ] Unit-test query building and response mapping
- [ ] Integration-test against local JanusGraph
- [ ] Verify stable canonical IDs and edge confidence fields
- [ ] Document interface for PP-29, PP-36, PP-39, and PP-40

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

- **Upstream:** PP-26 graph schema; PP-27 imported graph; PP-10 API/service boundary definitions.
- **Consumers / coordination:** PP-26, PP-27, PP-29, PP-36, PP-39, PP-40
- **Contract rule:** publish graph schema version, key semantics, traversal contract, load/query metrics, and limitations. Coordinate schema changes with every consumer.

## 8. Completion notes

Record graph backend/version, source dataset version, load counts, query examples, test results, and deviations.
