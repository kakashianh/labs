## 1. Objective

Cung cấp API đọc kết quả spatial SIR đã persist ở KAN-24, phục vụ bản đồ CesiumJS và bảng thống kê.

## 2. Dependencies

| Dependency | Required state |
|---|---|
| KAN-24 | Đã persist time series theo `simulation_id`, `building_id`, `time_step`; run status chuyển sang `completed` sau khi lưu kết quả |
| KAN-13/KAN-14 | Building và region metadata |
| Existing API conventions | Axum, SQLx, AppError, OpenAPI và pagination patterns |

### Verified storage contract from KAN-24

- `simulation_runs`: `id`, `request_id`, `request_payload`, `population_source`, `status`, `created_at`, `simulation_config`, `completed_at`.
- Status hiện được DB constraint cho phép là `initialized` và `completed`; chưa có trạng thái `running` hoặc `failed`.
- `simulation_building_results` có khóa chính `(simulation_id, building_id, time_step)`; lưu `time`, `susceptible`, `infected`, `recovered`.
- `buildings.region_id` là FK bắt buộc tới `regions.id`. Mỗi building hiện thuộc một region theo schema.
- KAN-24 lưu kết quả và cập nhật run status trong transaction; trạng thái `completed` chỉ được công bố cùng transaction kết quả.

## 3. Scope

### In scope

- Query simulation run metadata/status.
- Query per-building S/I/R tại một `time_step`.
- Query aggregate S/I/R cho toàn simulation hoặc một `region_id`.
- Filter theo `simulation_id`, `time_step`, tùy chọn `region_id`.
- Trả về IDs, population, S/I/R và metadata tối thiểu phục vụ map styling.
- Validate unknown run/time_step và invalid filters.
- Pagination cho danh sách building results; thứ tự ổn định theo `building_id`.

### Out of scope

- Running simulation (KAN-24).
- Risk classification (KAN-26).
- Hotspot computation (KAN-27).
- Frontend map rendering (KAN-28).
- Trả full geometry trong từng row nếu building API hiện tại có thể cung cấp riêng.
- Bổ sung lifecycle status `running`/`failed` vào schema; nếu cần phải tạo ticket/migration riêng.

## 4. API Design

Endpoint path và naming phải theo convention hiện có. Resource shape dự kiến:

- GET simulation run detail/status.
- GET building states của một run tại một `time_step`.
- GET aggregate S/I/R của run, có thể scope theo `region_id`.

Query contract:

- Bắt buộc `simulation_id` và `time_step` cho state/aggregate query.
- `region_id` optional; khi có phải validate region tồn tại và filter qua `buildings.region_id`.
- Danh sách building states hỗ trợ `limit` và `offset` (hoặc pagination convention sẵn có), giới hạn max hợp lý; sort cố định `building_id ASC`.
- Response có `simulation_id`, `time_step`, `time` (time lấy từ cùng snapshot), `building_id`, `region_id`, `population`, `susceptible`, `infected`, `recovered`.
- Population lấy từ `simulation_building_states` cùng `simulation_id` + `building_id`, không lấy population hiện tại/ngoài run.
- Aggregate trả `simulation_id`, `time_step`, `time`, `population`, `susceptible`, `infected`, `recovered` và số building; tính trên toàn bộ tập kết quả thỏa filter.
- Aggregate tuyệt đối không tính trên trang hiện tại. Pagination chỉ áp dụng cho endpoint danh sách building.
- Aggregate và detail dùng cùng điều kiện `simulation_id` + `time_step` + optional `region_id`; không được trộn run hoặc time step.
- Empty semantics: run đã completed nhưng không có building result phù hợp region thì trả danh sách rỗng và aggregate bằng 0; time_step không tồn tại trong run là lỗi rõ ràng.
- Numeric fields là JSON number (f64), không stringify và không rounding ở API; test tổng dùng floating-point tolerance phù hợp.

### Run lifecycle and partial-result semantics

- Run không tồn tại: not-found.
- Run status `initialized`: kết quả chưa sẵn sàng; trả lỗi conflict (hoặc status mapping theo AppError convention) với mã lỗi ổn định như `simulation_not_completed`, không trả result rỗng giả.
- Run status `completed`: cho phép đọc kết quả.
- Schema hiện không có `running`/`failed`; không viết logic giả định hai trạng thái này. Nếu sau này lifecycle mở rộng, cập nhật contract riêng.
- Do KAN-24 persist results và status `completed` trong cùng transaction, reader không được thấy run completed với kết quả mới chỉ persist một phần.

