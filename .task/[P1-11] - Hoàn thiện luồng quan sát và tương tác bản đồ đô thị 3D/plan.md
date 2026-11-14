# KAN-8 – Hoàn thiện luồng quan sát và tương tác bản đồ đô thị 3D

## 1. Objective

Tích hợp và kiểm thử end-to-end các thành phần bản đồ đã được triển khai ở P1-09 và P1-10, đảm bảo người dùng có thể mở bản đồ, quan sát công trình từ PostGIS, điều hướng, chọn công trình và xem thông tin chi tiết một cách nhất quán.

Đây là task tích hợp và hoàn thiện luồng sử dụng, không phải task triển khai lại các API hoặc component đã có.

**Expected result:**
- Người dùng mở được bản đồ và quan sát công trình từ dữ liệu PostGIS.
- Các thao tác điều hướng và chọn công trình hoạt động đồng thời.
- Panel thông tin đồng bộ với công trình đang chọn.
- Loading, empty và error state nhất quán.
- Có checklist hoặc integration tests cho luồng end-to-end trên dataset mẫu.

## 2. Scope

### In scope
- Rà soát kết quả P1-09 và P1-10.
- Tích hợp data loading, Cesium rendering, selection và detail panel.
- Kiểm tra luồng tải dữ liệu từ Backend đến Frontend.
- Chuẩn hóa trạng thái loading, empty, error và selected.
- Kiểm tra tương tác camera khi panel mở/đóng.
- Kiểm tra tính nhất quán ID và dữ liệu giữa map entity và detail panel.
- Thực hiện integration/E2E testing hoặc checklist thủ công có thể lặp lại.
- Sửa lỗi tích hợp phát hiện trong phạm vi luồng bản đồ.

### Out of scope
- Viết lại API P1-07/P1-08.
- Xây dựng lại Cesium data source hoặc BuildingInfoPanel nếu đã hoàn thành ở P1-09/P1-10.
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

### Step 1 – Review Outputs from P1-09 and P1-10

- [ ] Kiểm tra data loading flow và API client của P1-09.
- [ ] Kiểm tra Cesium data source và building entity IDs.
- [ ] Kiểm tra selection state và click handler của P1-10.
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
2. Các lỗi tích hợp trong phạm vi P1-09/P1-10 được xử lý.
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


---

## 10. API and frontend integration contract (2026-11-13)

This ticket is the integration/acceptance layer for P1-09 and P1-10. It must validate their outputs rather than implement a second version of their responsibilities.

### Backend endpoints

| Purpose | Endpoint | Contract |
|---|---|---|
| Region list | `GET /api/v1/regions` | `{ data: RegionListItem[] }`; each item has UUID `id`, `code`, `name`, nullable `description` |
| Region detail | `GET /api/v1/regions/{region_id}` | `{ data: RegionDetail }`; includes GeoJSON Geometry boundary |
| Building list | `GET /api/v1/buildings` | `{ data: BuildingDto[], pagination: { page, page_size, total, total_pages } }` |
| Building detail | `GET /api/v1/buildings/{building_id}` | `{ data: BuildingDetail }`; detail lookup by UUID |

Building list query parameters:
- `page`: default 1, positive integer.
- `page_size`: default 100, positive integer, maximum 500.
- `region_id`: optional UUID filter.
- Empty collection is valid: `data: []` and `pagination.total_pages: 0`.

Each building list/detail DTO contains `id` (UUID), `region_id` (UUID), `external_id`, nullable `name`, nullable `building_type`, and `geometry` as a GeoJSON Geometry. The current dataset contains 431 buildings, but this is a dataset snapshot, **not** a fixed application constant.

### Data-loading and Cesium rules

- Follow P1-09's pagination aggregation: request pages until `page >= pagination.total_pages`; do not assume one page or hard-code 431.
- The API returns Geometry objects, not GeoJSON Features or a FeatureCollection. The P1-09 adapter must create Features and a FeatureCollection (or perform an equivalent explicit Cesium mapping).
- Preserve `building.id` UUID as the stable Cesium entity key and as the detail API path parameter. Do not substitute `external_id`, entity index or display name.
- Do not make N+1 detail requests. The list payload is sufficient for initial map rendering; fetch detail only after a user selects a building.
- Geometry uses GeoJSON coordinate order and the API currently exposes MultiPolygon geometries in EPSG:4326; validate placement against the known study-area extent rather than silently swapping coordinates.
- Region list does not contain boundary geometry; request region detail only if the map workflow actually needs the boundary.

### Ownership boundaries

