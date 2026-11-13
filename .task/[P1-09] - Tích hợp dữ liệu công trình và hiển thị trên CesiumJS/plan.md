# KAN-16 – Tích hợp dữ liệu công trình và hiển thị trên CesiumJS

## 1. Objective

Tích hợp Frontend React + CesiumJS với các API GIS đã triển khai ở KAN-15 và KAN-7 để tải và hiển thị dữ liệu khu vực/công trình trên bản đồ 3D.

Mục tiêu của ticket là đưa dữ liệu công trình thực tế từ PostGIS lên CesiumJS, thay thế trạng thái chỉ hiển thị bản đồ nền ở KAN-11.

**Expected result:**
- Frontend tải được danh sách khu vực và công trình từ Backend.
- Geometry công trình được hiển thị đúng vị trí trên CesiumJS.
- Có thể phân biệt/chọn công trình trên bản đồ.
- Dữ liệu hiển thị khớp với response API và khu vực nghiên cứu.
- Có trạng thái loading, empty và error cơ bản.

## 2. Scope

### In scope
- Kiểm tra API contract của KAN-15 và KAN-7.
- Cấu hình API base URL cho Frontend.
- Xây dựng API client/service và TypeScript types.
- Tải dữ liệu khu vực/công trình.
- Chuyển GeoJSON thành Cesium data source/entities.
- Hiển thị geometry công trình trên CesiumJS.
- Zoom tới khu vực nghiên cứu hoặc extent dữ liệu.
- Xử lý chọn công trình ở mức cơ bản.
- Quản lý loading/error/empty state.
- Kiểm tra hiệu năng hiển thị với dataset mẫu.

### Out of scope
- Hoàn thiện toàn bộ thao tác bản đồ và panel thông tin (KAN-17).
- Mô phỏng SIR và hiển thị trạng thái dịch tễ.
- Màu sắc theo mức độ nguy cơ dịch bệnh.
- Mô hình 3D building mesh có độ cao thực nếu dataset chưa có dữ liệu tương ứng.
- Tối ưu streaming/tiling cho dữ liệu đô thị quy mô rất lớn.
- Phân tích spatial hoặc clustering trên Frontend.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| UI | React + TypeScript | Giao diện |
| 3D GIS | CesiumJS | Hiển thị geometry |
| Build Tool | Vite | Frontend build |
| HTTP | API client hiện có hoặc fetch wrapper | Gọi Backend |
| Backend API | Rust + Axum | Cung cấp dữ liệu |
| Database | PostgreSQL + PostGIS | Nguồn dữ liệu GIS |

Không tạo backend/service mới. Frontend chỉ gọi API Backend đã có; không kết nối trực tiếp Supabase/PostGIS từ trình duyệt.

### KAN-14 data contract (verified)

- Supabase chứa 1 region và 431 buildings trong snapshot đã nghiệm thu.
- `regions.id`, `buildings.id`, `buildings.region_id` là UUID.
- `regions.boundary` và `buildings.footprint` là `MultiPolygon`, EPSG:4326, 2D.
- `buildings.name` nullable (326/431 bản ghi thiếu tên); `building_type` là thuộc tính phân loại được map từ nguồn GeoJSON.
- Dataset không cung cấp height/floors đáng tin cậy để dựng extrusion; mặc định hiển thị footprint trên mặt đất.
- RLS đang bật trên các bảng Supabase; Frontend chỉ gọi Rust API, không dùng database credential/service-role key.

### Verified API contract after KAN-15 and KAN-7 merge

