# KAN-19 – Xây dựng API khởi chạy mô phỏng SIR và trả kết quả theo thời gian

## 1. Objective

Xây dựng REST API để Frontend gửi cấu hình mô hình SIR, Backend validate request, gọi simulation engine từ KAN-18 và trả về chuỗi kết quả S/I/R theo từng time step.

API chịu trách nhiệm tiếp nhận request, điều phối module tính toán và chuẩn hóa HTTP response. Không triển khai lại công thức SIR trong handler.

**Expected result:**
- Request/response DTO rõ ràng, ổn định.
- Request hợp lệ được chuyển tới SIR engine.
- Response chứa trạng thái S/I/R theo thứ tự thời gian, bao gồm trạng thái ban đầu.
- Request sai nhận HTTP status và error body phù hợp.
- Có API/integration tests cho success và failure cases.

## 2. Scope

### In scope
- Rà soát SIR input/output types và errors từ KAN-18.
- Thiết kế endpoint khởi chạy mô phỏng SIR.
- Xây dựng request DTO và mapping sang domain input.
- Validate payload và tái sử dụng domain validation.
- Gọi simulation engine, ánh xạ kết quả thành response DTO.
- Chuẩn hóa HTTP status/error response.
- API/integration tests và tài liệu endpoint.

### Out of scope
- Triển khai lại thuật toán SIR (KAN-18).
- Xây dựng giao diện cấu hình hoặc biểu đồ (KAN-9, KAN-20).
- Lưu lịch sử mô phỏng hoặc quản lý scenario.
- Mô phỏng theo từng công trình, lây nhiễm không gian hoặc hotspot (Phase 2).
- Chạy mô phỏng bất đồng bộ qua queue/job system.
- Authentication/authorization nếu chưa thuộc nền tảng MVP.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Backend | Rust + Axum | REST endpoint |
| Simulation | SIR module từ KAN-18 | Tính toán domain |
| Serialization | Serde | JSON request/response |
| Testing | Rust integration tests | Kiểm thử API |

Tái sử dụng router, error handling và response convention hiện có. Không tạo Python service hoặc backend riêng.

## 4. API Contract

Endpoint đề xuất, điều chỉnh theo convention hiện tại:

~~~http
POST /api/v1/simulations/sir
Content-Type: application/json
~~~

Request minh họa:

~~~json
{
  "population": 10000,
  "susceptible": 9990,
  "infected": 10,
  "recovered": 0,
  "beta": 0.3,
  "gamma": 0.1,
  "dt": 1.0,
  "steps": 100
}
~~~

Response minh họa:

~~~json
{
  "population": 10000,
  "time_step_count": 101,
  "points": [
    {
      "time_step": 0,
      "time": 0.0,
      "susceptible": 9990.0,
      "infected": 10.0,
      "recovered": 0.0
    }
  ]
}
~~~

Response thực tế chứa toàn bộ points; ví dụ chỉ minh họa cấu trúc. points bao gồm trạng thái ban đầu tại time_step = 0 và các bước từ 1 đến steps.

Error response đề xuất:

~~~json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Simulation parameters are invalid",
    "details": []
  }
}
~~~

HTTP mapping:
- 200 OK: mô phỏng hoàn tất.
- 400 Bad Request hoặc 422 Unprocessable Entity: payload không hợp lệ; chọn một convention nhất quán.
- 500 Internal Server Error: lỗi nội bộ không dự kiến.
- Không trả stack trace hoặc thông tin nội bộ cho client.

## 5. Implementation Plan

### Step 1 – Inspect SIR Engine and API Conventions
- [ ] Kiểm tra public interface và error types của KAN-18.
- [ ] Kiểm tra Axum router, route prefix và JSON extractor.
- [ ] Kiểm tra convention validation và error response.
- [ ] Xác nhận giới hạn steps từ SIR engine.
- [ ] Không sao chép công thức SIR vào handler.

### Step 2 – Define Request and Response DTOs
- [ ] Tạo request DTO gồm population, susceptible, infected, recovered, beta, gamma, dt và steps.
- [ ] Tạo response point DTO gồm time_step, time, susceptible, infected, recovered.
- [ ] Tạo response DTO chứa population, số điểm và points.
- [ ] Tạo error response DTO theo convention Backend.
- [ ] Giữ DTO độc lập với internal implementation details của engine.