## 5. Implementation Plan

### Step 1 – Inspect Stored Result Schema and API Conventions

- [ ] Review KAN-24 migrations/repositories và xác nhận transaction/status update.
- [ ] Inspect existing building/region response DTOs và error mapping.
- [ ] Chốt endpoint names, pagination convention và OpenAPI examples.
- [ ] Chốt aggregate numeric semantics: giữ f64, không round ở API; test với tolerance.
- [ ] Xác nhận query lấy population từ initial state thuộc đúng simulation run.

### Step 2 – Design Query Strategy

- [ ] Query run metadata/status trước khi đọc result.
- [ ] Query building states theo `simulation_id + time_step`.
- [ ] Join `simulation_building_states` theo cả `simulation_id` và `building_id` để lấy population.
- [ ] Join `buildings` để lấy `region_id`; không giả định region mapping ngoài schema.
- [ ] Query aggregate riêng trên toàn bộ tập đã filter, không bị ảnh hưởng bởi limit/offset.
- [ ] Dùng cùng predicate/filter cho detail và aggregate.
- [ ] Add pagination/limit và stable ordering theo `building_id`.
- [ ] Check indexes với EXPLAIN; tận dụng PK/index `(simulation_id, time_step, building_id)` hiện có và bổ sung index chỉ khi query plan chứng minh cần thiết.

### Step 3 – Implement Repository and Service

- [ ] Add parameterized SQLx queries.
- [ ] Validate UUID, `time_step >= 0`, `region_id` và pagination bounds.
- [ ] Prevent cross-run/time mixing bằng composite join/filter đầy đủ.
- [ ] Return explicit not-found/status errors.
- [ ] Return `simulation_not_completed` cho run `initialized`.
- [ ] Ensure aggregate and detail queries share identical filters.
- [ ] Ensure aggregate is independent from pagination.

### Step 4 – Implement Axum Routes and DTOs

- [ ] Add routes under existing API version.
- [ ] Define typed query/response DTOs.
- [ ] Register routes in router.
- [ ] Update OpenAPI schema/contract tests.
- [ ] Document pagination, region filter, lifecycle errors and numeric semantics.

### Step 5 – Tests and Verification

- [ ] Test valid per-building state query.
- [ ] Test aggregate whole simulation.
- [ ] Test aggregate by region.
- [ ] Test unknown simulation_id.
- [ ] Test initialized run returns not-ready status/error and no partial result.
- [ ] Test time_step outside stored range.
- [ ] Test invalid region and pagination bounds.
- [ ] Test stable deterministic ordering.
- [ ] Test aggregate remains unchanged across different limit/offset values.
- [ ] Test aggregate totals equal sum of all matching building states when unpaginated (with floating-point tolerance).
- [ ] Test filters do not mix simulation_id/time_step/region_id.
- [ ] Run API integration tests with Testcontainers PostGIS.

## 6. Acceptance Criteria

- [ ] API returns correct per-building S/I/R for requested `simulation_id + time_step`.
- [ ] API returns aggregate S/I/R for whole run and selected region.
- [ ] Population and region metadata belong to the same building/run.
- [ ] Aggregate is computed across all filtered buildings, never only the current page.
- [ ] Responses include required building/region identifiers.
- [ ] Unknown run, initialized run, invalid region and unknown time_step are handled consistently.
- [ ] Pagination and stable ordering supported.
- [ ] Numeric values remain JSON numbers without API-side rounding.
- [ ] OpenAPI and docs match implementation.
- [ ] Tests prove filters do not mix runs/time steps and pagination does not alter aggregate.

## 7. Deliverables

1. Read APIs and typed DTOs.
2. SQLx query/repository implementation.
3. OpenAPI updates and API documentation.
4. Unit/integration tests.

## 8. Definition of Done

KAN-25 hoàn thành khi Frontend có thể truy vấn chính xác trạng thái từng building và thống kê tổng hợp từ cùng một simulation run/time_step, hỗ trợ region filter, pagination, lifecycle/error handling và API contract được kiểm thử.
