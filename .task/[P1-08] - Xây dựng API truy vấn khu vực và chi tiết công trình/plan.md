# KAN-7 – Xây dựng API truy vấn khu vực và chi tiết công trình

> **Task:** P1-08. **Dependencies:** KAN-13 (PostGIS schema), KAN-14 (data ingestion), KAN-15 (Building List API).  
> **Nguyên tắc:** Chỉ triển khai Region List, Region Detail và Building Detail. Không triển khai lại `GET /api/v1/buildings` của KAN-15.

## 1. Objective

Xây dựng API Rust + Axum để Frontend lấy danh sách khu vực nghiên cứu, chi tiết một khu vực và chi tiết một công trình theo UUID. API dùng dữ liệu PostgreSQL/PostGIS từ KAN-14, giữ response contract nhất quán với KAN-15 và cung cấp dữ liệu cho KAN-16.

## 2. Scope

### In scope
- Xác minh schema regions/buildings và conventions Backend sau KAN-15.
- DTO/response contract cho region list, region detail, building detail.
- Repository/query PostGIS, Axum handlers/routes, UUID validation và not-found.
- GeoJSON MultiPolygon EPSG:4326.
- Unit Tests, Integration Tests bằng Testcontainers + PostGIS.
- Mở rộng OpenAPI spec/Swagger UI hiện có từ KAN-15.
- CI kiểm tra format, lint, Unit/Integration Tests và Docker build theo workflow hiện tại.

### Out of scope
- `GET /api/v1/buildings` có pagination/filter (KAN-15).
- CRUD regions/buildings, mô phỏng SIR, dữ liệu dịch tễ.
- Phân tích không gian/lân cận ngoài truy vấn geometry cần thiết.
- CesiumJS integration (KAN-16).
- Authentication/authorization mới nếu chưa được xác định trong MVP.

## 3. Dependencies and Verified Database Contract

### 3.1 Dependencies

| Task | Trách nhiệm | Quan hệ |
|---|---|---|
| KAN-13 / P1-05 | PostGIS schema | Source of truth cho columns, constraints, geometry |
| KAN-14 / P1-06 | Data ingestion/validation | Source of truth cho dữ liệu đã nạp |
| KAN-15 / P1-07 | Building List API | P1-08 tái sử dụng DB pool, AppError, DTO/response conventions, OpenAPI và Testcontainers harness |
| KAN-7 / P1-08 | Region list/detail + building detail | Phạm vi plan này |
| KAN-16 | Frontend/CesiumJS | Consume region list, region detail, building list và building detail |

### 3.2 Schema contract

Đối chiếu migration `backend/src/db/migrations/20261112000000_create_postgis_regions_and_buildings.sql` với database target KAN-14 trước khi code để phát hiện schema drift.

| Table/column | Definition theo migration | API implication |
|---|---|---|
| `regions.id` | UUID PK | `region_id` là UUID |
| `regions.code` | VARCHAR(64), unique, not null | Mã khu vực |
| `regions.name` | VARCHAR(255), not null | Tên khu vực |
| `regions.description` | TEXT, nullable | DTO nullable |
| `regions.boundary_method` | VARCHAR(64), not null | Có thể expose metadata |
| `regions.source_reference` | TEXT, nullable | DTO nullable |
| `regions.boundary` | geometry(MultiPolygon, 4326), not null | GeoJSON MultiPolygon WGS84 |
| `buildings.id` | UUID PK | `building_id` là UUID |
| `buildings.external_id` | VARCHAR(128), unique, not null | Mã OSM, không thay UUID |
| `buildings.osm_type` | VARCHAR(16), not null | Chỉ expose nếu UI cần |
| `buildings.osm_id`, `osm_way_id` | BIGINT nullable, constrained by osm_type | Không suy diễn field |
| `buildings.region_id` | UUID FK, not null | Region reference |
| `buildings.name` | TEXT, nullable | DTO nullable |
| `buildings.building_type` | TEXT, nullable | DTO nullable |
| `buildings.footprint` | geometry(MultiPolygon, 4326), not null | GeoJSON MultiPolygon WGS84 |

PostGIS extension trên Supabase target nằm trong schema `extensions`. Tái sử dụng cách KAN-15 xử lý `search_path`/schema qualification khi gọi spatial functions.

### 3.3 Data rules
- ID nội bộ là UUID; không dùng `regions.code` hoặc `buildings.external_id` thay UUID path parameter.
- Geometry trả về là GeoJSON `MultiPolygon`, SRID 4326, tọa độ longitude/latitude.
- `description`, `source_reference`, `name`, `building_type` giữ đúng nullable.
- Không tự thêm height, floors, population, area hoặc thuộc tính không có trong schema/nguồn dữ liệu.
- Chỉ select và expose field thuộc API contract; không serialize raw database row.
- Region list ordering ổn định theo `code ASC` (unique).
- Không nhúng buildings vào Region Detail; dùng KAN-15 để lấy building list theo `region_id`.

