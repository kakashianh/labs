# Contract: Gene Ontology annotation

## Purpose

Represent an assertion associating a protein with a Gene Ontology term and its evidence.

## Fields

| Field | Meaning | Required |
|---|---|---:|
| protein_id | Canonical protein reference | Yes |
| go_id | GO term identifier | Yes |
| evidence_code | GO evidence code | Yes |
| qualifier | Assertion qualifier, including NOT where present | When supplied |
| reference | Supporting publication/database reference | When available |
| assigned_by | Assigning source/authority | When available |
| source_version | Annotation release/version | Yes |
| schema_version | Contract version | Yes |

## Semantics

- Preserve the original evidence code and qualifier.
- An annotation absence is not equivalent to a NOT-qualified annotation.
- Preserve provenance and source release to support reproducibility.
- Validate GO identifiers against the relevant ontology release in the owning data task.
- Do not interpret evidence codes as numeric confidence scores.
