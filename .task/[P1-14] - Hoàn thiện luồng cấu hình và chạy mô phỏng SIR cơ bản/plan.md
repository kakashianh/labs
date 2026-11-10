# KAN-9 – Hoàn thiện luồng cấu hình và chạy mô phỏng SIR cơ bản

## 1. Objective

Hoàn thiện luồng Backend từ cấu hình mô hình đến khi thực thi SIR và trả kết quả thông qua API. Task tích hợp simulation engine ở KAN-18 với endpoint ở KAN-19, xác nhận contract và đảm bảo toàn bộ luồng chạy ổn định.

Đây là task tích hợp và xác minh luồng xử lý; không viết lại thuật toán hoặc xây dựng API thứ hai.

**Expected result:**
- Request model thống nhất giữa API và SIR engine.
- Cấu hình hợp lệ đi qua validation → simulation → response.
- Cấu hình sai bị từ chối với lỗi nhất quán.
- Response có thứ tự time_step và dùng được cho trực quan hóa.
- Có integration tests cho toàn bộ luồng.

## 2. Scope

### In scope
- Rà soát kết quả KAN-18 và KAN-19.
- Tích hợp API handler với SIR engine.
- Xác nhận mapping request DTO sang domain input.
- Xác nhận response contract cho Frontend.
- Hoàn thiện validation/error mapping xuyên suốt.
- Kiểm tra giới hạn số bước và bảo vệ tài nguyên.
- Kiểm thử luồng end-to-end ở tầng Backend.
- Cập nhật tài liệu cấu hình và chạy mô phỏng.

### Out of scope
- Triển khai lại thuật toán SIR (KAN-18).
- Xây dựng endpoint trùng chức năng KAN-19.
- Xây dựng form cấu hình Frontend hoặc biểu đồ (KAN-20).
- Lưu lịch sử mô phỏng hoặc quản lý scenario.
- Mô phỏng theo từng công trình hoặc lây nhiễm không gian (Phase 2).
- Chạy mô phỏng nền qua queue/worker.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Backend | Rust + Axum | Điều phối API |
| Domain | SIR engine từ KAN-18 | Tính toán |
| Serialization | Serde | JSON contract |
| Testing | Rust integration tests | Kiểm thử luồng |

Tái sử dụng module và API hiện tại trong một Backend Rust + Axum. Không tạo service mới.

## 4. End-to-End Backend Flow

~~~text
Frontend / API Client
        |
        v
POST /api/v1/simulations/sir
        |
        v
Deserialize Request DTO
        |
        v
Validate API + Domain Input
        |
        v
Map to SIR Engine Input
        |
        v
Execute SIR Engine (KAN-18)
        |
        v
Map Domain Result to Response DTO
        |
        v
Return ordered S/I/R time series
~~~

## 5. Implementation Plan

### Step 1 – Review KAN-18 and KAN-19
- [ ] Kiểm tra public interface của SIR engine.
- [ ] Kiểm tra API route/handler và request/response DTO.
- [ ] Đối chiếu field names, types và numeric precision.
- [ ] Đối chiếu quy ước beta/gamma/dt và đơn vị thời gian.
- [ ] Xác nhận time_step = 0 là trạng thái ban đầu.
- [ ] Ghi nhận điểm chưa thống nhất trước khi chỉnh sửa.

### Step 2 – Finalize Simulation Request Contract
- [ ] Xác nhận population là tổng dân số N.
- [ ] Xác nhận susceptible, infected, recovered là trạng thái ban đầu.
- [ ] Xác nhận beta/gamma là tốc độ theo đơn vị 1/time.
- [ ] Xác nhận dt là độ dài một bước thời gian.
- [ ] Xác nhận steps có giới hạn tối đa được công bố.
- [ ] Đảm bảo Frontend/API client và Backend dùng cùng field names/types.
- [ ] Không bổ sung tham số ngoài khả năng của SIR engine.
- [ ] Cập nhật API docs và request sample nếu contract thay đổi.