## 4. API Contract

Base prefix: `/api/v1`.

| Method | Endpoint | Purpose | Owner |
|---|---|---|---|
| GET | `/api/v1/regions` | Danh sách khu vực | P1-08 |
| GET | `/api/v1/regions/{region_id}` | Chi tiết khu vực | P1-08 |
| GET | `/api/v1/buildings/{building_id}` | Chi tiết công trình | P1-08 |
| GET | `/api/v1/buildings` | Danh sách có pagination/filter | KAN-15 |

### 4.1 Shared response/error contract
- Success envelope: `{ "data": ... }`, thống nhất KAN-15. Region list có data array; detail có data object.
- Error envelope theo AppError hiện có: `{ "error": "..." }`.
- HTTP 400: UUID path parameter sai định dạng.
- HTTP 404: UUID hợp lệ nhưng record không tồn tại.
- HTTP 500: lỗi database/internal, chỉ trả thông báo generic; không leak SQL, credentials, connection string hoặc stack trace.
- Region list rỗng trả HTTP 200 với `data: []`, không trả 404.

### 4.2 GET /api/v1/regions
MVP hiện có phạm vi nghiên cứu nhỏ nên không thêm pagination trong P1-08. Nếu dataset mở rộng, pagination là thay đổi contract cần thống nhất riêng.

Region list item:
- `id`: UUID string.
- `code`, `name`: string.
- `description`: string hoặc null.

Ordering: `code ASC`.

Example:
~~~json
{
  "data": [
    {
      "id": "00000000-0000-4000-8000-000000000001",
      "code": "BEN_THANH",
      "name": "Bến Thành",
      "description": null
    }
  ]
}
~~~

### 4.3 GET /api/v1/regions/{region_id}
- Path parameter `region_id` là UUID, validate trước khi query.
- DTO: `id`, `code`, `name`, `description`, `boundary_method`, `source_reference`, `geometry`.
- Nullable: description, source_reference.
- Geometry: GeoJSON MultiPolygon.

Example:
~~~json
{
  "data": {
    "id": "00000000-0000-4000-8000-000000000001",
    "code": "BEN_THANH",
    "name": "Bến Thành",
    "description": null,
    "boundary_method": "osm_admin_boundary",
    "source_reference": null,
    "geometry": {
      "type": "MultiPolygon",
      "coordinates": []
    }
  }
}
~~~

### 4.4 GET /api/v1/buildings/{building_id}
- Path parameter `building_id` là UUID, validate trước khi query.
- DTO: `id`, `region_id`, `external_id`, `name`, `building_type`, `geometry`.
- Nullable: name, building_type.
- Geometry lấy từ `buildings.footprint`, GeoJSON MultiPolygon.
- Không thêm pagination hoặc nhúng building khác.

Example:
~~~json
{
  "data": {
    "id": "00000000-0000-4000-8000-000000000002",
    "region_id": "00000000-0000-4000-8000-000000000001",
    "external_id": "way/123456",
    "name": null,
    "building_type": "residential",
    "geometry": {
      "type": "MultiPolygon",
      "coordinates": []
    }
  }
}
~~~

UUID/coordinates trong ví dụ chỉ minh họa contract, không phải fixture bắt buộc.

## 5. Implementation Plan

### Step 1 – Inspect Schema and Existing API Conventions
- [x] Xác nhận branch/base đã chứa KAN-15 sau khi PR #10 merge.
- [x] Kiểm tra Axum router, DB pool, repository/module structure và route registration.
- [x] Đọc migration regions/buildings và đối chiếu schema DB target KAN-14.
- [x] Xác nhận UUID PK, unique keys, nullable fields, FK, geometry type/SRID.
- [x] Kiểm tra PostGIS extension schema và cách gọi spatial functions theo KAN-15.
- [x] Kiểm tra AppError/error envelope, tránh tạo convention trùng lặp.
- [x] Kiểm tra OpenAPI JSON, Swagger UI và contract tests hiện có.
- [x] Chốt DTO theo schema và nhu cầu UI; không thêm field suy đoán.

### Step 2 – Define DTOs and Response Contracts
- [x] RegionListItem: id, code, name, description.
- [x] RegionDetail: id, code, name, description, boundary_method, source_reference, geometry.
- [x] BuildingDetail: id, region_id, external_id, name, building_type, geometry.
- [x] Dùng nullable types đúng schema.
- [x] Dùng GeoJSON value/DTO thống nhất KAN-15; geometry type là MultiPolygon.
- [x] Chuẩn hóa success envelope `{ data: ... }` và error envelope `{ error: ... }`.
- [x] Chốt 400 invalid UUID, 404 not found, 500 generic internal error.
- [x] Không nhúng buildings vào Region Detail; building list thuộc KAN-15.

