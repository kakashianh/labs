# KAN-18 – Triển khai thuật toán SIR và kiểm tra tham số đầu vào

## 1. Objective

Xây dựng module tính toán mô hình SIR cơ bản bằng Rust, mô phỏng sự thay đổi của ba nhóm dân số Susceptible (S), Infected (I) và Recovered (R) theo từng bước thời gian.

Module nhận trạng thái ban đầu và các tham số mô hình, kiểm tra tính hợp lệ, sau đó trả về chuỗi kết quả S/I/R. Logic tính toán phải độc lập với HTTP handler để có thể kiểm thử và tái sử dụng ở KAN-19.

**Expected result:**
- Có input/output data structures cho mô hình SIR.
- Validate đầy đủ trạng thái ban đầu và tham số.
- Tính được chuỗi S/I/R theo time step.
- S, I, R không âm và tổng dân số được bảo toàn trong sai số số học cho phép.
- Có unit tests cho trường hợp cơ bản, biên và dữ liệu không hợp lệ.
- Có tài liệu mô tả công thức và quy ước rời rạc hóa.

## 2. Scope

### In scope
- Định nghĩa SIR input, parameters và result structures.
- Quy ước đơn vị thời gian và đơn vị của beta/gamma.
- Validate N, S0, I0, R0, beta, gamma, dt và số bước mô phỏng.
- Cài đặt mô hình SIR rời rạc theo thời gian.
- Bảo toàn tổng dân số theo công thức.
- Xử lý sai số floating-point và giá trị biên.
- Tách simulation engine khỏi Axum/API layer.
- Unit tests và tài liệu kỹ thuật.

### Out of scope
- API khởi chạy mô phỏng (KAN-19).
- Tích hợp kết quả SIR lên CesiumJS.
- SIR theo từng công trình hoặc lây nhiễm giữa các công trình (Phase 2).
- Agent-based modeling, mô hình theo từng cá nhân/phòng.
- Mô hình lây truyền không gian hoặc tiếp xúc chi tiết.
- Can thiệp dịch tễ như cách ly/tiêm chủng/đóng cửa.
- Dự báo dịch bệnh thực tế hoặc hiệu chỉnh tham số từ dữ liệu y tế.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Language | Rust | Implement simulation engine |
| Numeric Type | f64 | Tính toán trạng thái S/I/R |
| Backend Framework | Axum (chỉ ở layer tích hợp sau) | Không đặt logic SIR trong HTTP handler |
| Testing | Rust unit tests | Kiểm thử thuật toán |

Module cần được tổ chức độc lập trong Backend hiện tại, ví dụ simulation/sir.rs hoặc vị trí phù hợp với cấu trúc repository. Không tạo Python service.

## 4. Mathematical Model and Time Convention

### 4.1. Initial State

Đầu vào gồm:
- N: tổng quy mô dân số.
- S0: số cá thể susceptible ban đầu.
- I0: số cá thể infected ban đầu.
- R0: số cá thể recovered ban đầu.
- beta (β): hệ số lây truyền, đơn vị 1/time.
- gamma (γ): hệ số hồi phục, đơn vị 1/time.
- dt: độ dài một bước thời gian.
- steps: số bước mô phỏng.

Điều kiện ban đầu:

\[
N > 0,\quad S_0 \geq 0,\quad I_0 \geq 0,\quad R_0 \geq 0
\]

\[
S_0 + I_0 + R_0 = N
\]

β và γ là các tốc độ theo đơn vị thời gian, không phải xác suất trực tiếp; do đó không áp đặt máy móc điều kiện β ≤ 1 hoặc γ ≤ 1.

### 4.2. Continuous SIR Equations

Mô hình SIR cơ bản:

\[
\frac{dS}{dt}=-\beta\frac{SI}{N}
\]

\[
\frac{dI}{dt}=\beta\frac{SI}{N}-\gamma I
\]

\[
\frac{dR}{dt}=\gamma I
\]

