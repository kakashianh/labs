# KAN-13 – Thiết kế schema PostGIS cho khu vực và công trình

## 1. Objective

Thiết kế và triển khai schema PostgreSQL/PostGIS để lưu trữ dữ liệu không gian nền của dự án 3D GIS Medical, tập trung vào ranh giới khu vực và footprint công trình.

Schema cần hỗ trợ truy vấn khu vực, truy vấn danh sách/chi tiết công trình ở các ticket API tiếp theo và có khả năng mở rộng cho dữ liệu mô phỏng ở các phase sau mà không đưa logic mô phỏng vào schema nền.

**Expected result:**
- PostgreSQL và PostGIS được sử dụng đúng trong database của dự án.
- Có migration tạo bảng khu vực và công trình.
- Geometry columns có kiểu hình học và SRID được xác định từ dữ liệu KAN-6.
- Có khóa, ràng buộc và spatial index cần thiết.
- Schema được tài liệu hóa để KAN-14 có thể import dữ liệu.

## 2. Scope

### In scope
- Xác định quan hệ giữa khu vực và công trình.
- Thiết kế bảng lưu ranh giới khu vực.
- Thiết kế bảng lưu footprint công trình.
- Khai báo geometry type và SRID.
- Thiết lập primary key, foreign key, unique constraint và NOT NULL phù hợp.
- Tạo spatial index và các index phục vụ truy vấn cơ bản.
- Viết migration và kiểm tra schema.

### Out of scope
- Chuẩn bị/làm sạch dữ liệu nguồn (KAN-6).
- Import dữ liệu đô thị (KAN-14).
- Xây dựng API Axum (KAN-15, KAN-7).
- Lưu trạng thái SIR theo công trình (Phase 2).
- Lưu scenario, intervention hoặc lịch sử mô phỏng (Phase 3).
- Thiết kế mô hình dữ liệu phòng/căn hộ hoặc agent-level.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Database | PostgreSQL | Lưu trữ dữ liệu quan hệ |
| Spatial extension | PostGIS | Geometry và truy vấn không gian |
| Migration | Công cụ migration đang dùng trong repository | Quản lý thay đổi schema |
| Backend | Rust + Axum | Kết nối database ở các ticket API |
| Spatial data | Geometry theo metadata KAN-6 | Lưu khu vực và footprint công trình |

Lưu ý:
- Không tự chọn SRID khi chưa đối chiếu metadata dữ liệu KAN-6.
- Geometry lưu trữ nên dùng CRS phù hợp với dữ liệu và các phép truy vấn dự kiến.
- Nếu cần phục vụ CesiumJS, chuyển geometry sang WGS84/GeoJSON ở lớp truy vấn hoặc API; không thay đổi CRS lưu trữ chỉ để thuận tiện hiển thị.

## 4. Proposed Data Model

Quan hệ chính:

```text
regions (1) ─────────── (N) buildings
```

### Table: regions

Lưu ranh giới các khu vực thuộc phạm vi nghiên cứu.

| Column | Suggested type | Constraint / Purpose |
|---|---|---|
| id | BIGSERIAL hoặc UUID | Primary key |
| code | VARCHAR | Mã khu vực, unique nếu có |
| name | VARCHAR | Tên khu vực |
| boundary | geometry(MultiPolygon, SRID) | Ranh giới khu vực |
| created_at | TIMESTAMPTZ | Thời điểm tạo |
| updated_at | TIMESTAMPTZ | Thời điểm cập nhật |

### Table: buildings

Lưu footprint và thông tin cơ bản của từng công trình.

| Column | Suggested type | Constraint / Purpose |
|---|---|---|
| id | BIGSERIAL hoặc UUID | Primary key |
| external_id | VARCHAR | ID từ nguồn dữ liệu, unique khi có |
| region_id | BIGINT/UUID | Foreign key đến regions |
| name | VARCHAR, nullable | Tên công trình nếu có |
| building_type | VARCHAR, nullable | Loại công trình nếu có |
| height_m | DOUBLE PRECISION, nullable | Chiều cao nếu nguồn cung cấp |
| floors | INTEGER, nullable | Số tầng nếu nguồn cung cấp |
| footprint | geometry(MultiPolygon, SRID) | Footprint công trình |
| created_at | TIMESTAMPTZ | Thời điểm tạo |
| updated_at | TIMESTAMPTZ | Thời điểm cập nhật |

Đây là mô hình đề xuất. Chốt kiểu ID, kiểu geometry và các cột thuộc tính theo dữ liệu thực tế từ KAN-6; không tạo thuộc tính giả hoặc bắt buộc các trường nguồn không cung cấp.

## 5. Implementation Plan

### Step 1 – Review Prepared GIS Data

Đọc metadata và kết quả chuẩn hóa từ KAN-6 trước khi tạo schema.

- [ ] Xác nhận danh sách lớp dữ liệu cần lưu.
- [ ] Xác nhận geometry type thực tế của region và building.
- [ ] Xác nhận SRID/CRS và đơn vị tọa độ.
- [ ] Xác nhận thuộc tính có sẵn và kiểu dữ liệu.
- [ ] Chốt chiến lược primary key và external ID.
- [ ] Chốt quan hệ giữa regions và buildings.

### Step 2 – Verify PostgreSQL and PostGIS Setup

Kiểm tra cấu hình database hiện có trong repository và môi trường Docker/local.

