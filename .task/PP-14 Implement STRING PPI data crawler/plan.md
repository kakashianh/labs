# PP-14 — Implement STRING PPI data crawler

## 1. Objective

Acquire high-confidence protein-protein interaction records with evidence and score provenance.

## 2. Scope, assumptions, and dependencies

PP-10 shared protein/PPI contracts; PP-12 configuration; PP-16 raw data conventions.

Related plans: [PP-12](../PP-12%20Design%20metadata%20database%20schema%20and%20application%20configuration/plan.md) · [PP-15](../PP-15%20Integrate%20UniProt%20ID%20mapping%20and%20Gene%20Ontology%20annotations/plan.md) · [PP-16](../PP-16%20Implement%20ingestion%20validation%20and%20raw%20data%20management/plan.md) · [PP-17](../PP-17%20/plan.md) · [PP-19](../PP-19%20/plan.md) · [PP-26](../PP-26%20/plan.md). These links describe the upstream/downstream contracts; coordinate changes rather than duplicating or redefining them.

## 3. Step-by-step implementation and to-do

### Step 1 — Define STRING query and confidence policy

Set organism to human (9606), pin STRING release, and define configurable confidence threshold (initial target >=700) and evidence policy.

**To-do**
- [ ] Record STRING release and API version
- [ ] Configure organism and confidence cutoff
- [ ] Decide whether to retain combined score and evidence-channel scores
- [ ] Define identifier mapping behavior for STRING protein IDs

### Step 2 — Implement interaction retrieval

Retrieve interactions in batches and preserve original source identifiers and evidence.

**To-do**
- [ ] Implement batch query and pagination/partitioning
- [ ] Capture endpoint parameters and retrieval metadata
- [ ] Handle rate limits, transient errors, and partial batch failures
- [ ] Support resumable retrieval

### Step 3 — Normalize and persist raw interaction data

Write immutable raw response/normalized landing records before downstream transformation.

**To-do**
- [ ] Preserve source STRING IDs for both endpoints
- [ ] Store combined score and available evidence channels
- [ ] Write deterministic partitioned files and manifest
- [ ] Avoid collapsing duplicate evidence prematurely

### Step 4 — Validate dataset handoff

Check organism, scores, endpoint validity, and coverage before Spark processing.

**To-do**
- [ ] Test self-loop, duplicate, missing-ID, and out-of-range score cases
- [ ] Report counts by confidence range
- [ ] Verify representative IDs can be mapped in PP-15
- [ ] Publish schema and manifest for PP-17/PP-19

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

- **Upstream:** PP-10 shared protein/PPI contracts; PP-12 configuration; PP-16 raw data conventions.
- **Consumers:** PP-12, PP-15, PP-16, PP-17, PP-19, PP-26
- **Handoff rule:** publish schemas, manifests, source release, and known limitations; notify consumers if any contract changes.

## 8. Completion notes

Record source versions, parameters, record counts, quality metrics, test commands/results, and deviations.
