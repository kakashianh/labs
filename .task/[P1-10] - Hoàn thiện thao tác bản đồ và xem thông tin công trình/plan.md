# KAN-17 – Hoàn thiện thao tác bản đồ và xem thông tin công trình

## 1. Objective

Hoàn thiện trải nghiệm tương tác trên bản đồ CesiumJS để người dùng có thể điều hướng bản đồ, chọn công trình và xem thông tin chi tiết của công trình được chọn.

Ticket này phát triển trên nền tảng hiển thị geometry và khả năng chọn công trình cơ bản đã có ở P1-09. Trọng tâm là quản lý selected state và trình bày thông tin từ API.

**Expected result:**
- Người dùng zoom, pan, rotate và tilt bản đồ ổn định.
- Click công trình sẽ mở popup/panel thông tin.
- Thông tin hiển thị đúng theo building ID và API response.
- Chọn công trình khác cập nhật nội dung panel.
- Bỏ chọn sẽ đóng panel hoặc xóa selected state theo quy ước giao diện.
- Thiếu thuộc tính không làm giao diện lỗi.

## 2. Scope

### In scope
- Rà soát Cesium Viewer và cơ chế chọn entity từ P1-09.
- Hoàn thiện thao tác điều hướng bản đồ.
- Quản lý selected building state trong React.
- Gọi API chi tiết công trình theo ID từ P1-08.
- Xây dựng popup/panel hiển thị thông tin công trình.
- Highlight công trình được chọn.
- Xử lý chuyển lựa chọn, bỏ chọn và dữ liệu thiếu.
- Đảm bảo panel không cản trở thao tác bản đồ ngoài vùng tương tác của panel.
- Kiểm thử các trạng thái tương tác.

### Out of scope
- Tải và render geometry công trình từ API (P1-09).
- Xây dựng lại API chi tiết công trình (P1-08).
- Tích hợp end-to-end toàn bộ luồng bản đồ (KAN-8).
- Hiển thị trạng thái SIR hoặc màu nguy cơ dịch bệnh.
- Chỉnh sửa thuộc tính công trình.
- Các chức năng tìm kiếm, lọc nâng cao hoặc quản lý nhiều lớp GIS.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| UI | React + TypeScript | Quản lý giao diện và state |
| 3D GIS | CesiumJS | Điều hướng và picking entity |
| API | Rust + Axum | Cung cấp building detail |
| Database | PostgreSQL + PostGIS | Nguồn dữ liệu công trình |
| Styling | CSS theo convention hiện có | Popup/panel |

Không tạo service/backend mới. Tái sử dụng API client và Cesium Viewer đã được xây dựng.

## 4. Interaction Flow

~~~text
User clicks a building
        |
        v
Cesium ScreenSpaceEventHandler
        |
        v
Resolve picked entity + building_id
        |
        v
Update selectedBuildingId
        |
        v
GET /api/v1/buildings/{building_id}
        |
        v
Loading detail state
        |
        v
Render building information panel
        |
        +---- Click another building --> Load new detail
        |
        +---- Click empty map --------> Clear selection / close panel
~~~

Endpoint và response phải khớp API contract thực tế của P1-08.

## 5. Implementation Plan

### Step 1 – Inspect Existing Map Selection and Building Detail API

- [ ] Kiểm tra Cesium entity properties được gắn ở P1-09.
- [ ] Xác nhận entity lưu building ID ổn định.
- [ ] Kiểm tra endpoint GET /api/v1/buildings/{building_id} của P1-08.
- [ ] Kiểm tra response DTO và các thuộc tính thực tế có thể hiển thị.
- [ ] Kiểm tra API client/service hiện có để tái sử dụng.
- [ ] Xác định quy ước click vào vùng trống để bỏ chọn.

### Step 2 – Implement Selected Building State

Quản lý trạng thái lựa chọn ở React thay vì để panel phụ thuộc trực tiếp vào Cesium internals.

State đề xuất:
- selectedBuildingId: ID công trình đang được chọn hoặc null.
- buildingDetail: dữ liệu chi tiết hiện tại.
- detailLoading: trạng thái tải.
- detailError: lỗi tải thông tin nếu có.