Prerequisites are now present on the default branch `project/3D-map-medical`: KAN-15/P1-07 (PR #10) and KAN-7/P1-08 (PR #11). Use the implementation/OpenAPI as the source of truth; do not recreate DTOs from assumptions.

| Endpoint | Response/use in P1-09 |
|---|---|
| `GET /api/v1/regions` | `{ data: RegionListItem[] }`; no pagination; ordered by `code ASC`. |
| `GET /api/v1/regions/{region_id}` | `{ data: RegionDetail }`; includes region boundary GeoJSON MultiPolygon. Call only if the viewer needs the boundary/extent. |
| `GET /api/v1/buildings` | `{ data: Building[], pagination: { page, page_size, total, total_pages } }`; supports `page`, `page_size` (default 100, maximum 500) and UUID `region_id` filter. |
| `GET /api/v1/buildings/{building_id}` | `{ data: BuildingDetail }`; available for a selected building lookup, but not required to render the building list because list items already contain geometry and properties. |

Building list items contain `id`, `region_id`, `external_id`, nullable `name`, nullable `building_type`, and `geometry` (GeoJSON MultiPolygon). Region list items contain `id`, `code`, `name`, and nullable `description`; geometry is only in Region Detail.

**Important integration rules:**
- Fetch buildings page-by-page until all pages indicated by `pagination.total_pages` are loaded; do not infer completeness from a single response or hard-code 431.
- For the current snapshot, `page_size=500` can fit 431 records in one page, but pagination logic must still work for larger datasets.
- API returns geometry as a GeoJSON Geometry nested in each DTO, not a GeoJSON FeatureCollection. Convert each building into a GeoJSON Feature (properties include at least `id`, `region_id`, `external_id`, `name`, `building_type`) and wrap them in a FeatureCollection before passing to `Cesium.GeoJsonDataSource`, or implement an equivalent explicit Cesium mapping.
- Keep `building.id` as the stable selection key. Do not use `external_id` as the internal identifier.
- Empty collections are valid; the regions endpoint returns `data: []`, and buildings returns an empty `data` array with pagination metadata.
- Do not call detail endpoints for every building (N+1 HTTP requests). Building detail is optional and belongs to a later selected-item flow if required.

## 4. Data Flow

~~~text
PostgreSQL / PostGIS
        |
        v
Rust + Axum API
        |
        +------------------------------+
        |                              |
        v                              v
GET /api/v1/regions          GET /api/v1/buildings
GET /api/v1/regions/{id}     GET /api/v1/buildings/{id}
        |                              |
        +---------------+--------------+
                        |
                        v
                Frontend API Client
                        |
                        v
                TypeScript Data Model
                        |
                        v
              GeoJSON -> Cesium DataSource
                        |
                        v
                 CesiumJS Viewer
                        |
                        v
             Building Footprints on Map
~~~

API paths và response contract đã được xác nhận từ Backend/OpenAPI trên default branch. Frontend chỉ sử dụng các endpoint được liệt kê ở mục 3; không hard-code contract khác với Backend.

## 5. Implementation Plan

### Step 1 – Inspect API Contract and Existing Cesium Setup

- [x] Kiểm tra CesiumViewer được tạo ở KAN-11.
- [x] Kiểm tra cách cấu hình API base URL trong Frontend; bổ sung `VITE_API_BASE_URL` với mặc định local `http://localhost:8000`.
- [ ] Kiểm tra response thực tế của GET /api/v1/regions.
- [ ] Kiểm tra response thực tế của GET /api/v1/buildings.
- [x] Xác nhận API trả geometry `MultiPolygon` từ `footprint`/`boundary`, SRID 4326 qua OpenAPI contract.
- [x] Xác nhận UUID được giữ dưới dạng string trong TypeScript; `name` được khai báo nullable.
- [x] Kiểm tra coordinate order longitude/latitude từ OpenAPI; không áp dụng phép đổi trục.
- [x] Kiểm tra pagination metadata và giới hạn `page_size` (tối đa 500).
- [x] Xác định và triển khai tải tuần tự tất cả trang theo `pagination.total_pages`.
- [x] Không giả định API trả toàn bộ công trình trong một response.

### Step 2 – Define Frontend GIS Types and API Client

Tạo TypeScript types phản ánh API contract.

Các kiểu dữ liệu cần thiết:
- Region list item.
- Building item.
- GeoJSON geometry/feature phù hợp với dữ liệu thực tế.
- Pagination response.
- API error response.

- [x] Tạo hoặc cập nhật thư mục src/types/ cho GIS types.
- [x] Tạo service gọi regions API.
- [x] Tạo service gọi buildings API.
- [x] Implement pagination fetching theo `pagination.total_pages`; hỗ trợ `page_size` và `region_id` filter.
- [x] Khai báo response types chính xác cho hai list endpoint đang dùng: Region list `{ data }`, Building list `{ data, pagination }`; detail endpoint không gọi ở P1-09 để tránh N+1 và thuộc luồng P1-10.
- [x] Cấu hình API base URL qua environment variable `VITE_API_BASE_URL`.
- [x] Xử lý HTTP error và parse response.
- [x] Tránh `any` trong phần mapping dữ liệu.

### Step 3 – Implement GIS Data Loading Flow

Tạo luồng tải dữ liệu tách khỏi component Cesium Viewer.

- [x] Tải danh sách khu vực khi khởi tạo màn hình bản đồ.
- [x] Tải danh sách công trình theo pagination; không áp dụng filter mặc định để hiển thị toàn bộ AOI.
- [x] Xử lý trạng thái loading.
- [x] Xử lý response rỗng.
- [x] Xử lý API timeout/network error.
- [x] Tránh gọi API lặp lại không cần thiết khi component re-render; request chỉ khởi tạo theo lifecycle của Viewer.
- [x] Đảm bảo component có thể cleanup khi request không còn cần thiết bằng `AbortController`.

### Step 4 – Convert GeoJSON to Cesium Data Source

Sử dụng khả năng load GeoJSON của CesiumJS hoặc cơ chế mapping tương ứng.

Yêu cầu:
- Giữ nguyên định danh công trình để có thể truy ngược khi click.
- Giữ các properties cần thiết cho popup ở KAN-17.
- Kiểm tra coordinate order và CRS; GeoJSON theo chuẩn sử dụng longitude/latitude (WGS84) khi truyền tới Cesium.
- Nếu dữ liệu PostGIS lưu ở CRS khác, Backend phải trả GeoJSON ở CRS phù hợp hoặc Frontend phải chuyển đổi có chủ đích; không được hiển thị sai vị trí do nhầm SRID.
- Chỉ dùng height/extrusion khi thuộc tính độ cao thực sự tồn tại và có đơn vị rõ ràng.
- Nếu không có height, hiển thị footprint dạng polygon trên mặt đất, không tự bịa chiều cao công trình.

- [x] Chuyển danh sách DTO thành GeoJSON FeatureCollection (geometry lấy từ DTO.geometry; properties giữ UUID và thuộc tính cần thiết).
- [x] Load FeatureCollection bằng `Cesium.GeoJsonDataSource`; geometry sai cấu trúc/empty được bỏ qua trước khi load.
- [x] Gắn building ID và properties vào Feature để Cesium giữ lại trên entity.
- [x] Cấu hình style mặc định cho footprint.
- [x] Kiểm tra `MultiPolygon` (kiểu geometry đã xác nhận của KAN-14) và bỏ qua feature có cấu trúc không hợp lệ.
- [x] Kiểm tra geometry null/empty được bỏ qua hoặc xử lý an toàn.
- [x] Không tạo building extrusion từ dữ liệu không có căn cứ.

### Step 5 – Integrate Data Source into Cesium Viewer

- [x] Tích hợp data loading flow vào trang bản đồ.
- [x] Add/remove data source đúng vòng đời component.
- [x] Tránh add trùng entities khi tải lại dữ liệu bằng một data source trong một Viewer lifecycle.
- [x] Zoom camera tới extent dữ liệu sau khi tải thành công.
- [x] Giữ thao tác camera cơ bản của Cesium Viewer.
- [x] Đảm bảo Viewer cleanup không để lại event handler/data source cũ.
- [x] Đảm bảo tải lại trang không tạo nhiều lớp công trình trùng nhau qua cleanup Viewer; cần xác nhận trực quan khi chạy app.

### Step 6 – Implement Basic Building Selection

Phạm vi chỉ cần xác nhận người dùng có thể chọn một công trình trên bản đồ; panel và trải nghiệm tương tác đầy đủ thuộc KAN-17.

- [x] Đăng ký click handler cho Cesium Viewer.
- [x] Xác định entity công trình được click.
- [x] Lấy building ID từ entity properties.
- [x] Lưu selected building ID trong React state.
- [x] Highlight công trình được chọn ở mức cơ bản.
- [x] Cleanup `ScreenSpaceEventHandler` khi component unmount.
- [x] Không gọi building detail API hoặc xây dựng panel chi tiết; phần này thuộc P1-10.

### Step 7 – Verify Data Accuracy and Performance

- [ ] Đối chiếu số công trình nhận từ API với số entity được hiển thị (tính đến pagination/filter); với snapshot hiện tại kỳ vọng 431 nếu không áp dụng filter.
- [ ] Kiểm tra vị trí công trình trên đúng khu vực nghiên cứu.
- [ ] Kiểm tra một số geometry đại diện bằng cách đối chiếu PostGIS/GeoJSON.
- [ ] Kiểm tra click chọn công trình trả đúng ID.
- [ ] Kiểm tra trường hợp API trả empty list.
- [ ] Kiểm tra trường hợp Backend không khả dụng.
- [ ] Kiểm tra thời gian tải và khả năng tương tác trên dataset mẫu.
- [ ] Nếu dataset quá lớn cho một lần render, ghi nhận giới hạn và chỉ áp dụng giải pháp tải phù hợp phạm vi MVP; không tự mở rộng thành 3D Tiles pipeline.

### Step 8 – Document and Final Verification

- [x] Cập nhật hướng dẫn cấu hình API URL.
- [x] Ghi rõ cách chạy Frontend và Backend đồng thời.
- [x] Ghi nhận API endpoints mà Frontend sử dụng.
- [x] Chạy TypeScript checks và production build (GitHub Actions frontend job).
- [ ] Kiểm tra browser console không có lỗi Cesium/API nghiêm trọng.
- [ ] Kiểm tra luồng end-to-end từ PostGIS đến Cesium Viewer.

### Test strategy

- Unit test API response mapping and TypeScript DTO types where practical.
- Unit test pagination aggregation (multiple pages, empty result, final partial page) without depending on the current 431-record snapshot.
- Unit test GeoJSON Geometry-to-FeatureCollection conversion and preservation of `building.id`/properties.
- Verify Cesium entity selection returns the original UUID string.
- Keep live Backend/PostGIS as the source of data in manual end-to-end verification; do not add Supabase browser access or embed credentials.

### Data access and privacy constraints

- Frontend chỉ gọi Rust + Axum API; không truy cập trực tiếp `public.buildings`/`public.regions`.
- Không đưa database password hoặc Supabase service-role key vào Vite environment variables được bundle cho browser.
- `name` null không làm lỗi TypeScript mapping, label hoặc selection.

## 6. Acceptance Criteria

- [ ] Frontend gọi thành công API regions và buildings.
- [ ] Có thể tải đủ công trình qua pagination dựa trên `total_pages`; không phụ thuộc page_size hiện tại.
- [ ] API DTO geometry được chuyển thành GeoJSON FeatureCollection/ Cesium entities đúng contract.
- [ ] Geometry công trình được hiển thị trên đúng vị trí bản đồ.
- [ ] Các properties/ID được giữ lại trong Cesium entity.
- [ ] Người dùng có thể click chọn một công trình.
- [ ] Không hiển thị sai vị trí do CRS/coordinate order.
- [ ] Không tự tạo chiều cao 3D khi dataset không cung cấp dữ liệu height hợp lệ.
- [x] Loading, empty và error state được xử lý.
- [x] Không tạo entities/data sources trùng khi component re-render hoặc tải lại.
- [x] Frontend production build thành công (GitHub Actions frontend job).

## 7. Deliverables

1. GIS TypeScript types.
2. Frontend API client/services cho regions và buildings.
3. Luồng tải dữ liệu GIS từ Backend.
4. Cesium data source hiển thị footprint công trình.
5. Cơ chế chọn công trình cơ bản.
6. Hướng dẫn cấu hình và chạy tích hợp.

## 8. Suggested Commit Sequence

~~~text
feat(frontend): define GIS API types and client

feat(frontend): load regions and paginated buildings

feat(map): convert building GeoJSON into Cesium entities

feat(map): render building footprints on Cesium viewer

feat(map): support basic building selection

test(frontend): verify GIS data rendering and API states

docs(frontend): document GIS map integration
~~~

## 9. Final Definition of Done

KAN-16 hoàn thành khi dữ liệu khu vực/công trình từ PostGIS đi qua Rust + Axum API và được Frontend tải, chuyển thành Cesium entities, hiển thị đúng vị trí trên bản đồ; người dùng có thể chọn công trình và các trạng thái loading/error/empty được xử lý.

Ticket này hoàn thiện tích hợp dữ liệu GIS lên bản đồ; chưa triển khai panel thông tin và thao tác bản đồ đầy đủ của KAN-17, cũng chưa hiển thị bất kỳ trạng thái SIR nào.
