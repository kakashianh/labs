# KAN-14 – Nạp và kiểm tra dữ liệu đô thị mẫu vào PostGIS

## 1. Objective

Nạp bộ dữ liệu đô thị mẫu đã chuẩn hóa từ KAN-6 vào schema PostgreSQL/PostGIS được triển khai tại KAN-13.

Đảm bảo dữ liệu khu vực và footprint công trình:
- Được import đầy đủ và đúng mapping.
- Giữ nguyên định danh OSM từ dữ liệu nguồn.
- Có SRID và geometry type đúng theo schema.
- Có quan hệ `buildings.region_id` hợp lệ.
- Có thể truy vấn không gian bằng PostGIS.
- Có quy trình import có thể chạy lại mà không tạo dữ liệu trùng.

**Expected result:**
- 1 region được import.
- 431 building features được import theo snapshot KAN-6.
- Các bản ghi được kiểm tra về thuộc tính, geometry, CRS và quan hệ.
- Có tài liệu/script để tái tạo quá trình import.

## 2. Dependencies

### KAN-6 – Chuẩn bị dữ liệu GIS

Input files:
- `data/sample/processed/hcm_boundary.geojson`
- `data/sample/processed/hcm_buildings.geojson`

Dataset snapshot:
- Khu vực: 1 feature.
- Công trình: 431 features.
- CRS: EPSG:4326.
- Boundary nguồn: Polygon.
- Building geometry nguồn: MultiPolygon 2D.
- Khu vực nghiên cứu: vùng mẫu quanh chợ Bến Thành, TP.HCM, diện tích xấp xỉ 1 km².
- `building_id` được định danh theo namespace `way/{osm_way_id}` hoặc `relation/{osm_id}`.

### KAN-13 – PostGIS Schema

Sử dụng schema đã được triển khai, không thiết kế lại nếu chưa phát hiện lỗi thực tế.

**regions**
- `id`: UUID, database tự sinh.
- `code`: mã khu vực, unique.
- `boundary`: `geometry(MultiPolygon, 4326)`.

**buildings**
- `id`: UUID, database tự sinh.
- `region_id`: UUID FK đến `regions.id`.
- `external_id`: định danh nguồn OSM, unique.
- `osm_type`, `osm_id`, `osm_way_id`.
- `name`, `building`.
- `footprint`: `geometry(MultiPolygon, 4326)`.

Các index chính:
- GiST trên `regions.boundary`.
- GiST trên `buildings.footprint`.
- B-tree trên `buildings.region_id`.

## 3. Scope

### In scope
- Kiểm tra PostGIS database và migration KAN-13.
- Kiểm tra dữ liệu GeoJSON đầu vào từ KAN-6.
- Xây dựng mapping nguồn – đích.
- Import regions và buildings.
- Chuyển đổi Polygon thành MultiPolygon khi cần.
- Gán `region_id` cho buildings.
- Đảm bảo import có thể chạy lại.
- Kiểm tra số lượng, thuộc tính, geometry, CRS và foreign key.
- Kiểm tra truy vấn không gian và index.
- Tài liệu hóa quy trình import.

### Out of scope
- Thu thập dữ liệu OSM mới.
- Thay đổi schema KAN-13 nếu không có lỗi được xác nhận.
- Xây dựng API GIS.
- Hiển thị dữ liệu bằng CesiumJS.
- Suy luận chiều cao, số tầng hoặc dân số từ footprint.
- Mô phỏng dịch bệnh hoặc tính toán SIR.
- Đồng bộ dữ liệu thời gian thực.

## 4. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Database | PostgreSQL 16 | Lưu trữ dữ liệu |
| Spatial extension | PostGIS 3.5 | Geometry và spatial query |
| Input | GeoJSON EPSG:4326 | Dữ liệu KAN-6 |
| Import | ogr2ogr hoặc staging + SQL | Nạp dữ liệu |
| Backend context | Rust + Axum | Ứng dụng sử dụng database |

Không tạo Python backend/service cho tác vụ import. Nếu cần script hỗ trợ, script phải có hướng dẫn chạy lại và không chứa credentials.

## 5. Implementation Plan

### Step 1 – Verify Database and Schema Readiness

Mục tiêu: xác nhận database đã sẵn sàng trước khi import.

- [ ] Khởi động PostgreSQL/PostGIS theo cấu hình repository.
- [ ] Kiểm tra kết nối database.
- [ ] Xác nhận PostGIS extension đã được bật.
- [ ] Chạy hoặc xác nhận migration KAN-13 đã hoàn tất.
- [ ] Kiểm tra bảng `regions` và `buildings`.
- [ ] Kiểm tra geometry type và SRID.
- [ ] Kiểm tra unique constraints, foreign key và GiST indexes.
- [ ] Ghi nhận cách cấu hình connection string qua environment variables.

**Expected result:** database và schema đáp ứng đầy đủ điều kiện import.

