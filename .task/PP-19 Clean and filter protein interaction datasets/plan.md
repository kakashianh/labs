# PP-19 — Clean and filter protein interaction datasets

## 1. Objective

Produce a biologically consistent high-confidence human PPI edge dataset suitable for graph construction and modeling.

## 2. Scope and dependencies

PP-14 STRING raw interactions; PP-15 identifier mappings; PP-17 Spark pipeline; PP-18 canonical IDs.

Related implementation plans: [PP-14](../PP-14%20undefined/plan.md) · [PP-15](../PP-15%20undefined/plan.md) · [PP-17](../PP-17%20Build%20Spark%20ingestion%20pipeline/plan.md) · [PP-18](../PP-18%20Normalize%20protein%20identifiers%20and%20integrate%20biological%20datasets/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-26](../PP-26%20Design%20JanusGraph%20protein%20interaction%20schema/plan.md) · [PP-27](../PP-27%20Import%20protein%20vertices%20and%20PPI%20edges/plan.md) · [PP-29](../PP-29%20Implement%20PPI%20network%20analytics%20and%20community%20detection/plan.md) · [PP-36](../PP-36%20Implement%20combined%20feature%20and%20graph-based%20prediction/plan.md). These are explicit handoff points; align schemas and versions rather than creating parallel definitions.

## 3. Step-by-step implementation and to-do

### Step 1 — Define PPI cleaning rules

Specify organism, confidence threshold, self-loop policy, duplicate edge policy, and evidence retention.

**To-do**
- [ ] Configure human organism filter and STRING release
- [ ] Set default high-confidence cutoff (e.g. combined score >=700), configurable per run
- [ ] Decide undirected edge canonicalization
- [ ] Define handling for self-interactions and duplicate evidence

### Step 2 — Normalize endpoints and filter records

Map both interaction endpoints to canonical protein IDs and reject invalid edges with reason codes.

**To-do**
- [ ] Map STRING protein IDs using PP-15/PP-18 outputs
- [ ] Remove or quarantine unmapped endpoints
- [ ] Apply confidence and organism filters
- [ ] Preserve original scores and evidence metadata

### Step 3 — Deduplicate and derive graph-ready edges

Create stable edge keys and ensure scores are represented consistently.

**To-do**
- [ ] Canonicalize endpoint ordering for undirected interactions
- [ ] Aggregate duplicate source rows according to documented policy
- [ ] Preserve max/combined score and evidence provenance
- [ ] Generate stable edge identifiers

### Step 4 — Profile and publish

Summarize network quality and provide validated edge data to graph and ML tasks.

**To-do**
- [ ] Report node/edge counts and score distribution
- [ ] Measure unmapped endpoint and duplicate rates
- [ ] Test edge uniqueness, endpoint validity, and score bounds
- [ ] Publish versioned PPI dataset for PP-20 and PP-26–PP-29

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

- **Upstream:** PP-14 STRING raw interactions; PP-15 identifier mappings; PP-17 Spark pipeline; PP-18 canonical IDs.
- **Consumers / coordination:** PP-14, PP-15, PP-17, PP-18, PP-20, PP-26, PP-27, PP-29, PP-36
- **Contract rule:** publish schema, sample records, dataset/run version, quality report, and known limitations. Coordinate any breaking change with every linked consumer.

## 8. Completion notes

Record job parameters, source versions, output URIs, row counts, quality metrics, test results, and deviations.
