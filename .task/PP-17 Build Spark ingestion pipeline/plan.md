# PP-17 — Build Spark ingestion pipeline

## 1. Objective

Create a repeatable PySpark batch pipeline that reads validated source manifests and writes curated, typed datasets.

## 2. Scope and dependencies

PP-16 validated raw manifests; PP-10 schema contracts; PP-11 runtime; PP-12 configuration.

Related implementation plans: [PP-16](../PP-16%20undefined/plan.md) · [PP-18](../PP-18%20Normalize%20protein%20identifiers%20and%20integrate%20biological%20datasets/plan.md) · [PP-19](../PP-19%20Clean%20and%20filter%20protein%20interaction%20datasets/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-21](../PP-21%20Implement%20PDB%2FmmCIF%20structure%20parser/plan.md) · [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md). These are explicit handoff points; align schemas and versions rather than creating parallel definitions.

## 3. Step-by-step implementation and to-do

### Step 1 — Set up Spark execution contract

Define Spark version, local execution mode, input manifest format, checkpoint strategy, and output partitions.

**To-do**
- [ ] Pin Spark/PySpark and Java compatibility
- [ ] Define job parameters for source snapshot, run ID, and output location
- [ ] Define structured job logs and metrics
- [ ] Define retry and failure semantics

### Step 2 — Implement source readers

Read AlphaFold metadata, STRING interactions, UniProt mappings, and GO annotations with explicit schemas.

**To-do**
- [ ] Implement schema-first readers for each validated source
- [ ] Read only manifest-listed files
- [ ] Validate schema compatibility before transformations
- [ ] Capture source release and run lineage

### Step 3 — Build orchestration and transformations

Create composable Spark stages with deterministic transformations and intermediate quality checks.

**To-do**
- [ ] Implement pipeline entry point and stage configuration
- [ ] Add partition-aware processing and avoid unnecessary collect()
- [ ] Add bad-record quarantine and stage-level metrics
- [ ] Make reruns deterministic for the same inputs

### Step 4 — Run sample and scale checks

Execute on a small fixed dataset, inspect plans, and estimate resource needs for the target corpus.

**To-do**
- [ ] Add Spark unit tests using local mode
- [ ] Compare output counts to source manifests
- [ ] Inspect shuffle, partition skew, and memory usage
- [ ] Publish curated stage outputs for PP-18–PP-20

## 4. Deliverables

- Tested implementation and reproducible job/output for this ticket.
- Versioned schemas, artifacts, and provenance metadata.
- Documentation and downstream handoff notes.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Output is deterministic for a fixed input snapshot and configuration.
- [ ] Data quality, schema, and lineage checks pass.
- [ ] Failure and invalid-data behavior is explicit and observable.
- [ ] Unit, integration, and representative data tests pass.

## 6. Testing strategy

- **Unit:** transformations, joins, filters, schema rules, and edge cases.
- **Integration:** Spark and relevant storage/database boundaries.
- **Data regression:** stable counts, uniqueness, referential integrity, and fixed-sample comparisons.
- **Performance:** inspect partitioning, shuffle, skew, and memory on representative volume.

## 7. Cross-task handoff

- **Upstream:** PP-16 validated raw manifests; PP-10 schema contracts; PP-11 runtime; PP-12 configuration.
- **Consumers / coordination:** PP-16, PP-18, PP-19, PP-20, PP-21, PP-34
- **Contract rule:** publish schema, sample records, dataset/run version, quality report, and known limitations. Coordinate any breaking change with every linked consumer.

## 8. Completion notes

Record job parameters, source versions, output URIs, row counts, quality metrics, test results, and deviations.
