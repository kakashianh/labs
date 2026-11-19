# ADR-002: Protein identity and identifier mapping

- Status: Accepted
- Date: 2026-11-19
- Owners: PP-10 architecture

## Context

Protein records are collected from multiple biological resources. Source identifiers may differ, be deprecated, be merged, or resolve ambiguously. Using a source-specific ID as the global project key would create inconsistent relationships across datasets and stores.

## Decision

- Use an immutable project-owned protein_id as the internal reference across project modules.
- Use canonical UniProt accession as the primary cross-system external protein identifier when a reliable mapping exists.
- Preserve every original source namespace and source_id in explicit mapping records.
- Represent mapping outcomes (resolved, merged, deprecated, unresolved, ambiguous) rather than dropping problematic identifiers.
- Preserve isoform distinctions where the source identifies them; do not silently collapse isoforms into a canonical sequence.

## Consequences

- Internal relationships remain stable if an external accession is updated.
- Source-level provenance and remapping remain auditable.
- Ingestion and normalization must handle unresolved and ambiguous mappings explicitly.
- Physical key formats and mapping-table constraints are implemented by PP-12 and data pipeline tasks.
