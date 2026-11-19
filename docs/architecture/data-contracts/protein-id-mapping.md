# Contract: Protein ID mapping

## Purpose

Preserve the relationship between identifiers used by external sources and the canonical protein entity.

## Fields

| Field | Meaning | Required |
|---|---|---:|
| source | Source namespace, such as a biological database | Yes |
| source_id | Identifier exactly as supplied by the source | Yes |
| protein_id | Internal canonical protein reference | When resolved or merged | Optional for deprecated mappings; forbidden for unresolved/ambiguous mappings |
| uniprot_accession | Canonical UniProt accession, when resolved | When resolved |
| mapping_status | Resolution outcome | Yes |
| source_version | Source release or mapping dataset version | Yes |
| mapping_evidence | Optional reason or evidence for the mapping | When available |

## Mapping status vocabulary

- resolved: a unique canonical protein mapping is available; internal UUID required.
- merged: the submitted identifier has been merged/replaced and resolves to a successor; successor internal UUID required.
- deprecated: the source identifier is retired; successor internal UUID is optional.
- unresolved: no reliable canonical mapping is available; internal UUID must be absent.
- ambiguous: multiple possible targets exist and cannot be safely reduced to one; internal UUID must be absent.

## Rules

- Preserve source_id byte-for-byte/case-sensitively as defined by the source namespace.
- Do not discard merged or deprecated identifiers after resolving them.
- Ambiguous and unresolved mappings must remain visible for quality review.
- Record the mapping source/version so the same transformation can be reproduced.


The internal protein UUID is project-owned and opaque. It is never derived from source_id or a UniProt accession. Mapping uniqueness is scoped by (source, source_id, source_version).
