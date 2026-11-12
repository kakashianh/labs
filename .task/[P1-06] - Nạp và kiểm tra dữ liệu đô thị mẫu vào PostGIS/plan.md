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
- Boundary nguồn trong GeoJSON hiện tại: MultiPolygon 2D.
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
- `name`, `building_type` (nhận giá trị từ thuộc tính nguồn `building`).
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
- Chuẩn hóa geometry sang MultiPolygon khi cần; boundary input hiện tại đã là MultiPolygon.
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
| Import | GDAL/ogr2ogr + PostgreSQL staging + SQL | Nạp và upsert dữ liệu |
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
| Boundary MultiPolygon | `regions.boundary` MultiPolygon, SRID 4326 |
| Database-generated | `regions.id` |

**Building mapping**

| Source GeoJSON | Target PostGIS |
|---|---|
| `building_id` | `external_id` |
| `osm_type` | `osm_type` |
| `osm_id` | `osm_id` |
| `osm_way_id` | `osm_way_id` |
| `name` | `name` |
| `building` | `building_type` |
| Geometry | `footprint` |
| Region UUID sau khi import | `region_id` |
| Database-generated | `id` |

**Mapping rules**
- [ ] Giữ nguyên prefix `way/` hoặc `relation/` trong `external_id`.
- [ ] Không sử dụng GDAL FID hoặc thứ tự feature làm business identity.
- [ ] Không hard-code UUID của region.
- [ ] Lấy `region_id` từ region đã được import thông qua `regions.code`.
- [ ] Xác nhận boundary nguồn là MultiPolygon; dùng `ST_Multi` để bảo đảm geometry tương thích schema đích.
- [ ] Không thay đổi schema chỉ để phù hợp với geometry nguồn.
- [ ] Giữ nguyên các thuộc tính nguồn nullable như `name` và `building`; map `building` vào `buildings.building_type`.
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
- [ ] Đối chiếu một số thuộc tính nguồn `name`, `building` với `buildings.name`, `buildings.building_type`.

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

## 6. Human-in-the-loop (HITL) and Approval Gates

KAN-14 sử dụng mô hình **automation-first, human-reviewed**: máy thực hiện import và các kiểm tra có thể tự động hóa; con người chỉ review các quyết định ảnh hưởng đến tính đúng đắn của dữ liệu và xác nhận chất lượng ở các checkpoint. Không yêu cầu kiểm tra thủ công từng building trong 431 features.

### Gate 1 – Approve Input and Mapping (Before Import)

**Thời điểm:** sau Step 2 và trước Step 4.

- [ ] Người review xác nhận đúng hai file đầu vào từ KAN-6.
- [ ] Xác nhận snapshot dự kiến: 1 region và 431 buildings.
- [ ] Xác nhận region code ổn định sẽ dùng để upsert.
- [ ] Review mapping `building_id` → `external_id`, OSM attributes và geometry.
- [ ] Xác nhận boundary MultiPolygon và quy tắc chuẩn hóa geometry tương thích schema.
- [ ] Chỉ bắt đầu import khi mapping và nguồn dữ liệu được chấp thuận.

**Evidence:** mapping checklist được đánh dấu approved hoặc ghi nhận trong báo cáo KAN-14.

### Gate 2 – Review Validation Exceptions

**Thời điểm:** sau Step 3 và bất cứ khi nào post-import validation ở Step 5 phát hiện bất thường.

- [ ] Tự động tạo danh sách/report các feature null, empty, invalid, duplicate hoặc sai CRS/extent.
- [ ] Người review kiểm tra các exception và quyết định một trong các hướng: sửa ở nguồn/quy trình chuẩn hóa, loại khỏi lần import với lý do rõ ràng, hoặc dừng import để điều tra.
- [ ] Mọi quyết định loại trừ hoặc sửa dữ liệu phải được ghi nhận cùng feature ID và lý do.
- [ ] Không tự động sửa geometry, drop record hoặc bỏ qua lỗi mà không có policy được duyệt.
- [ ] Sau khi xử lý, chạy lại pre-validation trước khi tiếp tục.

**Evidence:** validation report và exception disposition (resolved / approved exclusion / blocked).

