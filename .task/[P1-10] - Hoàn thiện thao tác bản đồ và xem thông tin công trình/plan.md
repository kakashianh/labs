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
- [x] Xác nhận entity lưu building ID ổn định tại `entity.properties.buildingId` (P1-09 `buildingsToFeatureCollection` → GeoJSON properties → Cesium entity properties).
- [ ] Kiểm tra endpoint GET /api/v1/buildings/{building_id} của P1-08.
- [ ] Kiểm tra response DTO và các thuộc tính thực tế có thể hiển thị.
- [x] Kiểm tra và tái sử dụng API client/service hiện có (`getJson`, `API_BASE_URL`, `ApiRequestError`).
- [ ] Xác định quy ước click vào vùng trống để bỏ chọn.

### Step 2 – Implement Selected Building State

Quản lý trạng thái lựa chọn ở React thay vì để panel phụ thuộc trực tiếp vào Cesium internals.

State đề xuất:
- selectedBuildingId: ID công trình đang được chọn hoặc null.
- buildingDetail: dữ liệu chi tiết hiện tại.
- detailLoading: trạng thái tải.
- detailError: lỗi tải thông tin nếu có.

- [x] Tạo state selectedBuildingId.
- [x] Tạo state dữ liệu detail và loading/error.
- [x] Khi chọn entity có building ID hợp lệ, cập nhật selectedBuildingId; không dùng `entity.id` làm fallback.
- [x] Khi click entity không phải công trình, không tạo selected building giả và giữ nguyên selection hiện tại.
- [x] Khi click vùng trống, clear selectedBuildingId và khôi phục highlight.
- [x] Tránh response cũ ghi đè dữ liệu khi người dùng chọn liên tiếp nhiều công trình bằng AbortController, đồng thời chỉ nhận detail có UUID khớp selection.

### Step 3 – Fetch Building Detail

- [x] Tái sử dụng API client của Frontend.
- [x] Gọi GET /api/v1/buildings/{building_id} khi selectedBuildingId thay đổi.
- [x] Hủy request trước đó khi selection thay đổi hoặc bỏ chọn bằng AbortController.
- [x] Reset detail state phù hợp khi bỏ chọn.
- [x] Xử lý 404 khi công trình không còn tồn tại.
- [x] Xử lý network/API error.
- [x] Không gọi API detail nếu entity không có building ID UUID hợp lệ.

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

- [x] Tạo BuildingInfoPanel component.
- [x] Render thông tin từ API response.
- [x] Hiển thị loading state.
- [x] Hiển thị error/not-found state.
- [x] Xử lý thuộc tính thiếu hoặc null.
- [x] Thêm thao tác đóng panel và đồng bộ clear selection.
- [x] Bổ sung CSS responsive cho panel; kiểm tra trực quan trên trình duyệt vẫn cần thực hiện.

### Step 5 – Highlight and Synchronize Selected Entity

- [x] Highlight entity đang được chọn.
- [x] Khôi phục style entity trước đó khi chọn công trình khác.
- [x] Khôi phục style mặc định khi bỏ chọn.
- [x] Đảm bảo selected state và panel luôn cùng building ID.
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

**Dependency note:** P1-08 đã được merge vào `project/3D-map-medical`. P1-09 hiện có implementation trên branch `feat/p1-09-cesium-building-integration`; trạng thái merge cần được xác nhận trước khi bắt đầu tích hợp P1-10. P1-10 chỉ bắt đầu tích hợp khi P1-09 đã hoàn tất implementation, CI liên quan pass và thay đổi đã được merge vào branch tích hợp. Không xem P1-09 là merged chỉ dựa trên việc đã có branch hoặc đã kiểm tra trực quan.


## 11. Detailed interaction rules and implementation safeguards (approved 2026-11-14)

### 11.1 Dependency gate and ownership

