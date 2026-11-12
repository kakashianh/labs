# KAN-13 – Thiết kế schema PostGIS cho khu vực và công trình

## 1. Objective

Thiết kế và triển khai schema PostgreSQL/PostGIS để lưu dữ liệu không gian nền của dự án 3D GIS Medical, bắt đầu với **một AOI mẫu khoảng 1 km² quanh chợ Bến Thành** và các building footprints OSM nằm trong AOI.

Đầu vào đã được chuẩn bị ở KAN-6:
- `hcm_boundary`: 1 MultiPolygon, EPSG:4326.
- `hcm_buildings`: 431 MultiPolygon features, EPSG:4326, geometry 2D.
- Nguồn building: Geofabrik Vietnam OSM extract tải ngày 2026-11-12 UTC; không phải snapshot lịch sử.
- `building_id` là định danh nguồn theo namespace OSM way/relation; không null và không trùng trong sample đã kiểm tra.

Schema cần hỗ trợ truy vấn khu vực, truy vấn danh sách/chi tiết công trình và import lặp an toàn ở KAN-14. Chỉ thiết kế dữ liệu GIS nền; không đưa logic mô phỏng vào schema này.

## 2. Scope

### In scope
- Xác định quan hệ giữa AOI/region và building.
- Thiết kế bảng lưu AOI và footprint công trình.
- Khai báo geometry type và SRID dựa trên dữ liệu KAN-6.
- Lưu OSM source identifiers và các thuộc tính nguồn có thật.
- Thiết lập primary key, foreign key, unique constraint và NOT NULL phù hợp.
- Tạo spatial index và index phục vụ truy vấn cơ bản.
- Tạo migration, kiểm tra schema và tài liệu mapping cho KAN-14.

### Out of scope
- Chuẩn bị/làm sạch dữ liệu nguồn (KAN-6).
- Import 431 features vào database (KAN-14).
- Xây dựng API Axum (các ticket API).
- Lưu trạng thái SIR theo công trình (Phase 2).
- Lưu scenario, intervention hoặc lịch sử mô phỏng (Phase 3).
- Mô hình phòng/căn hộ, tầng, dân số, occupancy hoặc agent-level.
- Tự suy diễn chiều cao/số tầng từ footprint.

## 3. Verified input contract from KAN-6

### 3.1 Layers

| Source layer | Geometry | SRID | Expected sample size | Ý nghĩa |
|---|---|---:|---:|---|
| `hcm_boundary` | MultiPolygon | 4326 | 1 | AOI chữ nhật tự định nghĩa quanh chợ Bến Thành, không phải boundary hành chính |
| `hcm_buildings` | MultiPolygon | 4326 | 431 | Building footprints 2D đã clip theo AOI |

AOI bounds (longitude, latitude):
- West: 106.693378
- South: 10.768079
- East: 106.702662
- North: 10.777061

AOI center: 10.77257° N, 106.69802° E.

### 3.2 Building source fields

| Source field | Logical type | Nullable | Ý nghĩa / mapping dự kiến |
|---|---|---:|---|
| `building_id` | string | No | External source key: `way/{osm_way_id}` hoặc `relation/{osm_id}` |
| `osm_type` | string | No | `way` hoặc `relation` |
| `osm_id` | integer/string theo driver | Yes | OSM relation ID nếu áp dụng |
| `osm_way_id` | integer/string theo driver | Yes | OSM closed-way ID nếu áp dụng |
| `name` | string | Yes | OSM `name` tag |
| `building` | string | Yes | OSM `building` tag |
| geometry | MultiPolygon 2D | No | Footprint công trình |

Các trường height, levels, population, occupancy và epidemiological state **không có trong data contract hiện tại**. Không tạo giá trị giả hoặc bắt buộc các trường này trong schema nền.

### 3.3 CRS strategy

