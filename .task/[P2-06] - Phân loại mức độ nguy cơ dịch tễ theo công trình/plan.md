## 1. Objective

Phân loại risk level cho mỗi building tại một simulation/time_step dựa trên tỷ lệ infected I/N; tổng hợp số building và population theo risk level ở cấp region.

Các ngưỡng trong task này phục vụ trực quan hóa và phân tích mô phỏng học thuật của đồ án, không phải tiêu chuẩn y tế hoặc ngưỡng cảnh báo dịch tễ chính thức.

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
- Phân loại riêng các building không đủ dữ liệu thành Unclassified.
- Tổng hợp số building, population và infected count theo region/risk.
- Cung cấp service/API contract cho KAN-28 integration.
- Unit and integration tests cho threshold boundaries và aggregation.

### Out of scope
- Dự đoán nguy cơ tương lai.
- Medical diagnosis hoặc khuyến nghị y tế.
- Thay thế infection model.
- Risk score dựa trên dữ liệu chưa có.
- Hard-code thresholds ở nhiều tầng code.

## 4. Risk Threshold Contract

### 4.1 Default thresholds (đã được duyệt)

| Risk level | Điều kiện | Ý nghĩa trong demo |
|---|---|---|
| Low | infection_ratio <= 0.05 | Tỷ lệ infected thấp |
| Medium | 0.05 < infection_ratio <= 0.15 | Tỷ lệ infected trung bình |
| High | infection_ratio > 0.15 | Tỷ lệ infected cao |
| Unclassified | Không tính được ratio hợp lệ | Thiếu hoặc không hợp lệ dữ liệu population |

- `risk_low_max = 0.05`.
- `risk_medium_max = 0.15`.
- Điều kiện hợp lệ: `0 <= risk_low_max < risk_medium_max <= 1`.
- Ratio được tính từ raw values: `infected / population`; không dùng số đã format/round.
- Boundary được inclusive ở upper bound của từng nhóm: đúng 5% là Low, đúng 15% là Medium.
- Các ngưỡng này là default configurable phục vụ mô phỏng/hiển thị của đồ án, không được diễn giải là chuẩn y tế.

### 4.2 Configuration and reproducibility

- Tập trung cấu hình tại domain/config layer; không hard-code thresholds rải rác trong API hoặc SQL.
- Mỗi lần phân loại phải có snapshot cấu hình để tái lập kết quả, gồm:
  - `version` (ví dụ `risk-v1`).
  - `source` (ví dụ `project_default` hoặc `request_override`).
  - `risk_low_max`.
  - `risk_medium_max`.
- Lưu snapshot trong `simulation_runs.simulation_config` dưới object riêng `risk_classification`; không ghi đè các tham số SIR hiện có. Nếu run chưa có object này, sử dụng default ở trên và lưu snapshot một cách an toàn trước/khi tạo kết quả phân loại.
- Khi có request override, validate đầy đủ và lưu đúng snapshot đã dùng; kết quả đã phân loại không được âm thầm đổi ngưỡng ở lần đọc sau.

### 4.3 Missing/zero population policy

- Population missing, null, hoặc `N <= 0`: risk level là `unclassified`, ratio là `null`, kèm reason rõ ràng.
- Không gán các trường hợp trên thành Low và không đưa chúng vào mẫu số của regional infection ratio.
- Vẫn expose số lượng building unclassified để frontend không làm mất dấu dữ liệu bị loại.
- Với population hợp lệ, kiểm tra S/I/R và ratio theo invariant của KAN-22/KAN-24 trước khi phân loại; dữ liệu không hợp lệ phải được xử lý như unclassified hoặc trả lỗi dữ liệu nhất quán theo domain contract, không tạo ratio giả.

## 5. Result and API Contract

### 5.1 Per-building classification

Mỗi item cần có tối thiểu:
- `building_id`, `region_id`.
- `population`, `infected`.
- `infection_ratio` (number hoặc null).
- `risk_level`: `low | medium | high | unclassified`.
- `unclassified_reason` (null nếu đã phân loại được).

### 5.2 Regional aggregation

- Group theo `region_id` và `risk_level`.
- Mỗi risk bucket trả `building_count`, `population`, `infected`.
- Regional infection ratio được tính bằng `SUM(infected) / SUM(population)` trên các building có population hợp lệ; tuyệt đối không lấy trung bình đơn giản các infection_ratio của từng building.
- Trả thêm `unclassified_building_count` (và thông tin population/infected nếu có thể xác định hợp lệ) để minh bạch dữ liệu bị loại.
- Aggregate phải tính trên toàn bộ tập building đã filter, không phụ thuộc pagination của endpoint danh sách.
- Region không có building đủ điều kiện vẫn trả aggregate zero/empty theo contract thống nhất.

### 5.3 API shape