### Step 2 – Prepare Input Data and Field Mapping

Mục tiêu: xác định chính xác dữ liệu nguồn và mapping sang schema KAN-13.

**Input files**
- [ ] Kiểm tra `data/sample/processed/hcm_boundary.geojson`.
- [ ] Kiểm tra `data/sample/processed/hcm_buildings.geojson`.
- [ ] Xác nhận CRS EPSG:4326.
- [ ] Xác nhận số lượng feature theo snapshot KAN-6.
- [ ] Kiểm tra encoding và kiểu dữ liệu thuộc tính.
- [ ] Kiểm tra cấu trúc geometry nguồn.

**Region mapping**

| Source | Target |
|---|---|
| Region code được xác định ổn định | `regions.code` |
| Boundary Polygon | `regions.boundary` MultiPolygon, SRID 4326 |
| Database-generated | `regions.id` |

**Building mapping**

| Source GeoJSON | Target PostGIS |
|---|---|
| `building_id` | `external_id` |
| `osm_type` | `osm_type` |
| `osm_id` | `osm_id` |
| `osm_way_id` | `osm_way_id` |
| `name` | `name` |
| `building` | `building` |
| Geometry | `footprint` |
| Region UUID sau khi import | `region_id` |
| Database-generated | `id` |

**Mapping rules**
- [ ] Giữ nguyên prefix `way/` hoặc `relation/` trong `external_id`.
- [ ] Không sử dụng GDAL FID hoặc thứ tự feature làm business identity.
- [ ] Không hard-code UUID của region.
- [ ] Lấy `region_id` từ region đã được import thông qua `regions.code`.
- [ ] Chuyển Polygon boundary thành MultiPolygon bằng `ST_Multi` hoặc cách tương đương.
- [ ] Không thay đổi schema chỉ để phù hợp với geometry nguồn.
- [ ] Giữ nguyên các trường nullable như `name` và `building`.
- [ ] Không tự suy luận height, floors, population hoặc epidemiology.

### Step 3 – Pre-import Data Validation

Mục tiêu: phát hiện vấn đề dữ liệu trước khi ghi vào bảng chính.

- [ ] Kiểm tra file GeoJSON có thể đọc được.
- [ ] Kiểm tra feature không có geometry.
- [ ] Kiểm tra geometry rỗng.
- [ ] Kiểm tra geometry không hợp lệ bằng công cụ phù hợp.
- [ ] Kiểm tra `building_id` null hoặc duplicate.
- [ ] Kiểm tra các trường OSM có nhất quán.
- [ ] Kiểm tra extent và CRS của dữ liệu.
- [ ] Ghi nhận các bản ghi lỗi và phương án xử lý.

**Expected result:** có báo cáo pre-import validation; không âm thầm bỏ qua dữ liệu lỗi.

### Step 4 – Implement Repeatable Import Process

Mục tiêu: xây dựng quy trình import có thể chạy lại an toàn.

**Import sequence**
1. Import region.
2. Tìm region bằng `regions.code` để lấy UUID.
3. Import buildings và gán UUID đó vào `region_id`.
4. Kiểm tra kết quả import.
5. Commit transaction nếu toàn bộ bước ghi dữ liệu thành công.

**Idempotency requirements**
- [ ] Sử dụng `regions.code` để nhận diện region ổn định.
- [ ] Upsert region theo unique key `code`.
- [ ] Upsert buildings theo unique key `external_id`.
- [ ] Không tạo region hoặc building duplicate khi chạy lại.
- [ ] Không hard-code UUID.
- [ ] Xử lý transaction/rollback khi import thất bại.
- [ ] Xác định rõ chính sách với bản ghi đã bị xóa khỏi nguồn: không tự động xóa dữ liệu đích nếu chưa có quy tắc đồng bộ được thống nhất.
- [ ] Ghi log số lượng bản ghi insert, update, skip và failure.

Có thể sử dụng:
- `ogr2ogr`.
- Staging table kết hợp SQL.
- Script hoặc command import phù hợp repository.

Không bắt buộc sử dụng một công cụ cụ thể nếu có phương án đơn giản và đáng tin cậy hơn.

### Step 5 – Validate Imported Records

Mục tiêu: xác nhận dữ liệu trong PostGIS khớp với dữ liệu nguồn.

**Record validation**
- [ ] Kiểm tra số lượng region.
- [ ] Kiểm tra số lượng buildings.
- [ ] Kiểm tra `external_id` không null.
- [ ] Kiểm tra `external_id` không duplicate.
- [ ] Kiểm tra namespace `way/` và `relation/`.
- [ ] Đối chiếu `osm_type`, `osm_id`, `osm_way_id` với nguồn.
- [ ] Đối chiếu một số thuộc tính `name`, `building`.