- [ ] Confirm P1-08 is available on the integration branch (already merged as of this plan update).
- [ ] Confirm P1-09 implementation is merged into `project/3D-map-medical` and its required CI checks pass before integrating P1-10.
- [ ] Re-read the actual P1-09 implementation at the time P1-10 starts; do not rely on assumed entity property names or earlier illustrative snippets.
- P1-08 owns backend region/building APIs and response DTOs.
- P1-09 owns fetching the paginated building collection, converting GeoJSON and creating Cesium entities with stable building UUID metadata; its existing basic selection behavior may be reused.
- P1-10 owns the React selected-building state, selected-detail request lifecycle, information panel, and synchronized highlight/clear behavior.
- P1-11 owns end-to-end acceptance across the complete map workflow.

### 11.2 Resolve the real entity ID contract before implementation

- [ ] Inspect the P1-09 source and identify the exact Cesium entity property key that stores the building UUID.
- [ ] Trace the value from the API DTO through GeoJSON properties to the picked Cesium entity.
- [ ] Add or update a focused test proving that the picked entity yields the original API `id` UUID.
- [ ] Use that UUID as the only path identifier for `GET /api/v1/buildings/{building_id}`; never derive it from display name, array index, or `external_id`.
- [ ] Confirm the existing API client base URL, error handling, and `response.data` unwrapping before adding a new service.

### 11.3 Explicit map and panel interaction behavior

| User action | Expected behavior |
|---|---|
| Click a building A | Select A, highlight A, clear any previous detail, and fetch A detail |
| Click building B while A is selected | Restore A's default style, select/highlight B, and fetch B detail |
| Click empty map space | Clear selected ID, cancel/invalidate pending detail request, restore highlight, and close/clear panel |
| Click a non-building entity | Do not create a building selection or issue a building detail request; preserve the current building selection unless the product behavior explicitly defines otherwise |
| Click panel content | Interact with the panel without triggering Cesium map picking |
| Close panel | Clear selected ID, invalidate pending request, and restore the selected entity's default style |
| Selected entity is no longer in the data source | Clear selection safely and ensure the panel does not present stale entity state |

The implementation must keep the selected UUID, panel detail UUID, and highlighted entity UUID consistent. The panel must not intercept map input outside its own bounds, and clicks inside the panel must not bubble into the Cesium picking handler.

### 11.4 Request lifecycle and stale-response protection

- [ ] When selected ID changes, immediately clear or mark the previous detail as non-current so A's information is not displayed as if it belonged to B.
- [ ] Abort the previous request with `AbortController` where supported, and also guard state updates using the current selected UUID/request identity.
- [ ] Apply a response only if its UUID still matches the current `selectedBuildingId`.
- [ ] On deselection or panel close, abort/invalidate the pending request and clear detail/loading/error state.
- [ ] Ensure a late success or failure from an older request cannot overwrite the latest selection or reopen a cleared panel.

### 11.5 Expanded test scenarios

- [ ] Select a valid building entity and assert the exact UUID is passed to the detail API.
- [ ] Select A then B; assert A's highlight is restored, B is highlighted, and only B's detail is shown.
- [ ] Select A → B → A rapidly; assert only the latest A request result is applied.
- [ ] Deselect while a detail request is pending; assert the panel remains closed after the request resolves.
- [ ] Resolve requests out of order; assert stale success and stale error responses are ignored.
- [ ] Click a non-building entity; assert no building detail request is made.
- [ ] Click inside the panel; assert Cesium picking does not clear or change the selection.
- [ ] Close the panel; assert selection is cleared and highlight is restored.
- [ ] Verify 400 invalid UUID, 404 missing building, network/server error, and nullable `name`/`building_type` fields.
- [ ] Verify panel interactions do not unexpectedly trigger camera movement or prevent map navigation outside the panel.
- [ ] Verify an entity removed from the data source cannot leave a stale highlight or stale detail panel.

### 11.6 Acceptance gate before marking KAN-17 done

KAN-17 can be marked Done only after:
1. P1-09 is merged and its CI gate is green.
2. The real entity UUID property path is verified against P1-09 source and covered by a test.
3. Selection, switching, deselection, close-panel, and highlight restoration behaviors pass.
4. Detail API success, 400, 404, nullable fields, and network failure behaviors pass.
5. Stale-response and pending-request cancellation/invalidation tests pass.
6. TypeScript checks, unit tests, and production build pass.
7. The relevant manual browser checks are recorded with their actual results.

Do not mark checklist items complete based only on planned behavior; check them off only after implementation or verification evidence exists.
