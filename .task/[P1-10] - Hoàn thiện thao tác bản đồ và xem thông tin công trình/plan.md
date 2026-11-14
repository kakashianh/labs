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

- [x] Kiểm tra Cesium entity properties được gắn ở P1-09.
- [x] Xác nhận entity lưu building ID ổn định tại `entity.properties.buildingId` (P1-09 `buildingsToFeatureCollection` → GeoJSON properties → Cesium entity properties).
- [x] Kiểm tra endpoint GET /api/v1/buildings/{building_id} của P1-08.
- [x] Kiểm tra response DTO và các thuộc tính thực tế có thể hiển thị.
- [x] Kiểm tra và tái sử dụng API client/service hiện có (`getJson`, `API_BASE_URL`, `ApiRequestError`).
- [x] Xác định và triển khai click vùng trống để bỏ chọn.

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
- [x] Không thay đổi geometry/dữ liệu gốc; chỉ lưu và thay đổi tạm thời polygon material để highlight.
- [ ] Xử lý an toàn khi entity không còn trong data source.

### Step 6 – Verify Map Navigation and Interaction

- [x] Kiểm tra zoom bằng chuột/scroll — người dùng xác nhận đạt.
- [x] Kiểm tra pan và rotate/tilt — người dùng xác nhận đạt.
- [x] Kiểm tra click chọn công trình sau khi camera đã di chuyển — người dùng xác nhận đạt.
- [x] Kiểm tra thao tác bản đồ khi panel đang mở — người dùng xác nhận đạt.
- [x] Kiểm tra click công trình A rồi công trình B — người dùng xác nhận đạt.
- [x] Kiểm tra click vùng trống để bỏ chọn — người dùng xác nhận đạt.
- [x] Kiểm tra đóng panel có đồng bộ selected state — người dùng xác nhận đạt.
- [x] Kiểm tra rapid selection không hiển thị nhầm detail response cũ — người dùng xác nhận đạt.

### Step 7 – Add Tests and Documentation

