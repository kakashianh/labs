# Contract: Structure metadata

## Purpose

Describe a protein structure artifact and its relationship to a protein, chain, and source.

## Fields

| Field | Meaning | Required |
|---|---|---:|
| structure_id | Stable structure record identifier | Yes |
| protein_id | Canonical protein reference | When mapped |
| source | Structure source/database | Yes |
| source_structure_id | Original source structure identifier | Yes |
| format | Artifact format, e.g. PDB or mmCIF | Yes |
| chain_id | Chain identifier in the artifact | When applicable |
| artifact_uri | URI locating the artifact | Yes |
| checksum | Content checksum and algorithm | When available |
| source_version | Source release/version | When available |
| coordinate_reference | Coordinate/reference-frame metadata | When applicable |
| schema_version | Contract version | Yes |

## Rules

- A structure may map to multiple proteins/chains and a protein may have multiple structures.
- Preserve source structure and chain identifiers.
- Store coordinate files as artifacts; metadata stores references and integrity information.
- Distinguish unavailable structure data from a confirmed absence of a structure.
- Record experimental/predicted provenance and relevant quality metadata in an implementation-specific extension.