- GeoJSON đầu vào là RFC 7946 / WGS 84 (EPSG:4326), tọa độ longitude/latitude.
- Phương án đơn giản cho KAN-13: lưu geometry theo SRID 4326 để giữ đúng CRS nguồn và tránh transform không cần thiết khi import.
- Với phép đo diện tích/khoảng cách theo mét, sử dụng `geography` hoặc `ST_Transform` sang CRS chiếu phù hợp (ví dụ UTM zone 48N / EPSG:32648) trong truy vấn/processing.
- Không dùng phép tính planar trực tiếp trên geometry EPSG:4326 để diễn giải kết quả theo mét.
- Chốt chiến lược SRID và kiểu cột trong Step 1 trước khi viết migration. Nếu repository đã có convention CRS khác, phải ghi rõ transform và giữ SRID nguồn có thể truy vết.

## 4. Proposed Data Model

Quan hệ cho sample hiện tại:

```text
regions (1) ─────────── (N) buildings
  Ben Thanh AOI             431 OSM footprints
```

### Table: regions

Lưu vùng nghiên cứu/AOI. Không đặt tên hoặc mô tả bảng theo nghĩa ranh giới hành chính vì polygon hiện tại do dự án tự định nghĩa.

| Column | Suggested type | Constraint / Purpose |
|---|---|---|
| id | UUID | Primary key nội bộ, default `gen_random_uuid()` |
| code | VARCHAR(64) | NOT NULL, UNIQUE; ví dụ mã AOI ổn định |
| name | VARCHAR(255) | NOT NULL; tên khu vực nghiên cứu |
| boundary | geometry(MultiPolygon, 4326) | NOT NULL; khớp layer `hcm_boundary` |
| boundary_method | VARCHAR(64) | NOT NULL; ví dụ `local_aoi` |
| source_reference | VARCHAR(128) | Nullable; tham chiếu nguồn/vị trí AOI |
| created_at | TIMESTAMPTZ | NOT NULL |
| updated_at | TIMESTAMPTZ | NOT NULL |

### Table: buildings

Lưu footprint và metadata OSM thực sự có trong KAN-6.

| Column | Suggested type | Constraint / Purpose |
|---|---|---|
| id | UUID | Primary key nội bộ, default `gen_random_uuid()` |
| external_id | VARCHAR(128) | NOT NULL, UNIQUE; map từ `building_id` |
| osm_type | VARCHAR(16) | NOT NULL; `way` hoặc `relation` |
| osm_id | BIGINT | Nullable; map từ source nếu là relation |
| osm_way_id | BIGINT | Nullable; map từ source nếu là closed way |
| region_id | UUID | NOT NULL, FK đến regions trong sample này |
| name | VARCHAR(255) | Nullable; OSM `name` |
| building_type | VARCHAR(64) | Nullable; map từ OSM `building` |
| footprint | geometry(MultiPolygon, 4326) | NOT NULL; footprint 2D |
| created_at | TIMESTAMPTZ | NOT NULL |
| updated_at | TIMESTAMPTZ | NOT NULL |

**Quyết định cần chốt ở Step 1:** kiểu khóa nội bộ (UUID) và kiểu dữ liệu OSM IDs sau khi kiểm tra giá trị thực tế. `external_id` phải giữ nguyên namespace `way/` hoặc `relation/`; không chỉ lưu số ID vì hai namespace khác nhau.

Nếu sau này có nguồn height/floors đáng tin cậy, có thể bổ sung ở migration riêng. Không thêm các cột không có dữ liệu vào scope hiện tại chỉ để dự phòng.

## 5. Implementation Plan

### Step 1 – Review Prepared GIS Data and Freeze Contract

Đọc trực tiếp KAN-6 metadata, data dictionary, quality report và GeoJSON trước khi tạo schema.

- [ ] Xác nhận file `hcm_boundary.geojson` và `hcm_buildings.geojson` đúng version bàn giao.
- [ ] Xác nhận 1 Polygon boundary và 431 MultiPolygon buildings.
- [ ] Xác nhận SRID 4326 và geometry 2D.
- [ ] Chốt mapping source fields sang database columns.
- [ ] Chốt UUID cho primary key nội bộ.
- [ ] Chốt kiểu lưu `osm_id`/`osm_way_id` là BIGINT sau khi xác nhận các giá trị nguồn là chuỗi số.
- [ ] Chốt geometry(MultiPolygon,4326) cho AOI và geometry(MultiPolygon,4326) cho footprints, hoặc ghi lại lý do nếu chọn CRS lưu trữ khác.
- [ ] Chốt quan hệ region/building và cách xử lý building không gắn được region.
- [ ] Ghi rõ AOI là polygon nghiên cứu tự định nghĩa, không phải đơn vị hành chính.

