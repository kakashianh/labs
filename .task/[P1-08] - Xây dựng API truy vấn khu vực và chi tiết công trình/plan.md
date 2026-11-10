# KAN-7 – Xây dựng API truy vấn khu vực và chi tiết công trình

## 1. Objective

Xây dựng các API Backend bằng Rust + Axum để Frontend truy vấn danh sách khu vực, thông tin chi tiết một khu vực và thông tin chi tiết một công trình.

Ticket này bổ sung các API tra cứu theo định danh, không trùng với API danh sách công trình có phân trang được triển khai ở KAN-15.

**Expected result:**
- Frontend có thể lấy danh sách khu vực nghiên cứu.
- Frontend có thể lấy thông tin và geometry của một khu vực.
- Frontend có thể lấy thông tin và geometry của một công trình theo ID.
- API trả dữ liệu nhất quán với schema và dữ liệu PostGIS.
- Có validation, xử lý not-found và tests.

## 2. Scope

### In scope
- Kiểm tra schema và dữ liệu regions/buildings.
- Thiết kế DTO cho region list, region detail và building detail.
- API lấy danh sách khu vực.
- API lấy chi tiết khu vực theo ID.
- API lấy chi tiết công trình theo ID.
- Serialize geometry sang GeoJSON hoặc định dạng thống nhất.
- Xử lý ID không hợp lệ/không tồn tại.
- Kiểm thử API và cập nhật tài liệu.

### Out of scope
- API danh sách công trình có pagination/filter (KAN-15).
- API chạy mô phỏng SIR.
- Dữ liệu trạng thái dịch tễ theo công trình.
- Phân tích quan hệ không gian/lân cận.
- Hiển thị dữ liệu trên CesiumJS (KAN-16).
- CRUD khu vực/công trình hoặc chỉnh sửa dữ liệu GIS.
- Authentication/authorization nếu chưa nằm trong phạm vi MVP.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| Backend | Rust | Ngôn ngữ triển khai |
| Web Framework | Axum | HTTP routing và handlers |
| Database | PostgreSQL + PostGIS | Truy vấn dữ liệu GIS |
| Database Access | Theo thư viện hiện có trong repository | Query và mapping |
| Spatial Format | GeoJSON | Truyền geometry tới Frontend |
| Testing | Rust test framework + HTTP integration tests | Kiểm thử |

Tái sử dụng database pool, routing, DTO conventions và error handling đã có. Không tạo thêm Python backend/service.

## 4. API Contract

Các endpoint đề xuất (điều chỉnh prefix nếu project đã có convention):

| Method | Endpoint | Purpose |
|---|---|---|
| GET | /api/v1/regions | Lấy danh sách khu vực |
| GET | /api/v1/regions/{region_id} | Lấy chi tiết khu vực |
| GET | /api/v1/buildings/{building_id} | Lấy chi tiết công trình |

Lưu ý:
- GET /api/v1/buildings (danh sách có pagination/filter) thuộc KAN-15.
- GET /api/v1/buildings/{building_id} là endpoint chi tiết riêng của ticket này.
- Kiểu ID phải theo schema thực tế, không mặc định UUID nếu database không dùng UUID.

Response region detail minh họa:

~~~json
{
  "data": {
    "id": "region-id",
    "properties": {},
    "geometry": {
      "type": "Polygon",
      "coordinates": []
    }
  }
}
~~~

Response building detail minh họa:

~~~json
{
  "data": {
    "id": "building-id",
    "region_id": "region-id",
    "properties": {},
    "geometry": {
      "type": "Polygon",
      "coordinates": []
    }
  }
}
~~~

Các field trong properties phải lấy từ schema/dataset thực tế. Không tự thêm height, floors, population hoặc thông tin không có nguồn dữ liệu.

## 5. Implementation Plan

### Step 1 – Inspect Schema and Existing API Conventions

- [ ] Kiểm tra routes/handlers/repositories đã có trong Axum Backend.
- [ ] Kiểm tra schema thực tế của regions và buildings.
- [ ] Xác định kiểu ID và các thuộc tính cần trả về.
- [ ] Xác định geometry type và SRID.
- [ ] Kiểm tra quan hệ buildings.region_id.
- [ ] Kiểm tra response/error conventions để tái sử dụng.

### Step 2 – Define DTOs and Response Contracts

- [ ] Tạo DTO danh sách khu vực.
- [ ] Tạo DTO chi tiết khu vực.
- [ ] Tạo DTO chi tiết công trình.
- [ ] Chuẩn hóa cách biểu diễn geometry bằng GeoJSON.
- [ ] Chỉ expose các thuộc tính cần thiết cho UI.
- [ ] Thống nhất response envelope và error format với KAN-15.
- [ ] Xác định behavior khi geometry không có hoặc không hợp lệ.

