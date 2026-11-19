# PP-13 — Implement AlphaFold structure data crawler

## 1. Objective

Discover and download AlphaFold Database (AFDB) predicted protein structures for the eligible protein cohort already registered in the PP-12 metadata database. Persist reproducible source metadata and mmCIF artifacts in MinIO, linked to the project's opaque internal protein UUIDs.

The crawler is a source-ingestion component only. It does not parse structures, assign chains, build graph/vector records, or train models.

## 2. Scope, assumptions, and dependencies

### Dependencies

- **PP-10:** canonical data contracts and identifier semantics.
- **PP-11:** local Docker Compose infrastructure, especially MinIO.
- **PP-12:** PostgreSQL models/migrations, typed configuration, artifact URI contract, and storage adapter.
- **PP-16:** ingestion validation and raw-data manifest conventions.
- **PP-21:** downstream mmCIF parsing and protein-chain association.

### Cohort and identity rules

- Read candidate proteins from PP-12 PostgreSQL metadata; do not create a second independent protein cohort.
- Initial project cohort is Homo sapiens (NCBI Taxonomy ID `9606`), but make taxonomy and sample/limit selection configurable rather than embedding the taxon throughout crawler code.
- Only query AFDB for proteins with a valid, resolved UniProt accession mapping.
- Use the PP-12 opaque `proteins.id` UUID as the internal `protein_id`. UniProt accessions and AFDB model identifiers remain external identifiers and must never be substituted for the internal UUID.
- Preserve unresolved, ambiguous, merged, deprecated, and missing mappings as explicit outcomes; do not turn them into “no structure”.
- Support one or more AFDB model records for an accession where the current API reports isoforms or fragments. Do not silently collapse distinct model entities.

### AlphaFold source contract

- Use the current official AlphaFold DB API contract and its current response field names. Do not depend on legacy fields such as `entryId`, `uniprotStart`, `uniprotEnd`, `uniprotSequence`, `isReviewed`, `isReferenceProteome`, or the removed `paeImageUrl`.
- Treat the official API schema as the contract; isolate response parsing in an adapter so upstream changes are localized.
- Obtain coordinate-file URLs from the API response. Do not synthesize download URLs or infer model versions from URL patterns.
- Prefer mmCIF as the primary stored coordinate format for PP-21. Store the exact source format and source URL in provenance.
- Record AFDB model entity identifier, UniProt accession, model/version information when supplied, sequence range, sequence length, reviewed/reference-proteome flags when supplied, and available confidence metadata without inventing missing values.
- AFDB API access is the supported retrieval path for individual models. Very long proteins represented only as overlapping proteome fragments and not exposed through the individual API must be reported as unsupported-by-API/skipped with a reason; bulk FTP proteome crawling is out of scope.
- Retain source attribution and the applicable AlphaFold DB license/publication citation in documentation and manifests.

### Out of scope

- PDB or STRING crawling, bulk FTP proteome downloads, Spark processing, structural parsing, chain-to-protein mapping, JanusGraph/Milvus writes, prediction/training, and full public API endpoints.
- Database schema changes. If PP-12 schema proves insufficient, stop and propose a separate migration/task rather than silently changing the schema here.

## 3. Step-by-step implementation and to-do

### Step 1 — Define cohort selection and API contract

Inspect PP-12 models and query the existing protein/mapping tables. Define deterministic selection, model identity, API response parsing, and the crawler's outcome categories.

**To-do**
- [ ] Implement configurable taxonomy filter (default `9606`), sample limit, and deterministic cohort ordering.
- [ ] Select only proteins with a resolved UniProt mapping; retain explicit skip reasons for other mapping states.
- [ ] Define typed models for current AFDB API response fields, including multiple model records/isoforms where returned.
- [ ] Document current endpoint, API schema reference, coordinate URL selection, and source metadata fields.
- [ ] Define distinct outcomes: discovered, downloaded, no model (confirmed 404), unsupported-by-API, skipped-invalid-mapping, retryable failure, and permanent failure.

### Step 2 — Implement the AlphaFold API client

