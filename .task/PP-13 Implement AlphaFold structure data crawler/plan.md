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
- Use the PP-12 opaque `proteins.protein_id` UUID as the internal `protein_id`. UniProt accessions and AFDB model identifiers remain external identifiers and must never be substituted for the internal UUID.
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
- Database schema changes. PP-12 has no direct `structures.protein_id` FK; PP-13 records the internal UUID in `structures.metadata_json.protein_id` until PP-21 creates chain associations. No migration is introduced.
- MinIO Community Edition's upstream container distribution has become unavailable to anonymous pulls; PP-11 Compose now uses the pinned `soldevelo/minio:2026.7.17` community-maintained packaging. Treat this as an explicit third-party S3-compatible dependency.

## 3. Step-by-step implementation and to-do

### Step 1 — Define cohort selection and API contract

Inspect PP-12 models and query the existing protein/mapping tables. Define deterministic selection, model identity, API response parsing, and the crawler's outcome categories.

**To-do**
- [x] Implement configurable taxonomy filter (default `9606`), sample limit (safe default 10; hard maximum 10,000), and deterministic cohort ordering.
- [x] Select only proteins with a resolved UniProt mapping; unresolved/ambiguous/deprecated mapping rows are excluded and are not mislabeled as no-model (per-mapping skip manifest rows remain a follow-up).
- [x] Define typed models for current AFDB API response fields, including multiple model records/isoforms where returned.
- [x] Document current endpoint, API schema reference, coordinate URL selection, and source metadata fields.
- [x] Define distinct outcomes for discovered/downloaded, confirmed no-model (404), unsupported-by-API, retryable failure, and permanent failure. Non-resolved mapping rows are excluded before querying AFDB and are never mislabeled as no-model; per-mapping skip manifest rows remain deferred.

### Step 2 — Implement the AlphaFold API client

Create a small HTTP client/adapter with bounded timeouts, conservative retries, and clear error classification.

**To-do**
- [x] Implement configurable base URL, request timeout, retry count/backoff, and user-agent through typed application settings.
- [x] Handle 404 as a confirmed no-model result; handle 429 and transient 5xx/network errors with bounded backoff and respect numeric/date `Retry-After` values.
- [x] Treat malformed JSON, schema drift, missing required model identifiers, and invalid coordinate URLs as explicit parse/contract failures.
- [x] Avoid retrying permanent client errors; redact credentials and avoid logging sensitive configuration.
- [x] Add deterministic HTTP fixtures for current API response examples, including multiple model records.

### Step 3 — Implement metadata discovery and resumability

Normalize each API model record and associate it with the internal protein UUID. Persist the logical structure/artifact metadata only after the coordinate object is uploaded and verified; no discovery-only success rows are created.

**To-do**
- [x] Extract model entity ID, accession, model version/release metadata, sequence start/end, sequence length, coordinate URLs, and available confidence/source metadata.
- [x] Preserve raw source metadata needed for audit/debugging in the existing JSON metadata field where appropriate.
- [x] Define a stable source structure identity and verify it against PP-12 uniqueness constraints before persistence.
- [x] Make persistence safe to resume: repeated API responses update/reuse the same logical structure and artifact records rather than creating duplicates. (Discovery-only rows are not created before a successful download.)
- [x] Distinguish confirmed absence of a model from upstream/API/network failure.
- [x] Preserve retrieval timestamp and source release/version when supplied; do not fabricate a version.

### Step 4 — Implement streaming download and MinIO storage

Download the selected mmCIF coordinate file from the API-provided URL and persist it through the PP-12 storage abstraction.

**To-do**
- [x] Stream response bytes to a temporary file with bounded timeouts and size limits; do not buffer large structure files fully in memory.
- [x] Validate HTTP status, accepted content-type variations, non-empty content, and a CIF `data_` block near the start before persisting an artifact.
- [x] Calculate SHA-256 and byte size while streaming.
- [x] Upload to MinIO in bounded chunks using a deterministic, path-safe object key derived from internal protein/model identity and version; never use untrusted URL path fragments directly.
- [x] Use the configured MinIO bucket/prefix and existing storage URI contract; do not hardcode credentials or endpoints.
- [x] Verify uploaded object size and streamed GET checksum/readback in a real-MinIO integration test.
- [x] Persist artifact success only after storage upload and readback verification succeed; leave failed transfers retryable and avoid dangling “successful” metadata.
- [x] Skip unchanged artifacts by checksum: HEAD size is checked, then an existing object is streamed back and reused only when its SHA-256 matches; otherwise it is replaced and verified.

