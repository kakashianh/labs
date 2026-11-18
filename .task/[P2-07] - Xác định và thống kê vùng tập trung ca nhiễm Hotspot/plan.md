## 1. Objective

Phát hiện các cụm building có nguy cơ/nhiễm cao và có quan hệ lân cận, thống kê mỗi hotspot và trả geometry đại diện để hiển thị trên bản đồ 3D.

Hotspot trong KAN-27 là cụm không gian theo quy tắc Connected Components trên adjacency graph; đây không phải kết quả kiểm định ý nghĩa thống kê.

## 2. Dependencies

| Dependency | Required state |
|---|---|
| KAN-21 | Neighbor/spatial relation; footprint-to-footprint distance |
| KAN-24 | Per-building S/I/R by simulation/time_step |
| KAN-25 | Query contract, completed-run and time_step validation |
| KAN-26 | Risk classification, Unclassified policy and configurable thresholds |

## 3. Scope

### In scope
- Chọn candidate buildings theo risk level và infected count.
- Gom cụm candidate buildings dựa trên footprint adjacency/radius.
- Giữ singleton component theo cấu hình.
- Thống kê member buildings, population, infected count and weighted infection ratio.
- Tạo geometry đại diện từ footprint members bằng PostGIS.
- Filter theo simulation_id/time_step/region.
- Trả effective configuration để client biết chính xác quy tắc đã dùng.
- Deterministic cluster identity và stable response ordering.

### Out of scope
- Getis-Ord Gi*, Moran's I hoặc spatial statistics nâng cao.
- Dự báo hotspot tương lai.
- Tự động gán vùng hành chính.
- Tạo polygon buffer/convex hull không có quy tắc rõ.
- Thay thế risk classification KAN-26.
- Persist lịch sử hotspot riêng; KAN-27 tính kết quả khi query và trả configuration snapshot trong response. Nếu cần lưu lịch sử kết quả, tạo scope/migration riêng.

## 4. Hotspot Detection Contract

### 4.1 Candidate condition

- Mặc định `minimum_risk_level = high`.
- Candidate khi đồng thời thỏa:
  - Building có risk classification hợp lệ từ KAN-26.
  - `risk_level >= minimum_risk_level` theo thứ tự `low < medium < high`.
  - `infected > 0`.
- Cho phép client chọn `medium` hoặc `high`; không cho chọn `unclassified`.
- Building Unclassified hoặc không có population hợp lệ không tham gia clustering.
- Không tự tính threshold/risk riêng trong KAN-27; sử dụng cùng risk configuration/snapshot và quy tắc KAN-26.
- Candidate được xác định độc lập cho từng `simulation_id + time_step + region_id`.

### 4.2 Adjacency and radius

- Hai candidate buildings có cạnh khi shortest footprint-to-footprint distance `d_ij <= radius_m`.
- Dùng đơn vị mét và phép đo geography phù hợp với dữ liệu footprint SRID 4326, nhất quán với KAN-21.
- `radius_m` bắt buộc hữu hạn, lớn hơn 0 và không vượt giới hạn KAN-21 (5,000 m).
- Không dùng infection-pressure weight của KAN-23 để quyết định cạnh; KAN-27 chỉ dùng điều kiện khoảng cách.
- Connected Components có tính bắc cầu: A nối B và B nối C thì A/B/C thuộc cùng component, kể cả khi khoảng cách A-C lớn hơn radius.
- Chỉ candidate nodes mới được tham gia graph; building không phải candidate không thể làm cầu nối giữa hai candidate.

### 4.3 Components and singleton policy

- Mỗi connected component là một hotspot.
- Mặc định `include_singletons = true`; candidate không có cạnh vẫn tạo hotspot với `member_count = 1`.
- Cho phép cấu hình `include_singletons=false` để loại component một member khỏi kết quả.
- Không có candidate hoặc tất cả candidate bị loại bởi singleton policy: trả `hotspots: []`, không phải lỗi.

### 4.4 Deterministic identity and ordering

- Mỗi component không chứa duplicate building.
- `cluster_id` là UUID nhỏ nhất theo thứ tự UUID của các `building_id` trong component. Đây là định danh ổn định trong cùng tập members, không phải ID lưu trữ lâu dài.
- Sắp xếp members theo `building_id ASC`.
- Sắp xếp hotspots theo `cluster_id ASC`.
- Cùng simulation/time_step/filter/config và cùng dữ liệu đầu vào phải cho cùng members, cluster_id và thứ tự kết quả.

## 5. Hotspot Statistics and Geometry Contract

### 5.1 Statistics

Mỗi hotspot trả tối thiểu:
- `cluster_id`, `simulation_id`, `time_step`, `region_id`.
- `member_count`, `population_total`, `infected_total`.
- `infection_ratio = infected_total / population_total`.
- `risk_level` đại diện: tính lại từ aggregate infection_ratio bằng đúng threshold snapshot KAN-26; không lấy max risk level của members.
- `building_ids` theo thứ tự ổn định.

