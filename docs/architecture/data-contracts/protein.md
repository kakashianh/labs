# Contract: Protein

## Purpose

Canonical logical record representing a protein entity across ingestion, processing, graph, vector, ML, and API modules.

## Fields

| Field | Meaning | Required | Notes |
|---|---|---:|---|
| protein_id | Stable internal identifier | Yes | Opaque, immutable project-owned UUIDv4; never derived from an external accession |
| uniprot_accession | Canonical UniProt accession | Yes when mapped | External identifier only; preserve its mapping in the Protein ID mapping contract |
| sequence | Amino-acid sequence | When available | Preserve sequence source/version and normalization policy |
| organism_id | Organism/taxon identifier | When known | Use a documented taxonomy namespace |
| source provenance | Origin of imported or derived record | Yes for imported data | Source name, source record ID, and source release/version |
| schema_version | Logical contract version | Yes | Version of this contract representation |

## Identity rules

- `protein_id` is generated and owned by this project as an opaque UUIDv4 and remains stable across source identifier changes.
- UniProt accession and identifiers from other providers are external keys recorded in `protein_id_mappings`; they are not internal primary keys.
- Never overwrite the original source identifier during normalization.
- Isoforms and canonical sequences must not be collapsed without an explicit policy; preserve isoform identity where the source distinguishes it.
- An unresolved or ambiguous accession must not be assigned a fabricated canonical identity or interpreted as negative biological evidence.

## Example

~~~json
{
  "protein_id": "550e8400-e29b-41d4-a716-446655440000",
  "uniprot_accession": "Q9Y261",
  "sequence": "MKWVTFISLLLLFSSAYS",
  "organism_id": "NCBITaxon:9606",
  "schema_version": "1.0"
}
~~~

The UUID is illustrative only. The sequence above is also illustrative and not a validated complete reference sequence. The UniProt accession is resolved through the separate mapping contract.
