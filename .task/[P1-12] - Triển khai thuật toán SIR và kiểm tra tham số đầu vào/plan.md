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

- [ ] Kiểm tra cấu trúc Rust Backend hiện tại.
- [ ] Xác định vị trí module simulation độc lập.
- [ ] Kiểm tra error/result conventions của project.
- [ ] Đảm bảo simulation engine không phụ thuộc Axum request/response types.
- [ ] Xác định nơi đặt unit tests và tài liệu công thức.

### Step 2 – Define Input, Output and Error Types

- [ ] Tạo SirInput.
- [ ] Tạo SirPoint cho kết quả từng time step.
- [ ] Tạo SirResult.
- [ ] Định nghĩa error types cho input không hợp lệ.
- [ ] Chốt cách biểu diễn thời gian: time = step × dt.
- [ ] Chốt output bao gồm trạng thái ban đầu tại time_step = 0 và các bước tiếp theo.

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

- [ ] Implement validation cho population.
- [ ] Implement validation cho S0/I0/R0.
- [ ] Kiểm tra tổng dân số với floating-point tolerance.
- [ ] Implement validation cho beta/gamma.
- [ ] Implement validation cho dt.
- [ ] Implement validation cho steps và maximum allowed steps.
- [ ] Trả lỗi có ý nghĩa, không panic khi input sai.

### Step 4 – Implement Discrete SIR Calculation

- [ ] Khởi tạo trạng thái tại time_step = 0.
- [ ] Tính infection flow từ S_t, I_t, N, beta và dt.
- [ ] Tính recovery flow từ I_t, gamma và dt.
- [ ] Giới hạn infection flow trong [0, S_t].
- [ ] Giới hạn recovery flow trong [0, I_t].
- [ ] Cập nhật S, I, R đồng thời từ trạng thái đầu bước.
- [ ] Kiểm tra và xử lý sai số số học rất nhỏ ở biên theo tolerance.
- [ ] Lưu kết quả tại từng time_step.
- [ ] Không làm tròn S/I/R giữa các bước.
- [ ] Đảm bảo engine trả result deterministic với cùng input.

### Step 5 – Enforce Invariants and Numerical Safety

Tại mỗi time step cần đảm bảo:
- S >= 0.
- I >= 0.
- R >= 0.
- S + I + R xấp xỉ N trong tolerance.
- Các giá trị đều hữu hạn.
- Infection flow không lớn hơn S.
- Recovery flow không lớn hơn I tại đầu bước.

- [ ] Kiểm tra các invariants sau mỗi lần cập nhật.
- [ ] Xử lý floating-point drift nhỏ bằng chiến lược có tài liệu, không âm thầm thay đổi kết quả lớn.
- [ ] Trả simulation error nếu xuất hiện NaN/infinity hoặc vi phạm conservation ngoài tolerance.
- [ ] Không sử dụng integer rounding trong vòng lặp mô phỏng.

### Step 6 – Add Unit Tests

Các nhóm test tối thiểu:

**Normal cases**
- [ ] S > 0, I > 0, R = 0 với beta/gamma dương.
- [ ] Có cả ba nhóm S/I/R ban đầu.
- [ ] Nhiều time steps cho kết quả liên tục và hữu hạn.

**Boundary cases**
- [ ] I0 = 0: không phát sinh ca nhiễm mới từ mô hình SIR này.
- [ ] S0 = 0: không có susceptible để phát sinh infection flow.
- [ ] R0 = N, S0 = I0 = 0.
- [ ] beta = 0: không có lây nhiễm mới.
- [ ] gamma = 0: không có hồi phục.
- [ ] steps = 1.
- [ ] dt có giá trị nhỏ hợp lệ.

**Invalid inputs**
- [ ] N <= 0.
- [ ] S0/I0/R0 âm.
- [ ] Tổng S0 + I0 + R0 không bằng N ngoài tolerance.
- [ ] beta hoặc gamma âm.
- [ ] dt <= 0.
- [ ] steps = 0 hoặc vượt maximum.
- [ ] Numeric input là NaN hoặc infinity.

**Invariant tests**
- [ ] S/I/R luôn không âm.
- [ ] S + I + R được bảo toàn qua mọi bước.
- [ ] Kết quả không chứa NaN/infinity.
- [ ] Cùng input cho cùng output.

### Step 7 – Document Formula and Usage

- [ ] Ghi công thức SIR liên tục.
- [ ] Ghi công thức explicit Euler rời rạc.
- [ ] Giải thích đơn vị của beta, gamma và dt.
- [ ] Ghi rõ initial state được trả ở time_step = 0.
- [ ] Ghi rõ tolerance cho kiểm tra tổng dân số.
- [ ] Thêm ví dụ input/output.
- [ ] Ghi giới hạn của mô hình SIR cơ bản và việc lựa chọn dt.

## 7. Acceptance Criteria

- [ ] Simulation engine nhận input hợp lệ và trả chuỗi S/I/R theo time step.
- [ ] Công thức và quy ước rời rạc hóa được thống nhất, ghi tài liệu.
- [ ] Từ chối input âm, NaN/infinity, tham số ngoài miền cho phép hoặc initial state không nhất quán.
- [ ] S, I, R không âm ở tất cả các bước.
- [ ] Tổng S + I + R được bảo toàn trong tolerance đã định nghĩa.
- [ ] Kết quả không bị làm tròn từng bước.
- [ ] Logic tính toán độc lập với Axum/HTTP layer.
- [ ] Có unit tests cho normal, boundary, invalid input và invariants.
- [ ] Không có lây nhiễm phụ thuộc khoảng cách hoặc giữa các công trình trong ticket này.

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
