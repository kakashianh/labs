# KAN-10 – Hoàn thiện luồng trực quan hóa và đồng bộ kết quả mô phỏng

## 1. Objective

Tích hợp luồng cấu hình và chạy mô phỏng Backend ở KAN-9 với thành phần trực quan hóa SIR ở KAN-20, đảm bảo người dùng có thể nhập tham số, chạy mô phỏng và xem kết quả trong một quy trình liên tục.

Trọng tâm là đồng bộ dữ liệu theo từng lần chạy, trạng thái giao diện và time_step đang chọn. Task này kiểm thử/tinh chỉnh tích hợp, không viết lại API, SIR engine hoặc chart component.

**Expected result:**
- Người dùng cấu hình và khởi chạy mô phỏng từ giao diện.
- Frontend gửi request và nhận kết quả từ API.
- Chart cùng chỉ số S/I/R hiển thị đúng response của lần chạy hiện tại.
- Chạy mô phỏng mới thay thế kết quả cũ nhất quán.
- Loading, success, empty và error states được đồng bộ.
- Có kiểm thử end-to-end cho cấu hình → chạy → xem kết quả.

## 2. Scope

### In scope
- Rà soát KAN-9 và KAN-20.
- Tích hợp form/configuration với SIR simulation API.
- Kết nối API response tới chart và metrics.
- Quản lý trạng thái của lần chạy mô phỏng hiện tại.
- Đồng bộ selected time_step giữa chart, slider và metrics.
- Xử lý loading, success, empty và error.
- Ngăn response cũ ghi đè kết quả request mới.
- Kiểm thử chạy lần đầu, chạy lại và lỗi API.
- Cập nhật hướng dẫn sử dụng.

### Out of scope
- Viết lại SIR engine (KAN-18) hoặc endpoint (KAN-19).
- Xây dựng lại chart/playback component (KAN-20).
- Lưu nhiều simulation runs hoặc so sánh scenario.
- Export báo cáo hoặc quản lý can thiệp.
- Gán kết quả SIR lên từng công trình hoặc bản đồ hotspot.
- Mô phỏng lây nhiễm không gian (Phase 2).

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Frontend | React + TypeScript + Vite | Form và state orchestration |
| API | Rust + Axum | Chạy mô phỏng |
| Chart | Component từ KAN-20 | Trực quan hóa |
| Testing | Tooling hiện có | Integration/E2E verification |

Tái sử dụng API client, form components và chart component. Không thêm backend/service hoặc thư viện lớn nếu không cần thiết.

## 4. End-to-End User Workflow

~~~text
Open SIR Simulation
        |
        v
Enter Initial State + Parameters
        |
        v
Validate Form Input
        |
        v
Submit Simulation Request
        |
        v
Show Loading State
        |
        v
POST /api/v1/simulations/sir
        |
        +---- API Error ------> Show Error
        |
        v
Receive Simulation Result
        |
        v
Set Current Run Result
        |
        v
Render SIR Chart + Current Metrics
        |
        v
Select / Play / Pause / Reset Time Step
~~~

Khi submit request mới, cần có quy tắc rõ ràng với kết quả cũ. Khuyến nghị giữ kết quả cũ trong lúc request mới tải nhưng đánh dấu loading; chỉ thay thế khi request mới thành công. Nếu request mới thất bại, hiển thị lỗi và không gắn nhầm dữ liệu cũ với tham số mới.

## 5. Implementation Plan

### Step 1 – Review Backend and Visualization Contracts
- [ ] Kiểm tra form/configuration data hiện có.
- [ ] Kiểm tra API endpoint và request DTO từ KAN-9.
- [ ] Kiểm tra response DTO và time_step convention.
- [ ] Kiểm tra props/state contract của SirResultChart từ KAN-20.
- [ ] Xác nhận cách hiển thị beta, gamma, dt và steps kèm đơn vị.
- [ ] Ghi nhận mismatch giữa API và Frontend.

### Step 2 – Integrate Configuration Form with Simulation API
- [ ] Kết nối submit handler với API client.
- [ ] Map form values sang đúng request DTO.
- [ ] Validate required fields và numeric ranges cơ bản ở Frontend.
- [ ] Không thay thế domain validation của Backend bằng Frontend validation.
- [ ] Disable submit hoặc chống double-submit khi request đang chạy.
- [ ] Hiển thị lỗi validation/API phù hợp.
- [ ] Không tự tính SIR ở Frontend.

### Step 3 – Manage Current Simulation Run State

State đề xuất:
- simulationStatus: idle, loading, success hoặc error.
- currentResult: response của lần chạy thành công hiện tại.
- request sequence nội bộ để phân biệt request.
- selectedTimeStep: mốc đang chọn trong currentResult.
- simulationError: lỗi request hiện tại nếu có.

- [ ] Tổ chức state ở component/page điều phối chung.
- [ ] Không để chart và metrics giữ các bản copy result độc lập.
- [ ] Gắn currentResult với một lần chạy cụ thể.
- [ ] Khi request mới thành công, thay currentResult bằng response mới.
- [ ] Reset selectedTimeStep về mốc đầu tiên khi thay result.
- [ ] Dừng playback của result cũ khi bắt đầu hoặc hoàn tất run mới.
- [ ] Ngăn response cũ ghi đè kết quả request mới bằng AbortController hoặc request sequence guard.
- [ ] Không cần database run ID nếu Backend chưa hỗ trợ persistent runs; dùng request identity nội bộ ở Frontend.