### Gate 3 – Visual QA in QGIS

**Thời điểm:** sau Step 5, trước khi chấp nhận kết quả cuối cùng.

- [ ] Mở boundary và buildings từ PostGIS trong QGIS.
- [ ] Kiểm tra vị trí AOI có đúng khu vực mẫu quanh chợ Bến Thành.
- [ ] Kiểm tra footprint buildings có hiển thị đúng hình dạng và không bị lệch vị trí.
- [ ] Kiểm tra các outlier rõ rệt: công trình nằm xa AOI, geometry bất thường hoặc vùng trống bất ngờ.
- [ ] Đối chiếu trực quan với GeoJSON nguồn KAN-6 khi cần.
- [ ] Người review ghi nhận kết quả: pass hoặc issue cần điều tra.

**Evidence:** kết quả checklist QA; lưu screenshot QGIS nếu repository/project yêu cầu bằng chứng trực quan.

### Gate 4 – Final Acceptance

**Thời điểm:** sau Step 6 và Step 7.

- [ ] Người review xác nhận count và mọi chênh lệch so với snapshot KAN-6 đã được giải thích.
- [ ] Xác nhận geometry, SRID, FK và unique constraints đạt yêu cầu.
- [ ] Xác nhận spatial query cơ bản cho kết quả hợp lý.
- [ ] Xác nhận re-import không tạo duplicate hoặc làm mất dữ liệu ngoài chính sách đã duyệt.
- [ ] Chấp thuận hoàn thành KAN-14 hoặc trả lại các mục chưa đạt để xử lý.

**Evidence:** validation report cuối cùng và trạng thái final acceptance.

### Responsibility Boundary

| Activity | Automation | Human |
|---|---|---|
| Đọc GeoJSON, đếm feature, kiểm tra schema | Execute | Review kết quả bất thường |
| Field mapping và region code | Chuẩn bị mapping | Approve trước import |
| Null/duplicate/CRS/geometry checks | Execute | Quyết định cách xử lý exception |
| Import/upsert/transaction/logging | Execute | Không thao tác thủ công từng record |
| SQL validation và spatial query | Execute | Review kết quả và sai lệch |
| QGIS visual QA | Hỗ trợ hiển thị | Xác nhận trực quan |
| Final acceptance | Tổng hợp evidence | Approve/reject |

**Stop conditions:** không import khi chưa qua Gate 1; dừng xử lý tự động khi có exception chưa có quyết định ở Gate 2; không đánh dấu KAN-14 Done khi Gate 3 hoặc Gate 4 chưa được chấp thuận.

---

## 7. Acceptance Criteria

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

## 8. Deliverables

1. Region và building data đã được nạp vào PostGIS.
2. Script/command hoặc hướng dẫn import có thể tái sử dụng.
3. Báo cáo pre-import validation.
4. Báo cáo post-import validation.
5. Kết quả kiểm tra spatial query và index.
6. Tài liệu hướng dẫn re-import và các giới hạn dữ liệu.

## 9. Suggested Commit Sequence

```text
chore(data): prepare GIS import configuration

feat(data): import sample region into PostGIS

feat(data): import sample building footprints into PostGIS

test(data): validate imported GIS records and spatial queries

docs(data): document sample dataset import procedure
```

## 10. Final Definition of Done

KAN-14 hoàn thành khi:
- Dữ liệu từ KAN-6 được import vào schema KAN-13.
- Mapping, định danh OSM và quan hệ region-building chính xác.
- Geometry và CRS vượt qua validation.
- Spatial queries cơ bản hoạt động.
- Re-import không tạo dữ liệu trùng.
- Có hướng dẫn tái tạo và báo cáo kết quả.

KAN-14 chỉ chịu trách nhiệm nạp và kiểm tra dữ liệu nền. Không triển khai API, Cesium visualization hoặc mô phỏng SIR trong ticket này.


## 11. Implementation Notes