Aggregation rules:
- `population_total = SUM(population)` và `infected_total = SUM(infected)` trên members hợp lệ.
- Ratio là weighted ratio từ tổng infected / tổng population; tuyệt đối không lấy trung bình các ratio cấp building.
- Không round số liệu trong backend/API.
- Candidate Unclassified bị loại trước clustering và không được đưa vào mẫu số.
- Validate S/I/R và population theo invariant KAN-22/KAN-24; dữ liệu vi phạm invariant phải trả lỗi dữ liệu nhất quán, không âm thầm tạo ratio giả.

### 5.2 Geometry

- Geometry đại diện là tập hợp footprint gốc của các member buildings; không tạo buffer, convex hull hoặc vùng bao mới.
- Dùng PostGIS collect và chuẩn hóa thành `MultiPolygon` (ví dụ `ST_Multi(ST_CollectionExtract(ST_Collect(...), 3))`).
- Geometry giữ SRID 4326; không gán SRID mới nếu không thực hiện transform.
- Kiểm tra geometry không null, không empty và valid trước khi serialize.
- API trả GeoJSON geometry theo RFC 7946-compatible coordinates (longitude, latitude); OpenAPI mô tả rõ kiểu `MultiPolygon`.
- Nếu geometry không hợp lệ, xử lý bằng lỗi dữ liệu rõ ràng hoặc repair có quy tắc được test; không âm thầm trả geometry sai.
- Không đơn giản hóa geometry ở backend trong KAN-27; frontend có thể chọn cách render các footprint theo member IDs.

### 5.3 Configuration snapshot

Response trả object `configuration` chứa effective values:
- `algorithm`: `connected_components`.
- `algorithm_version`: ví dụ `hotspot-v1`.
- `minimum_risk_level`.
- Risk threshold snapshot được sử dụng từ KAN-26.
- `radius_m`.
- `include_singletons`.

KAN-27 không persist snapshot/hotspot rows trong database ở scope hiện tại; response snapshot giúp client và log ghi nhận cấu hình thực tế. Kết quả có thể được tái lập khi dữ liệu nguồn và cấu hình không thay đổi.

## 6. API Design

Đề xuất một read endpoint:

```http
GET /api/v1/simulations/{simulation_id}/hotspots
    ?time_step=10
    &region_id={uuid}
    &minimum_risk_level=high
    &radius_m=100
    &include_singletons=true
```

- `simulation_id` và `time_step` bắt buộc.
- `region_id` optional; nếu truyền phải validate tồn tại và filter qua buildings.region_id.
- `minimum_risk_level` optional, default `high`; chỉ chấp nhận `medium|high`.
- `radius_m` optional, default `100 m`; phải ghi rõ default trong OpenAPI và không vượt 5,000 m.
- `include_singletons` optional, default `true`.
- Chỉ cho phép đọc run `completed`; reuse lifecycle/error semantics từ KAN-25.
- Unknown run/time_step/region và invalid query trả error theo AppError convention.
- Response top-level gồm `simulation_id`, `time_step`, `region_id` (nullable), `configuration`, `hotspots`, `hotspot_count`.
- Mỗi hotspot có fields tại mục 5.1 và `geometry` tại mục 5.2.
- Không cần pagination mặc định vì kết quả được nhóm theo components; nếu số hotspot lớn, cân nhắc pagination ở task riêng nhưng không được làm sai aggregate.

## 7. Implementation Plan

### Step 1 – Review Candidate and Adjacency Contracts
- [x] Kiểm tra risk threshold/config snapshot và route KAN-26.
- [x] Kiểm tra footprint geometry, SRID và neighbor distance contract KAN-21.
- [x] Kiểm tra S/I/R, population và completed-run/time_step contract KAN-24/KAN-25.
- [x] Chốt candidate: risk level >= configurable minimum (default High) và infected > 0.
- [x] Chốt singleton default true, có thể override.
- [x] Chốt radius_m default `100 m`, validation và maximum 5,000 m.
- [x] Chốt geometry MultiPolygon GeoJSON SRID 4326.
- [x] Chốt không persist hotspot result ở scope này; trả effective config snapshot.

### Step 2 – Implement Cluster Identification
- [x] Tạo deterministic connected-components service/query.
- [x] Chỉ tạo edge giữa hai candidate nếu footprint distance <= radius_m.
- [x] Đảm bảo candidate filtering xảy ra trước clustering; non-candidate không làm cầu nối.
- [x] Xử lý tính bắc cầu của connected components.
- [x] Không duplicate building trong một cluster.
- [x] Tạo cluster_id bằng UUID nhỏ nhất trong component.
- [x] Stable ordering cho member buildings và hotspot list.
- [x] Xử lý singleton policy, no candidate và disconnected candidates.

