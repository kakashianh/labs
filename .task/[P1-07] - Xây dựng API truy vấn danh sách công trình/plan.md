# KAN-15 – Xây dựng API truy vấn danh sách công trình

## 1. Objective

Xây dựng API Backend bằng Rust + Axum để truy vấn danh sách công trình đã được nạp vào PostgreSQL/PostGIS từ KAN-14.

API cung cấp dữ liệu công trình theo định dạng ổn định, có phân trang và bộ lọc cần thiết để Frontend có thể tải dữ liệu theo khu vực thay vì lấy toàn bộ dataset trong một request.

**Expected result:**
- Có endpoint truy vấn danh sách công trình.
- Hỗ trợ phân trang và lọc theo khu vực.
- Response có thông tin định danh, thuộc tính cần hiển thị và geometry ở định dạng thống nhất.
- Có validation, xử lý lỗi và kiểm thử API.
- API chỉ truy vấn dữ liệu GIS nền, chưa chứa logic mô phỏng dịch tễ.

## 2. Scope

### In scope
- Kiểm tra schema PostGIS và dữ liệu thực tế từ KAN-13/KAN-14.
- Thiết kế response DTO cho danh sách công trình.
- Xây dựng repository/query truy vấn PostgreSQL/PostGIS.
- Xây dựng endpoint danh sách công trình.
- Hỗ trợ pagination và filter theo region.
- Chuyển geometry sang GeoJSON hoặc định dạng không gian thống nhất cho frontend.
- Validation query parameters và xử lý lỗi.
- Unit/integration tests cho API.

### Out of scope
- API danh sách khu vực và chi tiết khu vực (KAN-7).
- API chi tiết một công trình (KAN-7).
- Hiển thị dữ liệu trên CesiumJS (KAN-16).
- Mô phỏng SIR hoặc trạng thái dịch tễ.
- Tính toán khoảng cách/lân cận giữa các công trình.
- Authentication/authorization nếu chưa nằm trong phạm vi MVP.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Backend | Rust | Ngôn ngữ triển khai |
| Web Framework | Axum | HTTP routing và handlers |
| Database | PostgreSQL + PostGIS | Lưu và truy vấn dữ liệu GIS |
| Database Access | Theo thư viện hiện có trong repository | Query và mapping dữ liệu |
| Spatial Format | GeoJSON | Truyền geometry tới Frontend |
| Testing | Rust test framework + HTTP integration tests | Kiểm thử |

Không đưa Python service/backend vào kiến trúc. Ưu tiên tái sử dụng cấu trúc config, database connection, error handling và response convention đã có trong Backend.

### KAN-14 database contract (verified)

Nguồn dữ liệu đích là Supabase PostgreSQL project đã được nạp và nghiệm thu ở KAN-14; không dùng lại GeoJSON làm runtime source.

| Table/column | Verified definition | API implication |
|---|---|---|
| `public.buildings.id` | UUID primary key | Trả về dưới dạng string; dùng làm `building_id` trong API |
| `region_id` | UUID FK tới `public.regions.id` | Filter bằng UUID, không dùng integer |
| `external_id` | Unique, mã OSM ổn định | Có thể expose như mã nguồn ngoài; không thay thế `id` |
| `name` | Nullable; 326/431 bản ghi đang NULL | DTO và UI phải chấp nhận null |
| `building_type` | Thuộc tính phân loại được map từ GeoJSON `building` | Không tự đổi tên thành `usage` nếu không có mapping rõ ràng |
| `footprint` | `geometry(MultiPolygon, 4326)` | Serialize thành GeoJSON MultiPolygon WGS84 |
| OSM attributes | Các cột nguồn OSM đã lưu theo migration KAN-13 | Chỉ expose các trường thực sự cần cho UI |

Snapshot hiện được nạp có 431 buildings và 1 region. Đây là baseline kiểm thử dữ liệu mẫu, không phải giới hạn cứng của API.

## 4. Implementation Plan

