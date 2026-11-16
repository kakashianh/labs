# KAN-20 – Trực quan hóa kết quả SIR trên giao diện

## 1. Objective

Xây dựng giao diện trực quan hóa chuỗi kết quả SIR do Backend API trả về, giúp người dùng theo dõi biến động Susceptible (S), Infected (I) và Recovered (R) theo thời gian.

Người dùng có thể xem biểu đồ, chọn time_step và duyệt kết quả bằng play/pause/reset. Các điều khiển này chỉ phát lại dữ liệu đã tính toán, không tự chạy lại thuật toán.

**Expected result:**
- Biểu đồ S/I/R theo time_step.
- Giá trị S/I/R và time tại time_step đang chọn.
- Slider/timeline và play/pause/reset.
- Empty/loading/error states.
- Biểu đồ và chỉ số luôn thuộc cùng một response mô phỏng.

## 2. Scope

### In scope
- Rà soát API response từ KAN-19/KAN-9.
- Tạo TypeScript types cho response SIR.
- Xây dựng chart với ba series S, I, R.
- Hiển thị giá trị tại time_step đang chọn.
- Xây dựng slider/timeline.
- Xây dựng play/pause/reset để duyệt kết quả.
- Xử lý empty/loading/error states ở mức component.
- Responsive layout và kiểm thử component/state.

### Out of scope
- Xây dựng lại API hoặc thuật toán SIR.
- Hoàn thiện toàn bộ luồng cấu hình/chạy end-to-end (KAN-9, KAN-10).
- Gán trạng thái SIR cho từng công trình trên Cesium.
- Hiển thị hotspot/risk level.
- So sánh nhiều lần chạy hoặc scenario.
- Lưu lịch sử hoặc export báo cáo.

## 3. Technical Stack and Decisions

| Component | Technology | Purpose |
|---|---|---|
| UI | React + TypeScript | Component và state |
| Chart | Native SVG trong React | Vẽ ba đường S/I/R, tránh thêm chart dependency cho biểu đồ đơn giản |
| API Types | TypeScript interfaces | Kiểu dữ liệu response |
| Styling | CSS theo convention hiện tại | Layout |
| Component Testing | Vitest + React Testing Library + jsdom | Test render và tương tác component |
| Pure Logic Testing | Vitest | Test mapping/validation/playback helpers nếu được tách riêng |

Quyết định triển khai:
- Dùng SVG chart tự xây dựng; không thêm thư viện chart ở task này.
- Repository hiện dùng Node built-in test runner cho service tests và chưa có React component testing setup. Khi triển khai task, bổ sung Vitest, React Testing Library và jsdom; cấu hình script riêng hoặc chuyển test script thống nhất sang Vitest, không làm mất các test hiện có.
- Trước khi thêm package, kiểm tra lại package.json và lockfile trên branch triển khai.
- Không cần thêm state-management library.

## 4. Component Design and Integration Boundary

~~~text
SirResultPanel
├── SirResultChart
├── SirCurrentMetrics
└── SirPlaybackControls
    └── TimelineSlider
~~~

Parent/component contract:

~~~typescript
interface SirResultPanelProps {
  result: SirSimulationResult | null;
  loading: boolean;
  error: string | null;
}
~~~

Responsibilities:
- Parent hiện có chịu trách nhiệm gọi API và truyền result, loading, error xuống panel.
- SirResultPanel điều phối selectedTimeStep và playback state.
- SirResultChart, SirCurrentMetrics và SirPlaybackControls nhận dữ liệu/trạng thái qua props.
- Visualization component không tự gọi API, không chứa request lifecycle và không chạy lại SIR.
- Mọi chart/metrics/control trong một lần hiển thị phải đọc từ cùng một object result.

Vị trí tích hợp vào màn hình hiện tại sẽ được xác định sau khi kiểm tra cấu trúc component trên branch triển khai. Không tự ý thay đổi luồng bản đồ/Cesium ngoài phần cần thiết để đặt panel.

## 5. Visualization Layout

~~~text
+------------------------------------------------------+
|                 SIR Simulation Result                |
+------------------------------------------------------+
|                                                      |
|              S / I / R Time Series Chart             |
|                                                      |
+------------------------------------------------------+
| Time step: [------------- slider -------------]      |
| Time: ...                                             |
|                                                      |
| Susceptible       Infected          Recovered         |
|     S                 I                 R             |
|                                                      |
|             [ Play ] [ Pause ] [ Reset ]              |
+------------------------------------------------------+
~~~

