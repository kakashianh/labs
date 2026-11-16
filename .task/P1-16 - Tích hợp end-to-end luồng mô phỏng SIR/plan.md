# P1-16 – Tích hợp end-to-end luồng mô phỏng SIR

## 1. Objective

Hoàn thiện luồng người dùng từ cấu hình tham số SIR đến khi Backend trả về kết quả và giao diện hiển thị chuỗi Susceptible (S), Infected (I), Recovered (R).

P1-16 phụ thuộc vào KAN-19 (SIR API) và KAN-20 (SIR result visualization). Không xây dựng lại thuật toán hoặc API.

**Expected result:**
- Người dùng nhập đủ tham số mô phỏng trên giao diện.
- Frontend validate các điều kiện cơ bản trước khi gửi request.
- Frontend gọi `POST /api/v1/simulations/sir` với đúng request contract.
- Loading/error/success states được quản lý rõ ràng.
- Kết quả response thật được truyền vào `SirResultPanel`.
- Chart, metrics, slider và playback hiển thị cùng một response.
- Không dùng dữ liệu giả để thay thế kết quả Backend.

## 2. Dependencies

| Dependency | Deliverable | Status |
|---|---|---|
| KAN-19 / P1-13 | `POST /api/v1/simulations/sir` | Implemented and CI verified |
| KAN-20 / P1-15 | `SirResultPanel`, chart, metrics, playback | Implemented; component interaction tests remain |
| P1-16 | Parent form + API request lifecycle + E2E verification | In progress |

Branch này được tạo từ `feat/p1-15-sir-visualization`; khi mở PR, base branch phải là `feat/p1-15-sir-visualization` để giữ dependency rõ ràng. Không merge hoặc tạo PR tự động.

## 3. Scope

### In scope
- Form nhập population, susceptible, infected, recovered, beta, gamma, dt và steps.
- Client-side validation khớp các giới hạn Backend.
- API client POST và typed request/response.
- Abort request khi component unmount hoặc request mới thay thế request cũ.
- Loading, validation error, API error và success state.
- Kiểm tra response tối thiểu trước khi render.
- Component interaction tests cho form và SIR result panel.
- Browser/E2E verification và CI.

### Out of scope
- Thay đổi SIR mathematical model hoặc numerical method.
- Persist simulation history/scenarios.
- Chạy simulation trong background queue.
- Gắn kết quả SIR lên từng building trên Cesium.
- Spatial transmission, hotspot hoặc risk prediction.
- Suy luận đơn vị thời gian; đơn vị do người dùng tự quy ước.

## 4. Request and Response Contract

### Request

```typescript
interface SirSimulationRequest {
  population: number;
  susceptible: number;
  infected: number;
  recovered: number;
  beta: number;
  gamma: number;
  dt: number;
  steps: number;
}
```

Endpoint: `POST /api/v1/simulations/sir`

### Response

```typescript
interface SirSimulationResult {
  population: number;
  point_count: number;
  points: SirPoint[];
}
```

Contract invariants:
- `point_count === points.length`.
- `point_count === steps + 1`.
- First point has `time_step = 0` and contains the initial S/I/R state.
- `time_step` increases sequentially.
- `time = time_step × dt`.
- Frontend formats values for display only; it does not round or recompute simulation values.

## 5. Validation Rules

Frontend validation is for usability only; Backend remains authoritative.

- Population is finite and greater than zero.
- S, I, R are finite and non-negative.
- `S + I + R` equals population within the Backend tolerance.
- Beta and gamma are finite and non-negative.
- dt is finite and greater than zero.
- steps is an integer in [1, 100000].
- `dt × steps` is finite and no greater than 10000 caller-defined time units.

The form must explain that beta, gamma and dt use a common caller-defined time unit; do not label it as days unless the user explicitly selects that unit.

## 6. UI / State Responsibilities

```text
HomePage
├── CesiumViewer
├── SirSimulationForm
│   ├── ParameterInputs
│   └── Validation / Request State
└── SirResultPanel
    ├── SirResultChart
    ├── SirCurrentMetrics
    └── SirPlaybackControls
```

- HomePage owns request lifecycle and passes `result/loading/error` to SirResultPanel.
- Form owns only editable parameter values and submit interaction.
- API client owns HTTP serialization and API error normalization.
- SirResultPanel remains presentation/playback-only; it must not call API or run the SIR model.
- A newer request aborts the previous request.
- Unmount aborts any pending request.
- A response is rendered only if it passes response validation and matches the submitted population/step count.
- No stale response may overwrite a newer request result.

