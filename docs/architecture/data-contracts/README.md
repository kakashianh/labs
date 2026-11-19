# Shared data contracts

These documents define logical, cross-module meanings. They are not physical PostgreSQL DDL, JanusGraph schema, Milvus collection definitions, Parquet schemas, or FastAPI response models. Those representations belong to PP-12, PP-20, PP-26, PP-30, and PP-39 respectively.

## Contract index

- [Protein](protein.md)
- [Protein ID mapping](protein-id-mapping.md)
- [PPI edge](ppi-edge.md)
- [GO annotation](go-annotation.md)
- [Structure metadata](structure-metadata.md)
- [Feature vector](feature-vector.md)
- [Embedding](embedding.md)
- [Prediction](prediction.md)

## Common conventions

- Use UTF-8 text and ISO 8601 timestamps with timezone where timestamps are needed.
- Use stable identifiers for references; do not use display names as primary keys.
- Preserve source and version provenance when a record is derived from external data.
- Define nullability explicitly in each physical schema. A missing field must not be silently interpreted as a negative biological assertion.
- Keep numeric semantics and units explicit. Do not mix probabilities, confidence scores, distances, and dimensionless features.
- Large payloads are stored as artifacts or in their designated data stores. Contracts carry references, checksums, and metadata.
- Version schemas independently from source dataset, feature, embedding, and model versions.