**Output:** data mapping và schema decision được chốt, đủ điều kiện triển khai migration.

### Step 2 – Verify PostgreSQL and PostGIS Setup

Kiểm tra cấu hình database hiện có trong repository và môi trường Docker/local.

- [ ] Kiểm tra PostgreSQL service/container hiện tại.
- [ ] Kiểm tra phiên bản PostgreSQL và khả năng cài PostGIS.
- [ ] Bật extension PostGIS trong database mục tiêu.
- [ ] Xác nhận `SELECT PostGIS_Full_Version();` chạy thành công.
- [ ] Không tạo database/service trùng lặp nếu repository đã có cấu hình phù hợp.
- [ ] Xác nhận migration tool và convention đang dùng trong repository.

**Output:** database mục tiêu sẵn sàng và xác định được cách chạy migration.

### Step 3 – Design Tables and Constraints

Thiết kế schema tối thiểu cho hai lớp dữ liệu nền.

- [ ] Tạo bảng `regions` với boundary MultiPolygon SRID đã chốt.
- [ ] Tạo bảng `buildings` với footprint MultiPolygon SRID đã chốt.
- [ ] Khai báo primary key nội bộ.
- [ ] Khai báo unique constraint cho `regions.code` và `buildings.external_id`.
- [ ] Khai báo foreign key `buildings.region_id`.
- [ ] Khai báo NOT NULL cho các trường bắt buộc.
- [ ] Khai báo CHECK constraint cho `osm_type` nếu phù hợp.
- [ ] Giữ OSM way/relation ID và namespace để truy vết nguồn.
- [ ] Không thêm height/floors/population hoặc dữ liệu mô phỏng chưa có nguồn.

**Output:** DDL/schema draft được review trước khi migration.

### Step 4 – Add Spatial and Query Indexes

- [ ] Tạo GiST index cho `regions.boundary`.
- [ ] Tạo GiST index cho `buildings.footprint`.
- [ ] Tạo B-tree index cho `buildings.region_id`.
- [ ] Unique constraint/index cho `external_id` và `regions.code`.
- [ ] Kiểm tra index được tạo thành công.
- [ ] Không tạo index dư thừa không gắn với truy vấn dự kiến.

**Output:** indexes phục vụ spatial filtering và truy vấn building theo region.

### Step 5 – Implement Database Migration

Sử dụng migration mechanism đang có trong repository. Nếu chưa có, chọn công cụ phù hợp với backend Rust hiện tại và thống nhất cách chạy local/Docker.

- [ ] Tạo migration bật PostGIS; database container sử dụng image PostGIS tương thích PostgreSQL 16.
- [ ] Tạo migration bảng `regions`.
- [ ] Tạo migration bảng `buildings`.
- [ ] Tạo constraints và indexes.
- [ ] SQLx embedded migrations trong repository hiện không khai báo convention down migration; rollback sẽ được ghi rõ bằng migration bù khi cần, không sửa migration đã áp dụng.
- [ ] Kiểm tra migration chạy trên database rỗng.
- [ ] Kiểm tra migration không phá vỡ cấu hình hiện có.

**Output:** schema có thể được dựng lại bằng migration.

### Step 6 – Verify Schema

- [ ] Xác nhận hai bảng tồn tại.
- [ ] Xác nhận geometry type và SRID đúng data contract KAN-6.
- [ ] Xác nhận foreign key và unique constraints hoạt động.
- [ ] Xác nhận GiST indexes tồn tại.
- [ ] Thử insert một region và một building mẫu hợp lệ.
- [ ] Thử insert dữ liệu sai để xác nhận constraint được thực thi.
- [ ] Kiểm tra spatial query cơ bản bằng `ST_Intersects`.
- [ ] Kiểm tra phép đo metric sử dụng geography hoặc transform phù hợp, không dùng đơn vị độ như mét.

