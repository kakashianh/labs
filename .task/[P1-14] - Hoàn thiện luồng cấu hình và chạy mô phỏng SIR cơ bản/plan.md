# KAN-9 – Hoàn thiện luồng cấu hình và chạy mô phỏng SIR cơ bản

## 1. Objective

Xác minh chất lượng luồng Backend từ API đến SIR engine và ngược lại, dựa trên hai phần đã được triển khai ở KAN-18 (SIR engine) và KAN-19 (SIR API).

Đây là task **integration verification và bổ sung kiểm thử còn thiếu**. Không triển khai lại thuật toán, không xây dựng API thứ hai và không lặp lại các kiểm thử đã được KAN-19 bao phủ.

**Expected result:**
- KAN-18 và KAN-19 được tích hợp trên branch nền.
- Request/response contract và quy ước thời gian nhất quán giữa API và domain.
- Kết quả mô phỏng có tính deterministic, các lần chạy độc lập và thỏa mãn invariants đã xác định.
- Các failure path còn thiếu được kiểm thử, không trùng lặp vô ích với KAN-19.
- Tài liệu phản ánh đúng hành vi thực tế của API.

## 2. Dependencies and Preconditions

| Dependency | Required state | Evidence / note |
|---|---|---|
| KAN-18 – SIR simulation engine | Đã merge vào `project/3D-map-medical` | Engine có public API `simulate`, `SirInput`, `SirError`, `SirResult` |
| KAN-19 – SIR simulation API | Đã có trong `project/3D-map-medical` | Đối chiếu GitHub compare: `feat/p1-13-sir-simulation-api` không còn commit riêng phía trước branch nền (base ahead 3, head ahead 0); CI success run [#43781923654](https://github.com/kakashianh/labs/actions/runs/43781923654) |
| KAN-20 – Frontend configuration/chart | Không phải dependency để kiểm thử Backend | Chỉ cần giữ response contract tương thích với Frontend |

**Gate trước implementation:**
- [x] Xác nhận KAN-18 và KAN-19 đã có mặt trên branch nền `project/3D-map-medical` (đã đối chiếu trạng thái branch).
- [ ] Nếu KAN-19 chưa merge, dừng trước khi tạo branch/viết code; không cherry-pick hoặc tạo bản API thay thế trong task này.
- [ ] Ghi nhận commit nền dùng để tạo branch KAN-9.

## 3. Scope

### In scope
- Rà soát contract và implementation KAN-18/KAN-19 sau khi đã tích hợp vào branch nền.
- Xác minh API handler thực sự gọi SIR engine hiện hữu.
- Xác minh mapping request DTO → domain input → response DTO.
- Kiểm thử các invariants số học, time series, tính deterministic và request isolation.
- Bổ sung những integration tests còn thiếu sau khi đối chiếu test hiện có.
- Xác minh error mapping và giới hạn tài nguyên.
- Chỉ cập nhật tài liệu khi phát hiện nội dung thiếu hoặc không khớp implementation.

### Out of scope
- Triển khai lại thuật toán SIR (KAN-18).
- Xây dựng endpoint hoặc request/response contract thứ hai (KAN-19).
- Lặp lại toàn bộ router tests đã có ở KAN-19 nếu không có lỗ hổng coverage.
- Xây dựng form cấu hình Frontend hoặc biểu đồ (KAN-20).
- Lưu lịch sử mô phỏng hoặc quản lý scenario.
- Mô phỏng theo từng công trình hoặc lây nhiễm không gian (Phase 2).
- Queue/worker hoặc xử lý mô phỏng nền.
- Thay đổi mô hình toán học hoặc tự ý điều chỉnh giới hạn domain trong phạm vi task tích hợp.

## 4. Existing Backend Flow to Verify

```text
API Client
   |
   v
POST /api/v1/simulations/sir
   |
   v
Deserialize request DTO
   |
   v
Map DTO -> SirInput
   |
   v
simulation::sir::simulate(...)
   |
   v
SirResult
   |
   v
Map domain result -> response DTO
   |
   v
HTTP response with ordered S/I/R time series
```

Luồng trên là luồng cần xác minh, không phải yêu cầu viết lại. Nếu implementation hiện tại khác mô tả, ghi nhận khác biệt và chỉ sửa khi đó là lỗi thực sự.

## 5. Contract and Domain Rules

### Request contract hiện tại
- `population`: tổng dân số N, hữu hạn và lớn hơn 0.
- `susceptible`, `infected`, `recovered`: trạng thái ban đầu S0, I0, R0, không âm và tổng khớp N theo tolerance của domain.
- `beta`, `gamma`: hệ số tốc độ, đơn vị 1/time, hữu hạn và không âm.
- `dt`: độ dài một bước thời gian, hữu hạn và lớn hơn 0.
- `steps`: số bước mô phỏng, trong giới hạn domain công bố.

### Time-series contract
- Điểm đầu tiên có `time_step = 0`, thể hiện trạng thái ban đầu.
- Tổng số điểm là `steps + 1`.
- Các điểm được trả theo thứ tự tăng dần của `time_step`.
- `time = time_step × dt`.
- Giữ precision của giá trị domain; không làm tròn từng bước trong API mapping.

### Error contract
- Lỗi request/validation thuộc nhóm client error: HTTP 400 theo `AppError` hiện tại.
- Response lỗi theo envelope hiện hành: `{"error":"..."}`; không tự thêm `error_code` nếu ứng dụng chưa định nghĩa contract đó.
- Numerical instability được map thành HTTP 500 với thông báo tổng quát, không làm lộ chi tiết nội bộ.
- JSON malformed, thiếu field hoặc sai kiểu dữ liệu được xử lý theo contract HTTP 400 hiện có.

Nếu phát hiện implementation không khớp các quy tắc trên, ghi nhận bằng test tái hiện trước khi sửa.

## 6. Implementation Plan

### Step 1 – Dependency and Existing Contract Verification
- [ ] Xác nhận KAN-18 và KAN-19 đã merge vào branch nền.
- [ ] Ghi nhận commit nền và trạng thái các dependency.
- [ ] Đọc public interface SIR engine và API handler thực tế.
- [ ] Đối chiếu field names/types giữa request DTO và `SirInput`.
- [ ] Đối chiếu mapping `SirResult` sang response DTO.
- [ ] Xác nhận beta/gamma có đơn vị (1/time), dt là duration mỗi bước.
- [ ] Xác nhận `time_step = 0`, `point_count = steps + 1` và thứ tự points.
- [ ] Lập danh sách mismatch hoặc coverage gap; nếu không có thì ghi rõ không cần sửa contract.

### Step 2 – Verify API-to-Engine Integration (No Reimplementation)
- [ ] Xác minh handler gọi trực tiếp engine KAN-18.
- [ ] Xác minh không có đoạn tính SIR thứ hai hoặc endpoint trùng lặp.
- [ ] Xác minh DTO mapping không đổi ý nghĩa hoặc làm mất precision.
- [ ] Xác minh domain errors được map đúng sang `AppError`.
- [ ] Xác minh response giữ điểm ban đầu và toàn bộ các bước theo thứ tự.
- [ ] Chỉ sửa code nếu Step 1/2 phát hiện lỗi có thể tái hiện.

### Step 3 – Verify Domain Invariants and Time Series
- [ ] Kiểm tra (S + I + R approx N) tại mọi điểm, dùng tolerance phù hợp với domain.
- [ ] Kiểm tra các giá trị S/I/R và time là hữu hạn.
- [ ] Kiểm tra S/I/R không âm đối với các input nằm trong miền ổn định được hỗ trợ; trường hợp mất ổn định phải trả domain error thay vì response sai.
- [ ] Kiểm tra `time = time_step × dt` tại mọi điểm.
- [ ] Kiểm tra số điểm bằng `steps + 1` và `time_step` tăng tuần tự.
- [ ] Không áp đặt invariant mới trái với semantics hoặc tolerance đã định nghĩa trong KAN-18.

### Step 4 – Verify Determinism and Request Isolation
- [ ] Gửi cùng một input nhiều lần và xác nhận kết quả tương đương/deterministic.
- [ ] Gửi hai request liên tiếp với bộ tham số khác nhau và xác nhận mỗi kết quả chỉ phụ thuộc request tương ứng.
- [ ] Xác minh không có mutable global/shared simulation state gây ảnh hưởng giữa các lần chạy.
- [ ] Nếu test isolation cần chạy đồng thời, xác định rõ cách kiểm thử phù hợp với kiến trúc hiện tại; không thêm concurrency/worker chỉ để phục vụ test.

### Step 5 – Close Only the Validation and Failure Coverage Gaps
- [ ] Đối chiếu test coverage hiện có ở KAN-18/KAN-19 trước khi bổ sung.
- [ ] Bổ sung domain-level tests cho non-finite values (NaN/Infinity) nếu KAN-18 chưa bao phủ; JSON request thông thường không biểu diễn được các giá trị này.
- [ ] Bổ sung API tests cho các trường hợp input JSON hợp lệ về cú pháp nhưng vi phạm domain constraints nếu còn thiếu.
- [ ] Xác minh steps vượt giới hạn và dt/steps không hợp lệ theo validation hiện hành.
- [ ] Xác minh numerical instability được trả về HTTP 500 với error envelope tổng quát nếu chưa được kiểm thử.
- [ ] Xác minh malformed/missing/wrong-type JSON chỉ bổ sung nếu coverage hiện có chưa đủ.
- [ ] Xác nhận request lỗi không tạo response mô phỏng thành công một phần.
- [ ] Không lặp lại nguyên bộ test đã có của KAN-19 chỉ để tăng số lượng test.

### Step 6 – Integration Test and Backend Regression
- [ ] Có test chứng minh đường đi API → engine → response bằng endpoint thực tế.
- [ ] Có test cho time-series invariants nếu chưa được KAN-19 bao phủ.
- [ ] Có test deterministic và request isolation.
- [ ] Có test cho các failure path còn thiếu đã xác định ở Step 5.
- [ ] Chạy test liên quan đến simulation/API.
- [ ] Chạy toàn bộ Backend test suite.
- [ ] Chạy formatter và Clippy theo CI workflow của repository.

### Step 7 – Documentation Reconciliation
- [ ] Rà soát `backend/docs/sir-api.md` đã mô tả đúng request/response hiện tại.
- [ ] Rà soát `backend/docs/sir-model.md` đã mô tả đúng domain rules và giới hạn SIR.
- [ ] Chỉ cập nhật tài liệu khi thiếu hoặc sai; không tạo tài liệu trùng lặp.
- [ ] Đảm bảo docs nêu rõ đơn vị beta/gamma/dt, giới hạn steps, error envelope và giới hạn mô hình tổng thể.
- [ ] Đảm bảo ví dụ request/response khớp với OpenAPI contract.

## 7. Acceptance Criteria

- [ ] KAN-18 và KAN-19 đã được tích hợp vào branch nền trước khi triển khai KAN-9.
- [ ] API sử dụng đúng SIR engine KAN-18, không có thuật toán hoặc endpoint trùng lặp.
- [ ] Request/response mapping và quy ước thời gian nhất quán.
- [ ] Response có điểm ban đầu, đúng số lượng điểm và thứ tự time_step.
- [ ] Kết quả thỏa mãn conservation, finite values và time-series invariants trong miền input được hỗ trợ.
- [ ] Cùng input cho kết quả deterministic.
- [ ] Các request độc lập, không rò rỉ state.
- [ ] Failure paths được kiểm thử theo coverage gap thực tế; không nhân đôi test không cần thiết.
- [ ] Backend regression tests, formatter và Clippy pass.
- [ ] Tài liệu API/domain khớp implementation.
- [ ] Không mở rộng sang Frontend, spatial SIR, persistence hoặc background processing.

## 8. Deliverables

1. Báo cáo ngắn đối chiếu contract và integration KAN-18/KAN-19.
2. Các sửa đổi nhỏ cần thiết để khắc phục mismatch thực tế (nếu có).
3. Tests bổ sung cho invariants, determinism, request isolation và failure coverage còn thiếu.
4. Kết quả Backend regression/CI.
5. Tài liệu được reconcile nếu cần.

## 9. Suggested Commit Sequence

Chỉ tạo commit tương ứng với thay đổi thực tế; không bắt buộc tạo commit nếu không có code/docs cần sửa.

```text
test(simulation): verify deterministic SIR execution
test(api): cover SIR request isolation and missing failure paths
fix(api): correct verified SIR integration mismatch
docs(api): reconcile SIR API and domain documentation
```

## 10. Final Definition of Done

KAN-9 hoàn thành khi luồng API hiện có được xác minh sử dụng đúng SIR engine KAN-18, contract nhất quán, kết quả có invariants và time-series semantics rõ ràng, các lần chạy deterministic và độc lập, các failure paths còn thiếu được kiểm thử, và Backend regression pass.

Task này xác nhận chất lượng tích hợp KAN-18/KAN-19; không tạo API/algorithm mới và không thay thế phạm vi Frontend của KAN-20.
