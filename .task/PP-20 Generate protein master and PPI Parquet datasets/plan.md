# PP-20 — Generate protein master and PPI Parquet datasets

## 1. Objective

Publish the curated canonical protein table and graph-ready PPI table as versioned Parquet datasets.

## 2. Scope and dependencies

PP-18 integrated protein data; PP-19 cleaned PPI edges; PP-17 Spark orchestration; PP-12 metadata.

Related implementation plans: [PP-17](../PP-17%20Build%20Spark%20ingestion%20pipeline/plan.md) · [PP-18](../PP-18%20Normalize%20protein%20identifiers%20and%20integrate%20biological%20datasets/plan.md) · [PP-19](../PP-19%20Clean%20and%20filter%20protein%20interaction%20datasets/plan.md) · [PP-21](../PP-21%20Implement%20PDB%2FmmCIF%20structure%20parser/plan.md) · [PP-26](../PP-26%20Design%20JanusGraph%20protein%20interaction%20schema/plan.md) · [PP-27](../PP-27%20Import%20protein%20vertices%20and%20PPI%20edges/plan.md) · [PP-30](../PP-30%20Design%20protein%20embedding%20generation%20pipeline/plan.md) · [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md). These are explicit handoff points; align schemas and versions rather than creating parallel definitions.

## 3. Step-by-step implementation and to-do

### Step 1 — Define curated dataset schemas

Freeze column names, types, nullability, partition keys, and dataset versioning rules.

**To-do**
- [ ] Define protein master schema and stable primary key
- [ ] Define PPI edge schema and stable edge key
- [ ] Include source/run lineage and quality flags
- [ ] Document schema evolution policy

### Step 2 — Implement Parquet writers

Write compressed, partitioned datasets with atomic publish semantics.

**To-do**
- [ ] Write protein master to versioned Parquet location
- [ ] Write PPI edges to versioned Parquet location
- [ ] Select partitioning based on access patterns and avoid tiny files
- [ ] Publish manifest only after successful output validation

### Step 3 — Add dataset validation and catalog metadata

Validate schemas, counts, relationships, and artifact checksums before marking a run complete.

**To-do**
- [ ] Check unique protein and edge keys
- [ ] Check every PPI endpoint exists in protein master
- [ ] Check non-null required fields and score ranges
- [ ] Record dataset version, source versions, counts, and checksums

### Step 4 — Prepare downstream handoff

Provide stable sample queries and dataset references to structural, graph, embedding, and ML tasks.

**To-do**
- [ ] Add sample PySpark/Parquet read examples
- [ ] Document refresh and rollback procedure
- [ ] Publish representative fixture dataset
- [ ] Notify PP-21, PP-26, PP-30, and PP-34 of schema freeze

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

- **Upstream:** PP-18 integrated protein data; PP-19 cleaned PPI edges; PP-17 Spark orchestration; PP-12 metadata.
- **Consumers / coordination:** PP-17, PP-18, PP-19, PP-21, PP-26, PP-27, PP-30, PP-34
- **Contract rule:** publish schema, sample records, dataset/run version, quality report, and known limitations. Coordinate any breaking change with every linked consumer.

## 8. Completion notes

Record job parameters, source versions, output URIs, row counts, quality metrics, test results, and deviations.
