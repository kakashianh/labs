# KAN-6 – Phân tích mô hình dữ liệu GIS và chuẩn bị dữ liệu đô thị mẫu

## 1. Objective

Xác định mô hình dữ liệu không gian phù hợp cho khu vực nghiên cứu và chuẩn bị bộ dữ liệu đô thị mẫu dùng xuyên suốt Phase 1.

Dữ liệu cần đủ để biểu diễn ranh giới khu vực và vị trí/phạm vi các công trình trên bản đồ 3D CesiumJS, đồng thời có thể nạp vào PostgreSQL/PostGIS ở các ticket tiếp theo.

**Expected result:**
- Xác định được nguồn dữ liệu và phạm vi khu vực nghiên cứu.
- Hiểu rõ cấu trúc, thuộc tính và hệ tọa độ của dữ liệu đầu vào.
- Có bộ dữ liệu mẫu đã làm sạch, thống nhất định dạng và sẵn sàng import.
- Có tài liệu mô tả dữ liệu, các giới hạn và quy tắc chuyển đổi.

## 2. Scope

### In scope
- Khảo sát nguồn dữ liệu GIS có thể sử dụng cho khu vực nghiên cứu.
- Chọn phạm vi không gian và tập dữ liệu mẫu phù hợp với MVP.
- Phân tích dữ liệu ranh giới khu vực và footprint công trình.
- Kiểm tra geometry, thuộc tính, hệ tọa độ và chất lượng dữ liệu.
- Chuẩn hóa định dạng, tên trường và giá trị thuộc tính.
- Chuẩn bị dữ liệu mẫu để thiết kế schema PostGIS và import.

### Out of scope
- Thiết kế schema hoặc tạo bảng PostGIS (KAN-13).
- Import dữ liệu vào database (KAN-14).
- Xây dựng API truy vấn GIS.
- Hiển thị dữ liệu trên CesiumJS.
- Bổ sung dữ liệu dân số/nhân khẩu học nếu nguồn dữ liệu không cung cấp đáng tin cậy.
- Xây dựng mô hình 3D chi tiết bên trong công trình.

## 3. Technical Context

| Component | Technology / Format | Purpose |
|---|---|---|
| GIS data | GeoJSON / GeoPackage / Shapefile hoặc định dạng nguồn phù hợp | Dữ liệu đầu vào |
| Spatial database | PostgreSQL + PostGIS | Lưu trữ dữ liệu không gian ở ticket tiếp theo |
| Coordinate system | CRS xác định từ dữ liệu nguồn | Đảm bảo phép đo và hiển thị đúng |
| 3D map | CesiumJS | Hiển thị dữ liệu sau khi chuẩn hóa |
| Backend | Rust + Axum | Backend của dự án; không xử lý thay thế bằng Python service |

Không mặc định một EPSG cụ thể trước khi kiểm tra metadata và phạm vi dữ liệu. Ghi nhận CRS nguồn, CRS lưu trữ và CRS xuất phục vụ Cesium trong tài liệu.

## 4. Implementation Plan

### Step 1 – Identify Study Area and Data Requirements

Xác định khu vực nghiên cứu và mức độ chi tiết cần thiết cho MVP.

Các lớp dữ liệu tối thiểu:
- Ranh giới khu vực nghiên cứu.
- Footprint (ranh giới mặt bằng) của từng công trình.
- Mã định danh ổn định cho từng công trình.
- Tên hoặc loại công trình nếu nguồn dữ liệu có cung cấp.

Thuộc tính bổ sung như chiều cao, số tầng hoặc loại công trình chỉ đưa vào nếu có nguồn dữ liệu đủ tin cậy.

- [ ] Ghi rõ tên và ranh giới khu vực nghiên cứu.
- [ ] Xác định lớp dữ liệu bắt buộc và thuộc tính tối thiểu.
- [ ] Xác định tiêu chí chọn dữ liệu mẫu (phạm vi, số lượng, mức độ đầy đủ).
- [ ] Ghi nhận các thuộc tính không có dữ liệu hoặc không đáng tin cậy.

### Step 2 – Survey and Select GIS Data Sources

Khảo sát nguồn dữ liệu phù hợp, ưu tiên dữ liệu có nguồn gốc rõ ràng, giấy phép sử dụng phù hợp và geometry có thể chuyển đổi sang định dạng GIS phổ biến.

Với mỗi nguồn, ghi nhận:
- Tên nguồn và URL/tài liệu tham chiếu.
- Ngày thu thập hoặc phiên bản dữ liệu.
- Phạm vi bao phủ.
- Định dạng và các lớp dữ liệu.
- CRS và đơn vị tọa độ.
- Giấy phép hoặc điều kiện sử dụng (nếu có).

- [ ] Liệt kê các nguồn dữ liệu có thể sử dụng.
- [ ] Chọn nguồn dữ liệu chính cho khu vực nghiên cứu.
- [ ] Lưu thông tin nguồn, thời điểm thu thập và điều kiện sử dụng.
- [ ] Kiểm tra dữ liệu có thể mở bằng công cụ GIS hoặc thư viện phù hợp.

### Step 3 – Inspect Data Structure and Geometry

Kiểm tra dữ liệu trước khi chuẩn hóa.

Các kiểm tra cần thực hiện:
- Geometry type (Polygon, MultiPolygon hoặc loại thực tế).
- Geometry rỗng, không hợp lệ hoặc bị lỗi.
- Feature trùng lặp và mã định danh bị thiếu/trùng.
- Thuộc tính thiếu, sai kiểu hoặc không thống nhất.
- CRS khai báo và extent tọa độ.
- Sự tương thích không gian giữa ranh giới khu vực và footprint công trình.

