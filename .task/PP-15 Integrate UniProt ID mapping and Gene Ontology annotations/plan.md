# PP-15 — Integrate UniProt ID mapping and Gene Ontology annotations

## 1. Objective

Resolve external protein identifiers to canonical UniProt accessions and collect GO annotations for supervised labels.

## 2. Scope, assumptions, and dependencies

PP-10 canonical protein/annotation contracts; PP-12 configuration; PP-14 STRING IDs.

Related plans: [PP-13](../PP-13%20Implement%20AlphaFold%20structure%20data%20crawler/plan.md) · [PP-14](../PP-14%20Implement%20STRING%20PPI%20data%20crawler/plan.md) · [PP-16](../PP-16%20Implement%20ingestion%20validation%20and%20raw%20data%20management/plan.md) · [PP-18](../PP-18%20/plan.md) · [PP-20](../PP-20%20/plan.md) · [PP-34](../PP-34%20/plan.md). These links describe the upstream/downstream contracts; coordinate changes rather than duplicating or redefining them.

## 3. Step-by-step implementation and to-do

### Step 1 — Define canonical identifier strategy

Use UniProt accession as canonical protein key while retaining STRING, Ensembl, and other source identifiers as cross-references.

**To-do**
- [ ] Define accession normalization and isoform handling
- [ ] Specify mapping confidence and ambiguous/unmapped states
- [ ] Define taxonomy filtering and reviewed/unreviewed policy

### Step 2 — Implement identifier mapping

Integrate UniProt ID mapping or appropriate UniProt endpoints with batching and caching.

**To-do**
- [ ] Implement source-ID to UniProt mapping adapter
- [ ] Cache mapping responses with source/version metadata
- [ ] Handle one-to-many, many-to-one, and unmapped IDs explicitly
- [ ] Add retry and rate-limit handling

### Step 3 — Retrieve and normalize GO annotations

Collect GO terms with evidence codes, ontology namespace, and provenance.

**To-do**
- [ ] Retrieve molecular function, biological process, and cellular component annotations
- [ ] Preserve evidence code, qualifier, reference, and annotation source
- [ ] Filter by target organism and define evidence inclusion policy
- [ ] Normalize GO identifiers and obsolete-term handling

### Step 4 — Publish mapping and label datasets

Produce joinable mapping and annotation tables for Spark and ML preparation.

**To-do**
- [ ] Generate mapping table with canonical accession and source IDs
- [ ] Generate GO annotation table with evidence metadata
- [ ] Report mapping coverage and ambiguity rates
- [ ] Provide fixtures for PP-18 and PP-34

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

- **Upstream:** PP-10 canonical protein/annotation contracts; PP-12 configuration; PP-14 STRING IDs.
- **Consumers:** PP-13, PP-14, PP-16, PP-18, PP-20, PP-34
- **Handoff rule:** publish schemas, manifests, source release, and known limitations; notify consumers if any contract changes.

## 8. Completion notes

Record source versions, parameters, record counts, quality metrics, test commands/results, and deviations.
