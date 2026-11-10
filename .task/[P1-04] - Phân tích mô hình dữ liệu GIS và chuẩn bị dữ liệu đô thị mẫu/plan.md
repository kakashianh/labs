# KAN-6 – Phân tích mô hình dữ liệu GIS và chuẩn bị dữ liệu đô thị mẫu

## 1. Objective

Chuẩn bị bộ dữ liệu GIS 2D từ OpenStreetMap (OSM) cho khu vực nghiên cứu Thành phố Hồ Chí Minh (TP.HCM), gồm ranh giới nghiên cứu và footprint công trình. Bộ dữ liệu sau chuẩn hóa là đầu vào cho KAN-13 (thiết kế schema PostGIS) và KAN-14 (import dữ liệu).

**Phương án đã thống nhất**
- Khu vực: TP. Hồ Chí Minh.
- Nguồn chính: OpenStreetMap; Vietnam extract của Geofabrik dùng làm dữ liệu PBF đầu vào.
- Mức chi tiết: 2D; chỉ footprint, không dựng khối 3D hoặc suy diễn chiều cao/số tầng.
- Định dạng bàn giao: GeoJSON theo RFC 7946, tọa độ WGS 84 (longitude, latitude; EPSG:4326).
- Công cụ: GDAL/OGR và QGIS; không thêm Python service vào backend Rust + Axum.

**Lưu ý phạm vi:** TP.HCM là phạm vi dữ liệu mục tiêu. Không tự thu hẹp sang một quận hoặc khu vực trung tâm nếu chưa được duyệt. Bước đầu phải xác minh boundary OSM đúng là ranh giới cấp thành phố; không chọn relation chỉ dựa trên tên gần giống.

## 2. Scope

### In scope
- Xác minh ranh giới hành chính TP.HCM từ OSM và lưu nguồn tham chiếu.
- Tải Vietnam OSM PBF từ Geofabrik.
- Trích xuất các đối tượng có tag `building=*` thành footprint.
- Cắt footprint theo boundary TP.HCM.
- Chuẩn hóa ID, thuộc tính, geometry và CRS.
- Kiểm tra dữ liệu bằng GDAL/OGR và QGIS.
- Tài liệu hóa nguồn, giấy phép, metadata, data dictionary, chất lượng và quy trình tái tạo.

### Out of scope
- Thiết kế hoặc tạo bảng PostGIS (KAN-13).
- Import dữ liệu vào database (KAN-14).
- Xây dựng GIS API hoặc hiển thị CesiumJS.
- Tạo mô hình 3D, chiều cao, số tầng, dân số hoặc dữ liệu dịch tễ.
- Cam kết OSM có đầy đủ 100% công trình của TP.HCM.

## 3. Data contract

### 3.1 Layers

| Layer | Geometry | Mục đích |
|---|---|---|
| `hcm_boundary` | MultiPolygon | Ranh giới phạm vi nghiên cứu TP.HCM |
| `hcm_buildings` | MultiPolygon | Footprint các công trình nằm trong boundary |

### 3.2 Building attributes

| Field | Kiểu logic | Nullable | Quy tắc |
|---|---|---:|---|
| `building_id` | string | No | Khóa ổn định theo nguồn: `way/{osm_way_id}` hoặc `relation/{osm_id}` |
| `osm_id` | integer/string theo driver | Yes | ID relation nếu feature là relation |
| `osm_way_id` | integer/string theo driver | Yes | ID way nếu feature là closed way |
| `osm_type` | string | No | `way` hoặc `relation`, suy ra từ ID nguồn có mặt |
| `name` | string | Yes | Tag OSM `name`, giữ null nếu thiếu |
| `building` | string | Yes | Giá trị tag OSM `building` |
| `geometry` | MultiPolygon, 2D | No | Footprint; giữ nguyên hình học nhiều phần |

Không tự tạo `height`, `building_levels`, dân số hoặc thuộc tính mô phỏng. Không dùng FID sinh tự động làm định danh nghiệp vụ.

### 3.3 CRS

- OSM PBF và boundary phải có CRS được GDAL nhận diện; không gán CRS giả để né lỗi.
- GeoJSON đầu ra được xuất theo RFC 7946, WGS 84 longitude/latitude (EPSG:4326).
- Nếu cần đo diện tích/khoảng cách, phải dùng CRS chiếu phù hợp hoặc phép đo geodesic; không tính diện tích trực tiếp từ đơn vị độ.
- KAN-13 quyết định SRID/kiểu geometry trong schema PostGIS dựa trên dữ liệu bàn giao.

## 4. Implementation plan

### Step 1 – Confirm study boundary and source

- [ ] Mở Overpass Turbo và tìm relation có `boundary=administrative` đại diện đúng TP.HCM.
- [ ] Kiểm tra tag `name`, `admin_level`, loại relation, phạm vi và hình dạng trên bản đồ; không hard-code admin_level hoặc relation ID trước khi xác minh.
- [ ] Export đúng một boundary feature thành GeoJSON và lưu tại `data/sample/source/hcm_boundary.geojson`.
- [ ] Ghi relation ID, URL truy vấn, ngày tải, attribution và ghi chú xác minh vào `metadata.md`.
- [ ] Kiểm tra file boundary có CRS hợp lệ và chỉ chứa đúng phạm vi thành phố.

### Step 2 – Acquire OSM PBF

