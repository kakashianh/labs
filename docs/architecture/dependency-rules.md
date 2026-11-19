# Dependency rules

## Dependency direction

~~~mermaid
flowchart TB
    Delivery[Delivery: API and batch entry points]
    App[Application orchestration]
    Domain[Domain contracts and rules]
    Adapters[Infrastructure adapters]
    Tech[External technologies]

    Delivery --> App
    App --> Domain
    App --> Adapters
    Adapters --> Domain
    Adapters --> Tech
~~~

The domain is the stable center. Application orchestration coordinates use cases. Infrastructure adapters translate between external technologies and the project's contracts. Delivery mechanisms (HTTP routes, CLI/jobs) invoke application use cases and should not own business rules.

## Allowed and prohibited dependencies

| Layer | May depend on | Must not depend on |
|---|---|---|
| domain | Python standard library and other domain modules | FastAPI, PySpark, SQLAlchemy/database drivers, JanusGraph clients, Milvus clients, object-storage SDKs |
| application/use cases | domain contracts; declared ports | HTTP request/response objects as core business inputs |
| infrastructure adapters | domain ports/contracts and technology SDKs | UI/frontend modules |
| API delivery | application use cases, API schemas, dependency wiring | Directly embedding domain rules in route functions |
| batch delivery | application/use cases and Spark-specific processing | FastAPI request lifecycle |

## Ports and adapters

- Define ports in the inner layer that needs a capability, using small interfaces/protocols.
- Implement ports in infrastructure-specific adapters in the owning task.
- Inject adapters at application boundaries; avoid constructing database or network clients inside domain objects.
- Return domain records or application results from adapters, not vendor-specific response objects.
- Keep API serialization and persistence mapping at the edge.

## Practical review checklist

- [ ] Does importing a domain module require any infrastructure dependency?
- [ ] Does a route handler contain orchestration only, with business rules delegated inward?
- [ ] Are Spark DataFrames confined to batch-processing boundaries?
- [ ] Are JanusGraph and Milvus SDK types hidden behind adapters?
- [ ] Are persistence models mapped to domain/application records rather than leaked across the codebase?
- [ ] Are external source identifiers preserved instead of being used as the sole internal identity?
- [ ] Are module dependencies acyclic and explicit?

## Enforcement

PP-10 documents the rules. Future implementation tasks should add automated import/layer checks when the relevant package structure exists. Do not add placeholder packages or speculative lint configuration in this architecture-only task.