Chart requirements:
- Dùng trục X là time_step; hiển thị time tương ứng tại tooltip/selected metrics.
- Có legend và nhãn series rõ ràng; không phụ thuộc màu sắc đơn thuần để phân biệt series.
- Giữ đúng thứ tự các điểm từ API; không nội suy hoặc sinh thêm dữ liệu.
- SVG phải co giãn theo container và có vùng hiển thị phù hợp trên màn hình nhỏ.

## 6. Data Contract

TypeScript types phải khớp response thực tế của KAN-19:

~~~typescript
export interface SirPoint {
  time_step: number;
  time: number;
  susceptible: number;
  infected: number;
  recovered: number;
}

export interface SirSimulationResult {
  population: number;
  point_count: number;
  points: SirPoint[];
}
~~~

Backend contract:
- point_count là tên field chính xác; không dùng time_step_count.
- point_count === points.length.
- points[0] là trạng thái ban đầu, time_step = 0.
- Số điểm bằng steps + 1.
- time_step tăng tuần tự; time = time_step × dt.
- S/I/R là số thực; không làm tròn dữ liệu gốc.
- Component nhận result hoàn chỉnh từ một lần chạy.
- Format số chỉ áp dụng ở lớp hiển thị.

Validation ở Frontend chỉ nhằm bảo vệ UI trước dữ liệu thiếu/sai contract; không thay thế validation của Backend. Không render chart nếu result không hợp lệ (ví dụ point_count lệch points.length, thiếu field số hoặc time_step không hợp lệ).

## 7. Implementation Plan

### Step 1 – Inspect API Contract and Frontend Structure
- [ ] Đối chiếu response với backend/src/routes/simulations.rs và backend/docs/sir-api.md.
- [ ] Xác nhận field names, numeric types và time_step convention.
- [ ] Kiểm tra package.json/lockfile và test tooling hiện tại.
- [ ] Kiểm tra component/layout convention và vị trí tích hợp panel.
- [ ] Không giả định response field chưa được Backend cung cấp.

### Step 2 – Define TypeScript Data Types and Validation
- [ ] Tạo SirPoint interface.
- [ ] Tạo SirSimulationResult interface dùng point_count.
- [ ] Xác định props contract cho SirResultPanel.
- [ ] Tạo helper kiểm tra response tối thiểu trước khi render.
- [ ] Đảm bảo component nhận result từ một lần chạy duy nhất.
- [ ] Không chuyển đổi hoặc làm tròn raw values.

### Step 3 – Implement SIR Time Series Chart
- [ ] Tạo component SirResultChart bằng React + SVG.
- [ ] Map points thành ba series susceptible, infected, recovered.
- [ ] Dùng time_step làm trục X.
- [ ] Hiển thị legend và nhãn trục phù hợp.
- [ ] Đảm bảo ba series dùng cùng tập mốc thời gian.
- [ ] Đồng bộ selected point/marker với time_step đang chọn.
- [ ] Xử lý points rỗng hoặc dữ liệu không hợp lệ.
- [ ] Không tự nội suy hoặc sinh thêm dữ liệu mô phỏng.

### Step 4 – Implement Current Time Step Metrics
- [ ] Tạo selectedTimeStep state tại SirResultPanel.
- [ ] Lấy point tương ứng từ result theo time_step.
- [ ] Hiển thị S, I, R và time tại mốc đang chọn.
- [ ] Đồng bộ marker/tooltip trên chart.
- [ ] Format số khi hiển thị nhưng giữ raw values.
- [ ] Khi result mới được truyền vào, reset selectedTimeStep về điểm đầu và dừng playback.
- [ ] Xử lý selectedTimeStep ngoài phạm vi khi result thay đổi.

