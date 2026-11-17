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

## 10. Approved Decisions and Review Additions (2026-11-16)

Các quyết định đã được người dùng duyệt:

- **Population source:** nhập thủ công (manual) theo từng building trong cấu hình simulation run. Không suy luận từ footprint, số tầng, loại công trình hoặc OSM metadata.
- **Numeric representation:** giữ f64/expected aggregate counts tương thích với SirInput và Phase 1. Không tự chuyển sang integer trong schema hoặc service.
- **Missing population:** building không được cung cấp population hợp lệ sẽ bị loại khỏi tập eligible và phải có exclusion reason trong kết quả khởi tạo; không tự gán mặc định.
- **Initial state:** N_i và I0_i phải được cung cấp/xác định rõ; R0_i chỉ mặc định 0 theo contract nếu không có recovered seed. Tính S0_i = N_i - I0_i - R0_i. Không tự mặc định infected seed cho building bất kỳ.
- **Validation:** N_i > 0, mọi giá trị finite/non-negative, I0_i + R0_i <= N_i; S+I+R=N theo tolerance tương thích Phase 1.
- **No fabricated data:** lưu source/method là manual input và thời điểm tạo run để có thể audit cấu hình.

### Additional review checklist
- [ ] API/schema nhận population manual gắn với building UUID, không nhận theo array index hoặc display name.
- [ ] Xử lý duplicate building IDs và unknown building IDs.
- [ ] Validate missing, zero, negative, NaN/Infinity (nếu đi qua serialization/domain) và over-population.
- [ ] Response phân biệt eligible buildings với excluded buildings và lý do loại.
- [ ] Persist initial values bằng kiểu dữ liệu tương thích f64 của Phase 1.
- [ ] Test rollback toàn transaction nếu contract yêu cầu atomic initialization và có một building input invalid.
- [ ] Không tạo population mặc định hoặc seed infection ngầm.


## 11. Approved Initialization Contract (2026-11-17)

Người dùng đã duyệt các quy tắc bổ sung sau để làm rõ request, validation, tính tái lập và tính nguyên tử của quá trình khởi tạo.

### 11.1 Request contract

Request nhận danh sách building tường minh, mỗi phần tử gắn với UUID:

```json
{
  "buildings": [
    {
      "building_id": "uuid-1",
      "population": 100.0,
      "initial_infected": 5.0,
      "initial_recovered": 0.0
    },
    {
      "building_id": "uuid-2",
      "population": 250.0,
      "initial_infected": 0.0,
      "initial_recovered": 0.0
    }
  ]
}
```

Quy tắc field:
- `building_id`: bắt buộc, UUID của building đã tồn tại.
- `population`: population manual; nếu không được cung cấp thì building được phân loại excluded với reason `missing_population`.
- `initial_infected`: bắt buộc đối với building có population; giá trị 0 phải được gửi tường minh để tránh seed infection ngầm.
- `initial_recovered`: optional, mặc định 0 nếu không gửi.
- `initial_susceptible`: backend tự tính bằng `population - initial_infected - initial_recovered`, không nhận từ client.

Việc chọn building ở ticket này dùng explicit UUID list. Không phụ thuộc KAN-21 neighbor API hoặc tự mở rộng tập building bằng spatial query; quan hệ lân cận được sử dụng ở các ticket mô phỏng tiếp theo.

### 11.2 Exclusion versus invalid request

Phân biệt rõ hai trường hợp:
- **Excluded building:** thiếu field population. Không tham gia run; trả về và lưu reason `missing_population`.
- **Invalid request:** population được cung cấp nhưng không hợp lệ (không finite hoặc <= 0), initial infected/recovered âm hoặc không finite, tổng I0 + R0 lớn hơn N, UUID sai định dạng, UUID trùng hoặc UUID không tồn tại. Reject toàn bộ request; không âm thầm bỏ qua input sai.

Nếu sau khi phân loại không còn eligible building nào, reject request và không tạo simulation run rỗng.

### 11.3 Persist exclusions and reproducibility

Bên cạnh initial state của eligible buildings, cần lưu exclusion để giải thích chính xác phạm vi run. Thiết kế bảng `simulation_building_exclusions` hoặc cấu trúc tương đương, tối thiểu gồm:
- `simulation_id`
- `building_id`
- `reason_code` (ở scope hiện tại gồm `missing_population`)
- `created_at`

Ràng buộc uniqueness phù hợp để một building không có nhiều exclusion record trùng trong cùng run. Nếu schema hiện tại có cấu trúc tương đương thì reuse, không tạo bảng trùng. Response khởi tạo phải phân biệt eligible và excluded, kèm reason code.

### 11.4 Atomicity and idempotency

- Validate cấu trúc request và các giá trị số trước khi ghi dữ liệu.
- Trong transaction: xác minh building UUID tồn tại, phân loại eligible/excluded, tạo simulation run, ghi initial states và exclusion records; commit tất cả cùng nhau.
- Bất kỳ lỗi persistence hoặc vi phạm contract nào trong quá trình ghi phải rollback toàn bộ run và dữ liệu liên quan.
- Bổ sung request ID/idempotency key do client cung cấp (hoặc cơ chế tương đương có uniqueness theo phạm vi phù hợp) để retry cùng một yêu cầu không tạo simulation run trùng. Cùng key nhưng payload khác phải trả lỗi conflict, không ghi đè run cũ.
- Không dựa riêng vào unique key `(simulation_id, building_id)` để giải quyết retry, vì mỗi lần retry có thể đã tạo một simulation_id mới.

### 11.5 Additional tests

- Missing population tạo exclusion record và reason code chính xác.
- Missing `initial_infected` ở building có population bị reject; explicit zero được chấp nhận.
- Missing `initial_recovered` được chuẩn hóa thành 0.
- Duplicate UUID, malformed UUID và unknown UUID reject toàn request.
- Invalid provided population / I0 / R0 và over-population reject toàn request.
- Tất cả buildings bị excluded thì không tạo run.
- Response và persistence khớp nhau giữa eligible states và exclusions.
- Lỗi khi ghi bất kỳ state/exclusion nào rollback run và toàn bộ records liên quan.
- Retry cùng idempotency key và cùng payload trả lại kết quả run đã tạo; cùng key nhưng payload khác trả conflict.
- Hai request đồng thời dùng cùng idempotency key không tạo hai run.

### 11.6 Implementation boundary

Các mục trong section này là contract đã duyệt cho KAN-22. Chưa có code, migration hoặc API nào được triển khai chỉ bởi việc cập nhật plan. Khi implement phải rà soát schema/migrations hiện tại trước, reuse cấu trúc tương đương và giữ tương thích SQLx embedded migrations cùng f64/Phase 1.
