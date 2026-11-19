# Contract: Protein embedding

## Purpose

Represent a model-generated vector used for similarity search or downstream ML.

## Fields

| Field | Meaning | Required |
|---|---|---:|
| protein_id | Canonical protein reference | Yes |
| embedding_type | Sequence, structure, or combined embedding family | Yes |
| dimension | Vector dimensionality | Yes |
| model_name | Encoder/model identity | Yes |
| model_version | Immutable model version or artifact digest | Yes |
| input_version | Sequence/structure/feature input version | Yes |
| vector_ref | Reference to vector in vector store or artifact | Yes |
| normalization | Vector normalization policy | Yes |
| generated_at | Generation timestamp | Yes |
| schema_version | Contract version | Yes |

## Rules

- Embeddings from different model versions or embedding types are not assumed comparable.
- Dimension and normalization policy must match the vector index configuration.
- Preserve the input version and model artifact identity.
- Milvus collection/index configuration and encoder implementation belong to PP-30–PP-33.
- Never store a large vector payload in a relational metadata record merely for convenience.
