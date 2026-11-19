# Contract: Protein-protein interaction edge

## Purpose

Represent an interaction assertion between two protein entities, including evidence and source confidence.

## Fields

| Field | Meaning | Required |
|---|---|---:|
| source_protein_id | First endpoint | Yes |
| target_protein_id | Second endpoint | Yes |
| confidence | Source interaction confidence score | When source provides it |
| evidence | Evidence records or references | Yes |
| source | Source database or experiment namespace | Yes |
| source_version | Source release/version | Yes |
| directionality | Whether the source explicitly asserts direction | Yes |
| schema_version | Contract version | Yes |

## Semantics

- Biological PPI is treated as undirected by default unless the source explicitly defines a directed relationship.
- For undirected records, normalize endpoint ordering using a stable deterministic rule before deduplication and persistence.
- Reject or explicitly classify self-interactions; do not silently discard them.
- Preserve multiple independent evidence records and source assertions even if they refer to the same normalized pair.
- confidence is a source-specific interaction confidence, not an ML probability. Document the source scale and any normalization separately.
- A missing confidence value is unknown, not zero.

## Example

~~~json
{
  "source_protein_id": "protein:P1",
  "target_protein_id": "protein:P2",
  "confidence": {
    "value": 0.85,
    "scale": "0..1",
    "meaning": "source interaction confidence"
  },
  "evidence": [{"type": "database", "reference": "source-record-123"}],
  "source": "example-source",
  "source_version": "2026-01",
  "directionality": "undirected",
  "schema_version": "1.0"
}
~~~

This is a contract example, not a claim about a real interaction.
