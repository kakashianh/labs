## 1. Objective

Định nghĩa và triển khai infection pressure giữa buildings dựa trên trạng thái infected, khoảng cách và beta; sử dụng adjacency từ KAN-21 làm đầu vào cho engine KAN-24.

Expected result:
- Xác định được building neighbors có thể ảnh hưởng đến mỗi building.
- Khoảng cách tác động theo đúng radius và distance semantics.
- Thay đổi beta/radius làm thay đổi infection pressure theo quy tắc đã công bố.
- Quy tắc deterministic, có unit tests và không mô phỏng cá nhân di chuyển.

## 2. Dependencies

| Dependency | Required state |
|---|---|
| KAN-21 | Spatial neighbors và distance_m đã được kiểm thử; KAN-21 đã hoàn tất theo xác nhận của người dùng |
| KAN-22 | Initial S/I/R và population per building; cần tích hợp code/data contract KAN-22 vào base branch hoặc cung cấp interface tương thích trước khi tích hợp KAN-23 |
| KAN-18 | SIR numerical conventions làm reference |

**Sequencing note:** CI success của KAN-22 trên feature branch không tự đồng nghĩa code đã có trong base branch. Trước khi triển khai/tích hợp KAN-23, xác nhận KAN-22 đã được merge vào base branch hoặc thống nhất cách tích hợp tương thích. Không tự giả định KAN-22 đã merge.

## 3. Scope

### In scope
- Định nghĩa adjacency list/graph từ spatial query.
- Định nghĩa distance-decay kernel.
- Tính infectious prevalence/pressure từ neighboring buildings.
- Cấu hình radius, beta và các hệ số cần thiết.
- Xử lý building không có neighbors, population thiếu/không hợp lệ và self contribution.
- Unit tests cho within/outside radius và sensitivity.

### Out of scope
- Individual movement/contact networks.
- Phòng, căn hộ, tầng hoặc agent-level simulation.
- Machine learning dự đoán lây nhiễm.
- Statistical hotspot detection nâng cao.
- Tự chọn tham số dịch tễ từ dữ liệu thực khi chưa có calibration.

## 4. Mathematical Contract

Công thức tại mục 10 là contract đã được người dùng duyệt. Implementation phải tuân thủ công thức này; mọi thay đổi công thức hoặc ý nghĩa tham số cần được duyệt lại trước khi thay đổi code.

- Neighbor j chỉ được xét nếu distance d_ij <= radius_m.
- Distance weight tuyến tính: w_ij = max(0, 1 - d_ij / radius_m).
- Weighted neighbor prevalence:
  p_neighbor_i = sum(w_ij * I_j/N_j) / sum(w_ij), nếu tổng weight > 0; nếu không thì 0.
- Local prevalence p_local_i = I_i/N_i.
- Effective pressure: lambda_i = beta * (p_local_i + alpha * p_neighbor_i), alpha = 1.0 theo quyết định đã duyệt.
- KAN-23 chỉ tính adjacency-derived weights, local/neighbor prevalence và infection pressure (lambda_i).
- KAN-24 sở hữu time loop và numerical integration: tính infection/recovery flows, cập nhật state theo từng time step và bảo toàn S+I+R. KAN-23 không triển khai time loop hoặc cập nhật state theo thời gian.

Đây là giả định mô phỏng học thuật cấp building, không phải epidemiological truth, mô hình đã hiệu chuẩn dịch tễ hay khuyến nghị y tế.

## 5. Data Contract

Neighbor record tối thiểu:
- source_building_id, target_building_id, distance_m.
- Current population and infected count/prevalence from the same simulation/time_step.
- Weight có thể được tính trong KAN-23 từ distance_m và radius_m; không cần coi weight từ upstream là nguồn sự thật.
- Deterministic ordering by IDs.
- No self-edge; no duplicate edge.
- Undirected adjacency chỉ khi nghiệp vụ xác nhận khoảng cách đối xứng; infection influence có thể tính có hướng nếu weighting/config khác nhau.

