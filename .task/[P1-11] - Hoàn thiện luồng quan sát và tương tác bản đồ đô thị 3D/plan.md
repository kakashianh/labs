# KAN-8 – Hoàn thiện luồng quan sát và tương tác bản đồ đô thị 3D

## 1. Objective

Tích hợp và kiểm thử end-to-end các thành phần bản đồ đã được triển khai ở KAN-16 và KAN-17, đảm bảo người dùng có thể mở bản đồ, quan sát công trình từ PostGIS, điều hướng, chọn công trình và xem thông tin chi tiết một cách nhất quán.

Đây là task tích hợp và hoàn thiện luồng sử dụng, không phải task triển khai lại các API hoặc component đã có.

**Expected result:**
- Người dùng mở được bản đồ và quan sát công trình từ dữ liệu PostGIS.
- Các thao tác điều hướng và chọn công trình hoạt động đồng thời.
- Panel thông tin đồng bộ với công trình đang chọn.
- Loading, empty và error state nhất quán.
- Có checklist hoặc integration tests cho luồng end-to-end trên dataset mẫu.

## 2. Scope

### In scope
- Rà soát kết quả KAN-16 và KAN-17.
- Tích hợp data loading, Cesium rendering, selection và detail panel.
- Kiểm tra luồng tải dữ liệu từ Backend đến Frontend.
- Chuẩn hóa trạng thái loading, empty, error và selected.
- Kiểm tra tương tác camera khi panel mở/đóng.
- Kiểm tra tính nhất quán ID và dữ liệu giữa map entity và detail panel.
- Thực hiện integration/E2E testing hoặc checklist thủ công có thể lặp lại.
- Sửa lỗi tích hợp phát hiện trong phạm vi luồng bản đồ.

### Out of scope
- Viết lại API KAN-15/KAN-7.
- Xây dựng lại Cesium data source hoặc BuildingInfoPanel nếu đã hoàn thành ở KAN-16/KAN-17.
- Thêm chức năng chỉnh sửa dữ liệu GIS.
- Mô phỏng SIR.
- Hiển thị trạng thái dịch tễ hoặc hotspot.
- Mở rộng thành hệ thống quản lý nhiều lớp bản đồ.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Frontend | React + TypeScript + Vite | Điều phối UI |
| 3D GIS | CesiumJS | Bản đồ và picking |
| Backend | Rust + Axum | GIS APIs |
| Database | PostgreSQL + PostGIS | Dữ liệu nguồn |
| Testing | Test framework hiện có / browser E2E nếu repository đã cấu hình | Kiểm thử tích hợp |

Ưu tiên dùng test tooling đang có trong repository. Không bổ sung framework E2E nặng nếu checklist tích hợp thủ công đáp ứng được MVP và chưa có hạ tầng tương ứng.

## 4. End-to-End Workflow

~~~text
Open Map Page
     |
     v
Load Regions + Buildings API
     |
     v
Render Building Footprints in Cesium
     |
     v
Navigate Map (Zoom / Pan / Rotate)
     |
     v
Select a Building
     |
     v
Fetch Building Detail
     |
     v
Show Information Panel
     |
     +------ Select another building
     |                 |
     |                 v
     |          Update selected detail
     |
     +------ Click empty map / close
                       |
                       v
                Clear selection
~~~

## 5. Implementation Plan

### Step 1 – Review Outputs from KAN-16 and KAN-17

- [ ] Kiểm tra data loading flow và API client của KAN-16.
- [ ] Kiểm tra Cesium data source và building entity IDs.
- [ ] Kiểm tra selection state và click handler của KAN-17.
- [ ] Kiểm tra BuildingInfoPanel và detail API integration.
- [ ] Xác nhận route/API response thực tế giữa Frontend và Backend.
- [ ] Ghi nhận các vấn đề tích hợp trước khi chỉnh sửa.

### Step 2 – Integrate Map, Selection and Detail Panel

- [ ] Kết nối selected building ID từ Cesium picking tới React state.
- [ ] Đảm bảo panel nhận detail data theo đúng selected ID.
- [ ] Đảm bảo đổi selection sẽ tải/cập nhật detail tương ứng.
- [ ] Đảm bảo bỏ chọn sẽ clear panel và highlight.
- [ ] Tránh duplicate event handlers khi Viewer/component re-render.
- [ ] Đảm bảo cleanup đúng với Cesium Viewer lifecycle.
- [ ] Không tạo state trùng lặp nếu có thể derive từ một selectedBuildingId duy nhất.

### Step 3 – Normalize Loading, Empty and Error States

Kiểm tra trạng thái ở các giai đoạn:
- Tải danh sách khu vực/công trình.
- Không có công trình phù hợp.
- Tải chi tiết công trình.
- API chậm hoặc timeout.
- Backend không khả dụng.
- Công trình không còn tồn tại hoặc detail API trả 404.

- [ ] Thống nhất loading indicator cho lần tải dữ liệu bản đồ.
- [ ] Có empty state khi không có dữ liệu công trình.
- [ ] Có error state và hành vi retry phù hợp nếu đã hỗ trợ.
- [ ] Có detail loading state khi chọn công trình.
- [ ] Không hiển thị detail cũ như thể thuộc selection mới.
- [ ] Đảm bảo lỗi detail API không làm Viewer bị unmount/crash.
- [ ] Không hiển thị trạng thái lỗi đồng thời với empty state nếu nguyên nhân thực tế là request thất bại.