### Step 3 – Implement Validation and Domain Mapping
- [ ] Kiểm tra JSON parsing, required fields và kiểu dữ liệu.
- [ ] Map request DTO sang SIR input của KAN-18.
- [ ] Tái sử dụng domain validation, không viết lại quy tắc.
- [ ] Map domain validation errors sang error code/message.
- [ ] Áp dụng giới hạn số bước để bảo vệ tài nguyên.
- [ ] Không để input không hữu hạn lọt qua domain validation.

### Step 4 – Implement Handler and Route
- [ ] Tạo Axum handler cho POST /api/v1/simulations/sir.
- [ ] Gọi SIR engine một lần cho mỗi request hợp lệ.
- [ ] Map domain result thành response DTO.
- [ ] Trả points theo thứ tự time_step tăng dần.
- [ ] Đảm bảo response gồm trạng thái ban đầu và đủ số bước.
- [ ] Đăng ký route vào router hiện tại.
- [ ] Không truy cập database nếu không có yêu cầu lưu kết quả.
- [ ] Không đặt logic tính toán trong HTTP handler.

### Step 5 – Normalize HTTP and Error Handling
- [ ] Chọn HTTP status nhất quán cho validation errors.
- [ ] Xử lý lỗi deserialize, domain validation và simulation.
- [ ] Không trả panic/stack trace hoặc chi tiết nội bộ.
- [ ] Đảm bảo error body có code/message ổn định cho Frontend.

### Step 6 – Add API and Integration Tests
- [ ] Test request hợp lệ trả HTTP success.
- [ ] Test trạng thái ban đầu tại time_step = 0.
- [ ] Test thứ tự time_step và số lượng points = steps + 1.
- [ ] Test tổng S + I + R được bảo toàn trong tolerance.
- [ ] Test thiếu field, sai kiểu và input không hợp lệ.
- [ ] Test steps vượt giới hạn.
- [ ] Test domain error được ánh xạ thành HTTP error.
- [ ] Test response không chứa stack trace.

### Step 7 – Document API
- [ ] Ghi endpoint, method, content type và request fields.
- [ ] Ghi đơn vị beta/gamma/dt.
- [ ] Ghi response schema và ý nghĩa time_step/time.
- [ ] Ghi error codes, HTTP status và ví dụ request/response.
- [ ] Ghi rõ API chỉ chạy SIR tổng thể, chưa có dữ liệu theo công trình.

## 6. Acceptance Criteria
- [ ] Có endpoint POST khởi chạy mô phỏng SIR.
- [ ] Request hợp lệ gọi module KAN-18 và trả chuỗi kết quả theo thời gian.
- [ ] Response chứa time_step, time, S, I, R theo thứ tự.
- [ ] Response bao gồm trạng thái ban đầu và các bước mô phỏng.
- [ ] API tái sử dụng domain validation và không lặp lại thuật toán.
- [ ] HTTP status và error response nhất quán.
- [ ] Có API/integration tests cho success và failure cases.
- [ ] Có tài liệu endpoint và ví dụ request/response.

## 7. Deliverables
1. Axum route và handler chạy SIR.
2. Request/response DTO.
3. Mapping giữa API DTO và domain SIR types.
4. Error response mapping.
5. API/integration tests.
6. Tài liệu API.

## 8. Suggested Commit Sequence

~~~text
feat(api): define SIR simulation request and response DTOs
feat(api): add SIR simulation endpoint
feat(api): map SIR validation and simulation errors
test(api): cover SIR simulation endpoint
docs(api): document SIR simulation endpoint
~~~

## 9. Final Definition of Done

KAN-19 hoàn thành khi API client có thể gửi request hợp lệ tới endpoint SIR, nhận chuỗi kết quả S/I/R theo time_step và nhận lỗi rõ ràng khi payload không hợp lệ. API tái sử dụng SIR engine của KAN-18, không triển khai lại thuật toán.

Phạm vi chỉ bao gồm API chạy mô phỏng SIR tổng thể; không lưu scenario/history và không trả trạng thái dịch tễ riêng theo từng công trình.
