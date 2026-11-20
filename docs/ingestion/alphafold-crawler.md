# AlphaFold DB crawler (PP-13)

## Scope

The crawler reads a bounded, preselected cohort from PP-12 and retrieves AlphaFold DB predictions by resolved UniProt accession. It stores mmCIF objects in the configured MinIO bucket and records source metadata in PostgreSQL. Structure parsing and chain association are owned by PP-21.

The initial taxonomy default is NCBI Taxonomy ID `9606`; callers must select candidates from PP-12 and pass the project's `proteins.protein_id` UUID with each resolved UniProt accession. Do not pass accessions as internal protein IDs.

## Current source contract

- API base: `https://alphafold.ebi.ac.uk/api`
- Prediction endpoint: `/prediction/{UniProt accession}`
- Current fields consumed: `modelEntityId`, `cifUrl`, `sequenceStart`, `sequenceEnd`, `sequence`, `latestVersion`, `isUniProtReviewed`, `isUniProtReferenceProteome`.
- Coordinate URLs are taken from `cifUrl`; the crawler does not construct a URL from an accession or model version.
- Multiple records are preserved as distinct model entities.
- 404 means no prediction was found through this API. Network/API failures are reported separately.
- Very long proteins available only as proteome archive fragments are outside the individual-API crawler scope and must be reported as unsupported rather than as a confirmed absence.

Official references:
- [AlphaFold DB API schema](https://alphafold.ebi.ac.uk/api-doc)
- [AlphaFold DB downloads](https://alphafold.ebi.ac.uk/download)
- [AlphaFold DB release notes](https://www.ebi.ac.uk/pdbe/news/alphafold-database-release-notes)
- [AlphaFold DB API breaking changes](https://www.ebi.ac.uk/pdbe/news/breaking-changes-afdb-predictions-api)

## Persistence contract

- `structures.source`: `alphafold_db`
- `structures.source_structure_id`: AFDB `modelEntityId`
- `structures.source_version`: `latestVersion`, or `unversioned` if upstream omits it
- `structures.structure_format`: `mmcif`
- `structures.storage_uri`: S3 URI returned by PP-12 `S3ArtifactStorage`
- `structures.metadata_json.protein_id`: canonical internal UUID, retained here because PP-12 has no direct `structures.protein_id` FK
- `artifacts`: MinIO URI, mmCIF format/content type, size, SHA-256, and source version

PP-13 does not create `structure_protein_chains` rows. PP-21 creates those associations after parsing coordinates.

## Operational notes

- Configure object storage through the existing `PROTEIN3D_OBJECT_STORAGE_*` settings and AFDB controls through `PROTEIN3D_ALPHAFOLD_*`.
- The default cohort limit is 10 records per run; the CLI enforces a hard maximum of 10,000. Use a small, deterministic cohort first. Do not launch a full proteome crawl as a validation step.
- MinIO bucket provisioning is an infrastructure responsibility (PP-11); the crawler now creates the configured bucket if absent. The Compose/test image is pinned to `soldevelo/minio:2026.7.17`, an unofficial community-maintained packaging of the MinIO S3-compatible server; upstream MinIO Community Edition is archived, so treat this image as an explicit third-party dependency rather than an official MinIO release.
- Keep the JSONL manifest with the dataset run. Each row reports internal protein UUID, accession, model ID, source URL, sequence range, artifact checksum/size, and outcome.
- Re-run the same cohort to validate deterministic object keys and metadata upserts. Verify object readback and database consistency before declaring the run successful.

## Running a bounded crawl

After PP-11 Compose services are healthy, apply the PP-12 migrations and configure the `PROTEIN3D_DATABASE_*` and `PROTEIN3D_OBJECT_STORAGE_*` environment variables. The default taxonomy is `9606` and the default sample limit is `10`.

```bash
uv run --locked protein3d alphafold-crawl --sample-limit 10 --manifest data/manifests/alphafold-sample.jsonl
```

The command creates a `dataset_runs` row and a `jobs` row, records per-model outcomes in the JSONL manifest, and prints run/job IDs and counts. Outcomes include `downloaded`, `no_model`, `unsupported`, and `failed`. Proteins longer than 2,700 residues are reported as `unsupported` for this individual-API workflow rather than being mislabeled as no-model. Re-run the same command with the same cohort/manifest to exercise deterministic object keys and metadata upserts.

## Attribution

AlphaFold DB data is distributed under CC BY 4.0. Retain source URL, retrieval time, source model/version, and cite the AlphaFold DB publication applicable to the dataset release in reports and downstream exports.
