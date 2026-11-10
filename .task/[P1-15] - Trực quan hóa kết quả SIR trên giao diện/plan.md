# KAN-20 – Trực quan hóa kết quả SIR trên giao diện

## 1. Objective

Xây dựng giao diện trực quan hóa chuỗi kết quả SIR do Backend API trả về, giúp người dùng theo dõi biến động Susceptible (S), Infected (I) và Recovered (R) theo thời gian.

Người dùng có thể xem biểu đồ, chọn time_step và duyệt kết quả bằng play/pause/reset. Các điều khiển này chỉ phát lại dữ liệu đã tính toán, không tự chạy lại thuật toán.

**Expected result:**
- Biểu đồ S/I/R theo time_step hoặc time.
- Giá trị S/I/R tại time_step đang chọn.
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
- Xây dựng play/pause/reset để duyệt chuỗi kết quả.
- Xử lý empty/loading/error states ở mức component.
- Responsive layout và kiểm thử dữ liệu mẫu.

### Out of scope
- Xây dựng lại API hoặc thuật toán SIR.
- Hoàn thiện toàn bộ luồng cấu hình/chạy end-to-end (KAN-9, KAN-10).
- Gán trạng thái SIR cho từng công trình trên Cesium.
- Hiển thị hotspot/risk level.
- So sánh nhiều lần chạy hoặc scenario.
- Lưu lịch sử hoặc export báo cáo.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| UI | React + TypeScript | Component và state |
| Chart | Thư viện biểu đồ hiện có hoặc lựa chọn nhẹ phù hợp repository | Vẽ S/I/R |
| API Types | TypeScript interfaces | Kiểu dữ liệu response |
| Styling | CSS theo convention hiện tại | Layout |
| Testing | Frontend test tooling hiện có | Component/state tests |

Kiểm tra package hiện tại trước khi thêm thư viện chart. Ưu tiên tái sử dụng dependency sẵn có.

## 4. Visualization Layout

~~~text
+------------------------------------------------------+
|                 SIR Simulation Result                |
+------------------------------------------------------+
|                                                      |
|              S / I / R Time Series Chart             |
|                                                      |
+------------------------------------------------------+
| Time step: [------------- slider -------------]      |
|                                                      |
| Susceptible       Infected          Recovered         |
|     S                 I                 R             |
|                                                      |
|             [ Play ] [ Pause ] [ Reset ]              |
+------------------------------------------------------+
~~~

Chart cần có legend/nhãn rõ ràng. Không phụ thuộc màu sắc đơn thuần để phân biệt series.

## 5. Data Contract

TypeScript types minh họa, điều chỉnh theo response thực tế:

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
  time_step_count: number;
  points: SirPoint[];
}
~~~

Quy ước:
- points[0] là trạng thái ban đầu, time_step = 0.
- Mỗi point tương ứng một mốc thời gian.
- Không tự tính lại S/I/R ở Frontend.
- Không làm tròn dữ liệu gốc; chỉ format số khi hiển thị.
- Component nhận result hoàn chỉnh từ một lần chạy.

## 6. Implementation Plan

### Step 1 – Inspect API Contract and Frontend Dependencies
- [ ] Kiểm tra response thực tế của KAN-19/KAN-9.
- [ ] Xác nhận field names, numeric types và time_step convention.
- [ ] Kiểm tra thư viện chart trong package.json.
- [ ] Kiểm tra component/layout convention.
- [ ] Xác định vị trí đặt SIR result visualization.
- [ ] Không giả định response field chưa được Backend cung cấp.

### Step 2 – Define TypeScript Data Types
- [ ] Tạo SirPoint interface.
- [ ] Tạo SirSimulationResult interface.
- [ ] Xác định props contract cho chart/result component.
- [ ] Mapping API response nếu naming khác TypeScript convention.
- [ ] Đảm bảo component nhận result từ một lần chạy duy nhất.

### Step 3 – Implement SIR Time Series Chart
- [ ] Tạo component SirResultChart.
- [ ] Map points thành ba series susceptible, infected, recovered.
- [ ] Dùng time_step hoặc time làm trục X theo contract.
- [ ] Hiển thị legend và nhãn trục phù hợp.
- [ ] Đảm bảo ba series dùng cùng tập mốc thời gian.
- [ ] Xử lý points rỗng hoặc dữ liệu không hợp lệ.
- [ ] Không tự nội suy hoặc sinh thêm dữ liệu mô phỏng.