## 7. Implementation Plan

### Step 1 – API Client and Request Types
- [x] Define `SirSimulationRequest` type.
- [x] Add reusable JSON POST support to API client.
- [x] Implement `simulateSir` for `POST /api/v1/simulations/sir`.
- [x] Normalize Backend `{ error }` responses into `ApiRequestError`.
- [x] Add unit tests for request method, URL, headers, payload, API errors and abort signal.

### Step 2 – Parameter Form and Validation
- [x] Add fields for all eight API parameters.
- [x] Provide a valid initial example configuration (N=1000, S=990, I=10, R=0).
- [x] Validate population, compartment totals, rates, dt, steps and maximum duration.
- [x] Keep time unit caller-defined and explain it in the UI.
- [x] Add responsive styling for the configuration panel.
- [ ] Add component tests for editing fields, validation messages and submit behavior.

### Step 3 – Request Lifecycle and Result Integration
- [x] Connect HomePage to `simulateSir`.
- [x] Manage loading, error and result state.
- [x] Abort in-flight request on unmount or replacement by a newer request.
- [x] Verify response contract, population and `point_count = steps + 1` before displaying.
- [x] Pass real API response to SirResultPanel; do not use mock data in the application.
- [ ] Test stale request handling and loading/error transitions at component level.

### Step 4 – SIR Result Component Tests
- [ ] Add Vitest + React Testing Library + jsdom without removing existing Node built-in tests.
- [ ] Test chart series and selected metrics render from response points.
- [ ] Test slider updates selected time step and metrics.
- [ ] Test Play/Pause/Reset and stop-at-last-point behavior with fake timers.
- [ ] Test playback cleanup on unmount, loading/error and result changes.
- [ ] Test malformed response and empty/loading/error states.

### Step 5 – Browser / E2E Verification
- [ ] Run the app with Backend API available.
- [ ] Submit the default valid configuration and verify HTTP 200.
- [ ] Verify the chart has `steps + 1` points and starts at time_step 0.
- [ ] Verify S/I/R metrics change when selecting another time step.
- [ ] Verify playback and reset behavior.
- [ ] Submit invalid initial totals and verify the frontend blocks the request.
- [ ] Submit Backend-invalid payload through API tests and verify HTTP 400 is shown safely.
- [ ] Verify request cancellation does not show stale results.

### Step 6 – Final Verification and Documentation
- [ ] Run frontend lint, all unit/component tests and production build.
- [ ] Run full GitHub Actions CI and record run URL + commit SHA.
- [ ] Update this plan with test evidence and remaining limitations.
- [ ] Do not mark complete while component tests or E2E acceptance criteria remain unchecked.

## 8. Acceptance Criteria

- [ ] User can configure all eight SIR parameters.
- [ ] Invalid parameters are rejected before API request with actionable feedback.
- [ ] Valid form submits the exact Backend request contract.
- [ ] Loading state is visible and duplicate submissions are prevented.
- [ ] API errors are shown without exposing internal stack traces.
- [ ] A successful response is validated and rendered in SirResultPanel.
- [ ] `point_count` matches `points.length` and equals `steps + 1`.
- [ ] Selected time step updates S/I/R metrics and time.
- [ ] Playback advances existing points only; it never calls the API again.
- [ ] Abort/stale response handling prevents old requests replacing newer results.
- [ ] Component tests cover the request lifecycle and playback lifecycle.
- [ ] Browser/E2E flow and CI pass.
- [ ] No simulated data is fabricated or assigned to individual Cesium buildings.

## 9. Current Implementation Notes

The first implementation pass is present in the parent branch and inherited by this branch:
- Added `SirSimulationRequest` and `simulateSir`.
- Added JSON POST support and API error handling.
- Added request validation and API unit tests.
- Added the SIR parameter form to HomePage.
- Connected HomePage request state to SirResultPanel.
- Added responsive configuration panel styles.

The first GitHub Actions run for this implementation is pending. Component tests (Vitest/RTL/jsdom) and browser E2E have not yet been completed. Do not mark P1-16 Done until these checks have evidence.

## 10. Definition of Done

P1-16 is complete only when a user can enter valid SIR parameters, submit them to the real Backend API, and inspect the returned S/I/R series through the visualization controls. Request lifecycle, response contract, component interactions, E2E behavior and CI must all be verified. No fake response may be used to satisfy the flow.
