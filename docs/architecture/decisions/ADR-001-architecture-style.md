# ADR-001: Architecture style and dependency direction

- Status: Accepted
- Date: 2026-11-19
- Owners: PP-10 architecture

## Context

The platform combines offline data engineering, structure processing, graph/vector storage, ML workflows, and an online API. Direct coupling between these technologies would make domain rules difficult to test and downstream components difficult to evolve independently.

## Decision

Use a domain-centered modular architecture with explicit application orchestration and infrastructure adapters (ports and adapters / hexagonal principles).

- Domain contracts remain framework-independent.
- Application use cases coordinate domain rules and declared ports.
- Infrastructure adapters implement ports for PostgreSQL, JanusGraph, Milvus, object storage, source APIs, and other external systems.
- FastAPI routes and batch entry points are delivery mechanisms, not owners of business rules.
- Spark-specific data structures remain at batch-processing boundaries.

## Consequences

**Positive**
- Domain contracts can be tested without external services.
- Infrastructure technologies can be replaced behind adapters.
- Batch and online delivery can reuse shared semantics without sharing transport-specific models.

**Trade-offs**
- Requires explicit mapping between persistence/transport models and domain records.
- Introduces interface and wiring work that should remain proportionate to each module's complexity.
