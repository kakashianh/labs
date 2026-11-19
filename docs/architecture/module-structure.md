# Module structure

## Target package layout

This is the intended logical organization under src/protein3d. It is a target, not a request to create empty packages ahead of their implementation tasks.

~~~text
src/protein3d/
├── domain/
│   ├── protein/
│   ├── interaction/
│   ├── annotation/
│   └── prediction/
├── ingestion/
│   ├── sources/
│   ├── adapters/
│   └── manifests/
├── processing/
│   ├── spark/
│   ├── normalization/
│   └── quality/
├── structures/
│   ├── parsers/
│   ├── coordinates/
│   └── features/
├── graph/
│   ├── repositories/
│   └── services/
├── embeddings/
│   ├── encoders/
│   └── repositories/
├── ml/
│   ├── datasets/
│   ├── models/
│   └── evaluation/
├── api/
│   ├── routes/
│   ├── schemas/
│   └── dependencies/
└── shared/
    ├── config/
    ├── logging/
    └── errors/
~~~

## Module responsibilities

| Module | Responsibility | Must not own |
|---|---|---|
| domain | Framework-independent entities, value objects, identifiers, and logical contracts | Database clients, web framework objects, Spark DataFrames |
| ingestion | Source clients/adapters, raw manifests, source-specific parsing | Canonical domain policy duplicated in individual connectors |
| processing | Batch transformations, normalization orchestration, data-quality checks | Online API request handling |
| structures | Structure file parsing, coordinate normalization, structural features | General source acquisition or model training |
| graph | Graph repository interfaces/adapters and graph-oriented services | Canonical protein identity policy |
| embeddings | Embedding generation adapters and vector repository integration | Owning generic protein/annotation schemas |
| ml | Dataset assembly, model lifecycle, evaluation, inference orchestration | API transport concerns |
| api | HTTP routes, request/response schemas, dependency wiring, application orchestration | Business rules embedded only in route handlers |
| shared | Cross-cutting configuration abstractions, logging setup, common errors | A catch-all for domain-specific business logic |

## Task ownership map

| Package area | Plan ownership |
|---|---|
| domain contracts and architecture | PP-10 |
| ingestion | PP-13–PP-17 |
| processing and canonical datasets | PP-17–PP-20 |
| structures | PP-21–PP-25 |
| graph | PP-26–PP-29 |
| embeddings | PP-30–PP-33 |
| ml | PP-34–PP-38 |
| api | PP-39–PP-40 |
| dashboard (outside Python package) | PP-41–PP-42 |

## Boundary guidance

- Keep source-specific fields in source adapters or provenance metadata; normalize shared concepts into domain contracts.
- Keep API DTOs separate from domain records. The API may omit or reshape fields without changing canonical data semantics.
- Keep Spark-specific schemas and transformations in processing modules. Domain contracts should remain usable without a Spark runtime.
- Keep graph and vector database representations behind repository/adapter boundaries.
- Do not create empty package directories solely to match this diagram. Add each package when its owning implementation task begins.