### Step 4 – Verify Map Navigation and Selection Together

- [ ] Kiểm tra zoom khi không có panel.
- [ ] Kiểm tra pan/rotate/tilt khi không có panel.
- [ ] Kiểm tra chọn công trình sau khi di chuyển camera.
- [ ] Kiểm tra camera vẫn tương tác được khi panel đang mở.
- [ ] Kiểm tra chọn liên tiếp nhiều công trình.
- [ ] Kiểm tra bỏ chọn bằng click vùng trống.
- [ ] Kiểm tra đóng panel và clear highlight.
- [ ] Kiểm tra không có click handler bị đăng ký lặp.

### Step 5 – Verify Data Consistency

- [ ] Đối chiếu building ID của Cesium entity với ID được gửi tới detail API.
- [ ] Đối chiếu thông tin panel với response detail API.
- [ ] Kiểm tra pagination không làm bỏ sót trang dữ liệu trong phạm vi dataset mẫu.
- [ ] Kiểm tra geometry hiển thị đúng khu vực nghiên cứu.
- [ ] Kiểm tra CRS/coordinate order không gây lệch vị trí.
- [ ] Kiểm tra record có thuộc tính null/thiếu vẫn hiển thị an toàn.
- [ ] Ghi nhận giới hạn hiệu năng với dataset mẫu.

### Step 6 – Execute Integration Tests / E2E Checklist

Tạo checklist có thể chạy lại theo môi trường local.

| Scenario | Expected result |
|---|---|
| Mở trang bản đồ khi Backend hoạt động | Bản đồ và công trình được tải |
| API trả danh sách rỗng | Hiển thị empty state |
| API lỗi/Backend tắt | Hiển thị error state, Viewer không crash |
| Chọn một công trình | Highlight và panel hiển thị đúng detail |
| Chọn công trình khác | Highlight và detail chuyển sang công trình mới |
| Click vùng trống | Selection và panel được clear |
| API detail chậm | Không hiển thị nhầm detail cũ |
| Điều hướng khi panel mở | Zoom/pan/rotate vẫn hoạt động |
| Tải lại trang | Không xuất hiện duplicate data source/entity |

- [ ] Tạo tài liệu checklist hoặc test suite theo khả năng repository.
- [ ] Thực thi từng scenario trên dataset đô thị mẫu.
- [ ] Ghi kết quả pass/fail và lỗi phát hiện.
- [ ] Sửa lỗi thuộc phạm vi tích hợp.
- [ ] Chạy lại toàn bộ checklist sau khi sửa.

### Step 7 – Final Verification and Documentation

- [ ] Chạy Backend và PostGIS theo hướng dẫn dự án.
- [ ] Chạy Frontend và xác nhận API base URL đúng.
- [ ] Chạy frontend checks/build.
- [ ] Chạy Backend tests liên quan nếu có thay đổi API integration.
- [ ] Kiểm tra browser console và network requests.
- [ ] Cập nhật README hoặc tài liệu chạy luồng bản đồ.
- [ ] Lưu checklist kết quả kiểm thử.

## 6. Acceptance Criteria

- [ ] Người dùng mở được bản đồ và quan sát công trình từ PostGIS.
- [ ] Zoom, pan và rotate/tilt không làm gián đoạn thao tác chọn công trình.
- [ ] Chọn công trình hiển thị đúng thông tin.
- [ ] Chọn công trình khác cập nhật đúng highlight và detail.
- [ ] Bỏ chọn clear panel/selection theo quy ước.
- [ ] Loading, empty và error state được xử lý nhất quán.
- [ ] API chậm hoặc lỗi không làm Cesium Viewer crash.
- [ ] Không có stale detail response làm sai thông tin công trình đang chọn.
- [ ] Có integration tests hoặc E2E checklist được thực thi và lưu kết quả.
- [ ] Frontend build và các kiểm thử liên quan thành công.

## 7. Deliverables

1. Luồng bản đồ end-to-end được tích hợp.
2. Các lỗi tích hợp trong phạm vi KAN-16/KAN-17 được xử lý.
3. Integration tests hoặc E2E checklist.
4. Kết quả kiểm thử trên bộ dữ liệu đô thị mẫu.
5. Tài liệu chạy và kiểm tra luồng bản đồ.

## 8. Suggested Commit Sequence

~~~text
refactor(map): integrate map selection and detail state

fix(map): normalize loading empty and error states

test(map): add end-to-end map interaction checklist

fix(map): resolve map workflow integration issues

docs(map): document end-to-end map verification
~~~

## 9. Final Definition of Done

KAN-8 hoàn thành khi toàn bộ luồng mở bản đồ → tải công trình → điều hướng → chọn công trình → xem chi tiết → đổi lựa chọn/bỏ chọn hoạt động nhất quán trên bộ dữ liệu đô thị mẫu, các trạng thái lỗi được xử lý và có kết quả kiểm thử tích hợp có thể đối chiếu.

Ticket này đóng gói và xác nhận chất lượng luồng bản đồ Phase 1; không mở rộng sang mô phỏng SIR hoặc phân tích dịch tễ.