Create a small HTTP client/adapter with bounded timeouts, conservative retries, and clear error classification.

**To-do**
- [ ] Implement configurable base URL, request timeout, retry count/backoff, and user-agent through typed application settings.
- [ ] Handle 404 as a confirmed no-model result; handle 429 and transient 5xx/network errors with bounded backoff and respect `Retry-After` when present.
- [ ] Treat malformed JSON, schema drift, missing required model identifiers, and invalid coordinate URLs as explicit parse/contract failures.
- [ ] Avoid retrying permanent client errors; redact credentials and avoid logging sensitive configuration.
- [ ] Add deterministic HTTP fixtures for current API response examples, including multiple model records.

### Step 3 — Implement metadata discovery and resumability

Normalize each API model record and associate it with the internal protein UUID without downloading coordinates yet.

**To-do**
- [ ] Extract model entity ID, accession, model version/release metadata, sequence start/end, sequence length, coordinate URLs, and available confidence/source metadata.
- [ ] Preserve raw source metadata needed for audit/debugging in the existing JSON metadata field where appropriate.
- [ ] Define a stable source structure identity and verify it against PP-12 uniqueness constraints before persistence.
- [ ] Make discovery safe to resume: repeated API responses update/reuse the same logical structure record rather than creating duplicates.
- [ ] Distinguish confirmed absence of a model from upstream/API/network failure.
- [ ] Preserve retrieval timestamp and source release/version when supplied; do not fabricate a version.

### Step 4 — Implement streaming download and MinIO storage

Download the selected mmCIF coordinate file from the API-provided URL and persist it through the PP-12 storage abstraction.

**To-do**
- [ ] Stream response bytes with bounded timeouts and size limits; do not buffer large structure files fully in memory.
- [ ] Validate HTTP status, expected content type (allow documented upstream variations), non-empty content, and basic mmCIF signature/format sanity.
- [ ] Calculate SHA-256 and byte size while streaming.
- [ ] Upload to MinIO using a deterministic, path-safe object key derived from source/model identity and version; never use untrusted URL path fragments directly.
- [ ] Use the configured MinIO bucket/prefix and existing storage URI contract; do not hardcode credentials or endpoints.
- [ ] Verify uploaded object metadata/readback in integration tests.
- [ ] Persist artifact success only after storage upload succeeds; leave failed transfers retryable and avoid dangling “successful” metadata.
- [ ] Skip unchanged artifacts by checksum and reuse existing object/metadata where safe.

### Step 5 — Integrate PP-12 metadata persistence

Use existing SQLAlchemy models and repository/storage abstractions. No schema migration is expected in this task.

**To-do**
- [ ] Create or update `structures` rows using the existing source/source-structure/version/format fields and internal protein UUID relationships supported by the schema.
- [ ] Create or update `artifacts` rows with backend, URI, format/content type, byte size, checksum algorithm/value, source provenance, and optional dataset/job linkage supported by PP-12.
- [ ] Associate each structure with the correct internal `protein_id`; never use a UniProt accession as the foreign key.
- [ ] Record run/job state and counts using the PP-12 lifecycle conventions where applicable.
- [ ] Make database writes transactional and idempotent; handle uniqueness conflicts by reloading/updating the existing logical record.
- [ ] Confirm actual PP-12 constraints and model field names before implementation. If a required relationship or field is absent, document the gap and request a separate schema change.

### Step 6 — Validation, tests, and sample execution

Test API, persistence, and object storage boundaries independently, then run a small reproducible sample.