**Geometry validation**
- [ ] Kiểm tra geometry không null.
- [ ] Kiểm tra geometry không empty.
- [ ] Kiểm tra `ST_IsValid(geometry) = true`.
- [ ] Kiểm tra SRID bằng `ST_SRID`.
- [ ] Kiểm tra geometry type đúng MultiPolygon.
- [ ] Kiểm tra extent trong PostGIS phù hợp với dữ liệu nguồn.
- [ ] Kiểm tra building có nằm trong hoặc giao với AOI theo spatial predicate được lựa chọn.

Lưu ý: `ST_Within` và `ST_Intersects` có ngữ nghĩa khác nhau. Cần chọn predicate phù hợp với yêu cầu kiểm tra; không mặc định coi mọi building giao với ranh giới là lỗi.

**Relationship validation**
- [ ] Tất cả building có `region_id` hợp lệ.
- [ ] Không có orphan records.
- [ ] Xác nhận foreign key hoạt động đúng.

**Expected result**
- Số lượng dữ liệu khớp snapshot KAN-6 hoặc mọi chênh lệch đều được giải thích.
- Không còn geometry lỗi chưa được xử lý.
- Không có quan hệ FK sai.

### Step 6 – Verify Spatial Queries and Index Usage

Mục tiêu: xác nhận dữ liệu sẵn sàng cho các phase API và visualization.

- [ ] Query buildings theo `region_id`.
- [ ] Query buildings giao với boundary bằng `ST_Intersects`.
- [ ] Query buildings nằm hoàn toàn trong boundary bằng `ST_Within` khi cần.
- [ ] Kiểm tra extent bằng `ST_Extent`.
- [ ] Kiểm tra kết quả query bằng cách đối chiếu với dữ liệu nguồn.
- [ ] Sử dụng `EXPLAIN` hoặc `EXPLAIN ANALYZE`.
- [ ] Xác nhận planner có thể cân nhắc GiST index khi phù hợp.
- [ ] Không bắt buộc index scan đối với dataset nhỏ nếu sequential scan hiệu quả hơn.

Không tối ưu hóa quá mức cho dataset mẫu 431 buildings.

### Step 7 – Re-import Test and Documentation

Mục tiêu: chứng minh quy trình có thể tái tạo.

- [ ] Chạy lại import trên cùng dataset.
- [ ] Xác nhận không phát sinh duplicate.
- [ ] Xác nhận số lượng và quan hệ dữ liệu không sai lệch.
- [ ] Kiểm tra upsert hoạt động đúng.
- [ ] Ghi lại prerequisite.
- [ ] Ghi lại câu lệnh chạy import.
- [ ] Ghi lại environment variables cần thiết, không ghi secret.
- [ ] Ghi lại kết quả validation.
- [ ] Ghi lại các giới hạn của dataset và những vấn đề còn tồn tại.

## 6. Acceptance Criteria

- [ ] PostGIS database đã có region và building data.
- [ ] Import đúng hai file đầu vào KAN-6.
- [ ] Số lượng dữ liệu được đối chiếu với snapshot nguồn.
- [ ] Field mapping tuân thủ schema KAN-13.
- [ ] `external_id` giữ nguyên định danh OSM và unique.
- [ ] Geometry có type MultiPolygon và SRID 4326.
- [ ] Geometry không null, empty hoặc invalid chưa xử lý.
- [ ] Buildings có `region_id` hợp lệ.
- [ ] Các spatial query cơ bản chạy thành công.
- [ ] GiST indexes được xác nhận tồn tại và có thể sử dụng.
- [ ] Re-import không tạo dữ liệu duplicate.
- [ ] Có tài liệu hoặc script tái tạo import.
- [ ] Không có credentials hoặc secret trong source control.
- [ ] Không tự suy luận height, floors, population hoặc epidemiology.

## 7. Deliverables

1. Region và building data đã được nạp vào PostGIS.
2. Script/command hoặc hướng dẫn import có thể tái sử dụng.
3. Báo cáo pre-import validation.
4. Báo cáo post-import validation.
5. Kết quả kiểm tra spatial query và index.
6. Tài liệu hướng dẫn re-import và các giới hạn dữ liệu.

## 8. Suggested Commit Sequence

```text
chore(data): prepare GIS import configuration

feat(data): import sample region into PostGIS

feat(data): import sample building footprints into PostGIS

test(data): validate imported GIS records and spatial queries

docs(data): document sample dataset import procedure
```

## 9. Final Definition of Done

KAN-14 hoàn thành khi:
- Dữ liệu từ KAN-6 được import vào schema KAN-13.
- Mapping, định danh OSM và quan hệ region-building chính xác.
- Geometry và CRS vượt qua validation.
- Spatial queries cơ bản hoạt động.
- Re-import không tạo dữ liệu trùng.
- Có hướng dẫn tái tạo và báo cáo kết quả.

KAN-14 chỉ chịu trách nhiệm nạp và kiểm tra dữ liệu nền. Không triển khai API, Cesium visualization hoặc mô phỏng SIR trong ticket này.