### Step 1 – Inspect Existing Backend and GIS Schema

Đọc code Backend và migration/schema đã được tạo ở KAN-13, đồng thời kiểm tra dữ liệu thực tế từ KAN-14.

- [ ] Kiểm tra cấu trúc thư mục Backend và cách khai báo Axum routes.
- [ ] Kiểm tra database connection/pool và repository conventions.
- [ ] Kiểm tra tên bảng, primary key, foreign key và các cột thực tế của buildings.
- [ ] Xác định geometry type và SRID đang được lưu.
- [ ] Xác định các thuộc tính công trình thực sự có trong dataset.
- [ ] Không tự giả định các field như height, floors, usage nếu dataset không có.

### Step 2 – Define Building List API Contract

**Schema-specific decisions:** `region_id` phải là UUID; `id` là UUID; `name` có thể null; geometry trả về phải là GeoJSON `MultiPolygon` với tọa độ longitude/latitude theo EPSG:4326. Response DTO không được ép `name` thành non-null.

Thiết kế endpoint theo convention hiện có của project.

Endpoint đề xuất:

~~~http
GET /api/v1/buildings
~~~

Query parameters tối thiểu:

| Parameter | Type | Required | Description |
|---|---|---|---|
| page | integer | No | Trang, mặc định 1 |
| page_size | integer | No | Số record/trang, có giới hạn tối đa |
| region_id | string/integer | No | Lọc công trình theo khu vực, theo kiểu ID thực tế |

Có thể bổ sung filter không gian/bounding box nếu cần cho cách tải dữ liệu của Frontend, nhưng chỉ triển khai khi phù hợp với dataset và được thống nhất trong codebase.

Response đề xuất:

