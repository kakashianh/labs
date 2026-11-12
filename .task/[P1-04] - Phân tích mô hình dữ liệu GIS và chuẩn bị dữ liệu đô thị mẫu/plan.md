# KAN-6 – Phân tích mô hình dữ liệu GIS và chuẩn bị dữ liệu đô thị mẫu

## 1. Objective

Chuẩn bị bộ dữ liệu GIS 2D từ OpenStreetMap (OSM) cho một khu vực đô thị mẫu nhỏ quanh chợ Bến Thành, Quận 1 cũ, TP.HCM. Bộ dữ liệu gồm polygon AOI và footprint công trình, làm đầu vào cho KAN-13 (thiết kế schema PostGIS) và KAN-14 (import dữ liệu).

**Phạm vi đã được duyệt**
- Khu vực: khoảng 1 km² (xấp xỉ 1 km × 1 km) quanh chợ Bến Thành.
- Tâm AOI: 10.77257° N, 106.69802° E.
- Bbox WGS 84: 106.693378–106.702662° E, 10.768079–10.777061° N.
- Ranh giới: polygon chữ nhật tự định nghĩa cho mục đích nghiên cứu, không phải ranh giới hành chính.
- Nguồn building: Geofabrik Vietnam OSM extract tải tại thời điểm chạy workflow.
- Mức chi tiết: footprint 2D; không dựng khối 3D hoặc suy diễn chiều cao/số tầng.
- Định dạng: GeoJSON RFC 7946, WGS 84 (EPSG:4326).

## 2. Scope

### In scope
- Tạo polygon AOI cố định quanh chợ Bến Thành.
- Tải Vietnam OSM PBF từ Geofabrik.
- Trích xuất đối tượng có tag `building=*`, clip footprint theo AOI.
- Chuẩn hóa ID, thuộc tính, geometry và CRS.
- Kiểm tra đầu ra bằng GDAL/OGR và QGIS.
- Tài liệu hóa nguồn, giấy phép, metadata, data dictionary, chất lượng và quy trình tái tạo.

### Out of scope
- Khôi phục ranh giới lịch sử toàn TP.HCM hoặc truy vấn OSM attic.
- Thiết kế/tạo bảng PostGIS (KAN-13) hoặc import database (KAN-14).
- Xây dựng GIS API hoặc hiển thị CesiumJS.
- Tạo mô hình 3D, chiều cao, số tầng, dân số hoặc dữ liệu dịch tễ.
- Cam kết OSM có đầy đủ 100% công trình trong AOI.

## 3. Data contract

### 3.1 Layers

| Layer | Geometry | CRS | Mục đích |
|---|---|---|---|
| `hcm_boundary` | Polygon | EPSG:4326 | AOI mẫu quanh chợ Bến Thành |
| `hcm_buildings` | MultiPolygon | EPSG:4326 | Footprint công trình được clip trong AOI |

### 3.2 Building attributes

| Field | Kiểu logic | Nullable | Quy tắc |
|---|---|---:|---|
| `building_id` | string | No | `way/{osm_way_id}` hoặc `relation/{osm_id}` |
| `osm_type` | string | No | `way` hoặc `relation` |
| `osm_id` | integer/string theo driver | Yes | ID relation nếu feature là relation |
| `osm_way_id` | integer/string theo driver | Yes | ID way nếu feature là closed way |
| `name` | string | Yes | Tag OSM `name` |
| `building` | string | Yes | Giá trị tag OSM `building` |
| `geometry` | MultiPolygon, 2D | No | Footprint; giữ hình học nhiều phần |

Không tự tạo `height`, `building_levels`, dân số hoặc thuộc tính mô phỏng. Không dùng FID sinh tự động làm định danh nghiệp vụ.

### 3.3 CRS and measurement
- GeoJSON đầu ra tuân thủ RFC 7946 / EPSG:4326 (longitude, latitude).
- Khi lưu trữ PostGIS, KAN-13 cần quyết định SRID lưu trữ dựa trên data contract; nếu cần đo khoảng cách/diện tích theo mét, dùng CRS chiếu phù hợp hoặc geography, không đo trực tiếp trên đơn vị độ.
- Dữ liệu hiện tại là 2D, không có Z/height được xác thực.

## 4. Implementation plan and execution result

### Step 1 – Define the study area
- [x] Chốt AOI khoảng 1 km² quanh chợ Bến Thành.
- [x] Định nghĩa tâm, bbox và polygon cố định trong workflow.
- [x] Loại bỏ phụ thuộc vào OSM relation lịch sử cấp thành phố.
- [x] Xác nhận AOI có 1 polygon feature.

### Step 2 – Acquire OSM PBF
- [x] Tải Vietnam extract từ https://download.geofabrik.de/asia/vietnam-latest.osm.pbf.
- [x] Workflow sử dụng PBF trong bước trích xuất; file nguồn lớn không được commit vào repository.
- [x] Ghi retrieval time, dung lượng và SHA-256 trong metadata.

**Kết quả lần chạy thành công**
- Retrieval UTC: `2026-11-12T03:29:24Z`.
- Kích thước PBF: `332,519,570` bytes.
- SHA-256: `5eb2bce83b0e93d43119de637311066ecc8815e045539514cc528a686731c7a8`.

### Step 3 – Inspect source data
- [x] Kiểm tra phiên bản GDAL và khả năng đọc OSM PBF.
- [x] Sử dụng GDAL/OGR để đọc và xử lý dữ liệu OSM.
- [x] Ghi phiên bản GDAL và provenance nguồn vào metadata.

