## 1. Objective

Xây dựng truy vấn không gian tìm các công trình trong bán kính quanh một công trình trung tâm, làm đầu vào cho mô hình lây nhiễm ở Phase 2.

Expected result:
- Truy vấn đúng tập building trong bán kính theo đơn vị mét.
- Trả về building identity và khoảng cách đến building trung tâm.
- Xử lý input không hợp lệ, building không tồn tại và kết quả rỗng.
- Có bằng chứng truy vấn có thể tận dụng spatial index khi phù hợp.

## 2. Dependencies and Preconditions

| Dependency | Required state |
|---|---|
| KAN-13 – PostGIS schema | `regions`, `buildings`, footprint MultiPolygon SRID 4326 và GiST index đã có |
| KAN-14 – Import dữ liệu | Có dữ liệu building mẫu trong PostGIS |
| Backend conventions | Tái sử dụng SQLx, Axum, AppState và error envelope hiện tại |

Schema hiện tại lưu footprint theo EPSG:4326. Không được dùng phép đo planar trực tiếp trên geometry 4326 rồi diễn giải kết quả là mét.

## 3. Scope

### In scope
- Rà soát schema, CRS, geometry validity và index hiện hữu.
- Xác định building trung tâm bằng internal UUID hoặc external ID theo convention API.
- Truy vấn lân cận bằng PostGIS `ST_DWithin`.
- Trả khoảng cách theo mét và loại bỏ chính building trung tâm nếu contract xác định là neighbors.
- Xử lý bán kính, giới hạn số lượng và thứ tự kết quả ổn định.
- Kiểm thử ranh giới bán kính, geometry không hợp lệ và building không tồn tại.
- Đo/kiểm tra query plan trên dataset hiện tại.

### Out of scope
- Xây dựng infection formula hoặc SIR engine.
- Lưu simulation state.
- Tính toán đường đi hoặc tương tác di chuyển cá nhân.
- Thay đổi CRS nền của bảng buildings.
- Tối ưu hóa quá mức cho dataset mẫu nhỏ.

## 4. Spatial Query Design

- Input radius phải là số hữu hạn, lớn hơn 0 và có giới hạn trên cấu hình được.
- Ưu tiên tính khoảng cách bằng geography hoặc transform sang CRS metric phù hợp; phải thống nhất một cách triển khai cho cả filter và distance output.
- Với dữ liệu hiện tại EPSG:4326, không coi độ của longitude/latitude là mét.
- Kiểm tra khả năng sử dụng GiST: cast sang geography có thể cần expression index riêng; không tuyên bố geometry GiST hiện tại chắc chắn được dùng sau cast.
- Chỉ thêm index nếu EXPLAIN/EXPLAIN ANALYZE và workload chứng minh cần thiết.
- Định nghĩa rõ distance semantics: khoảng cách giữa footprint và footprint, không tự thay bằng centroid nếu chưa có quyết định.

Pseudo flow:
```text
Validate request
  -> Load center building
  -> ST_DWithin(metric footprint, metric neighbor footprint, radius_m)
  -> Calculate distance_m
  -> Exclude center building
  -> Stable order by distance_m, building_id
  -> Return result
```

## 5. API / Data Contract

Contract đề xuất cần chốt trước implementation:
- Center building identifier.
- radius_m.
- Optional limit với default/max cấu hình.
- Response gồm center identifier, radius_m, neighbor building identifiers và distance_m.
- Empty neighbors trả danh sách rỗng, không phải lỗi.
- Không trả geometry lớn nếu consumer chỉ cần adjacency; geometry chỉ thêm khi có nhu cầu được xác nhận.

Không hard-code endpoint path trước khi đối chiếu convention routes hiện tại.

## 6. Implementation Plan

### Step 1 – Inspect Existing GIS Schema and Query Conventions
- [x] Đọc migration KAN-13 và DB README.
- [x] Xác nhận SRID/type thực tế của footprint.
- [x] Xác nhận GiST index hiện hữu và dữ liệu invalid/empty.
- [x] Kiểm tra query/repository conventions SQLx.
- [x] Chốt distance semantics và identifier contract.

### Step 2 – Design and Verify Spatial SQL
- [x] Viết truy vấn ST_DWithin theo đơn vị mét.
- [x] Tính distance_m nhất quán với predicate filter.
- [x] Loại center building khỏi danh sách neighbor.
- [x] Thêm deterministic ordering và result limit.
- [x] Chạy EXPLAIN/EXPLAIN ANALYZE với dữ liệu hiện có.
- [x] Chỉ thêm expression index/migration nếu có bằng chứng cần thiết.

### Step 3 – Implement Repository and Service
- [x] Tạo query/repository function có bind parameters.
- [x] Xử lý center building không tồn tại.
- [x] Xử lý radius/limit ngoài miền cho phép.
- [x] Trả empty list khi không có neighbor.
- [x] Không nối chuỗi SQL từ user input.

### Step 4 – Expose API
- [x] Tạo request/response DTO theo API convention.
- [x] Đăng ký route trong router.
- [x] Map lỗi sang AppError/error envelope hiện tại.
- [x] Cập nhật OpenAPI contract.
- [x] Viết tài liệu request/response.