**To-do**
- [ ] Unit-test cohort selection, API parsing, multiple model records, normalization, URL validation, error classification, checksum, and deterministic object-key generation.
- [ ] Cover 404, 429/Retry-After, 5xx, timeout, connection failure, malformed JSON, missing fields, invalid URL, empty response, and corrupt/non-mmCIF payload.
- [ ] Add PostgreSQL integration tests with Testcontainers for structure/artifact/run persistence, uniqueness, rollback, and repeated-run idempotency.
- [ ] Add MinIO integration tests for upload, metadata/checksum, readback, and retry after failed upload; use an actual MinIO service/container rather than mocks for this boundary.
- [ ] Test combined crawler flow with a small fixture-backed cohort and verify database-to-object-store consistency.
- [ ] Document and execute a small live sample against AFDB and the PP-11 Compose MinIO service when credentials/runtime are available; record exact sample size and results.
- [ ] Verify resume behavior by running the same sample twice and confirming no duplicate logical structures/artifacts.
- [ ] Emit a machine-readable manifest for PP-16 containing internal protein UUID, external accession, AFDB model ID/version, source URL, sequence range, format, storage URI, size, checksum, retrieval status, and skip/failure reason.

### Step 7 — Documentation and downstream handoff

Publish the operational and data contracts required by PP-16 and PP-21.

**To-do**
- [ ] Document setup/configuration, source attribution/license, rate-limit policy, small-sample execution, resume/retry, and troubleshooting.
- [ ] Document manifest schema and outcome/status semantics for PP-16.
- [ ] Document mmCIF artifact and metadata contract for PP-21; explicitly state that parsing and chain association happen downstream.
- [ ] Record known limitations, especially models/fragments unavailable through the individual API and upstream schema/version changes.
- [ ] Update this plan with test commands/results, sample counts, source metadata/version, and any deferred manual checks.

## 4. Deliverables

- Typed AlphaFold API adapter and current-response fixtures.
- Resumable/idempotent cohort discovery and mmCIF downloader.
- MinIO raw structure artifacts with SHA-256, size, and source provenance.
- PP-12 PostgreSQL structure/artifact/run metadata integration without schema changes.
- Machine-readable PP-16 manifest and PP-21 handoff contract.
- Unit/integration tests and operational documentation.

## 5. Acceptance criteria

- [ ] Cohort is sourced from PP-12 and selection is configurable and deterministic.
- [ ] Internal protein UUIDs and external UniProt/AFDB identifiers are never conflated.
- [ ] Current AFDB API fields and API-provided coordinate URLs are used; deprecated response fields and hand-built URLs are not relied upon.
- [ ] Multiple model entities/isoforms are represented without accidental collapsing.
- [ ] No-model, unsupported-by-API, invalid mapping, and transient/permanent failure outcomes are distinguishable.
- [ ] Valid mmCIF files are streamed to MinIO and verified by size/checksum/readback.
- [ ] PostgreSQL structure/artifact metadata remains consistent with object storage and uses existing PP-12 schema.
- [ ] Re-running the same cohort is idempotent and resumable.
- [ ] Manifest is consumable by PP-16 and artifact contract is documented for PP-21.
- [ ] Unit tests, PostgreSQL integration tests, and MinIO integration tests pass.
- [ ] A small live sample and resumed run are executed and their results recorded, or explicitly marked deferred with the reason (never reported as passed without execution).

## 6. Testing strategy

- **Unit:** API adapter/parser, selection, normalization, error classification, URL validation, checksums, object-key safety, manifest serialization.
- **PostgreSQL integration:** Testcontainers using the real PP-12 schema and migrations.
- **Object storage integration:** real MinIO service/container, upload/readback/checksum and failure/retry paths.
- **End-to-end sample:** small cohort against AFDB API and PP-11 Compose MinIO; repeat run to verify idempotency.
- **Quality gates:** Ruff, Mypy, pytest, migration checks, and relevant Compose validation.

## 7. Cross-task handoff

- **Upstream:** PP-10 contracts; PP-11 MinIO runtime; PP-12 metadata/config; PP-16 manifest/validation conventions.
- **Consumers:** PP-16 ingestion validation and raw-data management; PP-21 mmCIF parser.
- **Handoff rule:** publish stable manifest and artifact metadata semantics, source release/provenance, and known limitations. PP-21 owns coordinate parsing and chain association; PP-16 owns downstream raw-data quality validation.

## 8. Completion notes

Record AFDB API/schema reference and retrieval date, source release/version where available, cohort selection parameters, discovered/downloaded/skipped/failed counts, artifact checksum/size summaries, test commands/results, live sample and resume results, and deviations.