### Step 3 – Complete API-to-Engine Integration
- [ ] Đảm bảo handler gọi đúng SIR engine đã có.
- [ ] Loại bỏ đoạn tính SIR trùng lặp nếu phát hiện.
- [ ] Map domain errors về API errors.
- [ ] Giữ numeric precision cần thiết trong response.
- [ ] Đảm bảo points chứa time_step = 0 và các bước tiếp theo.
- [ ] Đảm bảo số lượng points theo quy ước steps + 1.
- [ ] Đảm bảo thứ tự time_step không đổi khi serialize.
- [ ] Đảm bảo mỗi request độc lập, không dùng state từ lần chạy trước.

### Step 4 – Verify Validation and Failure Paths
- [ ] Test population <= 0.
- [ ] Test S/I/R âm hoặc tổng ban đầu không khớp N.
- [ ] Test beta/gamma âm hoặc không hữu hạn ở domain layer.
- [ ] Test dt <= 0 hoặc steps = 0.
- [ ] Test steps vượt giới hạn.
- [ ] Test JSON sai định dạng hoặc thiếu field.
- [ ] Đảm bảo lỗi có HTTP status và error code đúng.
- [ ] Đảm bảo request lỗi không trả response thành công một phần.

### Step 5 – Verify Result Invariants and Determinism
- [ ] Xác nhận S, I, R không âm qua mọi time_step.
- [ ] Xác nhận S + I + R xấp xỉ population trong tolerance.
- [ ] Xác nhận output numeric values hữu hạn.
- [ ] Xác nhận time = time_step × dt.
- [ ] Gửi cùng request nhiều lần và xác nhận kết quả deterministic.
- [ ] Xác nhận hai request với tham số khác nhau không ảnh hưởng lẫn nhau.

### Step 6 – Add Integration Tests
- [ ] Test request hợp lệ đi qua API → engine → response.
- [ ] Test response schema và field bắt buộc.
- [ ] Test thứ tự time_step và số lượng points.
- [ ] Test invariants trên toàn bộ chuỗi kết quả.
- [ ] Test validation failure cases.
- [ ] Test hai lần chạy liên tiếp với hai bộ tham số khác nhau.
- [ ] Test engine error được chuyển thành API error.
- [ ] Chạy toàn bộ Backend test suite.

### Step 7 – Document Configuration and Execution
- [ ] Ghi luồng cấu hình và chạy mô phỏng.
- [ ] Ghi ví dụ request/response.
- [ ] Ghi ý nghĩa và đơn vị từng tham số.
- [ ] Ghi giới hạn số bước và quy tắc validation.
- [ ] Ghi error codes thường gặp.
- [ ] Ghi giới hạn SIR tổng thể của Phase 1.

## 6. Acceptance Criteria
- [ ] API sử dụng đúng SIR engine từ KAN-18.
- [ ] Contract request/response được thống nhất và tài liệu hóa.
- [ ] Request hợp lệ trả chuỗi S/I/R đúng thứ tự time_step.
- [ ] Response dùng được cho biểu đồ KAN-20.
- [ ] Validation và lỗi nhất quán từ API đến domain.
- [ ] Kết quả thỏa mãn invariants SIR đã định nghĩa.
- [ ] Hai lần chạy độc lập không rò rỉ state lẫn nhau.
- [ ] Có integration tests cho success, failure và repeated runs.
- [ ] Không phát sinh API hoặc thuật toán trùng lặp.

## 7. Deliverables
1. Luồng API → SIR engine được tích hợp.
2. Request/response contract đã thống nhất.
3. Validation và error mapping hoàn chỉnh.
4. Backend integration tests.
5. Tài liệu cấu hình và chạy SIR.

## 8. Suggested Commit Sequence

~~~text
refactor(simulation): align API DTOs with SIR domain types
fix(api): complete SIR engine integration and error mapping
test(api): verify SIR execution workflow and invariants
docs(api): document SIR configuration and execution flow
~~~

## 9. Final Definition of Done

KAN-9 hoàn thành khi request mô phỏng đi xuyên suốt từ API đến SIR engine và trả response ổn định, đúng contract, đúng thứ tự thời gian; input lỗi được xử lý rõ ràng và có integration tests xác nhận.

Task này đóng gói chất lượng luồng Backend ở KAN-18/KAN-19, không mở rộng sang mô phỏng theo từng công trình.