Input validation:
- population N phải finite và > 0.
- infected I phải finite, >= 0 và <= N.
- distance_m phải finite và >= 0.
- radius_m phải finite và > 0.
- beta và alpha phải finite và >= 0. Gamma và dt không thuộc input KAN-23; KAN-24 chịu trách nhiệm validate gamma/dt.
- Building bị loại khỏi tập eligible theo policy KAN-22 không được tham gia tính local prevalence hoặc neighbor contribution.
- Input thiếu/không hợp lệ phải được xử lý rõ ràng theo contract; không tự thay bằng 0 để tiếp tục tính pressure.

Interface gợi ý để tách KAN-23 khỏi time integration của KAN-24 (tên/kiểu cuối cùng có thể điều chỉnh theo convention hiện có của backend):

```rust
pub struct BuildingInfectionInput {
    pub building_id: Uuid,
    pub population: f64,
    pub infected: f64,
}

pub struct NeighborEdge {
    pub source_building_id: Uuid,
    pub target_building_id: Uuid,
    pub distance_m: f64,
}

pub struct InfectionPressure {
    pub building_id: Uuid,
    pub local_prevalence: f64,
    pub neighbor_prevalence: f64,
    pub infection_pressure: f64,
}
```

## 6. Implementation Plan

### Step 1 – Review Spatial and SIR Contracts
- [ ] Kiểm tra KAN-21 distance output và radius semantics.
- [ ] Kiểm tra KAN-22 population/state representation và eligible building set.
- [x] Xác nhận KAN-22 code/data contract hiện diện trên base branch.
- [x] Đối chiếu beta units với KAN-18; gamma và dt validation thuộc KAN-24.
- [ ] Xác định adjacency representation.
- [ ] Ghi rõ model assumptions và limitations.

### Step 2 – Confirm Approved Infection Formula
- [x] Người dùng duyệt self contribution và neighbor contribution: có cả hai.
- [x] Người dùng duyệt alpha = 1.0.
- [x] Người dùng duyệt distance semantics footprint-to-footprint.
- [x] Chốt boundary behavior: neighbor được xét khi d_ij <= radius_m; tại đúng radius weight = 0.
- [x] Chốt empty/zero-weight behavior: p_neighbor_i = 0.
- [x] Chốt beta và dt dùng cùng time unit với KAN-18.
- [ ] Ghi công thức vào backend/docs trước implementation.

### Step 3 – Implement Neighbor Pressure
- [x] Build deterministic adjacency list.
- [x] Validate population, infected state, distance, radius, beta và alpha theo Data Contract.
- [x] Tính distance weight theo công thức đã duyệt.
- [x] Tính local prevalence và weighted neighbor prevalence.
- [x] Tính lambda_i = beta * (p_local_i + alpha * p_neighbor_i).
- [x] Chỉ nhận eligible building states từ caller; building excluded không thể tham gia adjacency/input set.
- [x] Xử lý không có neighbor/weight sum bằng 0.
- [x] Reject duplicate undirected edge và self-edge.
- [ ] Không triển khai time loop hoặc state transition thuộc KAN-24.

### Step 4 – Tests
- [x] Local-only: không có neighbors thì p_neighbor = 0 nhưng local prevalence vẫn được tính.
- [x] Có cả local prevalence và infected neighbor thì pressure phản ánh cả hai thành phần.
- [x] Neighbor có I_j = 0 không đóng góp neighbor prevalence.
- [x] Neighbor gần hơn có weight lớn hơn neighbor xa hơn (với cùng radius và các điều kiện khác).
- [x] Neighbor tại đúng radius có weight = 0.
- [x] Neighbor ngoài radius không được đưa vào adjacency/không đóng góp.
- [x] Zero total weight và empty neighbors không phát sinh NaN/divide-by-zero.
- [x] Weighted neighbor prevalence nằm trong [0,1] khi mọi input state hợp lệ.
- [x] Thay đổi thứ tự adjacency input không làm thay đổi kết quả ngoài sai số floating-point cho phép.
- [x] Duplicate edge và self-edge được từ chối theo contract.
- [x] Reject population <= 0, non-finite population, infected < 0 hoặc infected > population.
- [x] Reject distance âm/không finite, radius <= 0/không finite, beta/alpha âm hoặc không finite.
- [x] Building bị exclude theo KAN-22 không tham gia tính toán (caller chỉ truyền eligible states).
- [x] Beta = 0 cho infection pressure = 0.
- [x] Đối chiếu kết quả với tính toán độc lập bằng fixture nhỏ.