### Step 5 – Implement Timeline and Playback Controls
- [ ] Tạo slider/timeline theo các time_step có trong points.
- [ ] Slider thay đổi sẽ cập nhật selectedTimeStep và metrics.
- [ ] Play tăng đến điểm kế tiếp theo thứ tự points.
- [ ] Playback interval mặc định 500 ms/điểm; không thay đổi dữ liệu mô phỏng hoặc time gốc.
- [ ] Pause dừng duyệt kết quả.
- [ ] Reset dừng playback và quay về điểm đầu tiên.
- [ ] Tự dừng khi tới time_step cuối.
- [ ] Disable hoặc xử lý hợp lý khi points rỗng/chỉ có một point.
- [ ] Play/pause không gọi API và không tính lại SIR.
- [ ] Cleanup timer khi component unmount, loading/error hoặc result thay đổi.
- [ ] Đảm bảo tối đa một playback timer hoạt động tại một thời điểm.

### Step 6 – Implement Empty, Loading and Error States
- [ ] Hiển thị empty state khi chưa có result.
- [ ] Hiển thị loading state khi parent đang chạy request.
- [ ] Hiển thị error state khi API request thất bại.
- [ ] Không render chart từ response lỗi hoặc dữ liệu thiếu/sai contract.
- [ ] Dừng playback khi chuyển loading/error hoặc thay result.
- [ ] Lỗi chart không làm các phần bản đồ bị crash.

### Step 7 – Add Component Tests and Responsive Layout
- [ ] Cấu hình Vitest + React Testing Library + jsdom theo stack hiện tại.
- [ ] Bảo toàn/chạy lại các service tests đang dùng Node test runner.
- [ ] Test chart render đủ ba series và đúng dữ liệu đầu vào.
- [ ] Test chọn time_step cập nhật S/I/R/time metrics.
- [ ] Test slider min/max và thay đổi giá trị.
- [ ] Test play/pause/reset và playback interval.
- [ ] Test tự dừng tại time_step cuối.
- [ ] Test empty points, one-point result và invalid response.
- [ ] Test result mới reset selectedTimeStep và dừng playback cũ.
- [ ] Test timer cleanup khi unmount/loading/error.
- [ ] Kiểm tra layout desktop và màn hình nhỏ.
- [ ] Chạy lint, TypeScript checks, toàn bộ frontend tests và production build.

## 8. Acceptance Criteria

- [ ] Biểu đồ hiển thị đúng S/I/R từ API response.
- [ ] Data contract sử dụng đúng point_count và points.
- [ ] Trục thời gian phản ánh đúng time_step; thời gian thực được hiển thị theo time.
- [ ] Chọn time_step cập nhật đúng S, I, R và time.
- [ ] Slider/timeline giới hạn trong các mốc dữ liệu thực tế.
- [ ] Play/pause/reset chỉ điều khiển duyệt kết quả đã tính.
- [ ] Playback chạy tuần tự, mặc định 500 ms/điểm, tự dừng tại mốc cuối và không tạo timer trùng.
- [ ] Timer được cleanup khi unmount, loading/error hoặc nhận result mới.
- [ ] Empty/loading/error states được thể hiện phù hợp.
- [ ] Kết quả và metrics thuộc cùng một simulation result.
- [ ] Không hiển thị trạng thái dịch tễ riêng theo từng công trình.
- [ ] Frontend lint, TypeScript checks, tests và production build thành công.

## 9. Deliverables

1. TypeScript types và response validation cho SIR result.
2. SirResultPanel component.
3. SirResultChart SVG component.
4. Current S/I/R/time metrics section.
5. Time-step slider/timeline.
6. Play/pause/reset controls với timer lifecycle an toàn.
7. Component tests và responsive layout.
8. Cấu hình test tooling cần thiết, không làm mất các test hiện có.

## 10. Suggested Commit Sequence

~~~text
feat(frontend): define SIR result types and validation
feat(frontend): add SVG SIR time-series chart
feat(frontend): add selected time-step metrics
feat(frontend): add SIR result playback controls
test(frontend): add SIR visualization component tests
docs(frontend): document SIR result visualization
~~~

## 11. Final Definition of Done

KAN-20 hoàn thành khi Frontend nhận SIR result theo đúng API contract (population, point_count, points) và hiển thị đúng ba chuỗi S/I/R; người dùng có thể chọn hoặc phát lại time_step, đồng thời cập nhật đúng metrics tại mốc đang chọn. Component xử lý empty/loading/error, kiểm soát timer lifecycle và không tính toán mô hình hoặc gọi lại API trong quá trình playback.

Phạm vi chỉ là kết quả SIR tổng thể của Phase 1; không gắn dữ liệu dịch tễ vào từng building entity trên Cesium.
