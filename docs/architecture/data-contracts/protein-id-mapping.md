# Contract: Protein ID mapping

## Purpose

Preserve the relationship between identifiers used by external sources and the canonical protein entity.

## Fields

| Field | Meaning | Required |
|---|---|---:|
| source | Source namespace, such as a biological database | Yes |
| source_id | Identifier exactly as supplied by the source | Yes |
| protein_id | Internal canonical protein reference | When resolved |
| uniprot_accession | Canonical UniProt accession, when resolved | When resolved |
| mapping_status | Resolution outcome | Yes |
| source_version | Source release or mapping dataset version | Yes |
| mapping_evidence | Optional reason or evidence for the mapping | When available |

## Mapping status vocabulary

- resolved: a unique canonical protein mapping is available.
- merged: the submitted identifier has been merged/replaced and resolves to a successor.
- deprecated: the source identifier is retired; a canonical successor may or may not be known.
- unresolved: no reliable canonical mapping is available.
- ambiguous: multiple possible targets exist and cannot be safely reduced to one.

## Rules

- Preserve source_id byte-for-byte/case-sensitively as defined by the source namespace.
- Do not discard merged or deprecated identifiers after resolving them.
- Ambiguous and unresolved mappings must remain visible for quality review.
- Record the mapping source/version so the same transformation can be reproduced.
