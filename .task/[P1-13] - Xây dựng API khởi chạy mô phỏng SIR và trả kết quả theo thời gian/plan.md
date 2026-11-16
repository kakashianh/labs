# KAN-19 – Xây dựng API khởi chạy mô phỏng SIR và trả kết quả theo thời gian

## 1. Objective

Xây dựng REST API để Frontend gửi cấu hình mô hình SIR, Backend validate request, gọi simulation engine từ KAN-18 và trả về chuỗi kết quả S/I/R theo từng time step.

API chịu trách nhiệm tiếp nhận request, điều phối module tính toán và chuẩn hóa HTTP response. Không triển khai lại công thức SIR trong handler.

**Expected result:**
- Request/response DTO rõ ràng, ổn định.
- Request hợp lệ được chuyển tới SIR engine.
- Response chứa trạng thái S/I/R theo thứ tự thời gian, bao gồm trạng thái ban đầu.
- Request sai nhận HTTP status và error body theo convention Backend hiện tại.
- Có API/integration tests chạy qua Axum router cho success và failure cases.

## 2. Dependencies and Preconditions

- [ ] KAN-18 / P1-12 (SIR simulation engine) được merge vào branch `project/3D-map-medical` trước khi triển khai KAN-19.
- [ ] Sau khi merge, xác nhận public interface có thể import từ `simulation::sir`, gồm `simulate`, `SirInput`, `SirError` và kiểu kết quả tương ứng.
- [ ] Xác nhận các variant hiện có của `SirError` và giới hạn đầu vào trực tiếp từ source KAN-18; không tự định nghĩa lại domain rules trong API.

**Dependency note:** Tại thời điểm lập plan, SIR engine mới nằm trên feature branch KAN-18 và chưa có trong `project/3D-map-medical`. Không copy hoặc viết lại engine trong KAN-19. Nếu KAN-18 chưa merge thì dừng phần tích hợp handler cho tới khi dependency sẵn sàng.

## 3. Scope

### In scope
- Rà soát SIR input/output types và errors từ KAN-18.
- Thiết kế endpoint khởi chạy mô phỏng SIR.
- Xây dựng request DTO và mapping sang domain input.
- Tái sử dụng domain validation.
- Gọi simulation engine, ánh xạ kết quả thành response DTO.
- Chuẩn hóa HTTP status/error response theo convention hiện có.
- API/integration tests và tài liệu endpoint.

### Out of scope
- Triển khai lại thuật toán SIR (KAN-18).
- Xây dựng giao diện cấu hình hoặc biểu đồ (KAN-9, KAN-20).
- Lưu lịch sử mô phỏng hoặc quản lý scenario.
- Mô phỏng theo từng công trình, lây nhiễm không gian hoặc hotspot (Phase 2).
- Chạy mô phỏng bất đồng bộ qua queue/job system.
- Authentication/authorization nếu chưa thuộc nền tảng MVP.
- Thay đổi error contract toàn cục của Backend.

## 4. Technical Design

| Component | Location / Technology | Responsibility |
|---|---|---|
| HTTP route | `backend/src/routes/simulations.rs` | Khai báo endpoint và handler |
| Route registration | `backend/src/routes/mod.rs`, `backend/src/router.rs` | Đăng ký route theo cấu trúc hiện tại |
| Domain engine | `backend/src/simulation/sir.rs` từ KAN-18 | Tính toán SIR; không đặt công thức trong handler |
| API DTO | Module route hoặc module DTO phù hợp | Tách HTTP contract khỏi domain types |
| Error mapping | `backend/src/error.rs` / `AppError` hiện có | Ánh xạ lỗi domain sang HTTP |
| Serialization | Serde | JSON request/response |
| Testing | Rust tests + Tower `ServiceExt::oneshot` | Kiểm thử endpoint qua router |

Tái sử dụng Axum router, `AppError` và response convention hiện có. Không tạo Python service hoặc backend riêng. API mô phỏng hiện tại là synchronous, không cần PgPool, Redis, queue hoặc background job.

## 5. API Contract

Endpoint:

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

Response minh họa (rút gọn):

