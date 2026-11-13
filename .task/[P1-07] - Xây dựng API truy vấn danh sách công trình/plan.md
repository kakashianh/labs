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
| region_id | UUID string | No | Lọc công trình theo public.regions.id; UUID không hợp lệ trả HTTP 400 |

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
        "type": "MultiPolygon",
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

Đây là cấu trúc minh họa. DTO đề xuất gồm id, region_id, external_id, name (nullable), building_type và geometry (MultiPolygon). Không trả toàn bộ cột nội bộ của database nếu Frontend không sử dụng. page mặc định 1; page_size mặc định 100, tối đa 500 (có thể điều chỉnh theo convention hiện có). total là số record sau filter; total_pages = ceil(total / page_size), bằng 0 khi không có record. Page vượt total_pages trả HTTP 200 với data: [] và metadata hiện tại, không trả 404.

- [ ] Chốt endpoint và query parameters.
- [ ] Chốt giới hạn page_size và giá trị mặc định (page=1, page_size=100, max page_size=500, trừ khi codebase đã có convention khác).
- [ ] Chốt semantics: total tính sau filter; total_pages=0 khi total=0; page vượt phạm vi trả HTTP 200 với data=[] và pagination metadata, không trả 404.
- [ ] Chốt response DTO và quy tắc serialize geometry.
- [ ] Thống nhất cách trả lỗi/validation theo convention Backend.
- [ ] Ghi rõ semantics của total và total_pages.

### Step 3 – Implement Database Query and Repository

Tạo query lấy danh sách công trình từ PostGIS.

Yêu cầu:
- Chỉ select các cột cần thiết.
- Hỗ trợ filter region_id khi được truyền vào.
- Pagination bằng LIMIT/OFFSET hoặc cơ chế phù hợp với repository hiện tại.
- ORDER BY id ASC (UUID PK duy nhất) hoặc ordering deterministic tương đương; không phân trang trên thứ tự không xác định.
- Serialize `footprint` sang GeoJSON bằng PostGIS (ví dụ `ST_AsGeoJSON(footprint)`) hoặc mapping tương đương. PostGIS extension của Supabase đang nằm trong schema `extensions`; kiểm tra `search_path`/schema qualification khi gọi spatial functions.
- Tính tổng số record phù hợp với filter để trả pagination metadata.

- [ ] Tạo repository/query cho danh sách buildings.
- [ ] Implement filter theo region_id.
- [ ] Implement pagination và ordering ổn định bằng unique key (id) để tránh record nhảy trang do thứ tự không xác định.
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

### Step 5 – Add Unit and Integration Tests

Tách rõ Unit Test và Integration Test để CI kiểm tra cả logic độc lập lẫn hành vi thực tế với PostgreSQL/PostGIS. Không dùng mock database để thay thế Integration Test.

#### 5.1 Unit Tests (không cần database/container)

- [ ] Test parse/validation query parameters: page, page_size, UUID region_id.
- [ ] Test defaults (page=1, page_size=100), page/page_size không hợp lệ và page_size vượt max 500.
- [ ] Test tính OFFSET, pagination metadata, total_pages=0 khi dataset rỗng.
- [ ] Test mapping database model → response DTO, đặc biệt name = NULL.
- [ ] Test mapping lỗi ứng dụng/database sang HTTP status và error response; không leak SQL/credentials.
- [ ] Test serialization GeoJSON/DTO với geometry MultiPolygon (nếu có logic serialize riêng trong ứng dụng).
- [ ] Không khởi tạo network, database hoặc Docker trong Unit Test.

#### 5.2 Integration Tests (PostgreSQL/PostGIS bằng Testcontainers)

Dùng crate testcontainers (Rust) hoặc API tương thích với codebase để khởi chạy PostGIS thực trong test. Chọn image tương thích PostgreSQL 17 (cùng major version với Supabase target KAN-14); pin tag cụ thể, không dùng latest.

- [ ] Tạo test harness khởi động PostGIS container, chờ database ready rồi tạo connection pool.
- [ ] Dùng database riêng trong container; chạy migrations/schema setup của ứng dụng (hoặc fixture SQL version-control) để test schema tương thích contract KAN-14.
- [ ] Seed fixture deterministic: ít nhất 2 region, nhiều buildings mỗi region, có name NULL và geometry MultiPolygon SRID 4326; không phụ thuộc dataset 431 record hoặc Supabase production.
- [ ] Test repository query: list không filter, filter theo region UUID, ordering ổn định, pagination và count theo filter.
- [ ] Test PostGIS ST_AsGeoJSON/spatial function trong container; xử lý schema extension/search_path tương thích môi trường test và Supabase.
- [ ] Test HTTP route qua Axum router/service: status, JSON fields, UUID validation, empty result và pagination metadata.
- [ ] Test name = NULL và geometry MultiPolygon được trả đúng GeoJSON type/coordinates.
- [ ] Test lỗi database có response an toàn; không cố tình làm hỏng shared Supabase.
- [ ] Dọn container/connection pool; test chạy lặp độc lập.

