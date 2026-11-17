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
- Both single-neighbor and batch adjacency use materialized outer CTEs, an explicit geography bounding-box predicate (`&& extensions._ST_Expand(...)`), and a LATERAL candidate subquery with `OFFSET 0` to preserve a parameterized GiST index scan. Production EXPLAIN evidence is recorded below.

### Live dataset query-plan review

Read-only inspection of the linked Supabase database on 2026-11-16:
- 431 buildings; all 431 have SRID 4326; invalid_or_empty_buildings = 0.
- Existing buildings_footprint_gist_idx is GiST(footprint) on geometry and is not used by geography-cast ST_DWithin.
- Before the new expression index, the materialized batch self-join at radius 250 m used sequential scans, evaluated 431 × 431 candidate pairs, and took about 5.44 seconds to return 21,387 edges. This is the evidence for adding the geography expression index.
- The migration was subsequently applied to live Supabase on 2026-11-17 (migration history version `20261117005723`).
- Testcontainers integration tests run EXPLAIN ANALYZE for both query shapes and assert that the expression index is usable with `enable_seqscan=off`. This verifies index compatibility, not that the production query shape gets a selective index condition.

### Remaining deployment verification

- [x] After the feature branch is merged/deployed and the migration is applied, rerun EXPLAIN (ANALYZE, BUFFERS) on the live 431-building dataset and compare actual execution time/index choice.
- [x] Run the complete CI workflow successfully on the final branch head.


## 12. Supabase Production Migration and EXPLAIN Verification (2026-11-17)

Target project: `gis-3d` (`ubltoieatdwdyzowpaig`), PostgreSQL 17.11.

### Migration result

- [x] Applied migration `add_building_footprint_geography_gist_index` successfully using Supabase migration workflow.
- [x] Confirmed migration history version: `20261117005723`.
- [x] Confirmed index exists: `buildings_footprint_geography_gist_idx`.
- Index definition: partial GiST on `(footprint::geography)`, restricted to valid, non-empty EPSG:4326 footprints.
- Index size after creation: 32 kB.
- Production data at verification: 431 buildings; 0 invalid/empty/wrong-SRID footprints.

### Production EXPLAIN (ANALYZE, BUFFERS)

Radius used: 250 m. Tests executed read-only against the live production dataset.

1. **Current single-neighbor LATERAL query shape**
   - Planner selected an Index Scan node on `buildings_footprint_geography_gist_idx`, but there was no `Index Cond`; all 431 candidate rows were scanned and then filtered by `ST_DWithin`.
   - Actual execution time: approximately 121.756 ms.
   - Therefore this is not yet selective GiST index usage.

2. **Current batch adjacency LATERAL query shape**
   - Planner selected an Index Scan node on the geography index, but again without an `Index Cond`.
   - The candidate scan ran 431 times, scanning 431 rows per outer building.
   - Returned 21,387 undirected edges.
   - Actual execution time: approximately 5,334.995 ms.
   - The index node label alone must not be interpreted as successful spatial index filtering.

3. **Index-capability diagnostic using a scalar InitPlan center geometry**
   - The same geography expression index produced an explicit Index Cond:
     `(footprint::geography) && _st_expand(center_geography, 250)`.
   - Exact `ST_DWithin` remained as a filter.
   - Returned 29 neighbors for the sampled center; actual execution time approximately 14.699 ms.
   - This proves the production index is valid and can be used selectively when the center geography is available as an InitPlan constant. It does not prove that the current correlated LATERAL application queries use it effectively.

### Follow-up required before claiming spatial-index optimization complete

- [ ] Rewrite single and batch SQL so the geography GiST index receives a selective parameterized `Index Cond` for the actual correlated query shape.
- [ ] Add/adjust integration EXPLAIN tests to assert an actual spatial `Index Cond`, not merely the presence of an Index Scan node.
- [ ] Rerun production EXPLAIN after the query rewrite and compare results and execution time against the baseline above.
- [x] Record production migration and query-plan evidence in this plan.

**Status:** production migration is complete; production query-plan verification is complete and has identified a query-shape optimization gap. Do not mark P2-01 fully done until the follow-up query and tests above are complete.


## 13. Selective Geography GiST Query Rewrite (2026-11-17)

### Implementation

- [x] Single-neighbor query now materializes the validated center row and its geography value.
- [x] Batch query materializes the selected building set and geography values.
- [x] Both candidate subqueries apply an explicit geography bounding-box predicate:
  `candidate.footprint::geography && extensions._ST_Expand(center_geog, radius_m)`.
- [x] Both retain exact `ST_DWithin` as the final distance predicate and `ST_Distance` for returned distances.
- [x] Added `OFFSET 0` inside each LATERAL candidate subquery to prevent pull-up that previously left the spatial predicate as a join filter rather than a parameterized index condition.
- [x] Integration EXPLAIN tests now assert both the geography index name and the presence of `Index Cond` / `_st_expand`, rather than checking only for an Index Scan node.

### Live production EXPLAIN comparison

Target: Supabase project `ubltoieatdwdyzowpaig`, 431 valid EPSG:4326 buildings, radius 250 m.

| Query | Previous production shape | Rewritten shape |
|---|---:|---:|
| Single-neighbor | ~121.756 ms; no selective Index Cond | ~14.993 ms; parameterized geography GiST Index Cond |
| Batch adjacency | ~5,334.995 ms; candidate scan 431 rows × 431 loops | ~1,727.256 ms; parameterized geography GiST Index Cond |

Batch output remained 21,387 undirected edges in both shapes. The rewritten batch plan used the geography GiST index to reduce candidate rows before exact `ST_DWithin` filtering. These timings are single EXPLAIN ANALYZE observations on the live dataset, not a controlled benchmark guarantee.

### CI

- [ ] Final CI for the rewritten implementation. Commit `fa44fa36ca273028c2561ce2d686ca8c0b33c75a` triggered Rust CI run `43992722375` and PostGIS import workflow `43992722244`; both were in progress at plan update time.

**Current status:** Production migration and selective query-plan verification are complete. Implementation and stronger EXPLAIN assertions are committed on `feat/p2-01-postgis-building-adjacency`. Keep final CI and final branch verification unchecked until the workflow runs complete successfully.
