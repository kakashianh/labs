# KAN-16 – Tích hợp dữ liệu công trình và hiển thị trên CesiumJS

## 1. Objective

Tích hợp Frontend React + CesiumJS với các API GIS đã triển khai ở KAN-15 và KAN-7 để tải và hiển thị dữ liệu khu vực/công trình trên bản đồ 3D.

Mục tiêu của ticket là đưa dữ liệu công trình thực tế từ PostGIS lên CesiumJS, thay thế trạng thái chỉ hiển thị bản đồ nền ở KAN-11.

**Expected result:**
- Frontend tải được danh sách khu vực và công trình từ Backend.
- Geometry công trình được hiển thị đúng vị trí trên CesiumJS.
- Có thể phân biệt/chọn công trình trên bản đồ.
- Dữ liệu hiển thị khớp với response API và khu vực nghiên cứu.
- Có trạng thái loading, empty và error cơ bản.

## 2. Scope

### In scope
- Kiểm tra API contract của KAN-15 và KAN-7.
- Cấu hình API base URL cho Frontend.
- Xây dựng API client/service và TypeScript types.
- Tải dữ liệu khu vực/công trình.
- Chuyển GeoJSON thành Cesium data source/entities.
- Hiển thị geometry công trình trên CesiumJS.
- Zoom tới khu vực nghiên cứu hoặc extent dữ liệu.
- Xử lý chọn công trình ở mức cơ bản.
- Quản lý loading/error/empty state.
- Kiểm tra hiệu năng hiển thị với dataset mẫu.

### Out of scope
- Hoàn thiện toàn bộ thao tác bản đồ và panel thông tin (KAN-17).
- Mô phỏng SIR và hiển thị trạng thái dịch tễ.
- Màu sắc theo mức độ nguy cơ dịch bệnh.
- Mô hình 3D building mesh có độ cao thực nếu dataset chưa có dữ liệu tương ứng.
- Tối ưu streaming/tiling cho dữ liệu đô thị quy mô rất lớn.
- Phân tích spatial hoặc clustering trên Frontend.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| UI | React + TypeScript | Giao diện |
| 3D GIS | CesiumJS | Hiển thị geometry |
| Build Tool | Vite | Frontend build |
| HTTP | API client hiện có hoặc fetch wrapper | Gọi Backend |
| Backend API | Rust + Axum | Cung cấp dữ liệu |
| Database | PostgreSQL + PostGIS | Nguồn dữ liệu GIS |

Không tạo backend/service mới. Frontend chỉ gọi API Backend đã có.

## 4. Data Flow

~~~text
PostgreSQL / PostGIS
        |
        v
Rust + Axum API
        |
        +------------------------------+
        |                              |
        v                              v
GET /api/v1/regions          GET /api/v1/buildings
GET /api/v1/regions/{id}     GET /api/v1/buildings/{id}
        |                              |
        +---------------+--------------+
                        |
                        v
                Frontend API Client
                        |
                        v
                TypeScript Data Model
                        |
                        v
              GeoJSON -> Cesium DataSource
                        |
                        v
                 CesiumJS Viewer
                        |
                        v
             Building Footprints on Map
~~~

API paths cần đối chiếu với route thực tế sau KAN-15/KAN-7; không hard-code một contract khác với Backend.

## 5. Implementation Plan

### Step 1 – Inspect API Contract and Existing Cesium Setup

- [ ] Kiểm tra CesiumViewer được tạo ở KAN-11.
- [ ] Kiểm tra cách cấu hình API base URL trong Frontend.
- [ ] Kiểm tra response thực tế của GET /api/v1/regions.
- [ ] Kiểm tra response thực tế của GET /api/v1/buildings.
- [ ] Kiểm tra geometry type, coordinate order và SRID/CRS của GeoJSON.
- [ ] Kiểm tra pagination metadata và giới hạn page_size.
- [ ] Xác định cách tải tất cả trang cần thiết trong phạm vi dataset mẫu.
- [ ] Không giả định API trả toàn bộ công trình trong một response.