Integration Test phải chạy trên PostGIS thật qua Testcontainers, không dùng SQLite hoặc mock thay thế. CI không kết nối Supabase và không cần Supabase secrets.

#### 5.3 CI Test Jobs

Mở rộng workflow Rust CI hiện có nếu có; tránh tạo workflow trùng lặp.

- [ ] Chạy trên ubuntu-latest với Docker socket/daemon khả dụng cho Testcontainers.
- [ ] Cài Rust toolchain và cache Cargo dependencies.
- [ ] Job Unit: cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings (nếu tương thích cấu hình), và cargo test --lib hoặc command tương đương.
- [ ] Job Integration: chạy Integration Tests với Testcontainers + PostGIS; đặt timeout hợp lý và lưu log khi fail nếu workflow hỗ trợ.
- [ ] Chạy cargo test để bảo đảm toàn bộ test suite, gồm integration test target, được thực thi.
- [ ] CI không cần Supabase credential, không chạy migration/seed trên project thật và không ghi secret vào log.
- [ ] Xác nhận CI xanh trên Pull Request/branch trước nghiệm thu KAN-15.

### Step 6 – Add OpenAPI Documentation and Verify API

Tạo OpenAPI specification làm API contract chính thức cho KAN-15; không chỉ ghi endpoint trong README.

- [x] Kiểm tra convention tài liệu API hiện có; repository chưa có OpenAPI framework.
- [x] Chọn OpenAPI 3.0.3 JSON version-controlled tại backend/openapi/openapi.json để giữ dependency runtime gọn; Swagger UI dùng trang tĩnh tại backend/openapi/swagger-ui.html.
- [x] Khai báo GET /api/v1/buildings, query parameters, defaults/limits, response 200/400/500 và error schema.
- [x] Khai báo DTO schema: UUID fields, nullable name, building_type, GeoJSON MultiPolygon và pagination metadata.
- [x] Mô tả semantics total, total_pages, empty result và page vượt phạm vi.
- [x] Thêm ví dụ request/response thành công và empty result; mô tả validation error.
- [x] Expose OpenAPI JSON tại /api-docs/openapi.json và Swagger UI tại /docs.
- [x] Thêm Unit Test parse OpenAPI JSON và kiểm tra endpoint, max page_size, MultiPolygon schema; khi contract thay đổi phải cập nhật spec và test cùng nhau.
- [x] Ghi hướng dẫn truy cập OpenAPI JSON/Swagger UI và chạy Backend + PostGIS local.
- [ ] Chạy formatter, linter, Unit Tests và Integration Tests.
- [ ] Kiểm tra endpoint bằng HTTP client và đối chiếu response với OpenAPI.
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
- [ ] Có Unit Tests cho validation, pagination, DTO/error mapping; không phụ thuộc database.
- [ ] Có Integration Tests chạy PostgreSQL/PostGIS thật bằng Testcontainers, bao phủ repository và Axum HTTP route.
- [ ] GitHub Actions chạy xanh Unit + Integration Tests; Integration Test không dùng Supabase credentials.
- [ ] Có OpenAPI 3.0.3 JSON version-controlled, được expose qua API và kiểm tra tính hợp lệ/những thành phần contract chính bằng Unit Test; Swagger UI truy cập được tại /docs.

## 6. Deliverables

1. Endpoint truy vấn danh sách công trình.
2. Repository/query và DTO liên quan.
3. Validation và error handling.
4. Unit Tests cho validation, pagination, DTO và error mapping.
5. Integration Tests với PostgreSQL/PostGIS qua Testcontainers.
6. GitHub Actions CI cho format, lint, Unit Tests và Integration Tests.
7. OpenAPI specification và Swagger UI (nếu phù hợp), kèm ví dụ request/response.

## 7. Suggested Commit Sequence

~~~text
feat(api): define building list response contract

feat(api): implement PostGIS building list query

feat(api): add paginated building list endpoint

test(api): add unit tests for validation and pagination

test(api): add PostGIS integration tests with Testcontainers

ci(api): run Rust unit and integration tests

docs(api): publish OpenAPI spec and Swagger UI
~~~

## 8. Final Definition of Done

KAN-15 hoàn thành khi Frontend có thể gọi API để lấy danh sách công trình theo trang và theo khu vực, nhận thuộc tính cùng geometry thống nhất; Unit Tests và Integration Tests (PostgreSQL/PostGIS qua Testcontainers) chạy xanh trên CI; OpenAPI contract phản ánh chính xác implementation.

Ticket này chỉ cung cấp API danh sách công trình; không triển khai API chi tiết công trình/khu vực hoặc tích hợp CesiumJS.