~~~json
{
  "population": 10000.0,
  "point_count": 101,
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

Response thực tế chứa toàn bộ points. `point_count` là số phần tử trong mảng `points`, bằng `steps + 1` vì kết quả bao gồm trạng thái ban đầu tại `time_step = 0` và các bước từ 1 đến `steps`.

### Units and semantics

- `beta` và `gamma` là các tốc độ (rate), có đơn vị nghịch đảo của đơn vị thời gian được chọn; không mặc định chúng là xác suất.
- `dt` dùng cùng đơn vị thời gian với mô hình. `time = time_step * dt`.
- Đơn vị thời gian do client/ngữ cảnh mô phỏng quy ước (ví dụ: ngày); API không tự suy luận đơn vị.
- S, I, R là số lượng tổng hợp và có thể là số thực do mô hình tính toán; không làm tròn từng bước trong API.

### Error response

Giữ nguyên error envelope hiện có của Backend:

~~~json
{
  "error": "Simulation parameters are invalid"
}
~~~

Không tạo envelope `code/message/details` riêng cho endpoint này. Nếu muốn thay đổi contract lỗi, cần thực hiện như một thay đổi toàn cục riêng, không nằm trong KAN-19.

### HTTP mapping

- `200 OK`: mô phỏng hoàn tất.
- `400 Bad Request`: JSON hợp lệ về cú pháp nhưng input/domain parameters không hợp lệ, ánh xạ qua `AppError::BadRequest`.
- `400 Bad Request` hoặc status mặc định hiện tại của Axum JSON extractor: JSON malformed / thiếu field / sai kiểu. Kiểm tra hành vi thực tế của extractor và chuẩn hóa có chủ đích; không để test phụ thuộc vào giả định chưa xác minh.
- `500 Internal Server Error`: lỗi numerical instability hoặc lỗi nội bộ không thể quy cho input validation; ánh xạ qua `AppError::Internal`, không tiết lộ chi tiết nội bộ.
- Không trả stack trace, panic detail hoặc thông tin nội bộ cho client.

## 6. Implementation Plan

### Step 1 – Verify Dependency, SIR Engine and API Conventions
- [ ] Xác nhận KAN-18 đã merge vào `project/3D-map-medical`.
- [ ] Kiểm tra public interface `simulation::sir::{simulate, SirInput, SirError}` và kiểu kết quả thực tế.
- [ ] Kiểm tra đầy đủ các variant của `SirError` để phân loại lỗi validation và numerical instability.
- [ ] Kiểm tra Axum router, route prefix và JSON extractor.
- [ ] Kiểm tra `AppError` và error response hiện tại (`{"error":"..."}`).
- [ ] Xác nhận giới hạn steps và các quy tắc validation từ SIR engine.
- [ ] Không sao chép công thức hoặc domain validation vào handler.

### Step 2 – Define Request and Response DTOs
- [ ] Tạo request DTO gồm population, susceptible, infected, recovered, beta, gamma, dt và steps.
- [ ] Tạo response point DTO gồm time_step, time, susceptible, infected, recovered.
- [ ] Tạo response DTO chứa population, `point_count` và points.
- [ ] Dùng `point_count` để diễn đạt chính xác số phần tử trong points (`steps + 1`), không dùng tên gây nhầm với số bước mô phỏng.
- [ ] Giữ DTO độc lập với internal implementation details của engine.
- [ ] Không expose trực tiếp domain error hoặc internal error details ra HTTP response.

### Step 3 – Implement Domain Mapping and Error Mapping
- [ ] Map request DTO sang `SirInput` của KAN-18.
- [ ] Gọi `simulate()` làm điểm vào domain; không gọi lặp `validate()` nếu `simulate()` đã tự validate.
- [ ] Map các lỗi input/domain validation của `SirError` sang `AppError::BadRequest` (HTTP 400).
- [ ] Map `SirError::NumericalInstability` hoặc lỗi tính toán nội bộ sang `AppError::Internal` (HTTP 500), không trả chi tiết nội bộ.
- [ ] Không gom tất cả `SirError` thành HTTP 400; phân biệt lỗi client và lỗi tính toán.
- [ ] Tái sử dụng giới hạn steps và các ràng buộc từ domain engine, không tạo bộ validation trùng lặp.
- [ ] Xác minh JSON deserialization xử lý missing fields, wrong types và malformed JSON như thế nào; nếu cần custom extractor/rejection mapping thì giữ thay đổi trong phạm vi route này.

### Step 4 – Implement Handler and Route
- [ ] Tạo `backend/src/routes/simulations.rs` cho POST `/api/v1/simulations/sir`.
- [ ] Handler chỉ deserialize request, map sang domain input, gọi `simulate()`, map result/error và trả response.
- [ ] Gọi SIR engine một lần cho mỗi request hợp lệ.
- [ ] Map domain result thành response DTO.
- [ ] Trả points theo thứ tự `time_step` tăng dần.
- [ ] Đảm bảo response gồm trạng thái ban đầu và đủ `steps + 1` points.
- [ ] Đăng ký module trong `backend/src/routes/mod.rs` và merge route trong `backend/src/router.rs` theo convention hiện tại.
- [ ] Không truy cập database/Redis nếu không có yêu cầu lưu kết quả.
- [ ] Không đặt logic tính toán trong HTTP handler.

### Step 5 – Normalize HTTP and Error Handling
- [ ] Dùng `AppError` hiện có để giữ error envelope nhất quán.
- [ ] Phân biệt lỗi deserialize/request, domain validation và numerical/internal error.
- [ ] Chốt status code thực tế cho malformed JSON/missing field/wrong type và ghi rõ trong API docs.
- [ ] Đảm bảo lỗi 500 không tiết lộ chi tiết nội bộ.
- [ ] Không trả panic/stack trace hoặc chi tiết triển khai cho client.

### Step 6 – Add Router-level API and Integration Tests
- [ ] Dùng `tower::ServiceExt::oneshot` gửi request qua Axum router thực tế; không chỉ test DTO hoặc helper mapping.
- [ ] Test request hợp lệ trả HTTP 200 và JSON response đúng schema.
- [ ] Test trạng thái ban đầu tại `time_step = 0` khớp request.
- [ ] Test thứ tự time_step và `point_count = points.len() = steps + 1`.
- [ ] Test mọi point có S/I/R hữu hạn, không âm và tổng S + I + R được bảo toàn trong tolerance domain.
- [ ] Test malformed JSON, thiếu field và sai kiểu dữ liệu.
- [ ] Test domain validation error (ví dụ population/state/rates/dt không hợp lệ) được ánh xạ HTTP 400.
- [ ] Test steps vượt giới hạn domain được ánh xạ HTTP 400.
- [ ] Test numerical instability được ánh xạ HTTP 500 và response không tiết lộ internal details.
- [ ] Test response error không chứa stack trace hoặc panic details.
- [ ] Không cần Testcontainers/PostGIS cho endpoint này nếu router test có thể khởi tạo state tối thiểu mà không truy cập DB.

### Step 7 – Document API
- [ ] Ghi endpoint, method, content type và request fields.
- [ ] Ghi rõ beta/gamma là rate, dt/time dùng cùng đơn vị thời gian và đơn vị do caller quy ước.
- [ ] Ghi response schema, ý nghĩa `point_count`, `time_step` và `time`.
- [ ] Ghi error envelope thực tế, error mapping và HTTP status đã kiểm chứng.
- [ ] Thêm ví dụ curl request và response.
- [ ] Ghi rõ API chỉ chạy SIR tổng thể, chưa có dữ liệu theo công trình.
- [ ] Cập nhật OpenAPI nếu dự án đang duy trì OpenAPI contract cho các endpoint hiện có.

## 7. Acceptance Criteria

- [ ] KAN-18 đã được merge và KAN-19 sử dụng trực tiếp SIR engine đó.
- [ ] Có endpoint POST khởi chạy mô phỏng SIR.
- [ ] Request hợp lệ gọi module KAN-18 và trả chuỗi kết quả theo thời gian.
- [ ] Response chứa time_step, time, S, I, R theo thứ tự.
- [ ] Response bao gồm trạng thái ban đầu và đúng `steps + 1` points.
- [ ] API tái sử dụng domain validation và không lặp lại thuật toán.
- [ ] Lỗi validation trả HTTP 400 theo error envelope hiện có; numerical/internal error trả HTTP 500 và không lộ chi tiết nội bộ.
- [ ] Có router-level API/integration tests cho success, malformed request, validation failure và internal failure mapping.
- [ ] Có tài liệu endpoint, units và ví dụ request/response.

## 8. Deliverables

1. Axum route và handler chạy SIR.
2. Request/response DTO.
3. Mapping giữa API DTO và domain SIR types.
4. Error mapping tích hợp với `AppError` hiện có.
5. Router-level API/integration tests.
6. Tài liệu API và ví dụ curl.

## 9. Suggested Commit Sequence

~~~text
feat(api): define SIR simulation request and response DTOs
feat(api): add SIR simulation endpoint
feat(api): map SIR validation and simulation errors
test(api): cover SIR simulation endpoint through router
docs(api): document SIR simulation endpoint and units
~~~

## 10. Final Definition of Done

KAN-19 hoàn thành khi API client có thể gửi request hợp lệ tới endpoint SIR, nhận chuỗi kết quả S/I/R theo time_step (bao gồm trạng thái ban đầu), và nhận lỗi HTTP rõ ràng theo error contract hiện tại khi payload không hợp lệ hoặc quá trình tính toán gặp lỗi nội bộ.

API tái sử dụng SIR engine của KAN-18, không triển khai lại thuật toán hoặc domain validation. Phạm vi chỉ bao gồm API chạy mô phỏng SIR tổng thể; không lưu scenario/history và không trả trạng thái dịch tễ riêng theo từng công trình.