### Step 2 – Define Frontend GIS Types and API Client

Tạo TypeScript types phản ánh API contract.

Các kiểu dữ liệu cần thiết:
- Region list item.
- Building item.
- GeoJSON geometry/feature phù hợp với dữ liệu thực tế.
- Pagination response.
- API error response.

- [ ] Tạo hoặc cập nhật thư mục src/types/ cho GIS types.
- [ ] Tạo service gọi regions API.
- [ ] Tạo service gọi buildings API.
- [ ] Implement pagination fetching nếu API phân trang.
- [ ] Cấu hình API base URL qua environment variable.
- [ ] Xử lý HTTP error và parse response.
- [ ] Tránh any không cần thiết trong phần mapping dữ liệu.

### Step 3 – Implement GIS Data Loading Flow

Tạo luồng tải dữ liệu tách khỏi component Cesium Viewer.

- [ ] Tải danh sách khu vực khi khởi tạo màn hình bản đồ.
- [ ] Tải danh sách công trình theo pagination và filter phù hợp.
- [ ] Xử lý trạng thái loading.
- [ ] Xử lý response rỗng.
- [ ] Xử lý API timeout/network error.
- [ ] Tránh gọi API lặp lại không cần thiết khi component re-render.
- [ ] Đảm bảo component có thể cleanup khi request không còn cần thiết.

### Step 4 – Convert GeoJSON to Cesium Data Source

Sử dụng khả năng load GeoJSON của CesiumJS hoặc cơ chế mapping tương ứng.

Yêu cầu:
- Giữ nguyên định danh công trình để có thể truy ngược khi click.
- Giữ các properties cần thiết cho popup ở KAN-17.
- Kiểm tra coordinate order và CRS; GeoJSON theo chuẩn sử dụng longitude/latitude (WGS84) khi truyền tới Cesium.
- Nếu dữ liệu PostGIS lưu ở CRS khác, Backend phải trả GeoJSON ở CRS phù hợp hoặc Frontend phải chuyển đổi có chủ đích; không được hiển thị sai vị trí do nhầm SRID.
- Chỉ dùng height/extrusion khi thuộc tính độ cao thực sự tồn tại và có đơn vị rõ ràng.
- Nếu không có height, hiển thị footprint dạng polygon trên mặt đất, không tự bịa chiều cao công trình.

- [ ] Implement hàm chuyển GeoJSON thành Cesium DataSource.
- [ ] Gắn building ID và properties vào entity.
- [ ] Cấu hình style mặc định cho footprint.
- [ ] Kiểm tra polygon/multipolygon nếu dataset có cả hai loại.
- [ ] Kiểm tra geometry null/empty được bỏ qua hoặc xử lý an toàn.
- [ ] Không tạo building extrusion từ dữ liệu không có căn cứ.

### Step 5 – Integrate Data Source into Cesium Viewer

- [ ] Tích hợp data loading flow vào trang bản đồ.
- [ ] Add/remove data source đúng vòng đời component.
- [ ] Tránh add trùng entities khi tải lại dữ liệu.
- [ ] Zoom camera tới extent của khu vực/dữ liệu sau khi tải thành công.
- [ ] Giữ thao tác camera cơ bản của Cesium Viewer.
- [ ] Đảm bảo Viewer cleanup không để lại event handler/data source cũ.
- [ ] Kiểm tra tải lại trang không tạo nhiều lớp công trình trùng nhau.

### Step 6 – Implement Basic Building Selection

Phạm vi chỉ cần xác nhận người dùng có thể chọn một công trình trên bản đồ; panel và trải nghiệm tương tác đầy đủ thuộc KAN-17.