- [ ] Tạo state selectedBuildingId.
- [ ] Tạo state dữ liệu detail và loading/error.
- [ ] Khi chọn entity có building ID, cập nhật selectedBuildingId.
- [ ] Khi click entity không phải công trình, không tạo selected building giả.
- [ ] Khi click vùng trống, clear selectedBuildingId theo quy ước UX.
- [ ] Tránh response cũ ghi đè dữ liệu khi người dùng chọn liên tiếp nhiều công trình; dùng AbortController hoặc kiểm tra ID của request hiện hành.

### Step 3 – Fetch Building Detail

- [ ] Tái sử dụng API client của Frontend.
- [ ] Gọi GET /api/v1/buildings/{building_id} khi selectedBuildingId thay đổi.
- [ ] Hủy request trước đó khi selection thay đổi nếu có thể.
- [ ] Reset detail state phù hợp khi bỏ chọn.
- [ ] Xử lý 404 khi công trình không còn tồn tại.
- [ ] Xử lý network/API error.
- [ ] Không gọi API detail nếu entity không có building ID hợp lệ.

### Step 4 – Implement Building Information Panel

Tạo component riêng, ví dụ BuildingInfoPanel.

Nội dung:
- Building ID hoặc mã định danh.
- Region reference nếu API cung cấp.
- Các thuộc tính mô tả thực sự có trong response.
- Geometry metadata chỉ khi có giá trị hữu ích cho người dùng.

Yêu cầu:
- Không hard-code field không tồn tại trong dataset.
- Field null/undefined/empty được ẩn hoặc hiển thị fallback nhất quán.
- Có loading, error và not-found state.
- Có nút đóng panel hoặc hành vi bỏ chọn rõ ràng.
- Panel có layout responsive, không che toàn bộ vùng bản đồ trên màn hình thông thường.

- [ ] Tạo BuildingInfoPanel component.
- [ ] Render thông tin từ API response.
- [ ] Hiển thị loading state.
- [ ] Hiển thị error/not-found state.
- [ ] Xử lý thuộc tính thiếu hoặc null.
- [ ] Thêm thao tác đóng panel.
- [ ] Kiểm tra panel trên các kích thước màn hình khác nhau.

### Step 5 – Highlight and Synchronize Selected Entity

- [ ] Highlight entity đang được chọn.
- [ ] Khôi phục style entity trước đó khi chọn công trình khác.
- [ ] Khôi phục style mặc định khi bỏ chọn.
- [ ] Đảm bảo selected state và panel luôn cùng building ID.
- [ ] Không làm thay đổi geometry hoặc dữ liệu gốc của entity.
- [ ] Xử lý an toàn khi entity không còn trong data source.

### Step 6 – Verify Map Navigation and Interaction

- [ ] Kiểm tra zoom bằng chuột/scroll.
- [ ] Kiểm tra pan và rotate/tilt.
- [ ] Kiểm tra click chọn công trình sau khi camera đã di chuyển.
- [ ] Kiểm tra thao tác bản đồ khi panel đang mở.
- [ ] Kiểm tra click công trình A rồi công trình B.
- [ ] Kiểm tra click vùng trống để bỏ chọn.
- [ ] Kiểm tra đóng panel có đồng bộ selected state.
- [ ] Kiểm tra rapid selection không hiển thị nhầm detail response cũ.

### Step 7 – Add Tests and Documentation

- [ ] Test selected state khi chọn entity hợp lệ.
- [ ] Test bỏ chọn khi click vùng trống.
- [ ] Test chuyển selection giữa hai công trình.
- [ ] Test API detail loading/success/error/not-found.
- [ ] Test field null/undefined không làm component crash.
- [ ] Test request race condition hoặc stale response.
- [ ] Cập nhật hướng dẫn thao tác bản đồ nếu cần.
- [ ] Chạy TypeScript checks và production build.

## 6. Acceptance Criteria