### Step 5 – Documentation and Integration Readiness
- [ ] Tài liệu hóa công thức, units, input validation và assumptions.
- [ ] Cung cấp interface pressure output cho KAN-24.
- [ ] Chạy backend tests, fmt, Clippy và CI.
- [ ] Đối chiếu kết quả mẫu bằng tính toán độc lập.

## 7. Acceptance Criteria

- [ ] Xác định được tập neighbors theo KAN-21.
- [ ] Pressure dùng infected state, distance và beta theo đúng Mathematical Contract.
- [ ] Local contribution luôn được tính cho building eligible, kể cả khi không có neighbors.
- [ ] Radius, beta và alpha được cấu hình/validate tập trung, không hard-code rải rác.
- [ ] Input không hợp lệ bị từ chối hoặc xử lý theo policy rõ ràng; không tạo NaN.
- [ ] Kết quả deterministic và có tests cho boundary, ordering và invalid inputs.
- [ ] KAN-23 cung cấp infection pressure để KAN-24 tích hợp, không sở hữu time loop/state transition.
- [ ] Công thức và giới hạn mô hình được tài liệu hóa.

## 8. Deliverables

1. Infection pressure model và adjacency representation.
2. Cấu hình parameter và validation.
3. Unit tests.
4. Tài liệu toán học/assumptions và interface pressure output cho engine.

## 9. Definition of Done

KAN-23 hoàn thành khi có quy tắc lây nhiễm cấp building đã được duyệt, sử dụng neighbor distance và infected state, có input validation, tests và interface rõ ràng để KAN-24 tích hợp. KAN-23 chỉ tính infection pressure; KAN-24 chịu trách nhiệm mô phỏng theo thời gian và cập nhật S/I/R.

## 10. Approved Infection Model (2026-11-16)

Người dùng đã duyệt các lựa chọn: **local infection = yes**, **alpha = 1**, **distance = footprint-to-footprint**. Dùng baseline dưới đây làm contract implementation; không thay công thức tùy ý trong KAN-24.

Với building i và neighbor j:

- p_local_i = I_i / N_i
- w_ij = max(0, 1 - d_ij / radius_m)
- p_neighbor_i = sum(w_ij * (I_j / N_j)) / sum(w_ij) khi tổng weight > 0; nếu không thì bằng 0.
- lambda_i = beta * (p_local_i + 1.0 * p_neighbor_i)

Trong đó:
- d_ij là khoảng cách ngắn nhất footprint-to-footprint theo mét (KAN-21).
- Neighbor chỉ được xét khi d_ij <= radius_m; tại đúng radius, weight bằng 0.
- Local prevalence luôn được tính, kể cả building không có neighbors.
- alpha = 1.0 là giá trị đã duyệt, nhưng nên biểu diễn thành config rõ ràng để dễ sensitivity analysis; không rải literal trong code.
- beta và dt dùng cùng time unit như KAN-18; lambda_i có đơn vị nghịch đảo thời gian.
- Nếu tổng neighbor weight bằng 0, p_neighbor_i = 0; không NaN/divide-by-zero.
- Mô hình này là giả định mô phỏng học thuật cấp building, không phải mô hình đã hiệu chuẩn dịch tễ hay khuyến nghị y tế.

New infection flow cho mỗi bước được KAN-24 tính theo min(S_i, lambda_i * S_i * dt). Recovery flow là min(I_i, gamma * I_i * dt). Cả hai flow được tính từ cùng snapshot đầu bước; áp dụng đồng thời để tạo state tiếp theo. KAN-23 không thực hiện các flow hoặc cập nhật state này.

### Additional review checklist
- [ ] Kiểm tra các validation rules trong Data Contract đã được implement và có tests.
- [ ] Test local-only, infected/uninfected neighbor, distance tăng, distance = radius và ngoài radius.
- [ ] Test weighted prevalence trong [0,1] với input hợp lệ.
- [x] Test alpha=1.0 được cấu hình tập trung, không hard-code rải rác.
- [ ] Test zero total weight và empty neighbors không phát sinh NaN.
- [ ] Test thứ tự adjacency input không ảnh hưởng kết quả ngoài sai số floating-point cho phép.
- [ ] Đối chiếu kết quả với tính toán độc lập bằng fixture nhỏ.