- **P1-09:** API client/list loading, pagination, GeoJSON-to-Cesium mapping, initial building rendering and stable entity identity.
- **P1-10:** camera/interaction polish, selected UUID state, detail API request, information panel and selected-entity highlight.
- **P1-11:** end-to-end verification, state consistency, error-path validation and fixes for integration defects. Do not duplicate the features owned by P1-09/P1-10.

### Additional integration acceptance checks

- [ ] Test more than one page using mocked pagination, even if the current 431-building snapshot fits in a single page at `page_size=500`.
- [ ] Test empty list and final partial page.
- [ ] Assert every rendered entity retains the source UUID and correct `region_id`.
- [ ] Assert selecting an entity calls the detail endpoint with its UUID and unwraps `response.data`.
- [ ] Assert null `name`/`building_type` do not crash or produce misleading content.
- [ ] Assert a failed list request is not represented as a successful empty dataset.
- [ ] Assert stale detail responses cannot replace the currently selected building.
- [ ] Assert camera interaction remains available while the information panel is open.

**Prerequisite status:** P1-07 and P1-08 APIs are merged; P1-09 plan has been aligned to those contracts. P1-11 implementation must still verify the actual frontend behavior and record test evidence; this documentation update does not mark P1-09, P1-10 or P1-11 implementation as complete.


---

## 11. Dependency and execution plan update (2026-11-14)

### 11.1 Current prerequisite status

- [x] P1-09 implementation is already merged into `project/3D-map-medical` via PR #12.
- [x] P1-10 implementation is present on `feat/p1-10-building-info-panel`; automated CI run #43576392119 passed the required checks.
- [x] P1-10 manual browser verification has been reported successful by the user, covering the interaction scenarios recorded in the P1-10 plan: camera navigation, selection after camera movement, A/B and rapid selection, empty-map deselection, non-building entity behavior, panel close/responsive layout, and map interaction while the panel is open.
- [ ] P1-10 has not yet been merged into `project/3D-map-medical`. Merge is a prerequisite before P1-11 integration work begins.

**Branching rule:** Do not implement P1-11 against an unmerged feature branch as if it were the integration baseline. After P1-10 is merged, create/continue the P1-11 implementation branch from the updated `project/3D-map-medical` and record the merge commit here.

### 11.2 Avoid duplicate work from P1-10

P1-10 has already implemented and manually verified the following, so P1-11 should validate the integrated workflow rather than re-implement these features:

- Cesium building selection using the stable building UUID.
- Selected building state and detail API request lifecycle.
- Building information panel, including loading/error/not-found and nullable-field handling.
- Switching selection, clearing selection, closing the panel and restoring highlight.
- AbortController cancellation and stale-response protection.
- Camera navigation and panel interaction behavior.

P1-11 must still verify these behaviors in the integrated application after the P1-10 merge, especially that the P1-09 data-loading/rendering lifecycle and P1-10 selection/detail lifecycle work together without duplicate handlers, stale entities or inconsistent state.

### 11.3 Refined P1-11 execution order

1. **Integration baseline:** Confirm P1-10 merge commit is included in `project/3D-map-medical`; inspect the merged P1-09/P1-10 source and confirm the actual runtime API configuration.
2. **Full data path:** Start Backend + PostGIS and Frontend; verify list API → pagination → GeoJSON adapter → Cesium entities → selected UUID → detail API → information panel.
3. **List-state coverage:** Prioritize P1-11-specific cases not owned by P1-10: empty building list, failed list request versus true empty response, multi-page pagination, final partial page, page reload and duplicate data-source prevention.
4. **Cross-component consistency:** Verify entity UUID and region ID preservation, detail response matching, coordinate placement/CRS and behavior when data source reloads or a selected entity disappears.
5. **Regression pass:** Re-run the key P1-10 interactions once against the merged integration baseline; do not duplicate implementation or add another detail state store.
6. **Evidence and documentation:** Record environment, dataset snapshot, CI run, browser/network observations, pass/fail and any known limitations in a repeatable checklist.

### 11.4 P1-10 edge case carried into integration

The P1-10 plan still tracks safe handling when a selected Cesium entity is removed from its data source. P1-11 must explicitly test data-source refresh/removal while a building is selected. If the current implementation leaves a stale highlight or detail panel, fix it within the integration scope and add a regression test. Do not mark this scenario as passed until it is actually exercised.

### 11.5 Completion boundary

P1-10's successful CI and browser checks do not automatically complete P1-11. P1-11 remains responsible for end-to-end evidence across the complete Backend/PostGIS-to-Cesium workflow, especially pagination, empty/error list states, coordinate correctness, lifecycle cleanup and repeatable integration verification.