- [ ] Zoom, pan, rotate và tilt hoạt động ổn định.
- [ ] Click công trình hiển thị đúng building ID và thông tin API cung cấp.
- [ ] Chọn công trình khác làm panel cập nhật đúng dữ liệu mới.
- [ ] Bỏ chọn làm đóng panel hoặc clear nội dung theo quy ước đã thống nhất.
- [ ] Selected highlight được cập nhật và khôi phục đúng.
- [ ] Thiếu thuộc tính không làm giao diện lỗi.
- [ ] Loading/error/not-found được xử lý.
- [ ] Response cũ không ghi đè thông tin của selection mới.
- [ ] Mở/đóng panel không làm mất khả năng điều hướng bản đồ.
- [ ] Frontend build và các tests liên quan thành công.

## 7. Deliverables

1. Selected building state management.
2. Building detail API integration.
3. BuildingInfoPanel component.
4. Selected entity highlight/clear behavior.
5. Tests cho selection và detail panel.
6. Hướng dẫn tương tác bản đồ cơ bản.

## 8. Suggested Commit Sequence

~~~text
feat(map): manage selected building state

feat(frontend): fetch building detail on selection

feat(frontend): add building information panel

feat(map): highlight and clear selected building

test(frontend): cover building selection and detail panel

docs(frontend): document map interaction behavior
~~~

## 9. Final Definition of Done

KAN-17 hoàn thành khi người dùng có thể điều hướng bản đồ, chọn một công trình để xem thông tin, chuyển sang công trình khác hoặc bỏ chọn; panel luôn phản ánh đúng selection hiện tại và không làm gián đoạn thao tác bản đồ.

Ticket này hoàn thiện tương tác và hiển thị thông tin; không triển khai lại phần tải/render dữ liệu GIS của P1-09.


---

## 10. API contract and dependency alignment (2026-11-13)

P1-10 consumes the merged backend APIs from P1-07/P1-08 and the Cesium building entities produced by P1-09. Do not use the old KAN-7 endpoint assumptions or infer a building identifier from OSM `external_id`.

### Building detail API — P1-08

```http
GET /api/v1/buildings/{building_id}
```

Success response shape:

```json
{
  "data": {
    "id": "<UUID>",
    "region_id": "<UUID>",
    "external_id": "way/...",
    "name": null,
    "building_type": "yes",
    "geometry": {
      "type": "MultiPolygon",
      "coordinates": []
    }
  }
}
```

- `building_id` path parameter must be a valid UUID. Invalid UUID returns 400; a valid UUID not found returns 404.
- `data.id` and `data.region_id` are UUIDs. `external_id` is the source identifier and is not the primary key used by the API.
- `name` and `building_type` are nullable. The frontend must render safe fallbacks and must not assume a name exists.
- `geometry` is a GeoJSON Geometry object (currently MultiPolygon), not a Feature or FeatureCollection. P1-10 normally does not need to render this detail geometry again.
- API response is wrapped in `{ "data": ... }`; read the detail DTO from `response.data`.

### P1-09 → P1-10 handoff

- P1-09 owns loading the paginated building collection, converting each Geometry to a GeoJSON Feature/FeatureCollection for Cesium, and attaching the stable UUID `id` to the corresponding entity properties.
- P1-10 owns selection state, detail fetching, panel and highlight behavior.
- Resolve the selected entity's UUID from the property attached by P1-09; do not use entity display name, array index, or `external_id` as the API path ID.
- Avoid duplicate data loading and do not issue one detail request per rendered building. Fetch detail only for the currently selected building.
- The optional region endpoints are `GET /api/v1/regions` and `GET /api/v1/regions/{region_id}`; use them only if the panel needs region metadata not already present.

### Updated verification checklist

- [ ] Confirm the actual P1-09 entity property name and UUID extraction path in the implemented frontend before coding.
- [ ] Confirm API base URL/configuration and response unwrapping in the existing API client.
- [ ] Verify success, invalid UUID (400), not found (404), nullable fields and network failure.
- [ ] Verify rapid selection does not allow stale requests to overwrite the current selected building.
- [ ] Verify panel and Cesium highlight are both keyed by the same UUID.

**Dependency note:** P1-09 and P1-08 are merged prerequisites. P1-10 must integrate with their existing outputs and must not recreate the building list/detail APIs or the Cesium data-loading pipeline.