### Step 3 – Implement Region List and Detail Queries

Danh sách khu vực:
- Truy vấn các region cần hiển thị trong phạm vi MVP.
- Chỉ lấy các cột cần thiết.
- Có thứ tự kết quả ổn định.

Chi tiết khu vực:
- Truy vấn theo region ID.
- Trả thuộc tính và geometry.
- Không trả dữ liệu buildings nhúng trong response nếu không có nhu cầu rõ ràng; danh sách buildings được lấy qua KAN-15.

- [ ] Implement repository query lấy danh sách regions.
- [ ] Implement repository query lấy region theo ID.
- [ ] Serialize region geometry sang GeoJSON.
- [ ] Kiểm tra kết quả khi database không có region.

### Step 4 – Implement Building Detail Query

Truy vấn một công trình theo building ID.

- [ ] Implement repository query lấy building theo ID.
- [ ] Trả ID, region reference, các thuộc tính có thật và geometry.
- [ ] Serialize geometry sang GeoJSON.
- [ ] Không đưa logic pagination của KAN-15 vào endpoint detail.
- [ ] Không giả định building có height/floors nếu dataset không cung cấp.

### Step 5 – Implement Axum Handlers and Routes

- [ ] Tạo handler GET /api/v1/regions.
- [ ] Tạo handler GET /api/v1/regions/{region_id}.
- [ ] Tạo handler GET /api/v1/buildings/{building_id}.
- [ ] Parse và validate path parameters theo kiểu ID thực tế.
- [ ] Đăng ký routes trong router hiện tại.
- [ ] Trả 404 khi ID hợp lệ nhưng không tồn tại.
- [ ] Trả 400 khi ID sai định dạng (nếu kiểu ID cho phép xác định lỗi định dạng).
- [ ] Mapping lỗi database thành response an toàn.
- [ ] Đảm bảo response envelope nhất quán với KAN-15.

### Step 6 – Add Tests

- [ ] Test lấy danh sách regions có dữ liệu.
- [ ] Test danh sách regions rỗng.
- [ ] Test lấy region detail tồn tại.
- [ ] Test region ID không tồn tại trả 404.
- [ ] Test lấy building detail tồn tại.
- [ ] Test building ID không tồn tại trả 404.
- [ ] Test ID sai định dạng.
- [ ] Test geometry response là GeoJSON hợp lệ.
- [ ] Test các thuộc tính trả về khớp schema.
- [ ] Test lỗi database được xử lý an toàn.

### Step 7 – Document and Verify

- [ ] Cập nhật OpenAPI/API documentation nếu project có sử dụng.
- [ ] Thêm request/response examples cho ba endpoints.
- [ ] Ghi rõ endpoint danh sách buildings thuộc KAN-15 để tránh trùng chức năng.
- [ ] Chạy formatter, linter và test.
- [ ] Kiểm tra API bằng HTTP client.
- [ ] Xác nhận dữ liệu response đủ cho luồng chọn khu vực và xem chi tiết công trình ở Frontend.

## 6. Acceptance Criteria

- [ ] GET /api/v1/regions trả danh sách khu vực.
- [ ] GET /api/v1/regions/{region_id} trả chi tiết khu vực.
- [ ] GET /api/v1/buildings/{building_id} trả chi tiết công trình.
- [ ] Geometry được trả về theo định dạng thống nhất, ưu tiên GeoJSON.
- [ ] ID và thuộc tính khớp schema PostGIS thực tế.
- [ ] ID không tồn tại trả 404.
- [ ] Input sai định dạng được xử lý phù hợp.
- [ ] Response/error format nhất quán với API danh sách công trình KAN-15.
- [ ] Có tests cho success, not-found, validation và geometry.
- [ ] API documentation được cập nhật.

## 7. Deliverables

1. API danh sách khu vực.
2. API chi tiết khu vực.
3. API chi tiết công trình.
4. Repository/query, DTO và handlers liên quan.
5. Tests và tài liệu API.

## 8. Suggested Commit Sequence

~~~text
feat(api): define region and building detail DTOs

feat(api): implement region list and detail queries

feat(api): implement building detail query

feat(api): expose region and building detail endpoints

test(api): cover region and building detail endpoints

docs(api): document region and building detail APIs
~~~

## 9. Final Definition of Done

KAN-7 hoàn thành khi Frontend có thể truy vấn danh sách khu vực, lấy chi tiết một khu vực và lấy chi tiết một công trình theo ID; dữ liệu và geometry phản ánh đúng PostGIS, các trường hợp not-found/invalid được xử lý rõ ràng và có tests.

Ticket này chỉ triển khai API tra cứu khu vực và chi tiết công trình; không triển khai API danh sách công trình của KAN-15 hoặc Cesium integration của KAN-16.