### Step 4 – Connect Result to Chart, Timeline and Metrics
- [ ] Truyền currentResult xuống chart/result components.
- [ ] Đồng bộ selectedTimeStep giữa slider, chart marker và metrics.
- [ ] Khi chọn time_step, lấy S/I/R từ đúng point trong currentResult.
- [ ] Play/pause/reset cập nhật cùng một selectedTimeStep state.
- [ ] Đảm bảo chart không hiển thị dữ liệu từ response trước.
- [ ] Kiểm tra result có một point hoặc points rỗng.
- [ ] Format số nhất quán nhưng không thay đổi raw data.

### Step 5 – Normalize Loading, Success, Empty and Error States
- [ ] Idle: hiển thị hướng dẫn hoặc trạng thái chưa có kết quả.
- [ ] Loading: hiển thị tiến trình request và ngăn submit trùng.
- [ ] Success: hiển thị chart và metrics từ currentResult.
- [ ] Error: hiển thị thông báo lỗi rõ ràng.
- [ ] Empty: xử lý response không có points như dữ liệu không hợp lệ/không có kết quả.
- [ ] Chốt UX khi chạy lại: giữ kết quả trước trong lúc loading hoặc ẩn chart; không trộn dữ liệu.
- [ ] Nếu request mới lỗi, phân biệt lỗi lần chạy mới với kết quả thành công trước đó.
- [ ] Không để lỗi mô phỏng làm mất bản đồ Cesium hoặc crash trang.

### Step 6 – Execute End-to-End Tests

| Scenario | Expected result |
|---|---|
| Mở trang lần đầu | Hiển thị trạng thái chưa có kết quả |
| Nhập tham số hợp lệ và submit | API được gọi và trả kết quả |
| API trả kết quả thành công | Chart và metrics hiển thị đúng |
| Chọn một time_step | Chart/slider/metrics đồng bộ |
| Play/pause/reset | Chỉ duyệt chuỗi kết quả hiện tại |
| Submit lần chạy mới | Kết quả mới thay thế kết quả cũ khi thành công |
| Request cũ trả về trễ | Không ghi đè kết quả của request mới |
| API validation error | Hiển thị lỗi phù hợp |
| Backend không khả dụng | Hiển thị error state, trang không crash |
| Response không có points | Không render chart sai hoặc crash |

- [ ] Tạo automated integration/E2E tests nếu repository có tooling phù hợp.
- [ ] Nếu chưa có E2E framework, tạo checklist thủ công có thể lặp lại.
- [ ] Chạy scenario với bộ tham số SIR mẫu.
- [ ] Ghi kết quả pass/fail.
- [ ] Sửa lỗi tích hợp trong phạm vi ticket.
- [ ] Chạy lại toàn bộ checklist sau khi sửa.

### Step 7 – Final Verification and Documentation
- [ ] Chạy Backend và Frontend theo hướng dẫn dự án.
- [ ] Xác nhận API base URL và CORS/configuration.
- [ ] Chạy Backend tests liên quan.
- [ ] Chạy TypeScript checks và Frontend production build.
- [ ] Kiểm tra browser console và network requests.
- [ ] Xác nhận request/response không mất precision ngoài chủ đích.
- [ ] Cập nhật hướng dẫn nhập tham số, chạy mô phỏng và đọc biểu đồ.
- [ ] Ghi rõ giới hạn mô phỏng SIR tổng thể trong Phase 1.

## 6. Acceptance Criteria
- [ ] Người dùng nhập cấu hình và chạy mô phỏng từ giao diện.
- [ ] Frontend gọi đúng API contract.
- [ ] Kết quả thành công hiển thị trên chart và metrics.
- [ ] Chart, slider và metrics đồng bộ cùng selectedTimeStep.
- [ ] Kết quả hiển thị thuộc đúng lần chạy mô phỏng.
- [ ] Chạy lại mô phỏng thay thế kết quả cũ đúng quy tắc.
- [ ] Response cũ không ghi đè response mới.
- [ ] Loading, success, empty và error states được xử lý.
- [ ] Play/pause/reset không gọi lại API hoặc tính lại SIR.
- [ ] Có E2E tests hoặc checklist được thực thi và lưu kết quả.
- [ ] Backend tests, Frontend checks/build liên quan thành công.
- [ ] Không có trạng thái SIR riêng theo từng công trình trong Phase 1.

## 7. Deliverables
1. Luồng form → API → current simulation result.
2. State orchestration cho lần chạy hiện tại.
3. Tích hợp chart, timeline và S/I/R metrics.
4. Xử lý loading/success/empty/error và repeated runs.
5. Integration/E2E tests hoặc checklist.
6. Hướng dẫn sử dụng mô phỏng SIR cơ bản.

## 8. Suggested Commit Sequence

~~~text
feat(frontend): connect SIR configuration form to API
refactor(frontend): centralize current simulation result state
feat(frontend): synchronize SIR chart timeline and metrics
fix(frontend): handle repeated simulation requests safely
test(frontend): verify end-to-end SIR simulation workflow
docs(frontend): document SIR simulation user flow
~~~

## 9. Final Definition of Done

KAN-10 hoàn thành khi người dùng cấu hình tham số, chạy SIR qua Backend API và xem biểu đồ cùng chỉ số S/I/R trong một luồng liên tục; việc chọn time_step và playback đồng bộ, chạy lại không hiển thị nhầm kết quả cũ, lỗi được xử lý và có kết quả kiểm thử đầu cuối.

Phase 1 chỉ hiển thị kết quả SIR tổng thể. Không gán trạng thái nhiễm cho từng building hoặc hiển thị mô phỏng dịch tễ không gian; các chức năng đó thuộc Phase 2.