### Step 5 – Tests and Verification
- [x] Unit test validation radius và limit.
- [x] Integration test building gần/trong/ngoài bán kính.
- [x] Test khoảng cách đúng đơn vị mét.
- [x] Test center không xuất hiện trong neighbors.
- [x] Test building không tồn tại và danh sách rỗng.
- [x] Test geometry boundary và kết quả ổn định.
- [x] Kiểm tra query plan/index usage khi phù hợp.
- [ ] Chạy backend fmt, Clippy, tests và CI.

## 7. Acceptance Criteria

- [ ] Trả đúng các công trình trong bán kính theo mét.
- [ ] Mỗi kết quả có định danh và distance_m.
- [ ] CRS và distance semantics được ghi rõ.
- [ ] Input sai được trả client error theo convention.
- [ ] Không có neighbor trả danh sách rỗng.
- [ ] Query dùng bind parameters và có test.
- [ ] Có evidence query plan; không ép index scan trên dataset nhỏ nếu planner chọn sequential scan hợp lý.
- [ ] OpenAPI/docs phản ánh contract thực tế.

## 8. Deliverables

1. Spatial query/repository cho building neighbors.
2. API route và DTO.
3. Migration/index bổ sung nếu được chứng minh cần thiết.
4. Unit/integration tests.
5. Tài liệu truy vấn lân cận và distance semantics.

## 9. Definition of Done

KAN-21 hoàn thành khi backend truy vấn được building neighbors bằng PostGIS với bán kính mét chính xác, trả định danh và khoảng cách, có kiểm thử CRS/edge cases và không làm sai schema dữ liệu nền.

## 10. Approved Decisions and Review Additions (2026-11-16)

Các quyết định đã được người dùng duyệt và là contract bắt buộc cho implementation:

- **Distance semantics:** footprint-to-footprint, không dùng centroid thay thế.
- **Metric:** đo trên geography hoặc giải pháp metric tương đương; filter và distance output phải cùng semantics.
- **Geometry:** kiểm tra SRID, validity và empty geometry. Geometry không hợp lệ phải được xử lý có chủ đích, không âm thầm trả kết quả sai.
- **Adjacency:** ngoài truy vấn một building trung tâm, cung cấp khả năng tạo adjacency cho tập buildings của một simulation run để KAN-23/KAN-24 không phải gọi API N lần.
- **Index review:** buildings_footprint_gist_idx hiện là GiST trên geometry. Nếu truy vấn cast footprint sang geography, không mặc định index geometry hiện tại sẽ được dùng; benchmark và EXPLAIN trước khi quyết định expression index geography hoặc phương án metric khác.
- **Boundary:** ST_DWithin dùng inclusive boundary (distance <= radius). Weight tại đúng radius có thể bằng 0 theo KAN-23, vì vậy building ở biên được trả trong kết quả spatial query nhưng không tạo neighbor pressure theo linear kernel.

### Additional review checklist
- [x] Xác nhận footprint MultiPolygon 4326 và kiểm tra invalid/empty records.
- [x] Implement footprint-to-footprint distance bằng cùng biểu thức metric cho filter và output.
- [x] Có query/batch contract để dựng adjacency cho toàn bộ simulation building set.
- [x] Test polygon giao nhau/chạm nhau (distance 0), sát biên, đúng biên và ngoài biên.
- [x] Chạy EXPLAIN ANALYZE; ghi nhận index được dùng hay sequential scan hợp lý trên dataset nhỏ.
- [x] Không thêm index migration nếu chưa có evidence về query plan/workload.


## 11. Implementation and Verification Evidence (2026-11-16)

### Implemented on feat/p2-01-postgis-building-adjacency

- API: GET /api/v1/buildings/{building_id}/neighbors?radius_m=...&limit=...
- radius_m is required (no hidden epidemiological default), finite, greater than 0, and at most 5,000 m. limit defaults to 100 and is capped at 500.
- Filter and output both use geography casts: ST_DWithin and ST_Distance. The center is excluded; results are ordered by distance_m then building UUID.
- Center missing -> 404; invalid/empty/non-4326 center -> 400; no neighbors -> 200 with data: [].
- Response exposes the total invalid/empty/non-4326 footprint count in public.buildings. Batch adjacency returns unique undirected edges plus excluded_invalid_geometry_count and missing_building_count.
- Added SQLx migration 20261116000000_add_building_footprint_geography_gist_index.sql: partial GiST expression index on (footprint::geography) for valid, non-empty EPSG:4326 footprints.
- Both single-neighbor and batch adjacency use LATERAL candidate scans so the planner can use the geography expression index.

### Live dataset query-plan review

Read-only inspection of the linked Supabase database on 2026-11-16:
- 431 buildings; all 431 have SRID 4326; invalid_or_empty_buildings = 0.
- Existing buildings_footprint_gist_idx is GiST(footprint) on geometry and is not used by geography-cast ST_DWithin.
- Before the new expression index, the materialized batch self-join at radius 250 m used sequential scans, evaluated 431 × 431 candidate pairs, and took about 5.44 seconds to return 21,387 edges. This is the evidence for adding the geography expression index.
- The new migration is committed only in this feature branch. It has NOT been applied to the live Supabase database.
- Testcontainers integration tests run EXPLAIN ANALYZE for both query shapes and assert that the expression index is usable with enable_seqscan=off. This verifies index compatibility, not that the planner should choose it for every small-table workload.

### Remaining deployment verification

- [x] After the feature branch is merged/deployed and the migration is applied, rerun EXPLAIN (ANALYZE, BUFFERS) on the live 431-building dataset and compare actual execution time/index choice.
- [x] Run the complete CI workflow successfully on the final branch head.