### Step 3 – Implement Region Repository Queries
- [x] Query region list, chỉ select field của RegionListItem.
- [x] Ordering ổn định bằng `code ASC`.
- [x] Query region theo UUID.
- [x] Serialize `boundary` sang GeoJSON bằng PostGIS.
- [x] Đảm bảo spatial functions hoạt động với extension schema/search_path như KAN-15.
- [x] Map query result sang DTO, không trả raw DB row.
- [x] Phân biệt record không tồn tại với database error.

### Step 4 – Implement Building Detail Repository Query
- [x] Query một building theo UUID.
- [x] Chỉ select id, region_id, external_id, name, building_type và footprint GeoJSON.
- [x] Serialize footprint thành GeoJSON MultiPolygon.
- [x] Map nullable name/building_type chính xác.
- [x] Không đưa pagination/filter list của KAN-15 vào detail query.
- [x] Phân biệt record không tồn tại với database error.

### Step 5 – Implement Axum Handlers and Routes
- [x] Handler GET /api/v1/regions.
- [x] Handler GET /api/v1/regions/{region_id}.
- [x] Handler GET /api/v1/buildings/{building_id}.
- [x] Parse UUID path parameter rõ ràng; invalid UUID trả 400.
- [x] UUID hợp lệ nhưng không có record trả 404.
- [x] Đăng ký routes vào router hiện tại, không tạo app entry point thứ hai.
- [x] Reuse AppError, DB pool, repository conventions và response envelope KAN-15.
- [x] Không expose SQL, database internals, credentials hoặc stack trace.
- [x] Đảm bảo route detail không xung đột GET /api/v1/buildings của KAN-15.

### Step 6 – Unit Tests
Unit Tests không khởi tạo network, database hoặc Docker.
- [x] UUID path parsing: valid/invalid.
- [x] DTO mapping cho region list/detail và building detail.
- [x] Nullable description/source_reference/name/building_type.
- [x] GeoJSON DTO/serialization với MultiPolygon.
- [x] NotFound -> HTTP 404; BadRequest -> HTTP 400.
- [x] Database/Internal -> HTTP 500 generic; không leak SQL/credentials.
- [x] OpenAPI JSON parse và contract assertions cho ba endpoint.

### Step 7 – Integration Tests with Testcontainers
Dùng cùng crate/test harness và PostGIS image pin cụ thể theo KAN-15. Không dùng SQLite/mock DB thay thế PostGIS Integration Tests; không kết nối Supabase thật trong CI.
- [x] Reuse/mở rộng Testcontainers harness hiện có, tránh duplicate lifecycle helper.
- [x] Khởi chạy PostgreSQL 17 + PostGIS container với image tag cố định, không dùng latest.
- [x] Tạo database/schema và chạy migrations của ứng dụng.
- [x] Seed fixture deterministic: 2 regions, 1 building, nullable attributes và MultiPolygon SRID 4326 (đủ bao phủ region list/detail và building detail; không cần 3 building cho API detail).
- [x] Test region list có dữ liệu, ordering code ASC và field contract.
- [x] Test region list rỗng -> HTTP 200, data [].
- [x] Test region detail tồn tại, invalid UUID -> 400, UUID không tồn tại -> 404.
- [x] Test building detail tồn tại, invalid UUID -> 400, UUID không tồn tại -> 404.
- [x] Test nullable fields và GeoJSON MultiPolygon type/coordinates.
- [x] Test response envelope/field names khớp OpenAPI.
- [x] Test database failure trả 500 generic bằng test DB/pool riêng hoặc fault injection; không làm hỏng shared Supabase.
- [x] Đảm bảo container/connection pool được dọn và test chạy độc lập/lặp lại.

### Step 8 – OpenAPI Documentation
Mở rộng OpenAPI JSON và Swagger UI do KAN-15 thiết lập; không tạo tài liệu/server thứ hai.
- [x] Bổ sung GET /api/v1/regions.
- [x] Bổ sung GET /api/v1/regions/{region_id} với UUID path parameter.
- [x] Bổ sung GET /api/v1/buildings/{building_id} với UUID path parameter.
- [x] Khai báo RegionListItem, RegionDetail, BuildingDetail, GeoJSON MultiPolygon và shared error schema.
- [x] Khai báo 200/400/404/500 cho từng endpoint.
- [x] Mô tả nullable fields, stable ordering và empty-list behavior.
- [ ] Thêm success/empty/not-found examples.
- [ ] Đảm bảo /api-docs/openapi.json và /docs tiếp tục phục vụ spec sau cập nhật.
- [x] Mở rộng OpenAPI contract tests cho paths, path params, schemas, response codes.
- [x] Không làm mất/thay đổi contract GET /api/v1/buildings của KAN-15.

