# PP-39 — Implement protein search and detail REST APIs

## 1. Objective

Expose protein lookup and detail endpoints that combine canonical metadata, structure summaries, annotations, and versioned prediction results.

## 2. Scope and dependencies

PP-10 defines API contracts; PP-12 owns metadata schema/configuration; PP-20 publishes the protein master; PP-25 provides structural features; PP-35/PP-36 provide model outputs.

Related plans: [PP-10](../PP-10%20Design%20project%20architecture%20and%20module%20structure/plan.md) · [PP-12](../PP-12%20Design%20metadata%20database%20schema%20and%20application%20configuration/plan.md) · [PP-20](../PP-20%20Generate%20protein%20master%20and%20PPI%20Parquet%20datasets/plan.md) · [PP-25](../PP-25%20Generate%20protein%20structure%20feature%20vectors/plan.md) · [PP-35](../PP-35%20Implement%20baseline%20protein%20function%20classification%20models/plan.md) · [PP-36](../PP-36%20Implement%20combined%20feature%20and%20graph-based%20prediction/plan.md) · [PP-38](../PP-38%20Evaluate%20models%20and%20analyze%20prediction%20performance/plan.md) · [PP-41](../PP-41%20Build%20protein%203D%20and%20PPI%20visualization%20dashboard/plan.md) · [PP-42](../PP-42%20Integrate%20prediction%20results%20and%20perform%20end-to-end%20testing/plan.md).

## 3. Step-by-step implementation and to-do

### Step 1 — Define API resources and contracts

Specify search filters, pagination, protein detail fields, prediction response, and version metadata.

**To-do**
- [ ] Define GET search endpoint and query parameters
- [ ] Define protein detail response keyed by canonical UniProt accession
- [ ] Define prediction summary, GO terms, and confidence/score semantics
- [ ] Define not-found, invalid ID, and unavailable-model responses
- [ ] Publish response examples and OpenAPI schemas

### Step 2 — Implement search and repository layer

Build efficient lookup using indexed metadata and bounded pagination.

**To-do**
- [ ] Implement repository queries against metadata/curated datasets
- [ ] Normalize accepted accession formats and aliases
- [ ] Add filters and deterministic pagination
- [ ] Avoid unbounded scans and N+1 data access
- [ ] Add query/index tests using representative protein fixtures

### Step 3 — Integrate structure and prediction details

Join structure metadata and model output by accession and version.

**To-do**
- [ ] Attach structure availability, chain/coverage, and confidence summary
- [ ] Attach predicted GO terms and model/dataset version
- [ ] Expose source/provenance and distinguish missing from negative prediction
- [ ] Validate response consistency across data versions
- [ ] Avoid fabricating missing structure or prediction fields

### Step 4 — Harden and document API

Add validation, tests, OpenAPI docs, and operational safeguards.

**To-do**
- [ ] Add request validation and response schemas
- [ ] Add unit and API integration tests
- [ ] Add timeouts, structured logs, and pagination limits
- [ ] Define health/readiness behavior for required data services
- [ ] Publish OpenAPI contract for PP-41 and PP-42

## 4. Deliverables

- Protein search and detail REST endpoints.
- Versioned response schemas and OpenAPI documentation.
- Tests, representative requests/responses, and operational notes.

## 5. Acceptance criteria

- [ ] All step-level to-dos are complete.
- [ ] Canonical protein identifiers and aliases resolve consistently.
- [ ] Search is paginated, deterministic, and bounded.
- [ ] Missing structure, annotation, and prediction data are represented explicitly.
- [ ] Model and dataset versions are exposed with prediction results.
- [ ] Unit and integration tests pass.

## 6. Testing strategy

- **Unit:** accession normalization, filters, pagination, response mapping, and error handling.
- **Integration:** metadata repository and model-output integration.
- **Contract:** validate OpenAPI schemas and stable response fields.
- **Regression:** fixed protein fixtures with and without structure/prediction data.

## 7. Cross-task handoff

- **Upstream:** PP-12, PP-20, PP-25, PP-35, PP-36, and PP-38.
- **Consumers:** PP-41 dashboard and PP-42 end-to-end validation.
- **Contract rule:** publish endpoint paths, schemas, pagination behavior, score semantics, and version fields. Coordinate breaking changes with every consumer.

## 8. Completion notes

Record endpoint examples, data/model versions, test commands/results, known limitations, and deviations.
