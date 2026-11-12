# PP-13 — Implement AlphaFold structure data crawler

## 1. Objective

Download and catalog AlphaFold-predicted structures for the target human protein set with reproducible source provenance.

## 2. Scope, assumptions, and dependencies

PP-10 schema contracts; PP-12 metadata/config; PP-16 ingestion validation conventions.

Related plans: [PP-12](../PP-12%20Design%20metadata%20database%20schema%20and%20application%20configuration/plan.md) · [PP-14](../PP-14%20Implement%20STRING%20PPI%20data%20crawler/plan.md) · [PP-16](../PP-16%20Implement%20ingestion%20validation%20and%20raw%20data%20management/plan.md) · [PP-21](../PP-21%20Implement%20PDB%2FmmCIF%20structure%20parser/plan.md). Keep source versions, identifiers, and schemas consistent with these linked tasks.

## 3. Step-by-step implementation and to-do

### Step 1 — Define query scope and source contract

Confirm Homo sapiens (Taxonomy ID 9606), accession format, AlphaFold endpoint/version, pagination, and expected mmCIF/PDB availability.

**To-do**
- [ ] Define eligible protein/accession selection and configurable limits
- [ ] Document AlphaFold API/URL patterns and metadata fields
- [ ] Define retry, timeout, rate-limit, and user-agent policy

### Step 2 — Implement metadata discovery

Fetch structure metadata without downloading all files at once; normalize source identifiers and retain source response metadata.

**To-do**
- [ ] Implement paginated/iterative metadata retrieval
- [ ] Extract accession, model version, download URL, sequence length, and confidence metadata when available
- [ ] Make discovery resumable and idempotent

### Step 3 — Implement structure download and storage

Download files to raw storage using deterministic paths and atomic writes.

**To-do**
- [ ] Stream downloads to avoid loading large structures into memory
- [ ] Verify HTTP status, content type, non-zero size, and checksum
- [ ] Save source URL, retrieval timestamp, checksum, and file format
- [ ] Skip unchanged files and retry transient failures

### Step 4 — Validate crawler output

Run on a small accession sample and hand off records to ingestion validation.

**To-do**
- [ ] Test missing structure, 404, throttling, timeout, and corrupt-file cases
- [ ] Compare discovered and downloaded counts
- [ ] Emit a machine-readable manifest for PP-16
- [ ] Document resume/re-run procedure

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

- **Upstream:** PP-10 schema contracts; PP-12 metadata/config; PP-16 ingestion validation conventions.
- **Consumers:** PP-12, PP-14, PP-16, PP-21
- **Handoff rule:** publish schemas, manifests, source release, and known limitations; notify consumers if any contract changes.

## 8. Completion notes

Record source versions, parameters, record counts, quality metrics, test commands/results, and deviations.
