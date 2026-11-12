# KAN-6 – Phân tích mô hình dữ liệu GIS và chuẩn bị dữ liệu đô thị mẫu

## 1. Objective

Chuẩn bị bộ dữ liệu GIS 2D từ OpenStreetMap (OSM) cho một khu vực đô thị mẫu nhỏ quanh chợ Bến Thành, Quận 1 cũ, TP.HCM. Bộ dữ liệu gồm ranh giới AOI và footprint công trình, làm đầu vào cho KAN-13 (thiết kế schema PostGIS) và KAN-14 (import dữ liệu).

**Phương án đã duyệt**
- Khu vực: khoảng 1 km² (xấp xỉ 1 km × 1 km) quanh chợ Bến Thành.
- Tâm AOI: 10.77257° N, 106.69802° E.
- Phạm vi WGS 84: 106.693378–106.702662° E, 10.768079–10.777061° N.
- Ranh giới: polygon chữ nhật tự định nghĩa cho mục đích nghiên cứu, không phải ranh giới hành chính.
- Nguồn building: Geofabrik Vietnam OSM extract, tải tại thời điểm chạy workflow.
- Mức chi tiết: 2D building footprints; không dựng khối 3D hoặc suy diễn chiều cao/số tầng.
- Định dạng bàn giao: GeoJSON RFC 7946, WGS 84 (EPSG:4326).
- Công cụ: GDAL/OGR, Osmium và QGIS; không thêm Python service vào backend Rust + Axum.

## 2. Scope

### In scope
- Tạo polygon AOI cố định quanh chợ Bến Thành.
- Tải Vietnam OSM PBF từ Geofabrik.
- Trích xuất đối tượng có tag `building=*` thành footprint.
- Clip footprint theo AOI.
- Chuẩn hóa ID, thuộc tính, geometry và CRS.
- Kiểm tra dữ liệu bằng GDAL/OGR và QGIS.
- Tài liệu hóa nguồn, giấy phép, metadata, data dictionary, chất lượng và quy trình tái tạo.

### Out of scope
- Khôi phục ranh giới lịch sử toàn TP.HCM hoặc truy vấn OSM attic.
- Thiết kế hoặc tạo bảng PostGIS (KAN-13).
- Import dữ liệu vào database (KAN-14).
- Xây dựng GIS API hoặc hiển thị CesiumJS.
- Tạo mô hình 3D, chiều cao, số tầng, dân số hoặc dữ liệu dịch tễ.
- Cam kết OSM có đầy đủ 100% công trình trong AOI.

## 3. Data contract

### 3.1 Layers

| Layer | Geometry | Mục đích |
|---|---|---|
| `hcm_boundary` | Polygon | Polygon AOI quanh chợ Bến Thành |
| `hcm_buildings` | MultiPolygon | Footprint công trình trong AOI |

### 3.2 Building attributes

| Field | Kiểu logic | Nullable | Quy tắc |
|---|---|---:|---|
| `building_id` | string | No | `way/{osm_way_id}` hoặc `relation/{osm_id}` |
| `osm_id` | integer/string theo driver | Yes | ID relation nếu feature là relation |
| `osm_way_id` | integer/string theo driver | Yes | ID way nếu feature là closed way |
| `osm_type` | string | No | `way` hoặc `relation` |
| `name` | string | Yes | Tag OSM `name` |
| `building` | string | Yes | Giá trị tag OSM `building` |
| `geometry` | MultiPolygon, 2D | No | Footprint; giữ nguyên hình học nhiều phần |

Không tự tạo `height`, `building_levels`, dân số hoặc thuộc tính mô phỏng. Không dùng FID sinh tự động làm định danh nghiệp vụ.

### 3.3 CRS

- AOI được định nghĩa theo WGS 84 longitude/latitude (EPSG:4326).
- GeoJSON đầu ra tuân thủ RFC 7946 / EPSG:4326.
- Khi đo diện tích/khoảng cách, dùng CRS chiếu phù hợp hoặc phép đo geodesic; không tính diện tích trực tiếp từ đơn vị độ.

## 4. Implementation plan

### Step 1 – Define the study area
- [x] Chốt AOI khoảng 1 km² quanh chợ Bến Thành.
- [x] Định nghĩa tâm, bbox và polygon cố định trong workflow.
- [x] Loại bỏ phụ thuộc vào OSM relation lịch sử cấp thành phố.

### Step 2 – Acquire OSM PBF
- [ ] Tải Vietnam extract: https://download.geofabrik.de/asia/vietnam-latest.osm.pbf.
- [ ] Lưu tại `data/sample/source/vietnam-latest.osm.pbf` (gitignored).
- [ ] Ghi ngày tải, URL, dung lượng và SHA-256.

