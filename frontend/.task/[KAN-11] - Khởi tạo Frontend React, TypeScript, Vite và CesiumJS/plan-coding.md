# KAN-11 – Khởi tạo Frontend React, TypeScript, Vite và CesiumJS

## 1. Objective

Khởi tạo nền tảng frontend cho dự án 3D GIS Medical bằng React, TypeScript và Vite.

Tích hợp CesiumJS để hiển thị một bản đồ 3D cơ bản trên trình duyệt.

**Expected result:**
- Frontend application chạy được trên local.
- React và TypeScript hoạt động.
- CesiumJS được tích hợp thành công.
- Hiển thị được Cesium Viewer với bản đồ nền.
- Không có lỗi runtime nghiêm trọng liên quan đến CesiumJS.

## 2. Scope

### In scope
- Khởi tạo React + TypeScript + Vite.
- Thiết lập cấu trúc thư mục frontend.
- Cài đặt và cấu hình CesiumJS.
- Tích hợp Cesium Viewer vào React.
- Cấu hình Cesium static assets.
- Thiết lập giao diện bản đồ toàn màn hình.
- Kiểm tra development server và production build.

### Out of scope
- Hiển thị dữ liệu tòa nhà 3D của khu vực nghiên cứu.
- Import GeoJSON hoặc dữ liệu GIS.
- Tích hợp PostGIS và Backend API.
- Mô phỏng dịch bệnh SIR.
- Hiển thị biểu đồ và thống kê.
- Quản lý các lớp dữ liệu GIS.

## 3. Technical Stack

| Component | Technology | Purpose |
|---|---|---|
| UI Framework | React | Xây dựng giao diện |
| Language | TypeScript | Static typing |
| Build Tool | Vite | Development server và build |
| 3D GIS Engine | CesiumJS | Hiển thị bản đồ 3D |
| Styling | CSS | Layout và giao diện cơ bản |
| Package Manager | npm | Quản lý dependencies |

## 4. Implementation Plan

### Step 1 – Initialize React + TypeScript + Vite

- [ ] Kiểm tra Node.js và npm đã được cài đặt.
- [ ] Khởi tạo React application sử dụng TypeScript template.
- [ ] Cài đặt dependencies.
- [ ] Chạy thử development server.
- [ ] Xác nhận trang React mặc định hiển thị thành công.

Expected commands:

```bash
cd frontend

npm create vite@latest . -- --template react-ts

npm install

npm run dev
```

Expected result:
- Có `package.json`.
- Có `tsconfig.json`.
- Có `vite.config.ts`.
- Có thư mục `src/`.
- Ứng dụng truy cập được qua local development URL.

### Step 2 – Setup Frontend Project Structure

Tổ chức source code để thuận tiện mở rộng trong các phase tiếp theo.

```text
frontend/
├── public/
│   └── cesium/
│
├── src/
│   ├── components/
│   │   └── map/
│   │       └── CesiumViewer.tsx
│   │
│   ├── pages/
│   │   └── HomePage.tsx
│   │
│   ├── styles/
│   │   └── global.css
│   │
│   ├── App.tsx
│   └── main.tsx
│
├── .env
├── .gitignore
├── index.html
├── package.json
├── package-lock.json
├── tsconfig.json
└── vite.config.ts
```

- [ ] Tạo component `CesiumViewer`.
- [ ] Tạo `HomePage` để chứa bản đồ.
- [ ] Tách CSS khỏi component chính.
- [ ] Xóa các thành phần demo mặc định không cần thiết của Vite.

### Step 3 – Install and Configure CesiumJS

Cài đặt CesiumJS:

```bash
npm install cesium
npm install -D vite-plugin-static-copy
```

Cấu hình Vite để copy Cesium static assets vào thư mục public.

Các assets cần được phục vụ gồm:
- Workers
- Assets
- Widgets
- ThirdParty

Cấu hình `vite.config.ts` sử dụng `vite-plugin-static-copy` để copy nội dung từ:

```text
node_modules/cesium/Build/Cesium/
```

đến:

```text
public/cesium/
```

Thiết lập `CESIUM_BASE_URL` trong `index.html`:

```html
<script>
  window.CESIUM_BASE_URL = "/cesium/";
</script>
```

Import Cesium widget stylesheet:

```typescript
import "cesium/Build/Cesium/Widgets/widgets.css";
```

- [ ] Cài đặt CesiumJS.
- [ ] Cấu hình static assets.
- [ ] Cấu hình `CESIUM_BASE_URL`.
- [ ] Import Cesium widgets CSS.
- [ ] Xác nhận không có lỗi tải Workers hoặc assets khi chạy ứng dụng.

### Step 4 – Implement Cesium Viewer Component

Tạo component:

```text
src/components/map/CesiumViewer.tsx
```

Responsibilities:
- Khởi tạo Cesium Viewer.
- Render Viewer vào một container HTML.
- Cấu hình bản đồ nền.
- Cấu hình camera ban đầu.
- Cleanup Viewer khi React component unmount.

