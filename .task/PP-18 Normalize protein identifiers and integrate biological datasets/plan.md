# PP-18 — Normalize protein identifiers and integrate biological datasets

## 1. Objective

Join source datasets through canonical UniProt accessions while preserving cross-reference and provenance details.

## 2. Scope and dependencies

PP-12 internal protein UUID and mapping schema; PP-15 identifier mappings and GO annotations; PP-17 Spark readers; PP-10 shared schemas.

Related implementation plans: [PP-12](../PP-12%20Design%20metadata%20database%20schema%20and%20application%20configuration/plan.md) · [PP-15](../PP-15%20Integrate%20UniProt%20ID%20mapping%20and%20Gene%20Ontology%20annotations/plan.md) · [PP-17](../PP-17%20Build%20Spark%20ingestion%20pipeline/plan.md) · [PP-19](../PP-19%20Clean%20and%20filter%20protein%20interaction%20datasets/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-21](../PP-21%20Implement%20PDB%2FmmCIF%20structure%20parser/plan.md) · [PP-26](../PP-26%20Design%20JanusGraph%20protein%20interaction%20schema/plan.md) · [PP-34](../PP-34%20Prepare%20multi-label%20protein%20function%20prediction%20dataset/plan.md). These are explicit handoff points; align schemas and versions rather than creating parallel definitions.

## 3. Step-by-step implementation and to-do

### Step 1 — Define join rules and conflict policy

Specify accession normalization, isoform policy, organism constraints, and how conflicting source values are retained. The UniProt accession is an external join key; the project-owned `protein_id` UUID remains the persisted internal identity.

**To-do**
- [ ] Define `protein_id` UUID resolution and external accession normalization functions
- [ ] Specify handling for ambiguous or missing mappings
- [ ] Define deduplication precedence without losing source provenance
- [ ] Document taxonomy and isoform rules

### Step 2 — Implement mapping joins in Spark

Join AlphaFold, STRING, UniProt, and GO data using explicit cardinality checks; preserve the resolved internal UUID and source accessions.

**To-do**
- [ ] Join source IDs to canonical accessions
- [ ] Preserve original source identifiers in cross-reference columns
- [ ] Detect one-to-many and many-to-many join expansion
- [ ] Emit unmapped and ambiguous records to separate outputs

### Step 3 — Integrate protein metadata and annotations

Build consistent protein records and annotation relationships for downstream processing.

**To-do**
- [ ] Combine protein metadata and sequence attributes while preserving `protein_id` UUID and external accession mappings
- [ ] Attach GO annotations with evidence codes
- [ ] Keep annotation namespaces and source references
- [ ] Record source snapshot and transformation version

### Step 4 — Validate integration quality

Measure mapping coverage and inspect examples across all sources.

**To-do**
- [ ] Test known accession mappings and edge cases
- [ ] Report join coverage and cardinality changes per source
- [ ] Verify no cross-organism records leak into target dataset
- [ ] Publish canonical integrated dataset for PP-20 and PP-34

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

- **Upstream:** PP-15 identifier mappings and GO annotations; PP-17 Spark readers; PP-10 shared schemas.
- **Consumers / coordination:** PP-15, PP-17, PP-19, PP-20, PP-21, PP-26, PP-34
- **Contract rule:** publish schema, sample records, dataset/run version, quality report, and known limitations. Coordinate any breaking change with every linked consumer.

## 8. Completion notes

Record job parameters, source versions, output URIs, row counts, quality metrics, test results, and deviations.