- [ ] Tải Vietnam extract: `https://download.geofabrik.de/asia/vietnam-latest.osm.pbf`.
- [ ] Lưu tại `data/sample/source/vietnam-latest.osm.pbf` (được gitignore).
- [ ] Ghi ngày tải, URL, dung lượng và SHA-256 vào metadata.
- [ ] Không commit PBF vào Git; người khác có thể tải lại từ URL nguồn.

### Step 3 – Inspect source data

- [ ] Kiểm tra phiên bản GDAL và khả năng đọc OSM PBF.
- [ ] Dùng `ogrinfo` kiểm tra các layer nguồn, đặc biệt `multipolygons`, field `building`, `osm_id`, `osm_way_id`.
- [ ] Kiểm tra CRS, extent, số feature và thuộc tính boundary.
- [ ] Ghi lại bất thường hoặc thiếu dữ liệu trước khi xử lý.

### Step 4 – Extract, clip and normalize

- [ ] Chạy `scripts/gis/prepare_osm_data.sh`.
- [ ] Chỉ lấy feature có tag `building IS NOT NULL`.
- [ ] Clip theo boundary TP.HCM đã xác minh.
- [ ] Tạo `building_id` theo namespace OSM way/relation; giữ ID nguồn để truy vết.
- [ ] Giữ Polygon/MultiPolygon dưới dạng MultiPolygon; không làm phẳng geometry.
- [ ] Xuất GeoJSON RFC 7946 / EPSG:4326.
- [ ] Không ghi đè PBF hoặc boundary gốc.

### Step 5 – Validate with GDAL and QGIS

- [ ] Đọc lại hai GeoJSON bằng `ogrinfo`.
- [ ] Xác nhận CRS và geometry type.
- [ ] Kiểm tra `building_id` null/trùng, geometry rỗng/invalid và feature ngoài boundary.
- [ ] Mở dữ liệu trong QGIS, đối chiếu các vùng đại diện và kiểm tra polygon bị cắt bất thường.
- [ ] Ghi số liệu thực tế vào `data_quality_report.md`; không điền số ước lượng.

### Step 6 – Document and handoff

- [ ] Hoàn thiện `metadata.md` bằng thông tin thực tế sau khi chạy pipeline.
- [ ] Hoàn thiện `data_dictionary.md` nếu cấu trúc thực tế có khác data contract.
- [ ] Hoàn thiện `data_quality_report.md`, gồm số feature, lỗi, cách xử lý và hạn chế.
- [ ] Bàn giao layers, fields, types, CRS, source IDs và caveats cho KAN-13.
- [ ] Kiểm tra hướng dẫn tái tạo từ đầu trên một môi trường sạch.

## 5. Repository layout

```text
data/sample/
├── source/
│   ├── README.md
│   └── hcm_boundary.geojson       # tải và xác minh thủ công; không phải file giả lập
└── processed/
    ├── README.md
    ├── hcm_boundary.geojson       # sinh bởi pipeline
    └── hcm_buildings.geojson      # sinh bởi pipeline

.task/[P1-04].../deliverables/
├── metadata.md
├── data_dictionary.md
└── data_quality_report.md

scripts/gis/
├── README.md
└── prepare_osm_data.sh
```

Các GeoJSON đầu ra chỉ được commit sau khi pipeline đã chạy và dữ liệu được kiểm tra. Không tạo dữ liệu mẫu giả để thay thế dữ liệu OSM thật.

## 6. Acceptance criteria

- [ ] Boundary đúng TP.HCM, có nguồn và OSM relation ID truy vết được.
- [ ] Có footprint từ OSM tag `building=*`, được clip theo boundary.
- [ ] CRS đầu ra là EPSG:4326 theo RFC 7946; CRS nguồn được ghi nhận.
- [ ] `building_id` không null và duy nhất; giữ lại OSM ID gốc.
- [ ] Geometry rỗng/invalid và feature ngoài phạm vi được thống kê, xử lý hoặc giải trình.
- [ ] Có metadata, data dictionary, quality report và hướng dẫn tái tạo.
- [ ] KAN-13 nhận được data contract và các giới hạn dữ liệu.

## 7. Deliverables

1. `hcm_boundary.geojson` và `hcm_buildings.geojson` sau khi chạy pipeline thành công.
2. Hướng dẫn tải boundary/PBF và tái tạo dữ liệu.
3. Metadata nguồn, phạm vi, CRS, giấy phép và checksum.
4. Data dictionary và báo cáo chất lượng.
5. Script xử lý có thể chạy lại.

## 8. Suggested commit sequence

```text
docs(gis): define HCMC OSM study area and data contract
build(gis): add reproducible OSM extraction pipeline
docs(gis): document dataset metadata and quality results
data(gis): add validated HCMC GeoJSON sample
```

## 9. Final Definition of Done

KAN-6 chỉ hoàn thành khi boundary TP.HCM đã được xác minh, bộ building footprints thật từ OSM đã được trích xuất/clip/chuẩn hóa, kết quả đọc lại được bằng công cụ GIS, chất lượng và giới hạn được báo cáo, và KAN-13 có đủ thông tin để thiết kế schema PostGIS.

**Trạng thái hiện tại:** tài liệu và pipeline được chuẩn bị trong nhánh này; dữ liệu GeoJSON và các số liệu chất lượng vẫn pending cho đến khi boundary OSM được xác minh và pipeline chạy trên dữ liệu nguồn.