- Endpoint danh sách per-building risk: filter theo `simulation_id`, `time_step`, `region_id`; hỗ trợ pagination và stable ordering theo `building_id`.
- Endpoint regional risk summary: filter theo `simulation_id`, `time_step`, `region_id`; không áp dụng pagination.
- Reuse validation, completed-run checks và repository/query logic từ KAN-25 khi phù hợp; tránh sao chép JOIN/filter logic.
- Cập nhật OpenAPI và error contract. Chốt route naming cụ thể theo convention hiện có ở bước inspect trước implementation.

## 6. Implementation Plan

### Step 1 – Inspect Result and Population Contracts
- [x] Đối chiếu KAN-22 population representation.
- [x] Đối chiếu KAN-24 result precision và S/I/R invariants.
- [x] Đối chiếu KAN-25 filters, completed-run validation và aggregation.
- [x] Xác định cách đọc/lưu `risk_classification` snapshot trong `simulation_runs.simulation_config`.
- [ ] Xác nhận migration có cần thiết không; ưu tiên tận dụng JSON config hiện có nếu đáp ứng được.

### Step 2 – Implement Risk Classification Domain
- [x] Tạo typed risk level enum: Low, Medium, High, Unclassified.
- [x] Tạo config tập trung với default Low <= 0.05, Medium <= 0.15, High > 0.15.
- [x] Validate thresholds: finite, nằm trong [0,1], và low < medium.
- [x] Tính I/N từ raw values, không dùng giá trị đã format.
- [x] Áp dụng boundary rules chính xác như mục 4.1.
- [x] Population missing/null/N<=0 trả Unclassified + reason, ratio null.
- [x] Domain deterministic; API POST lưu snapshot config đã sử dụng.

### Step 3 – Implement Regional Aggregation
- [x] Group building by region and risk level.
- [x] Aggregate building count, population và infected count cho từng bucket.
- [x] Tính regional infection ratio theo tổng infected / tổng population hợp lệ.
- [x] Theo dõi unclassified building count riêng.
- [x] Xử lý region không có building eligible khi truyền region_id cụ thể.
- [x] Đảm bảo aggregation không phụ thuộc page/page_size.

### Step 4 – Expose Result to API
- [x] Thêm API/service theo convention hiện có.
- [x] Filter simulation_id, time_step và region_id.
- [x] Trả per-building risk fields và regional summary theo contract mục 5.
- [ ] Reuse KAN-25 validation/repository pattern; query hiện tại dùng cùng JOIN và predicate đã kiểm tra ở KAN-25.
- [x] Cập nhật OpenAPI/docs và error contract.

### Step 5 – Tests and Verification
- [x] Viết unit tests cho Low, Medium, High và chính xác tại 5%/15% (chờ CI xác nhận).
- [x] Viết unit tests cho threshold invalid, non-finite và out-of-range (chờ CI xác nhận).
- [x] Viết unit tests cho population missing/null và N=0; N<0 bị reject (chờ CI xác nhận).
- [x] Viết unit tests cho weighted regional ratio với population khác nhau (chờ CI xác nhận).
- [x] Viết unit tests cho unclassified count và region empty (chờ CI xác nhận).
- [x] Thêm PostGIS integration test kiểm tra risk buckets và regional counts (chờ CI xác nhận).
- [x] API aggregate không nhận pagination; integration test kiểm tra aggregate toàn bộ dữ liệu (chờ CI xác nhận).
- [x] Thêm integration test cho persist threshold snapshot (chờ CI xác nhận).
- [ ] Chạy backend tests, fmt, Clippy và CI.

## 7. Acceptance Criteria

- [ ] Building có population hợp lệ được tính infection ratio từ raw I/N và phân loại nhất quán.
- [ ] Default thresholds là Low <= 5%, Medium > 5% và <= 15%, High > 15%; có thể cấu hình và được validate.
- [ ] Threshold boundary đúng 5% là Low, đúng 15% là Medium.
- [ ] N=0/missing population được gán Unclassified, ratio null và có reason; không gán Low giả.
- [ ] Mỗi lần phân loại có configuration snapshot để tái lập.
- [ ] Có tổng số building/population/infected theo risk và region, kèm số building unclassified.
- [ ] Regional ratio tính theo tổng infected / tổng population hợp lệ.
- [ ] Aggregate tính trên toàn bộ tập dữ liệu, không phụ thuộc pagination.
- [ ] API cung cấp đủ dữ liệu cho Frontend P2-08.
- [ ] Có unit/integration tests cho boundaries, missing data, aggregation và configuration.

## 8. Deliverables

1. Risk classification domain service.
2. Configurable thresholds và configuration snapshot.
3. Per-building risk API.
4. Regional risk aggregation API.
5. Unit/integration tests và OpenAPI docs.

## 9. Definition of Done

KAN-26 hoàn thành khi risk level Low/Medium/High/Unclassified được tính nhất quán từ I/N với ngưỡng cấu hình đã lưu snapshot, tổng hợp chính xác theo region và được cung cấp qua API để frontend sử dụng.
