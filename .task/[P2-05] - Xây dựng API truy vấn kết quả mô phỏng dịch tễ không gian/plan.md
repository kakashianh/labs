## 1. Objective

Cung cấp API đọc kết quả spatial SIR đã persist ở KAN-24, phục vụ bản đồ CesiumJS và bảng thống kê.

## 2. Dependencies

| Dependency | Required state |
|---|---|
| KAN-24 | Persisted per-building states indexed by simulation_id/building_id/time_step |
| KAN-13/KAN-14 | Building and region metadata |
| Existing API conventions | Axum, SQLx, AppError, OpenAPI and pagination patterns |

## 3. Scope

### In scope
- Query simulation run metadata/status.
- Query per-building S/I/R at a selected time_step.
- Query aggregate S/I/R for whole simulation or selected region.
- Filter by simulation_id, time_step and region_id.
- Return identifiers, population, S/I/R and fields required by map styling.
- Validate unknown run/time_step and invalid filters.
- Pagination/limit for large building sets.

### Out of scope
- Running simulation (KAN-24).
- Risk classification (KAN-26).
- Hotspot computation (KAN-27).
- Frontend map rendering (KAN-28).
- Returning full geometry in every row if existing building API can provide it separately.

## 4. API Design

Endpoint paths and naming must follow existing API conventions. Proposed resource shapes:
- GET simulation run detail/status.
- GET per-building states for a run and time_step.
- GET aggregate S/I/R for run, optionally scoped to region.
- Optional building detail lookup if needed by map selection.

Response contract:
- simulation_id, time_step, time.
- building_id and population, susceptible, infected, recovered.
- Aggregate totals must be computed from the same run/time_step snapshot.
- Explicit empty result semantics.
- Stable pagination metadata and deterministic ordering.
- Avoid returning floating values as strings.

The API must not return partially persisted data as a completed result. If run is running/failed, status must be explicit.

## 5. Implementation Plan

### Step 1 – Inspect Stored Result Schema and API Conventions
- [ ] Review KAN-24 migrations/repositories.
- [ ] Inspect existing building/region response DTOs.
- [ ] Confirm run lifecycle and completed/failed semantics.
- [ ] Confirm aggregate precision and rounding policy.
- [ ] Define endpoint/resource contract and OpenAPI examples.

### Step 2 – Design Query Strategy
- [ ] Query building states by simulation_id + time_step.
- [ ] Join only required building/region fields.
- [ ] Query aggregate totals at same time_step.
- [ ] Add optional region filter.
- [ ] Add pagination/limit and stable ordering.
- [ ] Check indexes with EXPLAIN where appropriate.

### Step 3 – Implement Repository and Service
- [ ] Add parameterized SQLx queries.
- [ ] Validate simulation_id/time_step/region_id.
- [ ] Prevent cross-run/time mixing.
- [ ] Return explicit not-found/status errors.
- [ ] Ensure aggregate and detail queries share identical filters.

### Step 4 – Implement Axum Routes and DTOs
- [ ] Add routes under existing API version.
- [ ] Define typed request/query and response DTOs.
- [ ] Register routes in router.
- [ ] Update OpenAPI schema/contract tests.
- [ ] Document pagination and numeric semantics.

### Step 5 – Tests and Verification
- [ ] Test valid per-building state query.
- [ ] Test aggregate whole simulation.
- [ ] Test aggregate by region.
- [ ] Test unknown simulation_id.
- [ ] Test time_step outside stored range.
- [ ] Test invalid region and pagination.
- [ ] Test running/failed run behavior.
- [ ] Test aggregate totals equal sum of returned eligible building states when unpaginated.
- [ ] Run API integration tests with PostGIS.

## 6. Acceptance Criteria

- [ ] API returns correct per-building S/I/R for requested time_step.
- [ ] API returns aggregate S/I/R by region and whole run.
- [ ] Responses include required building/region identifiers.
- [ ] Invalid or missing run/time_step handled consistently.
- [ ] Pagination and stable ordering supported where needed.
- [ ] OpenAPI and docs match implementation.
- [ ] Tests prove filters do not mix runs/time steps.

## 7. Deliverables

1. Read APIs and typed DTOs.
2. SQLx query/repository implementation.
3. OpenAPI updates and API documentation.
4. Integration tests.

## 8. Definition of Done

KAN-25 hoàn thành khi Frontend có thể truy vấn chính xác trạng thái từng building và thống kê tổng hợp từ cùng một simulation run/time_step, có pagination, error handling và API contract được kiểm thử.
