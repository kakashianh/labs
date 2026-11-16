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
- [ ] Đọc migration KAN-13 và DB README.
- [ ] Xác nhận SRID/type thực tế của footprint.
- [ ] Xác nhận GiST index hiện hữu và dữ liệu invalid/empty.
- [ ] Kiểm tra query/repository conventions SQLx.
- [ ] Chốt distance semantics và identifier contract.

### Step 2 – Design and Verify Spatial SQL
- [ ] Viết truy vấn ST_DWithin theo đơn vị mét.
- [ ] Tính distance_m nhất quán với predicate filter.
- [ ] Loại center building khỏi danh sách neighbor.
- [ ] Thêm deterministic ordering và result limit.
- [ ] Chạy EXPLAIN/EXPLAIN ANALYZE với dữ liệu hiện có.
- [ ] Chỉ thêm expression index/migration nếu có bằng chứng cần thiết.

### Step 3 – Implement Repository and Service
- [ ] Tạo query/repository function có bind parameters.
- [ ] Xử lý center building không tồn tại.
- [ ] Xử lý radius/limit ngoài miền cho phép.
- [ ] Trả empty list khi không có neighbor.
- [ ] Không nối chuỗi SQL từ user input.

### Step 4 – Expose API
- [ ] Tạo request/response DTO theo API convention.
- [ ] Đăng ký route trong router.
- [ ] Map lỗi sang AppError/error envelope hiện tại.
- [ ] Cập nhật OpenAPI contract.
- [ ] Viết tài liệu request/response.

### Step 5 – Tests and Verification
- [ ] Unit test validation radius và limit.
- [ ] Integration test building gần/trong/ngoài bán kính.
- [ ] Test khoảng cách đúng đơn vị mét.
- [ ] Test center không xuất hiện trong neighbors.
- [ ] Test building không tồn tại và danh sách rỗng.
- [ ] Test geometry boundary và kết quả ổn định.
- [ ] Kiểm tra query plan/index usage khi phù hợp.
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
