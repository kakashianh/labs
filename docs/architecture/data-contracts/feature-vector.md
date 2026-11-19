# Contract: Structural or computed feature vector

## Purpose

Represent a versioned set of derived numeric features associated with a protein or a specific structure/chain.

## Fields

| Field | Meaning | Required |
|---|---|---:|
| protein_id | Canonical protein reference | Yes |
| structure_id | Structure context, when applicable | When applicable |
| feature_type | Feature family/name | Yes |
| dimension | Number of feature values | Yes |
| values_ref | Inline small values or artifact reference | Yes |
| units | Units for dimensional quantities | When applicable |
| feature_version | Feature extraction definition/version | Yes |
| source_artifact_checksum | Input artifact checksum | When structure-derived |
| provenance | Tool/version/configuration metadata | Yes |
| schema_version | Contract version | Yes |

## Rules

- Define feature ordering and meaning for each feature_type/version.
- Dimension must match the number of values represented by the record.
- State units and coordinate conventions for dimensional/geometric features.
- Record parser/tool version and input artifact checksum for reproducibility.
- Large vectors should be stored in columnar/object storage or an appropriate feature store, not repeated in metadata rows.