**Output:** bằng chứng schema và spatial operations hoạt động.

### Step 7 – Document Schema and Handoff to KAN-14

- [ ] Tạo tài liệu schema hoặc cập nhật tài liệu database hiện có.
- [ ] Ghi mapping source field → database column.
- [ ] Ghi geometry type, SRID và quy tắc transform.
- [ ] Ghi cách tạo/giữ `external_id`.
- [ ] Ghi quy tắc import idempotent/upsert dựa trên `external_id`.
- [ ] Ghi thứ tự import: region trước, buildings sau để đáp ứng foreign key.
- [ ] Ghi rõ nguồn OSM snapshot và caveat dữ liệu 2D.
- [ ] Bàn giao hướng dẫn import cho KAN-14.

**Output:** KAN-14 có thể import sample dataset mà không phải tự suy đoán schema hoặc mapping.

## 6. Acceptance Criteria

- [ ] PostGIS extension được bật trong database mục tiêu.
- [ ] Có bảng `regions` và `buildings` theo data contract KAN-6.
- [ ] Geometry types khớp: AOI MultiPolygon, building MultiPolygon.
- [ ] SRID được chốt rõ ràng và nhất quán; mặc định giữ 4326 nếu không có lý do kỹ thuật khác.
- [ ] Giữ được source OSM IDs và `external_id` có namespace.
- [ ] Foreign key, unique và NOT NULL constraints hoạt động.
- [ ] Có GiST spatial indexes.
- [ ] Migration chạy thành công trên database rỗng.
- [ ] Có tài liệu mapping và hướng dẫn import idempotent cho KAN-14.
- [ ] Không đưa trạng thái SIR/scenario vào schema nền Phase 1.
- [ ] Không giả định AOI là ranh giới hành chính hoặc building có height/floors.

## 7. Deliverables

1. Migration bật PostGIS và tạo `regions`, `buildings`.
2. Schema PostgreSQL/PostGIS đã được kiểm tra.
3. Spatial indexes và relational constraints.
4. Tài liệu schema, CRS strategy và source-to-database mapping.
5. Hướng dẫn import cho KAN-14.

## 8. Suggested Commit Sequence

```text
chore(db): enable PostGIS extension
feat(db): add regions and buildings schema
perf(db): add spatial and relational indexes
docs(db): document GIS schema and KAN-14 import mapping
```

## 9. Dependencies and Handoff

- **Input:** KAN-6 / P1-04 đã cung cấp GeoJSON và data contract.
- **Consumer:** KAN-14 sẽ import dữ liệu vào schema này.
- **Downstream:** các ticket API sử dụng schema region/building; phase sau mở rộng bảng mô phỏng riêng.
- **Dependency note:** không cần chờ tái tạo độc lập trên môi trường sạch của KAN-6 để bắt đầu KAN-13; dùng đúng artifact/commit dataset đã được kiểm tra. Nếu KAN-6 thay đổi field hoặc CRS, phải cập nhật mapping trước khi merge migration.

## 10. Final Definition of Done

KAN-13 hoàn thành khi database có thể được dựng mới bằng migration, lưu AOI và building geometry với type/SRID chính xác, đảm bảo source identifiers, quan hệ/constraints và spatial indexes, đồng thời có tài liệu mapping để KAN-14 import 1 AOI cùng 431 building features một cách nhất quán.

**Trạng thái:** IMPLEMENTING. Đã xác minh SQLx embedded migrations, PostgreSQL 16 trong Docker Compose, và data contract thực tế. Quyết định: UUID internal PK; OSM numeric IDs là BIGINT; cả AOI và building footprint là MultiPolygon SRID 4326. Lưu ý quan trọng: metadata cũ mô tả AOI là Polygon nhưng GeoJSON bàn giao thực tế là MultiPolygon; schema phải theo dữ liệu thực tế.