### 4.3. Discrete-time Approximation

Sử dụng explicit Euler với bước thời gian dt. Tại mỗi bước, tính hai luồng chuyển trạng thái từ trạng thái đầu bước:

\[
\Delta S_{\text{infection}} =
\min\left(S_t,\ \beta\frac{S_t I_t}{N}\Delta t\right)
\]

\[
\Delta R_{\text{recovery}} =
\min\left(I_t,\ \gamma I_t\Delta t\right)
\]

Sau đó cập nhật đồng thời:

\[
S_{t+1}=S_t-\Delta S_{\text{infection}}
\]

\[
I_{t+1}=I_t+\Delta S_{\text{infection}}-\Delta R_{\text{recovery}}
\]

\[
R_{t+1}=R_t+\Delta R_{\text{recovery}}
\]

Lưu ý:
- Cả hai luồng chuyển trạng thái được tính từ S_t và I_t, không dùng giá trị vừa cập nhật trong cùng bước.
- Giới hạn infection flow không vượt S_t.
- Giới hạn recovery flow không vượt I_t.
- Dùng f64 trong quá trình mô phỏng; không làm tròn từng bước.
- Chỉ làm tròn khi format kết quả để hiển thị, không làm tròn dữ liệu tính toán.
- dt phải được chọn đủ nhỏ để explicit Euler có ý nghĩa; clamp là bảo vệ biên, không thay thế việc chọn bước thời gian hợp lý.

## 5. Data Structures

Cấu trúc minh họa, điều chỉnh theo convention của repository:

~~~rust
pub struct SirInput {
    pub population: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
    pub beta: f64,
    pub gamma: f64,
    pub dt: f64,
    pub steps: usize,
}

pub struct SirPoint {
    pub time_step: usize,
    pub time: f64,
    pub susceptible: f64,
    pub infected: f64,
    pub recovered: f64,
}

pub struct SirResult {
    pub population: f64,
    pub points: Vec<SirPoint>,
}
~~~

Có thể sử dụng enum/error type riêng cho các lỗi validation như:
- PopulationInvalid
- InitialStateInvalid
- InitialPopulationMismatch
- RateParameterInvalid
- TimeStepInvalid
- StepCountInvalid

Không cần giữ đúng tên enum minh họa nếu project đã có error convention phù hợp.

## 6. Implementation Plan

### Step 1 – Inspect Backend Structure and Define SIR Module Boundary

- [x] Kiểm tra cấu trúc Rust Backend hiện tại: crate `backend` có `src/lib.rs`, dùng `thiserror`; unit tests đặt cùng module.
- [x] Xác định module `backend/src/simulation/sir.rs`, export qua `backend::simulation`.
- [x] Dùng error enum riêng `SirError` với `thiserror`, không phụ thuộc AppError/Axum.
- [x] Simulation engine không phụ thuộc Axum request/response types hoặc database.
- [x] Tài liệu công thức đặt tại `backend/docs/sir-model.md`.

### Step 2 – Define Input, Output and Error Types

- [x] Tạo SirInput (dùng `recovered`, không đặt tên `R0` để tránh nhầm Basic Reproduction Number).
- [x] Tạo SirPoint cho kết quả từng time step.
- [x] Tạo SirResult.
- [x] Định nghĩa SirError bằng thiserror.
- [x] Chốt cách biểu diễn thời gian: time = step × dt.
- [x] Output bao gồm trạng thái ban đầu tại time_step = 0 và các bước tiếp theo.

### Step 3 – Implement Input Validation

Validate trước khi bắt đầu mô phỏng.

Quy tắc:
- N phải hữu hạn và lớn hơn 0.
- S0, I0, R0 phải hữu hạn và không âm.
- S0 + I0 + R0 phải bằng N trong tolerance cấu hình/được định nghĩa.
- β và γ phải hữu hạn và không âm.
- dt phải hữu hạn và lớn hơn 0.
- steps phải lớn hơn 0 và không vượt giới hạn bảo vệ tài nguyên được thống nhất.
- Không nhận NaN hoặc infinity ở bất kỳ numeric input nào.

