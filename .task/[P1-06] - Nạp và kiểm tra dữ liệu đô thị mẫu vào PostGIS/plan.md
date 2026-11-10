# KAN-14 – Nạp và kiểm tra dữ liệu đô thị mẫu vào PostGIS

## 1. Objective

Nạp bộ dữ liệu đô thị mẫu đã được chuẩn hóa ở KAN-6 vào schema PostgreSQL/PostGIS được tạo ở KAN-13.

Đảm bảo dữ liệu khu vực và footprint công trình được lưu đầy đủ, đúng CRS, đúng quan hệ và có thể truy vấn không gian để phục vụ các ticket API và hiển thị CesiumJS tiếp theo.

**Expected result:**
- Dữ liệu regions và buildings được import vào PostGIS.
- Số lượng bản ghi và geometry được đối chiếu với dữ liệu đầu vào.
- Không có lỗi nghiêm trọng về CRS, geometry hoặc quan hệ dữ liệu.
- Có quy trình import có thể chạy lại và tài liệu kiểm tra.

## 2. Scope

### In scope
- Kiểm tra database và migration KAN-13.
- Chuẩn bị dữ liệu đầu vào từ KAN-6.
- Thực hiện import lớp regions và buildings.
- Chuyển đổi geometry/CRS khi cần theo mapping đã thống nhất.
- Kiểm tra số lượng, thuộc tính, geometry và quan hệ.
- Xử lý lỗi import và dữ liệu không hợp lệ.
- Ghi lại quy trình để tái tạo dữ liệu mẫu.

### Out of scope
- Thu thập hoặc phân tích nguồn dữ liệu mới (KAN-6).
- Thay đổi thiết kế schema ngoài các lỗi được xác nhận (KAN-13).
- Xây dựng API truy vấn GIS.
- Hiển thị dữ liệu trên CesiumJS.
- Tính toán trạng thái dịch tễ hoặc mô phỏng SIR.
- Tự động đồng bộ dữ liệu thời gian thực từ nguồn bên ngoài.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Database | PostgreSQL | Lưu dữ liệu |
| Spatial extension | PostGIS | Lưu và kiểm tra geometry |
| Input | Bộ dữ liệu đã chuẩn hóa từ KAN-6 | Nguồn import |
| Import tooling | Công cụ phù hợp định dạng dữ liệu (ví dụ ogr2ogr hoặc SQL staging) | Nạp dữ liệu |
| Backend context | Rust + Axum | Ứng dụng sử dụng database sau khi import |

Ưu tiên công cụ import phù hợp với định dạng dữ liệu thực tế. Không tạo Python backend/service cho tác vụ này. Script hỗ trợ import nếu cần phải có hướng dẫn chạy lại và không chứa thông tin kết nối bí mật.

## 4. Implementation Plan

### Step 1 – Verify Database and Schema Readiness

Đảm bảo môi trường database đã sẵn sàng trước khi import.

- [ ] Khởi động PostgreSQL/PostGIS theo cấu hình repository.
- [ ] Kiểm tra kết nối tới database mục tiêu.
- [ ] Chạy migration KAN-13 trên database.
- [ ] Xác nhận regions và buildings tồn tại.
- [ ] Xác nhận geometry type, SRID, constraints và indexes.
- [ ] Ghi nhận cách cấu hình connection string qua environment variables.

### Step 2 – Prepare Input Data and Import Mapping

Sử dụng đúng bộ dữ liệu đã chuẩn hóa và metadata từ KAN-6.

Lập mapping rõ ràng:
- Lớp dữ liệu nguồn → bảng đích.
- Source field → database column.
- Source ID → external_id hoặc định danh tương ứng.
- Source CRS → storage SRID.
- Geometry type nguồn → geometry type đích.

- [ ] Xác định file đầu vào cuối cùng cho regions.
- [ ] Xác định file đầu vào cuối cùng cho buildings.
- [ ] Đối chiếu metadata và mapping với schema KAN-13.
- [ ] Kiểm tra encoding và kiểu dữ liệu thuộc tính.
- [ ] Kiểm tra lại số lượng feature đầu vào.
- [ ] Xác định cách xử lý record thiếu thuộc tính hoặc geometry không hợp lệ.

### Step 3 – Implement Repeatable Import Process

Chọn cách import phù hợp với định dạng dữ liệu và cấu trúc repository.

Có thể sử dụng:
- `ogr2ogr` cho các định dạng GIS được hỗ trợ.
- Staging table kết hợp SQL để kiểm tra và chuyển dữ liệu.
- Script/command phục vụ import nếu cần tự động hóa quy trình.

Quy trình nên tách bước nạp staging và bước ghi vào bảng chính khi cần kiểm tra/chuyển đổi dữ liệu. Không hard-code password hoặc connection string.

