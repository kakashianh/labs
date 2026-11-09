Dựa trên nội dung đồ án “Xây dựng hệ thống bản đồ đô thị ba chiều tích hợp mô phỏng dịch tễ không gian”, mình đề xuất xây dựng theo hướng Web-based 3D GIS + Spatial Analysis + Epidemic Simulation.

Mình khuyên sử dụng React + CesiumJS + Python FastAPI + PostgreSQL/PostGIS. Đây là stack phù hợp để vừa thể hiện được kiến thức GIS 3D, vừa có backend xử lý dữ liệu không gian và thuật toán mô phỏng SIR.

Mục tiêu là xây dựng một sản phẩm có thể demo trực quan, không biến đồ án thành một hệ thống GIS quá phức tạp.

# 1. Đề xuất Tech Stack

## Recommended Tech Stack

[React Logo, Interactive, JavaScript, Web Development, Library PNG](https://images.openai.com/static-rsc-4/Fl2k_M2MV9G60-G7WybCJohvSsz5vwqQsQppvr32F68EV081GHJtW9RJC9qzUmIyqYoKRib8QukYMDHeoi9S68vPmo6QWCkUSZ04wokMQon4iVyQtTjASZavNLZps_OHoxGIXYHG61QQxMlUTbtbDTcfPlPzUqtZndtSvw9CQCM?purpose=inline)

Frontend – React + TypeScript

Recommended

- React + Vite
- TypeScript
- Tailwind CSS
- Zustand (State Management)
- React Query (API State)

[CesiumJS – Cesium](https://images.openai.com/static-rsc-4/qjecgVZi9-hbZNjo1Sxb2OBb8lNKr99nT9ub74rV08GRuDVSP87aPJHrRk2iWg-fLqRhQgScsYl14TRLYiS9u8EwDpg25W80brSyliS_1G7jE9uw2x1uLKpzpgQ0SJWq4zX50znaVCEr9Tfa-H-EQZMAWLxKZ3Vq7-txAkBUQPE?purpose=inline)

3D GIS – CesiumJS

- CesiumJS
- 3D Tiles
- GeoJSON
- Terrain / 3D Buildings
- Entity & Primitive API

[File:Python logo 01.svg - Wikimedia Commons](https://images.openai.com/static-rsc-4/4WQnbUUJdvO4G_O54H-9RXYeiNC8ap-hadl7zQO_37z43QV44clQv_3mffdRKUsamUYFp5veqRIRVDSmAnDHlkj1JtcAh46t9rVGeEsTz7Kfg4qMWcGYIrrsCkO7nIRnHcOOcRu-Lm-tFOE47ydEtspq6XPPqxxyX0kFOlIlqkM?purpose=inline)

Backend – Python FastAPI

- FastAPI
- Pydantic
- SQLAlchemy
- GeoPandas
- Shapely
- NumPy / Pandas

[7 spatial databases for your enterprise - FME by Safe Software](https://images.openai.com/static-rsc-4/TfniKKHF-XQJ5rAQiDZNzwcsV-MScryMXjEAUgjZcxZN-jWIieoDwr1lr-Dx5fLRxnT79iwmlYKa20MXzm8pWZFC7Xp2K26elFazL9KIIL7InoC2pkIC8Ux18RCLSo6et22VbIobWMDXxELHHriFXUNaw0k6uZsy12E0LKQVaYk?purpose=inline)

Database – PostgreSQL + PostGIS

- PostgreSQL
- PostGIS Extension
- Spatial Index (GiST)
- SQLAlchemy / GeoAlchemy2

[Apache - The ASF (@TheASF) on X](https://images.openai.com/static-rsc-4/MvAglT4CX8V33jk0sC99QvblAmy3h8vQWp4IczHeO3_8nD2Eq1NYXK9hpdkFBKYiJrlLlTFjj2ViYZ4tKd2fLUzJWme_2fiqqEWlQs8lJfRv1KIvI9lO7n3_qvE3Qxqh2_2Bv3SAFwyjIYphJW0Mg69KZTT04Ei3B2rSHeJArQ4?purpose=inline)

Data Visualization

- Apache ECharts
- Recharts (nếu cần biểu đồ đơn giản)

[Learn How to Develop, Deploy, and Run Your Application with Docker | by Neeraj Kushwaha | Medium](https://images.openai.com/static-rsc-4/vpeCahKUt8HLTzNJcfyZuraqvPe4DYkQcZ6H_bYFxktcjGE9IzKNCy_2wzufjFg1CV4gCPUSKqyNqOP1LsjwqsGi_Ptm9bXovNu4KVQgDh00UrSckl0PXpeQz3Fy185kJss3Wgjsqi7965WWyoiWi3LAsk2u_qRKHUOJ4rIUl0s?purpose=inline)

Deployment

- Docker
- Docker Compose
- Nginx

## 1.1. Vai trò từng công nghệ

| Công nghệ  | Trách nhiệm                              |
| ---------- | ---------------------------------------- |
| React      | Xây dựng giao diện Dashboard             |
| CesiumJS   | Hiển thị bản đồ và tòa nhà 3D            |
| FastAPI    | Cung cấp REST API                        |
| GeoPandas  | Đọc, xử lý dữ liệu GIS                   |
| Shapely    | Phân tích hình học không gian            |
| NumPy      | Tính toán mô hình SIR                    |
| PostgreSQL | Lưu trữ dữ liệu hệ thống                 |
| PostGIS    | Truy vấn và phân tích dữ liệu không gian |
| ECharts    | Biểu đồ thống kê dịch bệnh               |
| Docker     | Đóng gói và triển khai                   |

Tại sao chọn CesiumJS thay vì Three.js?

- CesiumJS được thiết kế chuyên cho GIS 3D, có hệ tọa độ địa lý, terrain, imagery và 3D Tiles.
- Three.js là thư viện đồ họa 3D tổng quát, cần tự xây dựng thêm nhiều chức năng GIS.
- Với bài toán hiển thị tòa nhà theo tọa độ thực tế, CesiumJS phù hợp hơn.

Tham khảo:

- CesiumJS
- [CesiumJS Documentation](https://cesium.com/learn/cesiumjs/ref-doc/?utm_source=chatgpt.com)
- PostGIS
- [PostGIS Documentation](https://postgis.net/documentation/?utm_source=chatgpt.com)

# 2. Các tính năng cần có

Mình chia thành 6 module. Trong đó 4 module đầu là chức năng cốt lõi, 2 module sau giúp hệ thống hoàn thiện hơn.

## Module 1. 3D Urban Map – Bản đồ đô thị 3D

Core Feature

[1.1 Million 3D Buildings in New York City with 3D Tiles – Cesium](https://images.openai.com/static-rsc-4/WzKkDLX-adEvuy0JzK88dD345AW6wX1BeD-MKJZbJuSpH0h0XOpTFFZ2t8DoVcf9e3VZk0ZM5TbtGNzwLHMTOaba0oXWvUpKecCOZeaSJiGhC-e2SiG9wnizxHEUhow3YyPWXvKzgrJB309lrxT2VRZ7vi8Koum0EO6PgtfFIiM?purpose=inline)

[Inspect Features in Your 3D Tilesets with Cesium Stories – Cesium](https://images.openai.com/static-rsc-4/DL2j_MhnzgRFDt_uKSrLzvSKkG7gcw44bKCt-ERn_tjHKTrrdq64RvzS7262yCQwqwSv5uCHf--Mb0mEbqJkxK0SpkdOXV2ICJT0odUzqj4kJGVdXpgplZlUIyH8LfOgLdkNbUcHcMStuOF0haG4E0P-w8B6IP8QyeM440uc6iI?purpose=inline)

[Styling and Filtering 3D Tiles – Cesium](https://images.openai.com/static-rsc-4/ubP_dFE4Q9t73WYVk4-PdkwxuurKp07OLVZlc0U5SJTYjlKaJQMlpjxG8oCe3kNphApFxWtu09X0oaiFFPcNnXKWxSe8fQrZ1zngpcjGjbV_MYgGfQd64q5hscQBGdheJl7j0Rb_9poBHe-q64XfB0KOInIgkKzStMowmRlc3m8?purpose=inline)

9

Các tính năng:

| Feature                | Mô tả                                  |
| ---------------------- | -------------------------------------- |
| 3D Map Rendering       | Hiển thị bản đồ đô thị 3D              |
| Building Visualization | Hiển thị các tòa nhà dạng khối 3D      |
| Building Selection     | Click tòa nhà để xem thông tin         |
| Map Navigation         | Zoom, Pan, Rotate, Tilt                |
| Layer Management       | Bật/tắt các lớp dữ liệu                |
| Building Information   | Xem số tầng, chiều cao, dân số         |
| Risk Visualization     | Tô màu tòa nhà theo trạng thái dịch tễ |

Ví dụ khi click vào một tòa nhà:

Building Information

# Building A-102

Số tầng

15

Dân số

450

Đang nhiễm (I)

# 28

Đã hồi phục (R)

12

Trạng thái mô phỏng

High Infection Activity

## Module 2. Epidemic Simulation – Mô phỏng dịch bệnh

Core Feature

Đây là module quan trọng nhất về mặt thuật toán.

Sử dụng mô hình SIR:

- S (Susceptible): Người cảm nhiễm.
- I (Infected): Người đang nhiễm.
- R (Recovered): Người hồi phục.

Các tính năng:

| Feature               | Mô tả                                 |
| --------------------- | ------------------------------------- |
| Initialize Simulation | Khởi tạo dân số và trạng thái dịch tễ |
| Configure Parameters  | Cấu hình Beta, Gamma                  |
| Run Simulation        | Chạy mô phỏng                         |
| Pause Simulation      | Tạm dừng                              |
| Reset Simulation      | Đặt lại                               |
| Time Step Control     | Điều chỉnh bước thời gian             |
| Timeline Slider       | Tua đến thời điểm bất kỳ              |
| Real-time Update      | Cập nhật trạng thái trên bản đồ 3D    |

Ví dụ cấu hình mô phỏng:

## Simulation Configuration

Infection Rate (β)

## 0.30

Recovery Rate (γ)

## 0.10

Initial Infected Population

Simulation Duration (days)

30 days60 days100 days

Các giá trị trên là tham số minh họa, không phải tham số dịch tễ đã hiệu chỉnh.

Thiết kế mô phỏng với cấu hình này

Lưu ý: SIR cơ bản chỉ mô tả quần thể. Bạn cần xây dựng thêm Spatial Transmission Rules để mô phỏng sự lan truyền giữa các tòa nhà, ví dụ dựa trên khoảng cách hoặc hệ số tiếp xúc giả định.

## Module 3. Spatial Analysis – Phân tích không gian

Core Feature

Đây là phần giúp đề tài có giá trị GIS thực sự, thay vì chỉ là một ứng dụng hiển thị mô hình 3D.

Các tính năng nên có:

| Feature                      | Mô tả                                |
| ---------------------------- | ------------------------------------ |
| Building Population Analysis | Phân tích dân số từng tòa nhà        |
| Infection Distribution       | Phân bố ca nhiễm theo tòa nhà        |
| Spatial Proximity            | Xác định các tòa nhà lân cận         |
| Distance Analysis            | Tính khoảng cách giữa các tòa nhà    |
| Hotspot Visualization        | Trực quan hóa vùng có nhiều ca nhiễm |
| Risk Classification          | Phân loại mức nguy cơ                |
| Regional Aggregation         | Tổng hợp số liệu theo khu vực        |

Ví dụ:

- Tòa nhà A có 20 ca nhiễm.
- Tòa nhà B cách A 100m.
- Tòa nhà C cách A 500m.

Hệ thống sử dụng quy tắc không gian đã thiết kế để tính mức độ tương tác giả định giữa các tòa nhà.

Có thể dùng PostGIS với các hàm như:

```
ST_Distance()
ST_DWithin()
ST_Intersects()
ST_Contains()
```

Ví dụ truy vấn các tòa nhà nằm trong bán kính 500m:

```
SELECT b.id, b.name
FROM buildings b
WHERE ST_DWithin(
    b.geom::geography,
    :target_location::geography,
    500
);
```

## Module 4. Statistics & Dashboard – Thống kê dịch bệnh

Core Feature

[Jordan Poles' Blog](https://images.openai.com/static-rsc-4/wMqAMMQ2ASBXwQw-UaXh8MaTGH8wb062anpCT1_Boq9_cykfEnyz5c8ykzWMy4Li-5HkZziU_raZxMkesjL1aIsIY8B5q9tngNLuVC52Y6CpRkqf86NhdqlnjxEmWcCXhgoGqwcIGDY-zwyFBEqcv4ghuiQnlWMrQJ7dS_e8z2c?purpose=inline)

[Big Data Analytics Companies: What’s the Difference Today?](https://images.openai.com/static-rsc-4/zBwGJb6RVvKfbfGq1rdSneA_9t-Bl39re4B5RN9_BOgNz2kNU2EGY0FrTkGR46kZOglzWS9mGN1xl9oN8bQVUh6WTVorYtlYDlXDuZhioESlCHIgJ4ZDP17fwXU5hczj6Ao6zPfjlH_HfZYlmj9vCBW_8ZEgt90ZcTxyJvIOrA0?purpose=inline)

[GeoNet Community Discussions Covering Coronavirus ... - Esri Community](https://images.openai.com/static-rsc-4/SIP5yDSwpskD_vvVQvQ6eWyikICk5CmLaGIMgj_MN0pM4nr8qJlNSU_nn-egdH2Q0AGR4DqRAwZTwrWJ-pHLhcE2tmYTL2gywB82wKQpP3AfeOuCFjP63jV8_pPDEsMO6z0cL_1uulDcQjnaqvlz7S1sLnWe0oFeZHIGx77pvFQ?purpose=inline)

10

Các biểu đồ nên có:

1. Epidemic Curve: Biểu đồ S, I, R theo thời gian.
2. Infection Distribution: Số ca nhiễm theo tòa nhà.
3. Population Distribution: Phân bố dân số theo khu vực.
4. Risk Distribution: Phân bố các cấp độ nguy cơ.
5. Peak Infection: Thời điểm và số ca nhiễm cực đại.

Ví dụ dashboard:

## Epidemic Overview

Total Population

# 12,500

Currently Infected

# 1,240

Recovered

# 3,100

Buildings Affected

# 38

Epidemic Curve

Dữ liệu giả lập minh họa diễn biến SIR.

SusceptibleInfectedRecovered

03 N6 N9 N12 N0102030405060

Số liệu minh họa, không phải kết quả chạy mô hình thực tế.

## Module 5. Scenario Management – Quản lý và so sánh kịch bản

Highly Recommended

Đây là tính năng giúp đồ án có chiều sâu hơn.

Cho phép tạo nhiều kịch bản mô phỏng với các tham số khác nhau.

| Kịch bản             | Điều kiện                             |
| -------------------- | ------------------------------------- |
| Baseline             | Không có biện pháp can thiệp          |
| Spatial Lockdown     | Cách ly một số tòa nhà                |
| Mobility Restriction | Giảm hệ số tương tác giữa các tòa nhà |
| Low Transmission     | Giảm hệ số lây nhiễm β                |

Các chức năng:

- Tạo kịch bản.
- Chỉnh sửa tham số.
- Lưu kết quả mô phỏng.
- Chạy lại kịch bản.
- So sánh biểu đồ.
- So sánh tổng số ca nhiễm.
- So sánh đỉnh dịch và thời điểm đạt đỉnh.

Ví dụ bảng so sánh:

| Metrics        | Baseline | Lockdown |
| -------------- | -------- | -------- |
| Peak Infected  | 2,800    | 1,400    |
| Peak Day       | 30       | 42       |
| Total Infected | 8,000    | 4,200    |

Số liệu trên chỉ minh họa cách trình bày.

## Module 6. Data Management – Quản lý dữ liệu

Recommended

Các tính năng:

- Import dữ liệu GeoJSON.
- Import dữ liệu tòa nhà.
- Import dữ liệu dân số.
- Quản lý khu vực nghiên cứu.
- Kiểm tra dữ liệu không hợp lệ.
- Quản lý bộ dữ liệu mô phỏng.
- Export kết quả CSV / GeoJSON.

Nguồn dữ liệu có thể tham khảo:

- OpenStreetMap: Dữ liệu đường phố và footprint tòa nhà.
- Overpass API: Truy vấn đối tượng từ OpenStreetMap.
- QGIS: Kiểm tra, chỉnh sửa và chuẩn hóa dữ liệu GIS.

# 3. Kiến trúc tổng thể đề xuất

Mình khuyên sử dụng kiến trúc Modular Monolith, chưa cần Microservices.

Frontend – React + TypeScript

Dashboard · Simulation Controls · Charts

REST API / JSON

Backend – FastAPI

Building Module

GIS Data Module

Simulation Module

Spatial Analysis

Scenario Module

Statistics Module

Simulation Engine

Python · NumPy

GIS Processing

GeoPandas · Shapely

PostgreSQL + PostGIS

Buildings · Population · Simulation Results · Scenarios

Luồng hoạt động:

1. Người dùng mở bản đồ 3D.
2. React gọi API lấy dữ liệu tòa nhà.
3. FastAPI truy vấn PostGIS.
4. CesiumJS render các tòa nhà 3D.
5. Người dùng cấu hình mô phỏng.
6. FastAPI gọi Simulation Engine.
7. Engine tính toán trạng thái S, I, R theo từng bước thời gian.
8. Kết quả được lưu vào database.
9. Frontend lấy kết quả và cập nhật bản đồ, biểu đồ.

# 4. Thiết kế Database

Mình đề xuất các bảng chính sau:

| Table                      | Mục đích                                       |
| -------------------------- | ---------------------------------------------- |
| `regions`                  | Khu vực nghiên cứu                             |
| `buildings`                | Thông tin và geometry tòa nhà                  |
| `building_population`      | Dân số từng tòa nhà                            |
| `scenarios`                | Cấu hình kịch bản                              |
| `simulations`              | Thông tin lần chạy mô phỏng                    |
| `simulation_steps`         | Kết quả S, I, R theo thời gian                 |
| `building_epidemic_states` | Trạng thái dịch tễ từng tòa nhà theo thời gian |

Ví dụ cấu trúc `buildings`:

```
CREATE TABLE buildings (
    id SERIAL PRIMARY KEY,
    building_code VARCHAR(100) UNIQUE,
    name VARCHAR(255),
    floors INTEGER,
    height FLOAT,
    population INTEGER,
    geom GEOMETRY(POLYGON, 4326)
);
```

Ví dụ cấu trúc `simulation_steps`:

```
CREATE TABLE simulation_steps (
    id SERIAL PRIMARY KEY,
    simulation_id INTEGER,
    time_step INTEGER,
    susceptible INTEGER,
    infected INTEGER,
    recovered INTEGER,
    created_at TIMESTAMP
);
```

Nếu cần tối ưu truy vấn không gian, thêm GiST index cho `geom`.

# 5. Phân chia phạm vi triển khai

Không nên triển khai tất cả tính năng cùng lúc. Mình đề xuất chia thành 3 giai đoạn:

Phase 1 – MVP

Must Have

Mục tiêu: Có sản phẩm chạy được và demo được.

- Hiển thị bản đồ 3D bằng CesiumJS.
- Import dữ liệu tòa nhà.
- Hiển thị thông tin dân số, số tầng.
- Click tòa nhà xem chi tiết.
- Xây dựng thuật toán SIR.
- Play / Pause / Reset simulation.
- Hiển thị màu theo trạng thái dịch tễ.
- Biểu đồ SIR theo thời gian.

Phase 2 – GIS & Analytics

Should Have

Mục tiêu: Làm rõ yếu tố phân tích không gian.

- Phân tích khoảng cách giữa các tòa nhà.
- Xây dựng quy tắc lây lan theo không gian.
- Thống kê ca nhiễm theo tòa nhà.
- Phân loại mức nguy cơ.
- Heatmap / Hotspot visualization.
- Timeline Slider.

Phase 3 – Advanced Features

Nice to Have

Mục tiêu: Tăng chiều sâu phân tích và chất lượng demo.

- Quản lý nhiều kịch bản.
- So sánh Baseline và Lockdown.
- Dashboard phân tích tổng hợp.
- Export kết quả CSV.
- Phân tích kết quả qua nhiều lần chạy.
- Docker hóa và triển khai.

# 6. Những tính năng chưa nên làm

Để tránh vượt quá phạm vi đồ án, mình không khuyên triển khai ngay:

- Mô phỏng từng cá nhân (Agent-Based Simulation).
- Mô phỏng lây nhiễm theo từng căn hộ/phòng.
- Mô phỏng phát tán virus trong không khí.
- Real-time tracking dân cư.
- Hệ thống dự báo dịch bệnh thực tế.
- Kiến trúc Microservices, Kafka, Kubernetes.

Những chức năng này đòi hỏi dữ liệu và độ phức tạp cao hơn. Tài liệu của bạn cũng xác định giới hạn không mô phỏng chi tiết hành vi từng cá nhân và không xem kết quả SIR giả định là dự báo dịch bệnh chính thức.&#x20;

23730080_Tran_Anh_Duong_KLTN-IE402.docx



# 7. Kết luận – Stack mình khuyên chốt

# Final Tech Stack

| Layer          | Technology                |
| -------------- | ------------------------- |
| Frontend       | React + TypeScript + Vite |
| UI             | Tailwind CSS              |
| State          | Zustand                   |
| API State      | TanStack Query            |
| 3D GIS         | CesiumJS                  |
| Backend        | Python + FastAPI          |
| GIS Processing | GeoPandas + Shapely       |
| Simulation     | NumPy + SciPy             |
| Database       | PostgreSQL + PostGIS      |
| Charts         | Apache ECharts            |
| Deployment     | Docker Compose + Nginx    |

Recommended Architecture

Một lời khuyên quan trọng: Giá trị chính của đồ án không nằm ở việc sử dụng thật nhiều công nghệ mà ở 3 thành phần:

1. 3D GIS: Có thể hiển thị và tương tác với dữ liệu đô thị theo tọa độ địa lý.
2. Spatial Simulation: Có thuật toán SIR kết hợp quy tắc tương tác không gian giữa các tòa nhà.
3. Spatiotemporal Analytics: Có thể phân tích diễn biến dịch bệnh theo cả không gian và thời gian, đồng thời so sánh các kịch bản.

Đây cũng là ba trụ cột nên được ưu tiên khi triển khai và bảo vệ đồ án.

Tài liệu tham khảo định hướng: [CesiumJS](https://cesium.com/learn/cesiumjs/ref-doc/?utm_source=chatgpt.com) · [FastAPI](https://fastapi.tiangolo.com/?utm_source=chatgpt.com) · [PostGIS](https://postgis.net/documentation/?utm_source=chatgpt.com) · [GeoPandas](https://geopandas.org/en/stable/?utm_source=chatgpt.com).