### Step 9 – CI and Verification
Mở rộng workflow CI hiện tại, tránh tạo workflow trùng lặp.
- [x] Giữ Unit và Testcontainers Integration thành các job riêng như KAN-15.
- [x] Đảm bảo runner có Docker daemon/socket.
- [x] Chạy cargo fmt --check.
- [x] Chạy cargo clippy --all-targets --all-features -- -D warnings.
- [x] Chạy Unit Tests.
- [x] Chạy Integration Tests region/building detail với Testcontainers.
- [x] Giữ PostGIS Compose integration và Docker image build hiện có trong dependency graph phù hợp.
- [x] CI không cần Supabase credentials và không truy cập DB thật.
- [x] Xác nhận toàn bộ PR checks xanh trên commit implementation/OpenAPI.
- [x] Smoke test bằng HTTP client: list, detail, invalid UUID, not-found.
- [x] Xác nhận response đủ cho KAN-16 chọn region và xem building detail.

## 6. Database Access and Security

- Backend chỉ dùng server-side environment/secret manager.
- Không đưa DB password/service-role key vào Frontend, source code, response hoặc logs.
- Không tắt RLS và không tạo public read policy chỉ để API hoạt động.
- Chọn runtime DB role có quyền tối thiểu cần thiết.
- [ ] Xác minh query chạy bằng đúng runtime role/credential đã phê duyệt.
- [ ] Kiểm tra logs/error responses không chứa connection string, SQL nhạy cảm hoặc credentials.

## 7. Acceptance Criteria

- [x] GET /api/v1/regions trả regions theo code ASC.
- [x] Region list rỗng trả HTTP 200 với data [].
- [x] GET /api/v1/regions/{region_id} trả đúng region detail.
- [x] GET /api/v1/buildings/{building_id} trả đúng building detail.
- [x] UUID invalid trả 400; UUID hợp lệ nhưng không tồn tại trả 404.
- [x] Success/error envelope nhất quán KAN-15.
- [x] Region boundary và building footprint là GeoJSON MultiPolygon EPSG:4326.
- [x] Nullable fields đúng schema; không phát sinh field không có nguồn.
- [x] Unit Tests bao phủ validation, DTO, error mapping và OpenAPI.
- [x] Testcontainers Integration Tests chạy PostgreSQL 17/PostGIS thật.
- [x] Database failure trả 500 generic, không leak nội bộ.
- [x] OpenAPI bao gồm ba endpoint mới và không phá contract KAN-15.
- [ ] Swagger UI và OpenAPI JSON endpoint hiện có hoạt động.
- [x] GitHub Actions CI chạy xanh toàn bộ jobs liên quan.
- [x] API đủ dữ liệu cho KAN-16; không bao gồm CesiumJS trong ticket này.

## 8. Deliverables

1. Region list repository/query, DTO, handler và route.
2. Region detail repository/query, DTO, handler và route.
3. Building detail repository/query, DTO, handler và route.
4. Unit Tests cho validation, DTO, errors và OpenAPI.
5. Testcontainers Integration Tests với PostgreSQL/PostGIS.
6. OpenAPI JSON và Swagger UI cập nhật.
7. GitHub Actions CI cho Unit/Integration Tests và checks hiện có.
8. Hướng dẫn request/response và chạy test local.

## 9. Suggested Commit Sequence

~~~text
feat(api): define region and building detail contracts
feat(api): add region list and detail queries
feat(api): add building detail query
feat(api): expose region and building detail endpoints
test(api): add unit tests for region and building DTOs
test(api): add PostGIS integration tests for region and building details
docs(api): extend OpenAPI for region and building detail endpoints
ci(api): verify region and building detail API tests
~~~

## 10. Final Definition of Done

> **Implementation status:** Đã merge vào `project/3D-map-medical` qua PR #11 (merge commit `0d07cdb0d07aff58daaee9459f3048f0253857d0`); CI workflow #130 xanh. Các checklist còn mở bên dưới là những xác minh chưa có bằng chứng trực tiếp (runtime least-privilege role, Swagger UI smoke test và ví dụ response chi tiết).

KAN-7/P1-08 hoàn thành khi Frontend có thể lấy danh sách region theo thứ tự ổn định, lấy geometry/metadata của region theo UUID và lấy geometry/thuộc tính building theo UUID. APIs khớp schema PostGIS thực tế, dùng chung response/error/OpenAPI conventions của KAN-15, có Unit Tests và Testcontainers Integration Tests xanh trên CI. Không triển khai lại Building List API của KAN-15 và không bao gồm CesiumJS integration của KAN-16.