**Môi trường lần chạy thành công:** GDAL 3.12.2.

### Step 4 – Extract, clip and normalize
- [x] Chạy pipeline trên OSM PBF thật.
- [x] Trích xuất đối tượng có tag `building=*` và clip theo AOI.
- [x] Tạo `building_id` theo namespace OSM way/relation.
- [x] Giữ các trường ID nguồn để truy vết.
- [x] Chuẩn hóa footprint thành MultiPolygon 2D.
- [x] Xuất GeoJSON RFC 7946 / EPSG:4326.

**Kết quả:** 1 AOI boundary feature và 431 building features. File `hcm_buildings.geojson` có dung lượng 192,337 bytes.

### Step 5 – Validate with GDAL and QGIS
- [x] Đọc lại hai GeoJSON bằng `ogrinfo`.
- [x] Xác nhận layer đầu ra dùng EPSG:4326; building geometry là MultiPolygon.
- [x] Xác nhận `building_id` không null: 0.
- [x] Xác nhận nhóm `building_id` trùng: 0.
- [x] Mở cả hai layer trong QGIS và kiểm tra trực quan; người thực hiện xác nhận hiển thị/vị trí/phạm vi ở mức trực quan là ổn.

**Giới hạn của kết quả QA:** báo cáo tự động hiện có ghi nhận feature count, CRS/geometry inspection và kiểm tra ID. Không tự suy diễn rằng đã có thống kê riêng về số geometry invalid/empty hoặc số feature nằm ngoài AOI nếu report/log chưa cung cấp con số đó. Kiểm tra trực quan QGIS không thay thế kiểm tra topology tự động.

### Step 6 – Document and handoff
- [x] Cập nhật README, nguồn AOI và hướng dẫn pipeline theo phạm vi đã duyệt.
- [x] Ghi metadata nguồn, thời điểm tải, checksum và số lượng feature thực tế.
- [x] Cập nhật data dictionary và quality report theo kết quả workflow.
- [x] Ghi rõ giới hạn: dữ liệu OSM hiện thời, footprint 2D, không đảm bảo completeness.
- [x] Chuẩn bị data contract để KAN-13 thiết kế schema PostGIS.
- [ ] Kiểm thử tái tạo độc lập trên một môi trường sạch (nếu cần chứng minh reproducibility ngoài workflow CI hiện tại).

## 5. Repository layout

```text
data/sample/
├── source/
│   ├── README.md
│   └── hcm_boundary.geojson
└── processed/
    ├── README.md
    ├── hcm_boundary.geojson
    ├── hcm_buildings.geojson
    ├── metadata.md
    └── quality-summary.md

.task/[P1-04].../deliverables/
├── metadata.md
├── data_dictionary.md
└── data_quality_report.md

scripts/gis/
├── README.md
└── prepare_osm_data.sh
```

Không commit PBF 332 MB vào repository; dùng workflow artifact và các GeoJSON đã chuẩn hóa. Không tạo building giả.

## 6. Acceptance criteria

- [x] AOI đúng vị trí quanh chợ Bến Thành, kích thước xấp xỉ 1 km².
- [x] Có footprint thật từ OSM tag `building=*`, đã clip theo AOI.
- [x] CRS đầu ra EPSG:4326 / RFC 7946.
- [x] `building_id` không null và duy nhất; giữ OSM ID gốc.
- [x] Dữ liệu mở đọc được bằng GDAL/OGR.
- [x] Hai layer đã được mở và kiểm tra trực quan trong QGIS.
- [x] Có metadata, data dictionary, quality report và hướng dẫn tái tạo.
- [x] KAN-13 có data contract và giới hạn dữ liệu để thiết kế schema.
- [ ] Kiểm thử tái tạo độc lập trên môi trường sạch.

## 7. Deliverables and evidence

1. [x] `data/sample/processed/hcm_boundary.geojson`.
2. [x] `data/sample/processed/hcm_buildings.geojson` (431 features).
3. [x] Workflow/script/README cho AOI nhỏ.
4. [x] Metadata, data dictionary và quality report.
5. [x] Workflow thành công và kết quả được kiểm tra trực quan trong QGIS.
6. [ ] Tái tạo độc lập trên môi trường sạch (không phải điều kiện cản trở KAN-13 bắt đầu thiết kế).

**Evidence**
- Successful workflow run: https://github.com/kakashianh/labs/actions/runs/43148469271
- Data commit: https://github.com/kakashianh/labs/commit/d10eab3cdeb99c079a8bca46bace9f6af949891b

## 8. Suggested commit sequence

```text
docs(gis): define Ben Thanh sample AOI and data contract
build(gis): add reproducible OSM extraction pipeline
docs(gis): document dataset metadata and quality results
data(gis): add validated Ben Thanh GeoJSON sample
docs(gis): record QGIS visual QA and KAN-6 handoff
```

## 9. Definition of Done

KAN-6 đã đạt các đầu ra chính: AOI được xác định, building footprints thật từ OSM được trích xuất/clip/chuẩn hóa, GeoJSON đọc được bằng GDAL/OGR, ID được kiểm tra, dữ liệu đã được người thực hiện xác nhận trực quan trong QGIS, và KAN-13 có data contract để tiếp tục thiết kế PostGIS.

**Trạng thái:** DATASET READY / VISUAL QA PASSED. Còn một việc bổ sung không chặn handoff: thử tái tạo độc lập trên môi trường sạch. Dữ liệu là snapshot Geofabrik tại thời điểm workflow chạy, chỉ gồm footprint 2D và không đại diện ranh giới hành chính.