- [x] Test selected state khi chọn entity hợp lệ — người dùng xác nhận đạt qua kiểm thử trình duyệt.
- [x] Test bỏ chọn khi click vùng trống — người dùng xác nhận đạt qua kiểm thử trình duyệt.
- [x] Test chuyển selection giữa hai công trình — người dùng xác nhận đạt qua kiểm thử trình duyệt.
- [x] Bổ sung unit tests cho detail success/envelope, nullable fields, HTTP 400/404 và network error; người dùng xác nhận loading/error/not-found UI hoạt động đúng khi kiểm thử trình duyệt.
- [x] Bổ sung unit coverage cho nullable API fields; người dùng xác nhận component rendering hoạt động đúng khi kiểm thử trình duyệt.
- [x] Bổ sung AbortSignal cancellation test; người dùng xác nhận rapid-selection và stale-response guard hoạt động đúng khi kiểm thử trình duyệt.
- [x] Cập nhật frontend README về chọn công trình, detail panel và hành vi clear selection.
- [x] Chạy frontend lint, unit tests, TypeScript checks và production build (CI run #43576392119 — frontend job passed).

## 6. Acceptance Criteria

- [x] Zoom, pan, rotate và tilt hoạt động ổn định — người dùng xác nhận kiểm thử trình duyệt đạt.
- [x] Click công trình hiển thị đúng building ID và thông tin API cung cấp — người dùng xác nhận kiểm thử đạt.
- [x] Chọn công trình khác làm panel cập nhật đúng dữ liệu mới — người dùng xác nhận kiểm thử đạt.
- [x] Bỏ chọn làm đóng panel hoặc clear nội dung theo quy ước đã thống nhất — người dùng xác nhận kiểm thử đạt.
- [x] Selected highlight được cập nhật và khôi phục đúng — người dùng xác nhận kiểm thử đạt.
- [x] Thiếu thuộc tính không làm giao diện lỗi — unit tests và kiểm thử trình duyệt đạt.
- [x] Loading/error/not-found được xử lý — người dùng xác nhận kiểm thử trình duyệt đạt.
- [x] Response cũ không ghi đè thông tin của selection mới — người dùng xác nhận kiểm thử rapid-selection đạt.
- [x] Mở/đóng panel không làm mất khả năng điều hướng bản đồ — người dùng xác nhận kiểm thử đạt.
- [x] Frontend build và các tests liên quan thành công (CI run #43576392119: frontend lint, unit tests, TypeScript và Vite build đều pass).

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

**Dependency note (updated 2026-11-14):** P1-08 và P1-09 đã được merge vào `project/3D-map-medical`. P1-09 được merge qua PR #12 (merge commit `83d48b01133bc5f8c5536d27144edf7771df75c3`). P1-10 được triển khai trên branch `feat/p1-10-building-info-panel` từ branch tích hợp.


## 11. Detailed interaction rules and implementation safeguards (approved 2026-11-14)

### 11.1 Dependency gate and ownership

- [ ] Confirm P1-08 is available on the integration branch (already merged as of this plan update).
- [x] Confirm P1-09 implementation is merged into `project/3D-map-medical` via PR #12 before integrating P1-10.
- [x] Re-read the actual P1-09 implementation and verified the entity UUID property path before implementation.
- P1-08 owns backend region/building APIs and response DTOs.
- P1-09 owns fetching the paginated building collection, converting GeoJSON and creating Cesium entities with stable building UUID metadata; its existing basic selection behavior may be reused.
- P1-10 owns the React selected-building state, selected-detail request lifecycle, information panel, and synchronized highlight/clear behavior.
- P1-11 owns end-to-end acceptance across the complete map workflow.

### 11.2 Resolve the real entity ID contract before implementation

- [x] Inspect the P1-09 source and identify the exact Cesium entity property key that stores the building UUID (`buildingId`).
- [x] Trace the value from the API DTO through GeoJSON properties to the picked Cesium entity.
- [x] Verify through implementation review and browser selection testing that the picked entity yields the original API `id` UUID.
- [x] Use that UUID as the only path identifier for `GET /api/v1/buildings/{building_id}`; never derive it from display name, array index, or `external_id`.
- [x] Confirm the existing API client base URL/error handling and unwrap `response.data` in the detail service.

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

- [x] When selected ID changes, immediately clear the previous detail so A's information is not displayed as if it belonged to B.
- [x] Abort the previous request with `AbortController` and guard state updates using the requested/current UUID.
- [x] Apply a response only if its UUID still matches the requested/current `selectedBuildingId`.
- [x] On deselection or panel close, abort/invalidate the pending request and clear detail/loading/error state.
- [x] Ensure a late success or failure from an older request cannot overwrite the latest selection or reopen a cleared panel.

### 11.5 Expanded test scenarios

- [ ] Select a valid building entity and assert the exact UUID is passed to the detail API.
- [ ] Select A then B; assert A's highlight is restored, B is highlighted, and only B's detail is shown.
- [ ] Select A → B → A rapidly; assert only the latest A request result is applied.
- [ ] Deselect while a detail request is pending; assert the panel remains closed after the request resolves.
- [ ] Resolve requests out of order; assert stale success and stale error responses are ignored.
- [ ] Click a non-building entity; assert no building detail request is made.
- [ ] Click inside the panel; assert Cesium picking does not clear or change the selection.
- [ ] Close the panel; assert selection is cleared and highlight is restored.
- [x] Add service tests for 400 invalid UUID, 404 missing building, network error, and nullable `name`/`building_type` fields.
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


### Implementation progress — feature branch

Implementation is being carried out on `feat/p1-10-building-info-panel`, based on the merged P1-09 integration branch.

Implemented:
- Verified P1-09's real UUID path: `entity.properties.buildingId`; invalid/missing UUIDs are ignored and never fall back to the Cesium entity name or `external_id`.
- Added `fetchBuildingDetail`, which calls the P1-08 detail endpoint and unwraps the `data` envelope through the existing API client.
- Added the responsive `BuildingInfoPanel` with loading, error, not-found and nullable-field fallbacks.
- Added selection-driven detail loading with AbortController cleanup and a response UUID guard.
- Added selected-entity material restoration when switching, clearing or closing the panel.
- Added service tests for detail response unwrapping, nullable values, 400, 404, network errors and cancellation.

Still requires verification:
- [x] Run frontend lint, unit tests and production build in CI — workflow run #204 frontend job passed (Oxlint, 10/10 tests, `tsc -b`, `vite build`).
- [x] Browser-check A → B → A rapid selection and out-of-order response behavior — người dùng xác nhận đạt.
- [x] Browser-check click empty map, non-building entity, close panel, responsive panel layout and camera navigation — người dùng xác nhận đạt.
- [ ] Confirm no stale highlight remains if a selected entity is removed from its data source.
- [x] Cập nhật acceptance criteria sau khi người dùng xác nhận toàn bộ kiểm thử tương tác đạt.

Automated CI và các kiểm thử trình duyệt trong phạm vi P1-10 đã được xác nhận đạt. Trường hợp entity bị loại khỏi data source được giữ lại như một edge case riêng cần xác minh/xử lý trước khi tuyên bố toàn bộ vòng đời entity đã được bao phủ.
