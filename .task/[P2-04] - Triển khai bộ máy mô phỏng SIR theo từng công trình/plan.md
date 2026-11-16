## 1. Objective

Mở rộng SIR engine Phase 1 thành spatial building-level simulation. Mỗi building có population và S/I/R state riêng; infection pressure có thể đến từ building lân cận theo KAN-23.

Expected result:
- Tính trạng thái từng building theo time_step.
- Giữ conservation và domain invariants cho từng building.
- Persist kết quả gắn simulation_id, building_id và time_step.
- Truy vấn lại được run đã hoàn tất.
- Tái sử dụng SIR domain conventions; không tạo thuật toán SIR tổng thể thứ hai.

## 2. Dependencies

| Dependency | Required state |
|---|---|
| KAN-22 | Simulation run và per-building initial state |
| KAN-23 | Approved spatial infection-pressure contract |
| KAN-18 | Deterministic Euler SIR engine reference |
| KAN-21 | Spatial neighbor relationships |

## 3. Scope

### In scope
- Spatial SIR domain model.
- S/I/R transition theo từng building và từng bước thời gian.
- Kết hợp local recovery gamma với infection pressure KAN-23.
- Synchronous time-step update: mọi building ở bước t+1 dùng snapshot trạng thái bước t, tránh phụ thuộc thứ tự iteration.
- Persistence theo simulation_id/building_id/time_step.
- Transaction/batch persistence và giới hạn tài nguyên.
- Tests numerical invariants, no-seed, seed, spread và deterministic behavior.

### Out of scope
- Individual-level agent simulation.
- Movement, contact tracing hoặc mobility network.
- GPU/distributed simulation.
- Queue/worker nếu chưa được chứng minh cần thiết.
- Scenario comparison, interventions và advanced calibration.
- Thay đổi Phase 1 API contract nếu không cần thiết.

## 4. Numerical and State Invariants

- Mỗi building i có population N_i > 0.
- S_i, I_i, R_i không âm và hữu hạn.
- S_i + I_i + R_i = N_i trong tolerance đã định nghĩa.
- Recovery: dR_i/dt = gamma * I_i.
- Infection term lấy từ KAN-23; không tự viết lại công thức khác.
- Tất cả building được cập nhật đồng bộ từ snapshot ở time_step hiện tại.
- Không làm tròn intermediate values mỗi bước.
- Numerical instability phải fail run rõ ràng; không persist partial success như completed.
- Chốt rounding policy chỉ ở API/display boundary, không làm mất conservation.

## 5. Persistence and Resource Design

Trước implementation:
- Kiểm tra schema KAN-22.
- Ước lượng số rows = eligible_buildings × (steps + 1).
- Đặt max buildings, max steps, max rows hoặc giới hạn duration cấu hình.
- Tránh tải toàn bộ time series không giới hạn vào memory.
- Chọn batch insert/transaction strategy.
- Lưu run status (queued/running/completed/failed) chỉ nếu phù hợp execution model hiện tại; không giả định async job đã có.
- Lưu failure reason an toàn và không để run ở trạng thái completed khi persist thiếu.

## 6. Implementation Plan

### Step 1 – Review Dependencies and Freeze Engine Contract
- [ ] Đối chiếu initial-state schema KAN-22.
- [ ] Đối chiếu infection formula KAN-23 đã được duyệt.
- [ ] Xác định input/output types và time-step semantics.
- [ ] Xác định population/state precision.
- [ ] Ước lượng row volume và resource limits.

### Step 2 – Design Spatial SIR Domain
- [ ] Thiết kế pure domain types cho building state và simulation config.
- [ ] Tách domain computation khỏi database/HTTP.
- [ ] Xác định deterministic ordering.
- [ ] Xác định synchronous snapshot update.
- [ ] Tái sử dụng validation/error conventions của KAN-18.

### Step 3 – Implement Per-building Time-step Engine
- [ ] Tính infection pressure qua KAN-23 interface.
- [ ] Tính recovery theo gamma.
- [ ] Cập nhật S/I/R cho từng building.
- [ ] Validate finite/non-negative/conservation sau mỗi bước.
- [ ] Dừng run khi numerical instability.
- [ ] Xử lý zero infected seed và no-neighbor cases.
- [ ] Đảm bảo cùng input cho cùng kết quả.

### Step 4 – Persist Simulation Results
- [ ] Thiết kế result table/index nếu chưa có.
- [ ] Lưu simulation_id, building_id, time_step, time, S, I, R.
- [ ] Tạo unique key cho (simulation_id, building_id, time_step).
- [ ] Persist theo batch có transaction boundary rõ ràng.
- [ ] Không tạo duplicate khi retry.
- [ ] Không đánh dấu completed khi thiếu kết quả.
- [ ] Thêm indexes phục vụ truy vấn theo run/time/building.

### Step 5 – Tests
- [ ] No infected seed: không tự phát sinh infection.
- [ ] Initial infected seed: state được bảo toàn.
- [ ] Neighbor infected: infection lan theo formula KAN-23.
- [ ] Outside-radius building không nhận neighbor contribution.
- [ ] Mọi building bảo toàn N ở mọi time_step.
- [ ] Không có S/I/R âm hoặc non-finite.
- [ ] Deterministic across repeated runs.
- [ ] Failure trong persist không để run completed.
- [ ] Test giới hạn steps/buildings/row count.
- [ ] Integration tests với PostGIS/PostgreSQL.

### Step 6 – Performance and Documentation
- [ ] Benchmark dataset 431 buildings.
- [ ] Ghi thời gian compute và persist, memory/row count.
- [ ] Kiểm tra query patterns và indexes.
- [ ] Chỉ tối ưu sau khi có benchmark.
- [ ] Cập nhật docs về công thức, limits, persistence và failure semantics.
- [ ] Chạy backend tests, fmt, Clippy và CI.

## 7. Acceptance Criteria

- [ ] Có S/I/R cho mỗi building và time_step.
- [ ] Dùng neighbor infection rule đã duyệt.
- [ ] Conservation per building trong tolerance.
- [ ] Không có state âm/non-finite.
- [ ] Hỗ trợ no-seed, seed và spread scenarios.
- [ ] Kết quả persist theo simulation_id/building_id/time_step.
- [ ] Có thể truy xuất run và trạng thái.
- [ ] Có resource limits và tests cho failure paths.

## 8. Deliverables

1. Spatial SIR domain engine.
2. Persistence schema/repository.
3. Tests và benchmark.
4. Tài liệu model, limits và run lifecycle.

## 9. Definition of Done

KAN-24 hoàn thành khi engine mô phỏng đồng bộ SIR theo từng building, kết hợp infection pressure KAN-23, bảo toàn invariants và persist kết quả có thể truy xuất theo simulation run.