- [ ] Đăng ký click handler cho Cesium Viewer.
- [ ] Xác định entity công trình được click.
- [ ] Lấy building ID từ entity properties.
- [ ] Lưu selected building ID trong React state hoặc state management hiện có.
- [ ] Có thể highlight công trình được chọn ở mức cơ bản.
- [ ] Cleanup ScreenSpaceEventHandler khi component unmount.
- [ ] Không gọi building detail API hoặc xây dựng panel chi tiết nếu chưa cần cho acceptance của ticket này.

### Step 7 – Verify Data Accuracy and Performance

- [ ] Đối chiếu số công trình nhận từ API với số entity được hiển thị (tính đến pagination/filter).
- [ ] Kiểm tra vị trí công trình trên đúng khu vực nghiên cứu.
- [ ] Kiểm tra một số geometry đại diện bằng cách đối chiếu PostGIS/GeoJSON.
- [ ] Kiểm tra click chọn công trình trả đúng ID.
- [ ] Kiểm tra trường hợp API trả empty list.
- [ ] Kiểm tra trường hợp Backend không khả dụng.
- [ ] Kiểm tra thời gian tải và khả năng tương tác trên dataset mẫu.
- [ ] Nếu dataset quá lớn cho một lần render, ghi nhận giới hạn và chỉ áp dụng giải pháp tải phù hợp phạm vi MVP; không tự mở rộng thành 3D Tiles pipeline.

### Step 8 – Document and Final Verification

- [ ] Cập nhật hướng dẫn cấu hình API URL.
- [ ] Ghi rõ cách chạy Frontend và Backend đồng thời.
- [ ] Ghi nhận API endpoints mà Frontend sử dụng.
- [ ] Chạy TypeScript checks và production build.
- [ ] Kiểm tra browser console không có lỗi Cesium/API nghiêm trọng.
- [ ] Kiểm tra luồng end-to-end từ PostGIS đến Cesium Viewer.

## 6. Acceptance Criteria

- [ ] Frontend gọi thành công API regions và buildings.
- [ ] Có thể tải dữ liệu công trình qua pagination khi API yêu cầu.
- [ ] Geometry công trình được hiển thị trên đúng vị trí bản đồ.
- [ ] Các properties/ID được giữ lại trong Cesium entity.
- [ ] Người dùng có thể click chọn một công trình.
- [ ] Không hiển thị sai vị trí do CRS/coordinate order.
- [ ] Không tự tạo chiều cao 3D khi dataset không cung cấp dữ liệu height hợp lệ.
- [ ] Loading, empty và error state được xử lý.
- [ ] Không tạo entities/data sources trùng khi component re-render hoặc tải lại.
- [ ] Frontend production build thành công.

## 7. Deliverables

1. GIS TypeScript types.
2. Frontend API client/services cho regions và buildings.
3. Luồng tải dữ liệu GIS từ Backend.
4. Cesium data source hiển thị footprint công trình.
5. Cơ chế chọn công trình cơ bản.
6. Hướng dẫn cấu hình và chạy tích hợp.

## 8. Suggested Commit Sequence

~~~text
feat(frontend): define GIS API types and client

feat(frontend): load regions and paginated buildings

feat(map): convert building GeoJSON into Cesium entities

feat(map): render building footprints on Cesium viewer

feat(map): support basic building selection

test(frontend): verify GIS data rendering and API states

docs(frontend): document GIS map integration
~~~

## 9. Final Definition of Done

KAN-16 hoàn thành khi dữ liệu khu vực/công trình từ PostGIS đi qua Rust + Axum API và được Frontend tải, chuyển thành Cesium entities, hiển thị đúng vị trí trên bản đồ; người dùng có thể chọn công trình và các trạng thái loading/error/empty được xử lý.

Ticket này hoàn thiện tích hợp dữ liệu GIS lên bản đồ; chưa triển khai panel thông tin và thao tác bản đồ đầy đủ của KAN-17, cũng chưa hiển thị bất kỳ trạng thái SIR nào.
