# Protein3D-PPI Architecture

This directory is the logical architecture reference for Protein3D-PPI. It defines system boundaries, data flow, module responsibilities, shared data contracts, and architecture decisions.

## Start here

1. [System overview](system-overview.md) — system context, major components, and operating modes.
2. [Data flow](data-flow.md) — batch data lifecycle and online query path.
3. [Module structure](module-structure.md) — target Python package layout and ownership.
4. [Dependency rules](dependency-rules.md) — allowed dependency direction and adapter boundaries.
5. [Shared data contracts](data-contracts/README.md) — canonical identifiers and cross-module record semantics.
6. [Architecture decisions](decisions/README.md) — accepted ADRs and rationale.

## Scope and ownership

This documentation defines logical interfaces. It does not replace implementation plans:

| Concern | Owner |
|---|---|
| Repository conventions and developer tooling | PP-9 |
| Logical architecture and shared contracts | PP-10 |
| Local service topology and Docker Compose | PP-11 |
| PostgreSQL physical schema, migrations, application configuration | PP-12 |
| Ingestion and normalized datasets | PP-13–PP-20 |
| Structure parsing and structural features | PP-21–PP-25 |
| JanusGraph schema and graph operations | PP-26–PP-29 |
| Embeddings and vector search integration | PP-30–PP-33 |
| ML datasets, models, and evaluation | PP-34–PP-38 |
| REST API implementation | PP-39–PP-40 |
| Dashboard and visualization | PP-41–PP-42 |

## Contract change policy

- Treat the contracts in this directory as the logical source of truth for cross-module semantics.
- A change to a shared identifier or field meaning must update the owning contract and all affected consumer plans in the same coordinated change.
- Physical database, graph, vector, and API representations remain with their owning implementation tasks.
- Record material architecture changes as a new or amended ADR. Do not silently rewrite an accepted decision.

## Technology baseline

The project baseline is Python, PySpark, Biopython/Biotite, PostgreSQL, JanusGraph, Milvus, MinIO-compatible object storage, and FastAPI. See [ADR-001](decisions/ADR-001-architecture-style.md) and [ADR-003](decisions/ADR-003-data-storage-boundaries.md) for the architectural rationale and boundaries.

## Status

This architecture is the initial logical design baseline. Implementation tasks may refine technology-specific details while preserving the shared semantics documented here.