- GeoJSON boundary hiện tại thực tế đã là MultiPolygon 2D / EPSG:4326; importer vẫn dùng PROMOTE_TO_MULTI và ST_Multi để bảo đảm tương thích schema.
- Schema KAN-13 đặt tên cột đích là buildings.building_type; thuộc tính nguồn building phải được map vào cột này. Không thay đổi migration KAN-13 chỉ để giữ nguyên tên thuộc tính GeoJSON.
- Region metadata được map đầy đủ: name, area_note, boundary_method, source_reference; region code ổn định là hcm-ben-thanh-1km2.
- Implementation artifacts: scripts/gis/import_sample_to_postgis.sh và ba SQL files trong scripts/gis/sql/.
- Chạy validate trước để hoàn tất Gate 1; chỉ chạy apply sau khi người review chấp thuận. Gate 3 (QGIS Visual QA) và Gate 4 (Final Acceptance) vẫn cần người thực hiện xác nhận.


## 12. Human Review Record (2026-11-12)

- Gate 1: **Approved** by user. Input snapshot (1 region, 431 buildings), region code `hcm-ben-thanh-1km2`, and mapping `building` → `buildings.building_type` accepted.
- Gate 2: **Approved** by user. No blocking validation exceptions. 326 records with nullable `name` are retained as-is.
- Gate 3: **Approved** by user. User completed QGIS visual inspection of the KAN-6 GeoJSON source layers and confirmed the result looks correct.
- Gate 4: **Pending target database import**. CI import, post-import checks, and idempotency test passed on a disposable CI PostGIS database (1 region, 431 buildings). The user's local/project PostGIS database has not been changed. Run the importer against the intended target database and verify it before marking KAN-14 Done.



## 13. Supabase Target Database Import Record (2026-11-12)

**Target:** Supabase project `gis-3d` (`ubltoieatdwdyzowpaig`), PostgreSQL 17.11.

- [x] Confirmed target project is active and initially had no application tables or migration history.
- [x] Enabled PostGIS 3.3.7 in Supabase's dedicated `extensions` schema.
- [x] Applied KAN-13-equivalent `public.regions` and `public.buildings` schema, including UUID primary keys, unique constraints, OSM checks, FK, and GiST/B-tree indexes.
- [x] Enabled RLS on both public tables and left `anon` / `authenticated` without grants or policies. This intentionally keeps raw GIS tables private until an application access model is designed.
- [x] Imported the approved KAN-6 GeoJSON snapshot using client-supplied GeoJSON geometry converted with PostGIS; no server-side file access or local database credentials were used.
- [x] Upsert result: 1 region and 431 buildings.
- [x] Verified: 326 nullable names, 0 missing building types, 0 invalid/empty geometries, 0 wrong geometry type/SRID/dimensions, 0 orphan buildings, 0 duplicate external IDs, 0 buildings outside AOI.
- [x] Verified extent: `BOX(106.693378 10.768079,106.702662 10.777061)`.
- [x] Confirmed both tables exist with RLS enabled and expected columns/FK.
- [x] Human final acceptance: **Approved by user on 2026-11-12**. User reviewed the Supabase Table Editor results and confirmed the data looks correct ("check rất good").

**Supabase security note:** security advisor reports the informational `rls_enabled_no_policy` lint for both tables. This is intentional: RLS is enabled and access for `anon` and `authenticated` is revoked. Do not add broad read policies until the intended client/backend access pattern is specified. Performance advisor reports the new indexes as unused because the dataset has just been imported; this is expected for a fresh, small dataset and is not a failure.

**Environment note:** Supabase runs PostgreSQL 17.11 / PostGIS 3.3.7, whereas the local CI test used PostgreSQL 16 / PostGIS 3.5. Schema and import were adapted to Supabase's `extensions` schema for PostGIS. The GitHub KAN-13 migration remains the source schema definition for the local application; keep deployment-specific extension placement documented.


## 14. Final Acceptance Record (2026-11-12)

- Gate 4: **Approved** by user after reviewing the imported Supabase tables and confirming the result looks correct.
- KAN-14 data import and verification are accepted for the Supabase target.
- Scope note: Gate 3 evidence remains the user's visual QA of the KAN-6 GeoJSON source layers in QGIS; the Supabase table review is recorded separately as final target-data acceptance. No claim is made that PostGIS layers were independently inspected in QGIS.