- [ ] Tạo hướng dẫn hoặc script import có thể chạy lại.
- [ ] Import dữ liệu regions.
- [ ] Import dữ liệu buildings.
- [ ] Áp dụng CRS transformation đúng theo mapping.
- [ ] Ánh xạ ID và foreign key region_id.
- [ ] Ghi log số bản ghi thành công/thất bại.
- [ ] Đảm bảo import không tạo bản ghi trùng khi chạy lại (theo chiến lược upsert hoặc reset/reload được chọn).

### Step 4 – Validate Imported Records

Đối chiếu dữ liệu trong PostGIS với dữ liệu đầu vào.

Các kiểm tra:
- Số lượng records theo từng lớp.
- Giá trị ID và thuộc tính quan trọng.
- Geometry không null/rỗng và hợp lệ.
- SRID của geometry.
- Geometry nằm trong phạm vi nghiên cứu.
- Quan hệ buildings.region_id tham chiếu đúng region.
- Bounding box/extent có hợp lý so với khu vực nghiên cứu.

- [ ] So sánh số lượng regions đầu vào và đầu ra.
- [ ] So sánh số lượng buildings đầu vào và đầu ra.
- [ ] Kiểm tra ID null/trùng.
- [ ] Kiểm tra geometry null, empty hoặc invalid.
- [ ] Kiểm tra SRID thực tế bằng hàm PostGIS.
- [ ] Kiểm tra buildings có region_id không hợp lệ.
- [ ] Kiểm tra extent và một số geometry đại diện trên bản đồ GIS.
- [ ] Ghi nhận số record bị loại hoặc cần xử lý và lý do.

### Step 5 – Verify Spatial Queries and Index Usage

Thử các truy vấn không gian cơ bản để xác nhận dữ liệu sẵn sàng cho API và các phase tiếp theo.

Ví dụ truy vấn:
- Lấy các công trình thuộc một khu vực.
- Kiểm tra công trình giao/cắt ranh giới khu vực.
- Lấy extent của lớp công trình.
- Kiểm tra truy vấn có thể sử dụng spatial index khi phù hợp.

- [ ] Chạy truy vấn lọc buildings theo region_id.
- [ ] Chạy truy vấn không gian với ST_Intersects hoặc ST_Within phù hợp ngữ nghĩa dữ liệu.
- [ ] Kiểm tra kết quả truy vấn có số lượng hợp lý.
- [ ] Dùng EXPLAIN/EXPLAIN ANALYZE để kiểm tra kế hoạch truy vấn trên dataset mẫu.
- [ ] Xác nhận GiST index được planner cân nhắc khi phù hợp; không bắt buộc index scan với dataset quá nhỏ.

### Step 6 – Test Re-import and Document the Process

Đảm bảo có thể tái tạo dữ liệu mẫu trong môi trường phát triển mới.

- [ ] Thử chạy lại quy trình import theo hướng dẫn.
- [ ] Xác nhận không phát sinh dữ liệu trùng hoặc sai lệch ngoài dự kiến.
- [ ] Ghi rõ prerequisite và câu lệnh chạy.
- [ ] Ghi rõ biến môi trường cần thiết nhưng không ghi giá trị secret.
- [ ] Ghi kết quả kiểm tra: số lượng, SRID, geometry validity và quan hệ.
- [ ] Ghi lại các lỗi còn tồn tại và giới hạn của dataset.

## 5. Acceptance Criteria

- [ ] Database PostGIS đã có dữ liệu regions và buildings.
- [ ] Số lượng bản ghi được đối chiếu với dữ liệu đầu vào; mọi chênh lệch đều có giải thích.
- [ ] Geometry có SRID đúng và không còn lỗi nghiêm trọng chưa xử lý.
- [ ] Foreign key/quan hệ khu vực – công trình hợp lệ.
- [ ] Các truy vấn không gian cơ bản chạy thành công.
- [ ] Quy trình import có thể chạy lại theo tài liệu.
- [ ] Không có credentials hoặc secret được commit.
- [ ] Có báo cáo kết quả kiểm tra dữ liệu.

## 6. Deliverables

1. Dữ liệu regions và buildings được nạp vào PostgreSQL/PostGIS.
2. Script hoặc hướng dẫn import có thể tái sử dụng.
3. Kết quả kiểm tra số lượng, CRS, geometry và quan hệ.
4. Tài liệu quy trình import và các giới hạn dữ liệu.

## 7. Suggested Commit Sequence

```text
chore(data): prepare GIS import configuration

feat(data): import sample regions into PostGIS

feat(data): import sample building footprints into PostGIS

test(data): validate imported GIS records and spatial queries

docs(data): document sample dataset import procedure
```

## 8. Final Definition of Done

KAN-14 hoàn thành khi dữ liệu khu vực và công trình đã được nạp vào PostGIS, có thể truy vấn bằng SQL/PostGIS, vượt qua các kiểm tra về số lượng, CRS, geometry và quan hệ, đồng thời nhóm có thể tái tạo quy trình import trên môi trường phát triển.

Ticket này chỉ nạp dữ liệu GIS nền; chưa triển khai API, Cesium visualization hoặc mô phỏng SIR.