Các cấu hình Viewer ban đầu:
- Ẩn Animation.
- Ẩn Timeline.
- Ẩn Geocoder.
- Ẩn Base Layer Picker.
- Ẩn các control chưa cần thiết.
- Tắt các UI mặc định không cần cho MVP.

Sử dụng `useEffect` để quản lý vòng đời Cesium Viewer.

Lưu ý:
- Không khởi tạo Cesium Viewer trực tiếp ngoài React lifecycle.
- Phải gọi `viewer.destroy()` khi component unmount.
- Tránh khởi tạo nhiều Viewer trên cùng một DOM element.
- Container bản đồ phải có width và height xác định.

- [ ] Tạo Cesium Viewer component.
- [ ] Khởi tạo Viewer trong `useEffect`.
- [ ] Cấu hình camera mặc định.
- [ ] Cấu hình bản đồ nền.
- [ ] Cleanup Viewer đúng lifecycle.
- [ ] Kiểm tra không có lỗi khi React StrictMode mount/unmount.

### Step 5 – Implement Initial Map Page

Tạo trang HomePage chứa Cesium Viewer.

Expected layout:

```text
+---------------------------------------------+
|                                             |
|                                             |
|                                             |
|               Cesium 3D Map                 |
|                                             |
|                                             |
|                                             |
|                                             |
+---------------------------------------------+
```

Yêu cầu:
- Bản đồ chiếm toàn bộ vùng hiển thị chính.
- Không xuất hiện scrollbar ngoài ý muốn.
- Có thể zoom bằng chuột.
- Có thể xoay và nghiêng góc nhìn.
- Có thể di chuyển camera.
- Hiển thị được bản đồ nền.

Ở bước này có thể sử dụng OpenStreetMap imagery làm bản đồ nền để kiểm tra mà chưa cần dữ liệu 3D hoặc Cesium Ion token.

- [ ] Tích hợp CesiumViewer vào HomePage.
- [ ] Thiết lập layout full-screen.
- [ ] Kiểm tra zoom, rotate, tilt và pan.
- [ ] Kiểm tra hiển thị trên kích thước màn hình khác nhau.

### Step 6 – Configure Environment and Git

- [ ] Kiểm tra file `.env` hiện tại.
- [ ] Chỉ thêm environment variables thực sự cần thiết.
- [ ] Không commit API token hoặc secret lên Git.
- [ ] Cập nhật `.gitignore`.
- [ ] Đảm bảo `package-lock.json` được commit.
- [ ] Đảm bảo các Cesium static assets được generate/copy tự động khi chạy build, không cần commit `node_modules`.

Nếu sử dụng Cesium Ion token ở các bước mở rộng sau này, đặt token trong biến môi trường:

```env
VITE_CESIUM_ION_TOKEN=
```

Không bắt buộc token cho bản đồ nền OSM cơ bản ở ticket này.

### Step 7 – Verify Development and Production Build

Kiểm tra development:

```bash
npm run dev
```

Kiểm tra production build:

```bash
npm run build
```

Kiểm tra:
- [ ] Không có TypeScript errors.
- [ ] Không có lỗi import CesiumJS.
- [ ] Không có lỗi load Cesium Workers.
- [ ] Không có lỗi load Cesium static assets.
- [ ] Viewer hiển thị thành công.
- [ ] Các thao tác camera hoạt động.
- [ ] Production build hoàn tất thành công.

## 5. Acceptance Criteria

Ticket được xem là hoàn thành khi đáp ứng tất cả điều kiện:

- [ ] React + TypeScript + Vite được khởi tạo thành công trong `frontend/`.
- [ ] Có cấu trúc source code cơ bản.
- [ ] CesiumJS được cài đặt và tích hợp.
- [ ] Cesium static assets được cấu hình chính xác.
- [ ] Trình duyệt hiển thị được Cesium Viewer.
- [ ] Bản đồ nền hiển thị thành công.
- [ ] Có thể zoom, rotate, tilt và pan bản đồ.
- [ ] Không có lỗi runtime nghiêm trọng.
- [ ] `npm run build` chạy thành công.

## 6. Deliverables

Sau khi hoàn thành cần có:

1. React + TypeScript + Vite application.
2. CesiumJS integration.
3. `CesiumViewer` component.
4. Trang HomePage hiển thị bản đồ 3D.
5. Cấu hình Vite và Cesium static assets.
6. Source code được commit lên branch `project/3D-map-medical`.

## 7. Suggested Commit Sequence

```text
chore(frontend): initialize React TypeScript Vite project

chore(frontend): configure CesiumJS and static assets

feat(frontend): implement Cesium viewer component

feat(frontend): display initial 3D map page

test(frontend): verify Cesium viewer and production build
```

## 8. Final Definition of Done

KAN-11 hoàn thành khi mở frontend trên trình duyệt và nhìn thấy Cesium Viewer hoạt động ổn định, có thể tương tác với bản đồ nền và production build thành công.

Chưa yêu cầu hiển thị tòa nhà 3D, dữ liệu dân số hoặc kết quả mô phỏng dịch bệnh trong ticket này.