- [x] Implement validation cho population và S/I/R.
- [x] Kiểm tra tổng dân số với tolerance `1e-9 + 1e-9 × N`.
- [x] Implement validation cho beta/gamma (finite, không âm; không áp đặt upper bound 1 vì đây là rate).
- [x] Implement validation cho dt (finite, > 0).
- [x] Giới hạn steps trong 1..=100,000 và tổng thời lượng `dt × steps` không vượt 10,000 đơn vị thời gian.
- [x] Trả SirError có ý nghĩa, không panic khi input sai.

### Step 4 – Implement Discrete SIR Calculation

- [x] Khởi tạo trạng thái tại time_step = 0.
- [x] Tính infection/recovery flow từ trạng thái đầu bước; kiểm tra intermediate finite trước khi clamp.
- [x] Giới hạn infection flow trong [0, S_t] và recovery flow trong [0, I_t].
- [x] Cập nhật S, I, R đồng thời từ trạng thái đầu bước.
- [x] Kiểm tra finite, non-negative và population conservation sau mỗi bước; chỉ clamp sai số âm rất nhỏ.
- [x] Lưu kết quả tại từng time_step, không làm tròn giữa các bước.
- [x] Cùng input cho output deterministic.

### Step 5 – Enforce Invariants and Numerical Safety

Tại mỗi time step cần đảm bảo:
- S >= 0.
- I >= 0.
- R >= 0.
- S + I + R xấp xỉ N trong tolerance.
- Các giá trị đều hữu hạn.
- Infection flow không lớn hơn S.
- Recovery flow không lớn hơn I tại đầu bước.

- [x] Kiểm tra invariants sau mỗi lần cập nhật.
- [x] Xử lý drift bằng tolerance đã công bố; không âm thầm sửa sai lệch lớn.
- [x] Trả NumericalInstability nếu intermediate/output không finite hoặc conservation vượt tolerance.
- [x] Không sử dụng integer rounding trong vòng lặp mô phỏng.

### Step 6 – Add Unit Tests

Các nhóm test tối thiểu:

**Normal cases**
- [x] S > 0, I > 0, R = 0 với beta/gamma dương.
- [ ] Có cả ba nhóm S/I/R ban đầu (chưa có test riêng với cả S, I, R đều dương).
- [x] Nhiều time steps cho kết quả liên tục và hữu hạn.

**Boundary cases**
- [x] I0 = 0: không phát sinh ca nhiễm mới từ mô hình SIR này.
- [x] S0 = 0: không có susceptible để phát sinh infection flow.
- [x] R0 = N, S0 = I0 = 0.
- [x] beta = 0: không có lây nhiễm mới.
- [x] gamma = 0: không có hồi phục.
- [x] steps = 1.
- [x] dt có giá trị nhỏ hợp lệ.

**Invalid inputs**
- [x] N <= 0.
- [x] S0/I0/R0 âm.
- [x] Tổng S0 + I0 + R0 không bằng N ngoài tolerance.
- [x] beta hoặc gamma âm.
- [x] dt <= 0.
- [x] steps = 0 hoặc vượt maximum.
- [x] Numeric input là NaN hoặc infinity (đã cover đại diện ở state, rate, dt và intermediate).

**Invariant tests**
- [x] S/I/R luôn không âm.
- [x] S + I + R được bảo toàn qua mọi bước.
- [x] Kết quả không chứa NaN/infinity.
- [x] Cùng input cho cùng output.

### Step 7 – Document Formula and Usage

- [x] Ghi công thức SIR liên tục và explicit Euler rời rạc.
- [x] Giải thích đơn vị beta/gamma/dt và f64 là aggregate/expected counts.
- [x] Ghi rõ initial state được trả ở time_step = 0.
- [x] Ghi rõ tolerance, maximum steps và maximum duration.
- [x] Ghi giới hạn mô hình và quy ước lựa chọn dt.
- [x] Bổ sung ví dụ input/output một bước trong `backend/docs/sir-model.md`.

