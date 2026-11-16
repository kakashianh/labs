## 1. Objective

Phân loại risk level cho mỗi building tại một simulation/time_step dựa trên tỷ lệ infected I/N; tổng hợp số building và population theo risk level ở cấp region.

## 2. Dependencies

| Dependency | Required state |
|---|---|
| KAN-24 | Per-building S/I/R result |
| KAN-25 | Query contract for simulation/time_step and region |
| KAN-22 | Population per building |

## 3. Scope

### In scope
- Tính infection ratio = I/N.
- Phân loại Low, Medium, High bằng configurable thresholds.
- Tổng hợp số building, population và infected count theo region/risk.
- Xử lý population thiếu hoặc N=0.
- Cung cấp service/API contract cho KAN-25/KAN-28 integration.
- Unit and integration tests cho threshold boundaries.

### Out of scope
- Dự đoán nguy cơ tương lai.
- Medical diagnosis hoặc khuyến nghị y tế.
- Thay thế infection model.
- Risk score dựa trên dữ liệu chưa có.
- Hard-code thresholds ở nhiều tầng code.

## 4. Risk Threshold Contract

Jira yêu cầu Low/Medium/High nhưng chưa chỉ định ngưỡng. Cần chốt cấu hình trước implementation.

Proposed contract:
- risk_low_max: upper bound của Low.
- risk_medium_max: upper bound của Medium.
- High: ratio > risk_medium_max.
- 0 <= risk_low_max < risk_medium_max <= 1.
- Quy tắc boundary phải rõ ràng, ví dụ ratio <= low_max là Low; low_max < ratio <= medium_max là Medium; còn lại High.
- Config có version/source và được gắn với run hoặc được lưu cùng result metadata để kết quả có thể tái lập.
- Không trình bày các ngưỡng này như chuẩn y tế; đây là phân lớp mô phỏng của đồ án.

## 5. Implementation Plan

### Step 1 – Inspect Result and Population Contracts
- [ ] Đối chiếu KAN-22 population representation.
- [ ] Đối chiếu KAN-24 result precision.
- [ ] Đối chiếu KAN-25 filters và aggregation.
- [ ] Chốt cách xử lý missing/zero population.
- [ ] Chốt thresholds và configuration source với người dùng.

### Step 2 – Implement Risk Classification Domain
- [ ] Tạo typed risk level enum.
- [ ] Validate thresholds.
- [ ] Tính I/N từ raw values, không dùng giá trị đã format.
- [ ] Phân loại theo boundary rules.
- [ ] Xử lý N=0/missing theo policy rõ ràng (không tự coi là Low).
- [ ] Giữ deterministic result.

### Step 3 – Implement Regional Aggregation
- [ ] Group building by region and risk level.
- [ ] Aggregate building count.
- [ ] Aggregate population and infected count.
- [ ] Tính tỷ lệ tổng hợp theo tổng I/tổng N, không lấy trung bình đơn giản các ratio nếu population khác nhau.
- [ ] Xử lý region không có building eligible.

### Step 4 – Expose Result to API
- [ ] Thêm API/service theo convention hiện có.
- [ ] Filter simulation_id, time_step và region_id.
- [ ] Trả risk_level, infection_ratio và summary.
- [ ] Cập nhật OpenAPI/docs.
- [ ] Không lặp query logic nếu có thể compose với KAN-25 repository.

### Step 5 – Tests and Verification
- [ ] Test từng level và chính xác tại threshold boundaries.
- [ ] Test threshold invalid/out-of-range.
- [ ] Test N=0/missing population.
- [ ] Test weighted regional ratio.
- [ ] Test region empty.
- [ ] Test consistency giữa per-building risk và regional counts.
- [ ] Chạy backend tests, fmt, Clippy và CI.

## 6. Acceptance Criteria

- [ ] Mỗi building hợp lệ có infection ratio và risk level.
- [ ] Thresholds configurable và được validate.
- [ ] N=0/missing population xử lý an toàn, không gán Low giả.
- [ ] Có tổng số building/population/infected theo risk và region.
- [ ] Regional ratio tính theo tổng infected / tổng population.
- [ ] API có thể cung cấp dữ liệu cho Frontend.
- [ ] Có tests cho boundaries và aggregation.

## 7. Deliverables

1. Risk classification domain service.
2. Configurable thresholds.
3. Regional risk aggregation/API.
4. Unit/integration tests và docs.

## 8. Definition of Done

KAN-26 hoàn thành khi risk level Low/Medium/High được tính nhất quán từ I/N với ngưỡng cấu hình, tổng hợp chính xác theo region và được cung cấp qua API.