### Step 5 — Integrate PP-12 metadata persistence

Use existing SQLAlchemy models and repository/storage abstractions. No schema migration is expected in this task.

**To-do**
- [x] Create or update `structures` rows using existing source/source-structure/version/format fields; PP-12 has no direct `structures.protein_id` FK, so the internal UUID is recorded in `structures.metadata_json.protein_id` until PP-21 creates chain associations.
- [x] Create or update `artifacts` rows with backend, URI, format/content type, byte size, checksum algorithm/value, source provenance, and dataset/job linkage.
- [x] Preserve the correct internal `protein_id` UUID in structure metadata; never use a UniProt accession as an internal key or fabricate chain associations before PP-21.
- [x] Record run/job state and counts using the PP-12 lifecycle conventions.
- [x] Make database writes transactional per completed model and idempotent through PP-12 structure/artifact unique keys.
- [x] Confirm actual PP-12 constraints and model field names before implementation. The missing direct structure/protein FK is documented and handled through `metadata_json`; chain associations remain PP-21 scope.

### Step 6 — Validation, tests, and sample execution

Test API, persistence, and object storage boundaries independently, then run a small reproducible sample.

**To-do**
- [x] Add unit coverage for API parsing, multiple model records, URL validation, checksum/streaming, object-key safety, and long-protein unsupported outcomes. Candidate selection and persistence are covered by PostgreSQL integration tests.
- [x] Cover 404, 429/Retry-After, 5xx, timeout, connection failure, malformed JSON, missing fields, invalid URL, empty response, and corrupt/non-mmCIF payload.
- [x] Add PostgreSQL Testcontainers integration coverage for deterministic candidate selection, structure/artifact persistence, metadata linkage, repeated-upsert idempotency, and rollback when MinIO upload fails.
- [x] Add a real MinIO Testcontainers integration test for bucket creation, streaming upload, HEAD size, streamed GET checksum/readback, and injected upload failure followed by successful retry.
- [x] Test combined configured crawler flow with a fixture-backed cohort and real PostgreSQL + MinIO containers; verify database/object consistency, dataset/job lifecycle, upload-failure rollback, retry success, checksum readback, and subsequent object reuse.
- [x] Live AFDB + user-local PP-11 Compose sample is explicitly deferred: this execution environment cannot access the user's local Compose runtime or its private database/MinIO credentials. The command and exact bounded sample procedure are documented; no live sample is claimed as passed.
- [x] Verify resume behavior by running the same fixture-backed cohort through a failed upload, successful retry, and subsequent rerun; the existing MinIO object is reused and no duplicate logical structures/artifacts are created. Live cohort repetition remains deferred with the live sample.
- [x] Emit a machine-readable JSONL manifest for PP-16 containing internal protein UUID, external accession, AFDB model ID/version, source URL, sequence range, format, artifact key, size, checksum, retrieval status, and unsupported/failure reason.

### Step 7 — Documentation and downstream handoff

Publish the operational and data contracts required by PP-16 and PP-21.

**To-do**
- [x] Document setup/configuration, source attribution/license, bounded sample execution, resume/retry, and troubleshooting. Rate-limit behavior is implemented in the client.
- [x] Document manifest outcome/status semantics for PP-16.
- [x] Document mmCIF artifact and metadata contract for PP-21; explicitly state that parsing and chain association happen downstream.
- [x] Record known limitations, especially models/fragments unavailable through the individual API and upstream schema/version changes.
- [x] Update this plan with automated test commands/results, sample fixture details, source metadata/version handling, and deferred manual checks.

## 4. Deliverables

- Typed AlphaFold API adapter and current-response fixtures.
- Resumable/idempotent cohort discovery and mmCIF downloader.
- MinIO raw structure artifacts with SHA-256, size, and source provenance.
- PP-12 PostgreSQL structure/artifact/run metadata integration without schema changes.
- Machine-readable PP-16 manifest and PP-21 handoff contract.
- Unit/integration tests and operational documentation.

## 5. Acceptance criteria