## 7. Acceptance Criteria

- [x] Simulation engine nhận input hợp lệ và trả chuỗi S/I/R theo time step.
- [x] Công thức và quy ước rời rạc hóa được thống nhất, ghi tài liệu.
- [x] Từ chối input âm, NaN/infinity, tham số ngoài miền cho phép hoặc initial state không nhất quán.
- [x] S, I, R không âm ở tất cả các bước.
- [x] Tổng S + I + R được bảo toàn trong tolerance đã định nghĩa.
- [x] Kết quả không bị làm tròn từng bước.
- [x] Logic tính toán độc lập với Axum/HTTP layer.
- [x] Có unit tests cho normal, boundary, invalid input và invariants.
- [x] Không có lây nhiễm phụ thuộc khoảng cách hoặc giữa các công trình trong ticket này.

## 8. Deliverables

1. Rust SIR simulation module.
2. Input/output structs và validation errors.
3. Discrete SIR calculation implementation.
4. Unit tests cho thuật toán và các invariants.
5. Tài liệu công thức, đơn vị và quy ước thời gian.

## 9. Suggested Commit Sequence

~~~text
feat(simulation): define SIR input output and error types

feat(simulation): validate SIR parameters and initial state

feat(simulation): implement discrete-time SIR engine

test(simulation): cover SIR normal boundary and invalid cases

docs(simulation): document SIR equations and time convention
~~~

## 10. Final Definition of Done

KAN-18 hoàn thành khi Rust simulation module có thể nhận trạng thái ban đầu và tham số hợp lệ, tạo chuỗi kết quả S/I/R theo từng time step, từ chối input không hợp lệ, bảo toàn tổng dân số trong sai số số học cho phép và có unit tests cùng tài liệu công thức.

Đây là SIR cơ bản ở quy mô tổng thể; chưa có mô hình lây nhiễm giữa các công trình hoặc các yếu tố không gian của Phase 2.


## 11. Implementation progress (2026-11-15)

- Branch: `feat/p1-12-sir-simulation-engine`
- Base: `feat/p1-11-map-e2e-integration`
- Added `backend/src/simulation/sir.rs`, `backend/src/simulation/mod.rs`, and exported `backend::simulation` from `backend/src/lib.rs`.
- Added standalone technical notes at `backend/docs/sir-model.md`.
- Implemented validation, explicit Euler integration, per-step invariants, overflow checks, deterministic output, and unit tests.
- Policy decisions: population tolerance = `1e-9 + 1e-9 × N`; max steps = `100,000`; max duration = `10,000` caller-defined time units; fractional f64 values are expected/aggregate counts.
- CI verification: [workflow run #43726465661](https://github.com/kakashianh/labs/actions/runs/43726465661) completed successfully on commit `29c766081191d323dd24c58c24076966da4b9c3b`. Axum API integration tests, frontend lint/unit/build, Rust format + Clippy + tests + Docker image build, and Docker Compose + PostGIS migration verification all passed. Render deployment was skipped because it is manual.



## 12. CI verification and test checklist update (2026-11-16)

- Confirmed workflow run #43726465661 completed with conclusion `success`.
- Passed jobs:
  - Axum API integration tests with Testcontainers PostGIS.
  - Frontend lint, unit tests and production build.
  - Rust formatting, Clippy, unit tests and backend Docker image build.
  - Docker Compose + PostGIS migration verification.
- Render deployment job was skipped by design because deployment is manual; it is not a CI failure.
- Checked Step 6 and Acceptance Criteria against the implemented tests and CI evidence. The dedicated test for an initial state where all three S/I/R compartments are positive remains unchecked; existing normal-case test uses R = 0.
- CI verifies the committed branch state; no manual deployment or production runtime verification is claimed.