### Step 3 – Inspect source data
- [ ] Kiểm tra phiên bản GDAL và khả năng đọc OSM PBF.
- [ ] Dùng `ogrinfo` kiểm tra layer `multipolygons`, field `building`, `osm_id`, `osm_way_id`.
- [ ] Kiểm tra CRS, extent, số feature và thuộc tính.
- [ ] Ghi lại bất thường hoặc thiếu dữ liệu trước xử lý.

### Step 4 – Extract, clip and normalize
- [x] Có script `scripts/gis/prepare_osm_data.sh` để chuẩn hóa AOI, trích xuất `building=*`, clip và xuất GeoJSON (chưa xác nhận end-to-end).
- [ ] Chạy pipeline trên dữ liệu OSM thật.
- [ ] Tạo `building_id` theo namespace OSM way/relation.
- [ ] Giữ ID nguồn để truy vết.
- [ ] Giữ Polygon/MultiPolygon dưới dạng MultiPolygon.
- [ ] Xuất GeoJSON RFC 7946 / EPSG:4326.

### Step 5 – Validate with GDAL and QGIS
- [ ] Đọc lại hai GeoJSON bằng `ogrinfo`.
- [ ] Xác nhận CRS và geometry type.
- [ ] Kiểm tra ID null/trùng, geometry rỗng/invalid và feature ngoài AOI.
- [ ] Mở dữ liệu trong QGIS, kiểm tra clipping và một số vùng đại diện.
- [ ] Ghi số liệu thực tế vào quality report; không điền số ước lượng.

### Step 6 – Document and handoff
- [x] Cập nhật README, nguồn AOI và hướng dẫn pipeline theo phạm vi mới.
- [ ] Hoàn thiện metadata và quality report bằng thông tin từ workflow thực tế.
- [ ] Bàn giao layers, fields, types, CRS, source IDs và caveats cho KAN-13.
- [ ] Kiểm tra tái tạo trên môi trường sạch.

## 5. Repository layout

```text
data/sample/
├── source/
│   ├── README.md
│   └── hcm_boundary.geojson       # sinh từ AOI đã duyệt
└── processed/
    ├── README.md
    ├── hcm_boundary.geojson
    └── hcm_buildings.geojson

.task/[P1-04].../deliverables/
├── metadata.md
├── data_dictionary.md
└── data_quality_report.md

scripts/gis/
├── README.md
└── prepare_osm_data.sh
```

Không tạo dữ liệu building giả. Chỉ bàn giao GeoJSON sau khi pipeline đã chạy và QA.

## 6. Acceptance criteria

- [ ] AOI đúng vị trí và kích thước xấp xỉ 1 km² quanh chợ Bến Thành.
- [ ] Có footprint thật từ OSM tag `building=*`, được clip theo AOI.
- [ ] CRS đầu ra EPSG:4326 / RFC 7946.
- [ ] `building_id` không null và duy nhất; giữ OSM ID gốc.
- [ ] Geometry lỗi và feature ngoài phạm vi được thống kê, xử lý hoặc giải trình.
- [ ] Có metadata, data dictionary, quality report và hướng dẫn tái tạo.
- [ ] KAN-13 nhận được data contract và các giới hạn dữ liệu.

## 7. Deliverables

1. [ ] `hcm_boundary.geojson` và `hcm_buildings.geojson` từ pipeline thực tế.
2. [x] Workflow, script và README đã cập nhật cho AOI nhỏ.
3. [ ] Metadata nguồn, phạm vi, CRS, giấy phép và checksum.
4. [ ] Data dictionary và quality report đối chiếu dữ liệu thực tế.
5. [ ] Chạy thử thành công và xác nhận có thể tái tạo.

## 8. Suggested commit sequence

```text
docs(gis): define Ben Thanh sample AOI and data contract
build(gis): add reproducible OSM extraction pipeline
docs(gis): document dataset metadata and quality results
data(gis): add validated Ben Thanh GeoJSON sample
```

## 9. Definition of Done

KAN-6 chỉ hoàn thành khi AOI được xác minh, building footprints thật từ OSM đã được trích xuất/clip/chuẩn hóa, kết quả đọc lại được bằng công cụ GIS, chất lượng và giới hạn được báo cáo, và KAN-13 có đủ thông tin để thiết kế schema PostGIS.

**Trạng thái hiện tại:** workflow và tài liệu đã chuyển sang AOI khoảng 1 km² quanh chợ Bến Thành. Chưa chạy workflow thành công; chưa có building GeoJSON hoặc số liệu QA thực tế.