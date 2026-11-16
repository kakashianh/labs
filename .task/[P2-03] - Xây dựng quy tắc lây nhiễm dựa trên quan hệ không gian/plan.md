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
| KAN-21 | Spatial neighbors và distance_m đã được kiểm thử |
| KAN-22 | Initial S/I/R và population per building |
| KAN-18 | SIR numerical conventions làm reference |

## 3. Scope

### In scope
- Định nghĩa adjacency list/graph từ spatial query.
- Định nghĩa distance-decay kernel.
- Tính infectious prevalence/pressure từ neighboring buildings.
- Cấu hình radius, beta và các hệ số cần thiết.
- Xử lý building không có neighbors, population bằng 0/thiếu và self contribution.
- Unit tests cho within/outside radius và sensitivity.

### Out of scope
- Individual movement/contact networks.
- Phòng, căn hộ, tầng hoặc agent-level simulation.
- Machine learning dự đoán lây nhiễm.
- Statistical hotspot detection nâng cao.
- Tự chọn tham số dịch tễ từ dữ liệu thực khi chưa có calibration.

## 4. Proposed Mathematical Contract – Approval Required

Jira chưa chỉ định một công thức duy nhất. Không triển khai công thức tùy ý mà không ghi rõ giả định. Đề xuất baseline đơn giản, giải thích được:

- Neighbor j chỉ được xét nếu distance d_ij <= radius.
- Distance weight tuyến tính: w_ij = max(0, 1 - d_ij / radius).
- Weighted neighbor prevalence:
  p_neighbor_i = sum(w_ij * I_j/N_j) / sum(w_ij), nếu tổng weight > 0; nếu không thì 0.
- Local prevalence p_local_i = I_i/N_i.
- Effective pressure: lambda_i = beta * (p_local_i + alpha * p_neighbor_i), với alpha là neighbor influence multiplier cấu hình được.
- New infections per time step được engine tính từ lambda_i và S_i; KAN-24 chịu trách nhiệm numerical integration và conservation.

Đây là baseline proposal, không phải epidemiological truth. Trước implementation cần:
- Chốt có tính self prevalence hay chỉ neighbor transmission.
- Chốt alpha/default và giới hạn.
- Chốt radius semantics: khoảng cách footprint-to-footprint hay centroid.
- Chốt beta unit và dt compatibility.
- Ghi rõ đây là mô hình mô phỏng học thuật ở cấp building, không dùng làm khuyến nghị y tế.

Nếu người dùng không duyệt công thức, dừng trước khi hard-code model.

## 5. Data Contract

Neighbor record tối thiểu:
- source_building_id, target_building_id, distance_m, weight.
- Current population and infected count/prevalence from same simulation/time_step.
- Deterministic ordering by IDs.
- No self-edge; no duplicate edge.
- Undirected adjacency chỉ khi nghiệp vụ xác nhận khoảng cách đối xứng; infection influence có thể tính có hướng nếu weighting/config khác nhau.

## 6. Implementation Plan

### Step 1 – Review Spatial and SIR Contracts
- [ ] Kiểm tra KAN-21 distance output và radius semantics.
- [ ] Kiểm tra KAN-22 population/state representation.
- [ ] Đối chiếu beta, gamma, dt units với KAN-18.
- [ ] Xác định adjacency representation.
- [ ] Ghi rõ model assumptions và limitations.

### Step 2 – Approve Infection Formula
- [ ] Review baseline formula với người dùng.
- [ ] Chốt self contribution và neighbor contribution.
- [ ] Chốt distance kernel và alpha.
- [ ] Chốt boundary behavior tại distance = radius.
- [ ] Chốt empty-neighbor behavior.
- [ ] Chốt parameter ranges và units.
- [ ] Ghi công thức vào backend/docs trước implementation.

### Step 3 – Implement Neighbor Pressure
- [ ] Build deterministic adjacency list.
- [ ] Tính distance weight theo công thức đã duyệt.
- [ ] Tính weighted infectious prevalence.
- [ ] Bỏ qua building không đủ population hợp lệ theo policy KAN-22.
- [ ] Xử lý không có neighbor/weight sum bằng 0.
- [ ] Tránh double-count và self-edge.

### Step 4 – Tests
- [ ] Neighbor trong radius có ảnh hưởng.
- [ ] Neighbor ngoài radius không ảnh hưởng.
- [ ] Distance tăng làm weight giảm theo kernel đã chốt.
- [ ] I_j=0 không đóng góp infectious pressure.
- [ ] Beta=0 không tạo infection pressure.
- [ ] Empty neighbors cho kết quả đúng.
- [ ] Test deterministic và ordering.
- [ ] Test invalid N/state và extreme parameters.

### Step 5 – Documentation and Integration Readiness
- [ ] Tài liệu hóa công thức, units và assumptions.
- [ ] Cung cấp interface cho KAN-24.
- [ ] Chạy backend tests, fmt, Clippy và CI.
- [ ] Đối chiếu kết quả mẫu bằng tính toán độc lập.

## 7. Acceptance Criteria

- [ ] Xác định được tập neighbors theo KAN-21.
- [ ] Pressure dùng infected state, distance và beta.
- [ ] Radius, beta và các hệ số được cấu hình, không hard-code rải rác.
- [ ] Kết quả deterministic và có tests.
- [ ] Có test building trong/ngoài radius.
- [ ] Công thức và giới hạn mô hình được tài liệu hóa.

## 8. Deliverables

1. Infection pressure model và adjacency representation.
2. Cấu hình parameter và validation.
3. Unit tests.
4. Tài liệu toán học/assumptions cho engine.

## 9. Definition of Done

KAN-23 hoàn thành khi có quy tắc lây nhiễm cấp building đã được duyệt, sử dụng neighbor distance và infected state, có tests và interface rõ ràng để KAN-24 tích hợp.
