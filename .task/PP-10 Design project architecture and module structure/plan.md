# PP-10 — Design project architecture and module structure

## 1. Objective

Define stable boundaries and data contracts for ingestion, Spark, structure features, graph, vector search, ML, API, and UI.

## 2. Scope and design context

Translate the platform flow into modules with one-way dependencies. This task defines interfaces; implementation belongs to later tasks.

**Upstream dependencies:** PP-9 provides the repository conventions.

**Related plans:** `PP-11` · `PP-12` · `PP-13–PP-42`. Use the links below to keep contracts aligned; update this plan if an upstream interface changes.

## 3. Step-by-step implementation and to-do

### Step 1 — Map the end-to-end data flow

Trace external sources through raw storage, Spark normalization, feature extraction, graph/vector stores, ML, API, and dashboard.

**To-do**
- [ ] Draw a component diagram and data-flow diagram
- [ ] Identify batch jobs versus online request paths
- [ ] Mark ownership of each dataset and service

### Step 2 — Define module boundaries and dependency direction

Separate source adapters, domain schemas, processing jobs, repositories, model code, API routes, and presentation contracts.

**To-do**
- [ ] Define package/module responsibilities
- [ ] Enforce inward dependency direction and avoid circular imports
- [ ] Identify shared libraries versus task-specific code

### Step 3 — Specify shared identifiers and schemas

Define canonical protein ID, source ID mapping, PPI edge, GO annotation, structure metadata, feature vector, and prediction records.

**To-do**
- [ ] Define canonical UniProt accession policy and external ID mapping fields
- [ ] Specify PPI edge directionality, confidence score, and evidence metadata
- [ ] Specify versioned schemas and nullability for shared records
- [ ] Define dataset/model version and provenance fields

### Step 4 — Publish architecture decisions

Record decisions, trade-offs, and contract ownership so downstream tasks can implement consistently.

**To-do**
- [ ] Add architecture overview and module diagram to docs
- [ ] Add interface/schema contract documents
- [ ] Link each later task to the relevant contract
- [ ] Review architecture against Docker and persistence choices

## 4. Deliverables

- Implementation/configuration and documentation within this ticket's scope.
- Versioned interfaces and assumptions needed by dependent tasks.
- Automated tests and reproducible verification instructions.

## 5. Acceptance criteria

- [ ] All step to-dos are completed and reviewed.
- [ ] Contracts, configuration, and operational assumptions are documented.
- [ ] Failure cases are explicit; no credentials or generated large datasets are committed.
- [ ] Tests pass from the documented local environment.
- [ ] Downstream plans and interfaces remain consistent.

## 6. Testing strategy

- **Unit:** validate core rules, configuration, schemas, and edge cases.
- **Integration:** verify the relevant infrastructure or module boundary.
- **Regression:** preserve stable IDs and contracts with fixed fixtures.
- **Manual:** follow the documented clean-checkout workflow.

## 7. Cross-task links

- **Upstream:** PP-9 provides the repository conventions.
- **Downstream / coordination:** PP-11, PP-12, PP-13–PP-42
- **Rule:** consume upstream outputs as defined; do not silently redefine shared identifiers, schemas, or configuration. If a contract must change, update the owning plan and all affected consumers in the same change set.

## 8. Completion notes

Record implementation decisions, commands and test results, artifacts produced, data/source versions, and deviations from this plan.
