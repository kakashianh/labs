## 1. Objective

Thiết lập trạng thái dịch tễ ban đầu S (Susceptible), I (Infected), R (Recovered) cho từng building tham gia một simulation run.

Expected result:
- Mỗi building tham gia có population và trạng thái S/I/R hợp lệ.
- Có thể chỉ định một hoặc nhiều building có ca nhiễm ban đầu.
- Trạng thái được gắn với một simulation run và có thể tái tạo.
- Không tạo dân số giả từ footprint, diện tích hoặc số tầng không có nguồn.

## 2. Dependencies and Preconditions

| Dependency | Required state |
|---|---|
| KAN-13/KAN-14 | Building identity và footprint đã có trong PostGIS |
| KAN-18/KAN-19 | SIR domain/API Phase 1 làm reference về validation và time semantics |
| KAN-21 | Building selection/neighbor query sẵn sàng nếu cần xác định phạm vi mô phỏng |

Population hiện chưa có trong data contract OSM sample. Vì vậy population phải là input do người dùng/cấu hình/dataset có nguồn cung cấp; không suy ra population từ footprint hoặc floors.

## 3. Scope

### In scope
- Chốt nguồn population cho mỗi building.
- Định nghĩa building eligibility khi thiếu population.
- Nhận cấu hình population và initial infected.
- Khởi tạo S/I/R theo từng building.
- Kiểm tra S + I + R = N, mọi giá trị không âm và không vượt N.
- Gắn trạng thái với simulation_id.
- Thiết kế persistence cho initial state và test tính toàn vẹn.

### Out of scope
- Suy luận dân số từ diện tích footprint.
- Mô hình occupancy theo tầng/phòng hoặc agent-level.
- Quy tắc lây nhiễm giữa buildings (KAN-23).
- Tính toán tiến hóa SIR theo thời gian (KAN-24).
- Lưu nhiều scenario/intervention nâng cao.

## 4. Population and Initial-State Policy

Phải chốt một trong các nguồn population được hỗ trợ trước khi code:
1. Population được nhập rõ ràng theo building trong cấu hình simulation.
2. Population đến từ một dataset bên ngoài có nguồn, license và mapping được ghi nhận.
3. Building không có population hợp lệ bị loại khỏi simulation với lý do được trả về rõ ràng.

Không tự động gán population mặc định cho toàn bộ building.

Initial state contract:
- N_i > 0 và hữu hạn.
- I0_i do người dùng cấu hình; mặc định 0 chỉ khi quy tắc nghiệp vụ được chấp thuận.
- R0_i mặc định 0 nếu không có recovered seed được nhập.
- S0_i = N_i - I0_i - R0_i.
- 0 <= I0_i + R0_i <= N_i.
- Chốt rõ integer counts hay floating population. Nếu counts là người, ưu tiên integer ở initial state; engine có thể tạo fractional expected counts trong các bước số học và cần policy hiển thị/lưu nhất quán.

## 5. Persistence Design

Rà soát trước khi tạo migration. Thiết kế tối thiểu có thể gồm:
- simulation_runs: metadata, region/scope, parameters, status, timestamps.
- simulation_building_states: simulation_id, building_id, population, initial_s, initial_i, initial_r.
- Unique key (simulation_id, building_id).
- FK đến simulation_runs và buildings.
- Constraints/checks cho population và state values.

Không tạo bảng trùng nếu schema/branch đã có cấu trúc tương đương. Migration mới phải tương thích SQLx embedded migrations.

## 6. Implementation Plan

### Step 1 – Inspect Data Availability and Decide Population Source
- [ ] Kiểm tra schema buildings và attributes thực có.
- [ ] Xác nhận hiện chưa có population/floors trong nguồn OSM sample.
- [ ] Đề xuất và chốt population input/source với người dùng.
- [ ] Định nghĩa behavior với building thiếu population.
- [ ] Chốt kiểu dữ liệu và rounding policy.

### Step 2 – Define Simulation and Initial-State Contract
- [ ] Xác định simulation run identity.
- [ ] Định nghĩa request cho building population và initial infected/recovered.
- [ ] Định nghĩa default initial state có thể kiểm soát.
- [ ] Chốt cách chọn building: region, explicit list hoặc spatial selection.
- [ ] Định nghĩa validation và response cho building bị loại.

### Step 3 – Design Database Schema
- [ ] Rà soát các migration hiện tại để tránh duplicate.
- [ ] Thiết kế simulation run metadata nếu chưa có.
- [ ] Thiết kế per-building initial state.
- [ ] Thêm FK/unique/check constraints.
- [ ] Chốt delete/update behavior và transaction boundary.
- [ ] Ghi migration strategy và indexes.

### Step 4 – Implement Initialization Service
- [ ] Load building set được chọn.
- [ ] Validate population và initial infected/recovered.
- [ ] Tính S0 từ N, I0, R0.
- [ ] Reject hoặc báo rõ building thiếu population.
- [ ] Tạo run và initial states trong một transaction.
- [ ] Đảm bảo request retry không tạo trạng thái trùng.

### Step 5 – Tests and Verification
- [ ] Test population dương và missing population.
- [ ] Test infected seed ở một hoặc nhiều buildings.
- [ ] Test tổng S+I+R bằng N.
- [ ] Test negative/over-population/unknown building.
- [ ] Test transaction rollback khi một building invalid.
- [ ] Test idempotency/unique constraint.
- [ ] Test database integration với PostGIS.
- [ ] Chạy backend fmt, Clippy, tests và CI.

## 7. Acceptance Criteria

- [ ] Mỗi building tham gia có N, S0, I0, R0.
- [ ] S0+I0+R0=N theo kiểu dữ liệu và tolerance đã chốt.
- [ ] Không có giá trị âm hoặc vượt N.
- [ ] Có thể cấu hình một/nhiều infected seed buildings.
- [ ] Building thiếu population không bị gán giá trị giả.
- [ ] Initial states gắn với một simulation_id.
- [ ] Có kiểm thử validation, transaction và persistence.

## 8. Deliverables

1. Population/initial-state contract.
2. Schema/migrations cho simulation run và initial states.
3. Initialization service.
4. Tests và tài liệu nguồn population/giới hạn dữ liệu.

## 9. Definition of Done

KAN-22 hoàn thành khi mỗi building đủ điều kiện được khởi tạo S/I/R hợp lệ, có nguồn population rõ ràng, seed infections cấu hình được và trạng thái được lưu an toàn theo simulation run.
