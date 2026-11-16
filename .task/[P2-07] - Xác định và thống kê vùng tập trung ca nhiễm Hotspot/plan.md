## 1. Objective

Phát hiện các cụm building có nguy cơ/nhiễm cao và có quan hệ lân cận, thống kê mỗi hotspot và trả geometry đại diện để hiển thị trên bản đồ 3D.

## 2. Dependencies

| Dependency | Required state |
|---|---|
| KAN-21 | Neighbor/spatial relation |
| KAN-24 | Per-building S/I/R by simulation/time_step |
| KAN-25 | Query contract |
| KAN-26 | Risk classification and configurable threshold |

## 3. Scope

### In scope
- Chọn candidate buildings theo threshold/risk level.
- Gom cụm candidate buildings dựa trên adjacency/spatial relationship.
- Thống kê member buildings, population, infected count and ratio.
- Tạo geometry đại diện cho hotspot bằng PostGIS.
- Filter theo simulation_id/time_step/region.
- Xử lý no-hotspot và cluster đơn lẻ.
- Có cấu hình threshold và deterministic cluster identity trong một result snapshot.

### Out of scope
- Getis-Ord Gi*, Moran's I hoặc spatial statistics nâng cao.
- Dự báo hotspot tương lai.
- Tự động gán vùng hành chính.
- Tạo polygon buffer không có quy tắc rõ.
- Thay thế risk classification KAN-26.

## 4. Proposed Clustering Approach

Ưu tiên connected components trên adjacency graph:
1. Candidate = buildings đạt điều kiện risk/infected threshold.
2. Edge nối hai candidate nếu thỏa neighbor relation/radius đã định nghĩa.
3. Mỗi connected component là một hotspot.
4. Singleton component có được giữ hay loại phải cấu hình/chốt rõ.
5. Geometry đại diện có thể là ST_Collect/ST_Union footprint của members; nếu cần polygon đơn giản hóa, phải có tolerance và không làm sai topology.
6. Hotspot stats gồm member_count, population_total, infected_total, infection_ratio.

Cách này là clustering theo quy tắc không gian, không phải kết quả kiểm định thống kê significance.

## 5. Implementation Plan

### Step 1 – Review Candidate and Adjacency Contracts
- [ ] Kiểm tra risk threshold KAN-26.
- [ ] Kiểm tra neighbor graph/radius KAN-21.
- [ ] Chốt candidate condition (High only hay threshold cấu hình).
- [ ] Chốt xử lý singleton components.
- [ ] Chốt geometry output semantics.

### Step 2 – Implement Cluster Identification
- [ ] Tạo deterministic connected-components service/query.
- [ ] Chỉ nối candidate buildings theo adjacency rule đã chốt.
- [ ] Không duplicate building trong một cluster.
- [ ] Xác định stable ordering/identity cho response.
- [ ] Xử lý no candidate và disconnected candidates.

### Step 3 – Aggregate Hotspot Statistics
- [ ] Đếm member buildings.
- [ ] Tính tổng population và infected.
- [ ] Tính ratio = total infected / total population.
- [ ] Xử lý missing/zero population theo KAN-22/KAN-26.
- [ ] Gắn simulation_id/time_step/region.

### Step 4 – Generate Spatial Output
- [ ] Tạo geometry từ member footprints bằng PostGIS.
- [ ] Chốt GeometryCollection/MultiPolygon output contract.
- [ ] Đảm bảo SRID được khai báo và nhất quán.
- [ ] Không serialize geometry quá lớn nếu map chỉ cần boundary đơn giản.
- [ ] Kiểm tra geometry validity và empty result.

### Step 5 – Expose API and Test
- [ ] Expose hotspot query theo simulation_id/time_step/region.
- [ ] Cập nhật OpenAPI/docs.
- [ ] Test adjacent candidates tạo cùng cluster.
- [ ] Test non-adjacent candidates tạo cluster riêng.
- [ ] Test singleton/no-hotspot.
- [ ] Test geometry type/SRID/validity.
- [ ] Test aggregate stats khớp members.
- [ ] Chạy backend tests, fmt, Clippy và CI.

## 6. Acceptance Criteria

- [ ] Xác định được hotspot theo simulation/time_step.
- [ ] Mỗi hotspot có member buildings và aggregate stats.
- [ ] Có geometry để Frontend render.
- [ ] Candidate threshold có thể cấu hình.
- [ ] Không có hotspot trả empty list.
- [ ] Không yêu cầu thuật toán spatial statistics nâng cao.
- [ ] Geometry có SRID/type rõ và tests.

## 7. Deliverables

1. Hotspot clustering service/query.
2. Aggregate stats and spatial geometry output.
3. Read API.
4. Integration tests và docs.

## 8. Definition of Done

KAN-27 hoàn thành khi hệ thống gom được các building đạt ngưỡng thành connected spatial hotspots, thống kê chính xác và trả geometry hợp lệ cho CesiumJS.
