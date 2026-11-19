# ADR-003: Data and artifact storage boundaries

- Status: Accepted
- Date: 2026-11-19
- Owners: PP-10 architecture

## Context

The platform stores relational metadata, interaction relationships, vector embeddings, large biological/columnar files, and model artifacts. Putting every payload in one database would create inappropriate coupling and inefficient access patterns.

## Decision

Use stores according to access pattern and data responsibility:

| Store | Logical responsibility |
|---|---|
| PostgreSQL | Relational metadata, provenance, job/run records, artifact references, and application-owned relational state |
| JanusGraph | Protein interaction graph and graph traversal workloads |
| Milvus | Embedding vectors and similarity-search index |
| MinIO-compatible object storage | Large raw files, PDB/mmCIF, Parquet datasets, model artifacts, and other immutable/versioned artifacts |

Relational metadata should reference large artifacts using URI, checksum, format, size, and provenance where applicable. It should not duplicate large binary or columnar payloads.

## Consequences

- Each store has a clear access-pattern boundary and an owning implementation plan.
- Cross-store consistency is coordinated through identifiers, version metadata, and reproducible jobs rather than assuming a distributed transaction.
- PP-11 provisions local services; PP-12 owns PostgreSQL schema/configuration; PP-20 and later tasks implement datasets and store-specific schemas/integrations.
- Object storage URI conventions and lifecycle policies must be finalized by the relevant implementation task.