### Step 4 – Implement Current Time Step Metrics
- [ ] Tạo selectedTimeStep state.
- [ ] Lấy point tương ứng từ result theo time_step.
- [ ] Hiển thị S, I, R tại mốc đang chọn.
- [ ] Đồng bộ marker/tooltip trên chart nếu thư viện hỗ trợ.
- [ ] Format số khi hiển thị nhưng giữ raw values.
- [ ] Xử lý selectedTimeStep ngoài phạm vi khi result thay đổi.

### Step 5 – Implement Timeline and Playback Controls
- [ ] Tạo slider/timeline theo time_step có trong points.
- [ ] Slider thay đổi sẽ cập nhật selectedTimeStep và metrics.
- [ ] Play tăng time_step theo thứ tự.
- [ ] Pause dừng duyệt kết quả.
- [ ] Reset quay về time_step đầu tiên.
- [ ] Tự dừng khi tới time_step cuối.
- [ ] Disable hoặc xử lý hợp lý khi points rỗng/chỉ có một point.
- [ ] Play/pause không gọi lại API và không tính lại SIR.
- [ ] Cleanup timer khi component unmount hoặc result thay đổi.
- [ ] Tránh tạo nhiều playback timer đồng thời.

### Step 6 – Implement Empty, Loading and Error States
- [ ] Hiển thị empty state khi chưa có result.
- [ ] Hiển thị loading state khi parent đang chạy request.
- [ ] Hiển thị error state khi API request thất bại.
- [ ] Không render chart từ response lỗi hoặc dữ liệu thiếu.
- [ ] Dừng playback khi chuyển loading/error hoặc thay result.
- [ ] Lỗi chart không làm các phần bản đồ bị crash.

### Step 7 – Add Tests and Responsive Layout
- [ ] Test chart hiển thị đúng ba series.
- [ ] Test chọn time_step cập nhật S/I/R metrics.
- [ ] Test slider min/max và thay đổi giá trị.
- [ ] Test play/pause/reset.
- [ ] Test tự dừng tại time_step cuối.
- [ ] Test empty points và one-point result.
- [ ] Test result mới reset selectedTimeStep/playback phù hợp.
- [ ] Kiểm tra layout desktop và màn hình nhỏ.
- [ ] Chạy TypeScript checks và production build.

## 7. Acceptance Criteria
- [ ] Biểu đồ hiển thị đúng S/I/R từ API response.
- [ ] Trục thời gian phản ánh đúng time_step/time.
- [ ] Chọn time_step cập nhật đúng S, I, R.
- [ ] Slider/timeline giới hạn trong các mốc dữ liệu thực tế.
- [ ] Play/pause/reset chỉ điều khiển duyệt kết quả đã tính.
- [ ] Playback tự dừng tại mốc cuối và không tạo timer trùng.
- [ ] Empty/loading/error states được thể hiện phù hợp.
- [ ] Kết quả và metrics thuộc cùng một simulation result.
- [ ] Không hiển thị trạng thái dịch tễ riêng theo từng công trình.
- [ ] Frontend checks/build và component tests thành công.

## 8. Deliverables
1. TypeScript types cho SIR result.
2. SirResultChart component.
3. Current S/I/R metrics section.
4. Time-step slider/timeline.
5. Play/pause/reset controls.
6. Component tests và responsive layout.

## 9. Suggested Commit Sequence

~~~text
feat(frontend): define SIR result types
feat(frontend): add SIR time-series chart
feat(frontend): add selected time-step metrics
feat(frontend): add SIR result playback controls
test(frontend): cover SIR chart and playback behavior
docs(frontend): document SIR result visualization
~~~

## 10. Final Definition of Done

KAN-20 hoàn thành khi Frontend nhận SIR result và hiển thị đúng ba chuỗi S/I/R, cho phép người dùng chọn hoặc phát lại time_step, đồng thời cập nhật đúng chỉ số tại mốc đang chọn. Component xử lý empty/loading/error và không tính toán mô hình ở Frontend.

Phạm vi chỉ là kết quả SIR tổng thể của Phase 1; không gắn dữ liệu dịch tễ vào từng building entity trên Cesium.