- [ ] Thống kê số lượng feature theo từng lớp.
- [ ] Kiểm tra geometry rỗng và geometry không hợp lệ.
- [ ] Kiểm tra ID trùng lặp hoặc thiếu.
- [ ] Kiểm tra CRS, extent và đơn vị tọa độ.
- [ ] Kiểm tra công trình nằm ngoài phạm vi khu vực nghiên cứu.
- [ ] Ghi lại các vấn đề dữ liệu và phương án xử lý.

### Step 4 – Clean and Normalize the Dataset

Chuẩn hóa dữ liệu để schema PostGIS và quy trình import có thể xử lý nhất quán.

Quy tắc:
- Chuẩn hóa tên lớp và tên thuộc tính theo snake_case.
- Chuẩn hóa kiểu dữ liệu và giá trị null.
- Tạo hoặc chuẩn hóa mã định danh duy nhất, ổn định cho mỗi công trình.
- Sửa geometry không hợp lệ khi có thể sửa mà không làm thay đổi sai lệch phạm vi đáng kể.
- Chuyển đổi CRS có kiểm soát; không gán CRS mới chỉ để làm thay đổi metadata.
- Giữ lại bản dữ liệu gốc để có thể đối chiếu.

- [ ] Tạo bản dữ liệu làm việc, không ghi đè dữ liệu gốc.
- [ ] Chuẩn hóa tên trường và kiểu dữ liệu.
- [ ] Xử lý ID thiếu/trùng theo quy tắc được ghi nhận.
- [ ] Sửa hoặc loại bỏ geometry lỗi theo tiêu chí rõ ràng.
- [ ] Chuẩn hóa CRS và ghi lại phép chuyển đổi.
- [ ] Xuất dữ liệu đã chuẩn hóa ở định dạng thống nhất.

### Step 5 – Prepare Import-ready Sample Data

Chuẩn bị cấu trúc thư mục và dữ liệu đầu ra để KAN-13/KAN-14 sử dụng.

Ví dụ cấu trúc (điều chỉnh theo cấu trúc repository hiện tại):

```text
data/
└── sample/
    ├── source/          # Dữ liệu gốc, nếu được phép lưu trong repository
    ├── processed/       # Dữ liệu đã chuẩn hóa
    └── metadata.md      # Nguồn, CRS, schema thuộc tính và ghi chú
```

Không commit dữ liệu dung lượng lớn hoặc dữ liệu có điều kiện giấy phép không cho phép. Trong trường hợp đó, lưu hướng dẫn tải/tái tạo và checksum thay vì đưa dữ liệu vào Git.

- [ ] Chuẩn bị lớp ranh giới khu vực ở định dạng đã chọn.
- [ ] Chuẩn bị lớp footprint công trình ở định dạng đã chọn.
- [ ] Tạo metadata mô tả nguồn, CRS, số lượng feature và thuộc tính.
- [ ] Ghi hướng dẫn tái tạo/tải dữ liệu nếu không thể commit dữ liệu.
- [ ] Kiểm tra dữ liệu đầu ra có thể đọc lại bằng công cụ GIS.

### Step 6 – Validate Dataset and Handoff

Đối chiếu dữ liệu đầu ra với dữ liệu nguồn và xác nhận các điều kiện trước khi bàn giao cho ticket thiết kế schema.

- [ ] Xác nhận dữ liệu có CRS rõ ràng.
- [ ] Xác nhận ID công trình duy nhất và không null.
- [ ] Xác nhận geometry hợp lệ theo tiêu chí đã chọn.
- [ ] Xác nhận các lớp dữ liệu nằm trong phạm vi nghiên cứu.
- [ ] Ghi lại giới hạn và các thuộc tính không có sẵn.
- [ ] Bàn giao tên lớp, tên trường, kiểu dữ liệu và CRS cho KAN-13.

## 5. Acceptance Criteria

Ticket được xem là hoàn thành khi:
- [ ] Nguồn dữ liệu và phạm vi nghiên cứu được ghi nhận rõ ràng.
- [ ] Có lớp ranh giới khu vực và lớp footprint công trình phù hợp MVP.
- [ ] CRS nguồn và quy tắc chuyển đổi được xác định, không suy đoán.
- [ ] Dữ liệu đã được kiểm tra geometry, ID và thuộc tính.
- [ ] Có bộ dữ liệu mẫu đã chuẩn hóa hoặc hướng dẫn tái tạo dữ liệu.
- [ ] Có metadata đủ để thiết kế schema PostGIS và thực hiện import.
- [ ] Các giới hạn/chất lượng dữ liệu được ghi nhận.

## 6. Deliverables

1. Bộ dữ liệu đô thị mẫu đã chuẩn hóa hoặc hướng dẫn tái tạo.
2. Tài liệu metadata: nguồn, phạm vi, CRS, geometry và thuộc tính.
3. Báo cáo ngắn về chất lượng dữ liệu và các vấn đề đã xử lý.
4. Quy ước ID và tên trường làm đầu vào cho KAN-13.

## 7. Suggested Commit Sequence

```text
docs(gis): document study area and source datasets

data(gis): add normalized sample region and building data

docs(gis): add dataset metadata and validation notes
```

## 8. Final Definition of Done

KAN-6 hoàn thành khi nhóm có thể giải thích dữ liệu đô thị lấy từ đâu, biểu diễn những đối tượng nào, sử dụng CRS nào, chất lượng ra sao và có thể dùng trực tiếp bộ dữ liệu đã chuẩn hóa làm đầu vào thiết kế schema PostGIS ở KAN-13.

Ticket này chỉ chuẩn bị dữ liệu không gian nền; chưa triển khai database, API hoặc mô phỏng dịch tễ.