- [x] Cohort is sourced from PP-12 and selection is configurable and deterministic.
- [x] Internal protein UUIDs and external UniProt/AFDB identifiers are never conflated.
- [x] Current AFDB API fields and API-provided coordinate URLs are used; deprecated response fields and hand-built URLs are not relied upon.
- [x] Multiple model entities/isoforms are represented without accidental collapsing.
- [x] No-model, unsupported-by-API, and retryable/permanent failures are distinguishable. Non-resolved mappings are excluded from the AFDB cohort and are not mislabeled as no-model; per-mapping skip manifest rows are deferred.
- [x] Valid mmCIF files are streamed to MinIO and verified by size/checksum/readback.
- [x] PostgreSQL structure/artifact metadata remains consistent with object storage and uses existing PP-12 schema.
- [x] Re-running the same fixture-backed cohort is idempotent and resumable; the live AFDB cohort repetition is deferred.
- [x] Manifest is consumable by PP-16 and artifact contract is documented for PP-21.
- [x] Unit tests, PostgreSQL integration tests, and MinIO integration tests pass in GitHub Actions run `44656513813`.
- [x] A small live sample and resumed run are explicitly marked deferred with the reason (the user's local Compose runtime and secrets are not accessible from this execution environment; no live run is reported as passed).

## 6. Testing strategy

- **Unit:** API adapter/parser, selection, normalization, error classification, URL validation, checksums, object-key safety, manifest serialization.
- **PostgreSQL integration:** Testcontainers using the real PP-12 schema and migrations.
- **Object storage integration:** real MinIO-compatible service/container, bucket creation, upload/readback/checksum, deterministic object reuse, and injected upload-failure followed by a successful retry.
- **End-to-end sample:** fixture-backed one-protein cohort against Testcontainers PostgreSQL + MinIO-compatible storage; injected first upload failure leaves no structure/artifact metadata, the next run uploads successfully, and the third run reuses the object. A live AFDB + user-local Compose run is deferred as documented above.
- **Quality gates:** Ruff, Mypy, pytest, migration checks, and relevant Compose validation.

## 7. Cross-task handoff

- **Upstream:** PP-10 contracts; PP-11 MinIO runtime; PP-12 metadata/config; PP-16 manifest/validation conventions.
- **Consumers:** PP-16 ingestion validation and raw-data management; PP-21 mmCIF parser.
- **Handoff rule:** publish stable manifest and artifact metadata semantics, source release/provenance, and known limitations. PP-21 owns coordinate parsing and chain association; PP-16 owns downstream raw-data quality validation.

## 8. Completion notes

- **Automated validation:** GitHub Actions [run 44656513813](https://github.com/kakashianh/labs/actions/runs/44656513813) passed Ruff lint, Ruff format, Mypy, unit tests (47 passed), PostgreSQL/MinIO integration tests (6 passed), package/CLI smoke checks, and Docker Compose validation.
- **Reproduction commands:** `uv run --locked ruff check .`; `uv run --locked ruff format --check .`; `uv run --locked mypy src tests`; `uv run --locked pytest -m unit`; `uv run --locked pytest -m integration`.
- **Fixture-backed sample:** taxonomy `9606`, sample limit `1`, UniProt accession `Q9Y261`, AFDB model entity `AF-Q9Y261-F1`, source version `6`. The first configured run simulated a MinIO upload interruption and verified no structure/artifact row was committed; the second run uploaded and verified the mmCIF object; the third run reused the object after SHA-256 readback. Assertions confirmed one logical structure, one artifact, one failed then two successful dataset/job runs.
- **Source/API contract:** current AFDB fields and API-provided `cifUrl`; model version is persisted as returned. No live API response was used in the fixture test.
- **Deferred manual check:** run `uv run --locked protein3d alphafold-crawl --sample-limit 10 --manifest data/manifests/alphafold-live-sample.jsonl` from the user's PP-11 Compose/Codespaces environment after its database and MinIO services are available. This environment cannot access that private runtime or credentials, so live AFDB download and live second-run counts are not claimed as passed.
- **Known follow-up:** per-mapping skip manifest rows for unresolved/ambiguous/deprecated mappings are not implemented in PP-13. Those mapping rows are excluded from the AFDB cohort and are not mislabeled as no-model.
