# Contract: Protein

## Purpose

Canonical logical record representing a protein entity across ingestion, processing, graph, vector, ML, and API modules.

## Fields

| Field | Meaning | Required | Notes |
|---|---|---:|---|
| protein_id | Stable internal identifier | Yes | Immutable project identifier; representation chosen by implementation owner |
| uniprot_accession | Canonical UniProt accession | Yes when mapped | Canonical cross-system protein key |
| sequence | Amino-acid sequence | When available | Preserve sequence source/version and normalization policy |
| organism_id | Organism/taxon identifier | When known | Use a documented taxonomy namespace |
| source provenance | Origin of imported or derived record | Yes for imported data | Source name, source record ID, and source release/version |
| schema_version | Logical contract version | Yes | Version of this contract representation |

## Identity rules

- UniProt accession is the canonical external protein identifier when a valid mapping exists.
- protein_id is the stable internal reference used by project-owned records and relationships.
- Never overwrite the original source identifier during normalization.
- Isoforms and canonical sequences must not be collapsed without an explicit policy; preserve isoform identity where the source distinguishes it.
- An unresolved accession must not be assigned a fabricated canonical identity.

## Example

~~~json
{
  "protein_id": "protein:Q9Y261",
  "uniprot_accession": "Q9Y261",
  "sequence": "MKWVTFISLLLLFSSAYS",
  "organism_id": "NCBITaxon:9606",
  "schema_version": "1.0"
}
~~~

The sequence above is illustrative only and not a validated complete reference sequence.