### Step 3 – Aggregate Hotspot Statistics
- [x] Đếm member buildings.
- [x] Tính tổng population và infected từ cùng simulation/time_step.
- [x] Tính ratio = total infected / total population, không average per-building ratios.
- [x] Phân loại aggregate risk_level bằng threshold snapshot KAN-26.
- [x] Loại Unclassified khỏi candidate và denominator; validate invariant dữ liệu hợp lệ.
- [x] Gắn simulation_id/time_step/region và configuration snapshot.

### Step 4 – Generate Spatial Output
- [x] Collect footprint members bằng PostGIS.
- [x] Chuẩn hóa output thành MultiPolygon.
- [x] Giữ SRID 4326 và GeoJSON coordinates theo longitude/latitude.
- [x] Kiểm tra geometry validity, null và empty result.
- [x] Không tạo buffer/convex hull hoặc simplify geometry trong task này.
- [x] Kiểm tra kích thước response với dataset hiện có.

### Step 5 – Expose API and Test
- [x] Expose hotspot query theo simulation_id/time_step/region và config.
- [x] Reuse completed-run, time_step, region validation theo KAN-25.
- [x] Cập nhật OpenAPI/docs và error contract.
- [x] Test adjacent candidates tạo cùng cluster.
- [x] Test non-adjacent candidates tạo cluster riêng.
- [x] Test transitive connectivity: A-B, B-C, A-C > radius vẫn là một component.
- [x] Test non-candidate không làm cầu nối hai candidate.
- [x] Test singleton included/excluded.
- [x] Test no candidate và all-singleton-excluded trả empty list.
- [x] Test minimum_risk_level High/Medium và infected=0 bị loại.
- [x] Test radius boundary: distance đúng radius được nối; lớn hơn radius không nối.
- [x] Test deterministic cluster_id và stable ordering.
- [x] Test population/infected/ratio aggregate khớp members, ratio weighted.
- [x] Test aggregate risk level theo KAN-26 thresholds.
- [x] Test geometry type MultiPolygon, SRID 4326, validity và non-empty.
- [x] Test invalid radius/config, unknown simulation/time_step/region và initialized run.
- [x] Chạy backend tests, fmt, Clippy và CI.

## 8. Acceptance Criteria

- [x] Xác định hotspot theo simulation/time_step và filter region.
- [x] Candidate mặc định là building High có infected > 0; có thể cấu hình minimum risk level Medium/High.
- [x] Candidate chỉ được nối theo footprint distance <= radius_m.
- [x] Connected Components có tính bắc cầu và non-candidate không làm cầu nối.
- [x] Singleton được giữ mặc định và có thể cấu hình loại bỏ.
- [x] Mỗi hotspot có member buildings, deterministic cluster_id và stable ordering.
- [x] Population/infected/ratio được aggregate chính xác; ratio dùng SUM(I)/SUM(N).
- [x] Aggregate risk level dùng threshold snapshot KAN-26.
- [x] Geometry là MultiPolygon SRID 4326 hợp lệ, non-empty và renderable bằng CesiumJS.
- [x] Response chứa effective configuration snapshot.
- [x] Không có hotspot trả empty list.
- [x] Có unit/integration tests cho graph, boundary, stats, geometry và API errors.
- [x] Không yêu cầu thuật toán spatial statistics nâng cao.

## 9. Deliverables

1. Hotspot connected-components service/query.
2. Aggregate hotspot statistics and risk level.
3. PostGIS MultiPolygon geometry output.
4. Read API and OpenAPI documentation.
5. Unit/integration tests.

## 10. Definition of Done

KAN-27 hoàn thành khi hệ thống xác định được connected spatial hotspots từ các building đạt candidate condition, thống kê chính xác theo simulation/time_step/region, tạo geometry MultiPolygon SRID 4326 hợp lệ và cung cấp API/configuration snapshot được kiểm thử để frontend CesiumJS sử dụng.

## 11. Implementation Progress

- Đã tạo API `GET /api/v1/simulations/{simulation_id}/hotspots` trên nhánh `feat/p2-07-hotspot-detection`.
- Đã triển khai candidate filtering theo KAN-26 snapshot, Connected Components, singleton policy, deterministic `cluster_id`, aggregate statistics và PostGIS GeoJSON `MultiPolygon`.
- Effective configuration được trả trong response; default `radius_m = 100 m`, maximum `5,000 m`.
- Đã bổ sung OpenAPI và Testcontainers integration test cho transitive component, radius boundary (inclusive), singleton filtering, aggregate statistics và geometry.
- Đã thêm integration test vào CI workflow.
- Các TODO phía trên chưa được tick cho đến khi CI và kiểm thử thực tế xác nhận thành công.

- CI GitHub Actions đã được người dùng xác nhận thành công; các kiểm thử trong workflow hoàn tất.
