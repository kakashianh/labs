# PP-16 — Implement ingestion validation and raw data management

## 1. Objective

Create a consistent raw-data landing and validation layer for all crawlers before processing.

## 2. Scope, assumptions, and dependencies

PP-10 source contracts; PP-11 storage services; PP-12 metadata/config; PP-13–PP-15 crawlers.

Related plans: [PP-13](../PP-13%20Implement%20AlphaFold%20structure%20data%20crawler/plan.md) · [PP-14](../PP-14%20Implement%20STRING%20PPI%20data%20crawler/plan.md) · [PP-15](../PP-15%20Integrate%20UniProt%20ID%20mapping%20and%20Gene%20Ontology%20annotations/plan.md) · [PP-17](../PP-17%20Build%20Spark%20ingestion%20pipeline/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-21](../PP-21%20Implement%20PDB%2FmmCIF%20structure%20parser/plan.md). Keep source versions, identifiers, and schemas consistent with these linked tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Define raw-zone layout and manifests

Standardize source/version/date partitions, immutable raw objects, and manifest metadata.

**To-do**
- [ ] Define directory/object naming convention per source and release
- [ ] Define manifest schema with URI, checksum, size, retrieval time, and source parameters
- [ ] Separate raw, validated, and rejected records

### Step 2 — Build reusable validation framework

Implement schema, required-field, type, range, and referential checks without silently dropping records.

**To-do**
- [ ] Add validators for structure metadata, PPI rows, ID mappings, and GO annotations
- [ ] Classify errors as reject, quarantine, or warning
- [ ] Produce row-level reason codes and aggregate quality reports

### Step 3 — Implement integrity and idempotency

Protect ingestion from duplicate downloads, partial files, and accidental source replacement.

**To-do**
- [ ] Verify checksums and atomic file writes
- [ ] Detect duplicate source records using stable keys
- [ ] Preserve source snapshots and version lineage
- [ ] Make re-ingestion safe and repeatable

### Step 4 — Test source-to-raw workflow

Run all crawlers through common validation and publish quality summaries.

**To-do**
- [ ] Test malformed, missing, duplicate, and unexpected schema fields
- [ ] Verify rejected records are inspectable and recoverable
- [ ] Verify manifests reference exact raw objects
- [ ] Publish validated datasets and quality report for PP-17

## 4. Deliverables

- Working crawler/validation capability within this ticket's scope.
- Versioned raw/normalized output, manifests, and source provenance.
- Automated tests, representative fixtures, and operational documentation.

## 5. Acceptance criteria

- [ ] Every step to-do is complete.
- [ ] Retrieval is reproducible, resumable, and idempotent where applicable.
- [ ] Source version, parameters, checksums, and provenance are retained.
- [ ] Invalid or ambiguous records are reported, not silently discarded.
- [ ] Unit and integration tests pass; sample output is validated.

## 6. Testing strategy

- **Unit:** adapters, parsers, validators, normalization, and error classification.
- **Integration:** source API/storage and metadata database boundaries.
- **Data quality:** counts, uniqueness, required fields, identifier coverage, and score ranges.
- **Manual:** execute a small sample and a resumed run from documented instructions.

## 7. Cross-task handoff

- **Upstream:** PP-10 source contracts; PP-11 storage services; PP-12 metadata/config; PP-13–PP-15 crawlers.
- **Consumers:** PP-13, PP-14, PP-15, PP-17, PP-20, PP-21
- **Handoff rule:** publish schemas, manifests, source release, and known limitations; notify consumers if any contract changes.

## 8. Completion notes

Record source versions, parameters, record counts, quality metrics, test commands/results, and deviations.