- [ ] Kiểm tra PostgreSQL service/container hiện tại.
- [ ] Kiểm tra phiên bản PostgreSQL và khả năng cài PostGIS.
- [ ] Bật extension PostGIS trong database mục tiêu.
- [ ] Xác nhận truy vấn `SELECT PostGIS_Full_Version();` chạy thành công.
- [ ] Không tạo database/service trùng lặp nếu repository đã có cấu hình phù hợp.

### Step 3 – Design Tables and Constraints

Thiết kế schema tối thiểu cho hai lớp dữ liệu nền.

Yêu cầu:
- Mỗi region có định danh ổn định.
- Mỗi building có định danh nội bộ và có thể lưu external ID.
- Building tham chiếu region bằng foreign key khi dữ liệu xác định được quan hệ.
- Geometry column có geometry type và SRID rõ ràng.
- Các trường bắt buộc được giới hạn ở dữ liệu thực sự cần thiết.
- Có unique constraint phù hợp để tránh import trùng.

- [ ] Chốt tên schema/table và naming convention.
- [ ] Chốt primary key cho regions và buildings.
- [ ] Khai báo foreign key buildings.region_id.
- [ ] Khai báo geometry columns với type/SRID đã xác nhận.
- [ ] Thêm NOT NULL, UNIQUE và CHECK constraint cần thiết.
- [ ] Xác định cách xử lý building không gắn được region nếu có trường hợp này.

### Step 4 – Add Spatial and Query Indexes

Tạo index phục vụ truy vấn theo không gian và quan hệ.

- [ ] Tạo GiST index cho regions.boundary.
- [ ] Tạo GiST index cho buildings.footprint.
- [ ] Tạo B-tree index cho buildings.region_id.
- [ ] Tạo unique index/constraint cho external_id nếu phù hợp.
- [ ] Kiểm tra index được tạo thành công.
- [ ] Tránh tạo index dư thừa không gắn với truy vấn dự kiến.

### Step 5 – Implement Database Migration

Sử dụng migration mechanism đang có trong repository. Nếu chưa có, chọn một công cụ migration phù hợp với backend Rust hiện tại và thống nhất cách chạy trong local/Docker.

Migration cần:
- Bật PostGIS extension (nếu quyền môi trường cho phép).
- Tạo bảng regions và buildings.
- Tạo constraints và indexes.
- Có hướng dẫn rollback hoặc migration down nếu công cụ hỗ trợ.

- [ ] Tạo migration bật PostGIS.
- [ ] Tạo migration bảng regions.
- [ ] Tạo migration bảng buildings.
- [ ] Tạo constraints và indexes.
- [ ] Kiểm tra migration chạy trên database rỗng.
- [ ] Kiểm tra migration không phá vỡ cấu hình hiện có.

### Step 6 – Verify Schema

Kiểm tra schema bằng SQL và metadata catalog của PostGIS.

- [ ] Xác nhận hai bảng tồn tại.
- [ ] Xác nhận geometry type và SRID đúng với metadata KAN-6.
- [ ] Xác nhận foreign key và unique constraints hoạt động.
- [ ] Xác nhận GiST indexes tồn tại.
- [ ] Thử insert một region và một building mẫu hợp lệ.
- [ ] Thử insert dữ liệu sai để xác nhận constraint được thực thi.
- [ ] Xác nhận migration có thể chạy lại theo quy trình dự án.

### Step 7 – Document Schema for Data Import

Ghi rõ mapping giữa dữ liệu chuẩn hóa từ KAN-6 và các cột database để KAN-14 thực hiện import thống nhất.

- [ ] Tạo tài liệu schema hoặc cập nhật tài liệu database hiện có.
- [ ] Ghi mapping source field → database column.
- [ ] Ghi quy tắc chuyển đổi CRS/geometry khi import.
- [ ] Ghi cách tạo/giữ external_id.
- [ ] Bàn giao hướng dẫn import cho KAN-14.

## 6. Acceptance Criteria

- [ ] PostGIS extension được bật trong database mục tiêu.
- [ ] Có bảng regions và buildings theo mô hình đã thống nhất.
- [ ] Geometry type và SRID khớp với dữ liệu KAN-6.
- [ ] Quan hệ khóa ngoại và các constraint cần thiết hoạt động.
- [ ] Có spatial index cho geometry columns.
- [ ] Migration chạy thành công trên database rỗng.
- [ ] Có tài liệu mapping để import dữ liệu.
- [ ] Không đưa trạng thái SIR/scenario vào schema nền của Phase 1.

## 7. Deliverables

1. Migration tạo PostGIS extension, regions và buildings.
2. Schema PostgreSQL/PostGIS đã được kiểm tra.
3. Spatial indexes và relational constraints.
4. Tài liệu schema và mapping dữ liệu đầu vào.

## 8. Suggested Commit Sequence

```text
chore(db): enable PostGIS extension

feat(db): add regions and buildings schema

perf(db): add spatial and relational indexes

docs(db): document GIS schema and import mapping
```

## 9. Final Definition of Done

KAN-13 hoàn thành khi database có thể tạo mới từ migration, lưu được geometry của khu vực và công trình với SRID chính xác, đảm bảo quan hệ/constraint và có spatial index để các ticket API và import sử dụng.

Schema này chỉ quản lý dữ liệu GIS nền; dữ liệu mô phỏng dịch tễ sẽ được thiết kế ở phase tương ứng.