~~~json
{
  "data": [
    {
      "id": "building-id",
      "region_id": "region-id",
      "properties": {},
      "geometry": {
        "type": "Polygon",
        "coordinates": []
      }
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 100,
    "total": 250,
    "total_pages": 3
  }
}
~~~

Đây là cấu trúc minh họa. Kiểu ID, properties và metadata phải khớp schema thực tế. Không trả toàn bộ cột nội bộ của database nếu Frontend không sử dụng.

- [ ] Chốt endpoint và query parameters.
- [ ] Chốt giới hạn page_size và giá trị mặc định.
- [ ] Chốt response DTO và quy tắc serialize geometry.
- [ ] Thống nhất cách trả lỗi/validation theo convention Backend.
- [ ] Ghi rõ semantics của total và total_pages.

### Step 3 – Implement Database Query and Repository

Tạo query lấy danh sách công trình từ PostGIS.

Yêu cầu:
- Chỉ select các cột cần thiết.
- Hỗ trợ filter region_id khi được truyền vào.
- Pagination ổn định bằng LIMIT/OFFSET hoặc cơ chế phù hợp với repository hiện tại.
- Có ORDER BY xác định để kết quả giữa các trang ổn định.
- Serialize `footprint` sang GeoJSON bằng PostGIS (ví dụ `ST_AsGeoJSON(footprint)`) hoặc mapping tương đương. PostGIS extension của Supabase đang nằm trong schema `extensions`; kiểm tra `search_path`/schema qualification khi gọi spatial functions.
- Tính tổng số record phù hợp với filter để trả pagination metadata.

- [ ] Tạo repository/query cho danh sách buildings.
- [ ] Implement filter theo region_id.
- [ ] Implement pagination và ordering ổn định.
- [ ] Chuyển geometry sang GeoJSON.
- [ ] Implement query đếm tổng record theo filter.
- [ ] Xử lý trường hợp geometry null/không hợp lệ theo quy tắc dữ liệu đã thống nhất.
- [ ] Tránh N+1 query khi lấy danh sách.

### Step 4 – Implement Axum Handler and Route

Tạo handler nhận query parameters, validate input và gọi repository.

- [ ] Tạo request/query struct.
- [ ] Validate page >= 1.
- [ ] Validate page_size > 0 và không vượt giới hạn.
- [ ] Validate `region_id` theo UUID (không nhận integer).
- [ ] Tạo handler truy vấn danh sách công trình.
- [ ] Đăng ký route GET /api/v1/buildings.
- [ ] Trả response theo API contract đã chốt.
- [ ] Mapping lỗi database và lỗi input thành HTTP status phù hợp.
- [ ] Không để lộ SQL, credentials hoặc thông tin nội bộ trong error response.

### Step 5 – Add Tests

Kiểm thử cả logic query và HTTP endpoint.

- [ ] Test danh sách công trình không truyền filter.
- [ ] Test pagination và page_size boundary.
- [ ] Test filter theo UUID `region_id` hợp lệ.
- [ ] Test `name = NULL` vẫn serialize response thành công.
- [ ] Test geometry `MultiPolygon` EPSG:4326 được trả đúng GeoJSON.
- [ ] Test region_id không tồn tại hoặc sai định dạng.
- [ ] Test page/page_size không hợp lệ.
- [ ] Test response geometry có GeoJSON hợp lệ.
- [ ] Test pagination metadata khớp số record.
- [ ] Test empty result trả response hợp lệ.
- [ ] Test database error được xử lý an toàn.

### Step 6 – Document and Verify API

- [ ] Cập nhật API documentation/OpenAPI nếu project đang sử dụng.
- [ ] Thêm ví dụ request/response.
- [ ] Hướng dẫn cách chạy Backend và database để kiểm thử.
- [ ] Chạy formatter và linter theo cấu hình Rust hiện có.
- [ ] Chạy toàn bộ test liên quan.
- [ ] Kiểm tra endpoint bằng HTTP client.
- [ ] Xác nhận response đủ dữ liệu để KAN-16 tích hợp.

### Database access and security

- Backend kết nối Supabase bằng connection string/secret ở server-side environment; không đưa DB password/service-role key vào Frontend.
- Các bảng `public.regions` và `public.buildings` đang bật RLS và chưa cấp quyền đọc rộng cho `anon`/`authenticated`. Không tắt RLS hoặc tạo policy public chỉ để API hoạt động.
- Chọn role/credential Backend có quyền tối thiểu cần thiết; giữ secret ngoài source code và log.
- [ ] Kiểm thử truy vấn qua đúng role mà Backend sẽ sử dụng.

## 5. Acceptance Criteria

- [ ] Endpoint GET /api/v1/buildings hoạt động.
- [ ] Trả danh sách công trình từ PostGIS.
- [ ] Hỗ trợ pagination với giới hạn page_size.
- [ ] Hỗ trợ filter region_id.
- [ ] Geometry được serialize theo định dạng thống nhất, ưu tiên GeoJSON.
- [ ] Response không phụ thuộc vào các field không tồn tại trong dataset.
- [ ] Input sai được trả lỗi phù hợp.
- [ ] Empty result được xử lý đúng.
- [ ] Có tests cho filter, pagination, geometry và lỗi.
- [ ] API documentation hoặc hướng dẫn request/response được cập nhật.

## 6. Deliverables

1. Endpoint truy vấn danh sách công trình.
2. Repository/query và DTO liên quan.
3. Validation và error handling.
4. Tests cho API.
5. Tài liệu API request/response.

## 7. Suggested Commit Sequence

~~~text
feat(api): define building list response contract

feat(api): implement PostGIS building list query

feat(api): add paginated building list endpoint

test(api): cover building filters pagination and geometry

docs(api): document building list endpoint
~~~

## 8. Final Definition of Done

KAN-15 hoàn thành khi Frontend có thể gọi API để lấy danh sách công trình theo trang và theo khu vực, nhận được thuộc tính cùng geometry ở định dạng thống nhất, và các trường hợp hợp lệ/không hợp lệ đều được kiểm thử.

Ticket này chỉ cung cấp API danh sách công trình; không triển khai API chi tiết công trình/khu vực hoặc tích hợp CesiumJS.
