https://facebookanhduong.atlassian.net/browse/KAN-12

# KAN-12 – Khởi tạo Backend FastAPI và Docker Compose

## 1. Mục tiêu

Khởi tạo nền tảng Backend cho hệ thống bản đồ đô thị 3D tích hợp mô phỏng dịch tễ không gian.

Sau khi hoàn thành ticket, hệ thống cần có:

- Backend sử dụng FastAPI và Python.
- Cấu trúc source code rõ ràng, dễ mở rộng.
- Health-check endpoint để kiểm tra trạng thái backend.
- Dockerfile để đóng gói backend thành Docker image.
- Docker Compose để khởi chạy backend và các dịch vụ nền tảng.
- Có thể chạy backend bằng môi trường local hoặc Docker.

**Phạm vi ticket chỉ tập trung vào khởi tạo nền tảng, chưa triển khai nghiệp vụ GIS hoặc mô phỏng dịch tễ.**

## 2. Phạm vi công việc

### 2.1. In Scope

- Khởi tạo Python project.
- Cài đặt và cấu hình FastAPI.
- Thiết lập cấu trúc thư mục backend.
- Tạo API health-check.
- Cấu hình CORS để Frontend có thể gọi Backend.
- Quản lý biến môi trường.
- Tạo Dockerfile.
- Tạo Docker Compose.
- Kiểm tra khả năng khởi chạy backend bằng Docker.
- Viết README hướng dẫn chạy dự án.

### 2.2. Out of Scope

Không triển khai trong ticket này:

- API quản lý dữ liệu tòa nhà.
- API quản lý khu vực địa lý.
- Tích hợp PostgreSQL/PostGIS và thiết kế database nghiệp vụ (nếu được tách thành ticket riêng).
- Xử lý GeoJSON, 3D Tiles hoặc dữ liệu GIS.
- Thuật toán SIR.
- Mô phỏng dịch bệnh theo thời gian.
- API thống kê và phân tích không gian.
- Authentication và Authorization.
- Redis, Kafka hoặc hệ thống message queue.
- Microservices.

## 3. Tech Stack

| Thành phần | Công nghệ | Mục đích |
|---|---|---|
| Programming Language | Python 3.12 | Ngôn ngữ Backend |
| Web Framework | FastAPI | Xây dựng REST API |
| ASGI Server | Uvicorn | Chạy ứng dụng FastAPI |
| Validation | Pydantic | Validate request/response |
| Environment | python-dotenv hoặc pydantic-settings | Quản lý cấu hình |
| Containerization | Docker | Đóng gói Backend |
| Orchestration | Docker Compose | Khởi chạy các dịch vụ |
| API Documentation | Swagger / OpenAPI | Tài liệu API tự động |

**Định hướng:** Sử dụng Modular Monolith cho Backend MVP. Chưa cần chia thành nhiều service độc lập.

## 4. Implementation Plan

### Step 1. Khởi tạo Backend Project

Tại thư mục root của repository, tạo thư mục `backend`.

Cấu trúc ban đầu:

```text
3D-map-medical/
├── frontend/
├── backend/
└── README.md
```

Khởi tạo virtual environment:

```bash
cd backend

python -m venv .venv
```

Kích hoạt môi trường:

Windows:

```bash
.venv\Scripts\activate
```

Linux / macOS:

```bash
source .venv/bin/activate
```

Cài đặt dependencies:

```bash
pip install fastapi "uvicorn[standard]" pydantic-settings
```

Tạo file `requirements.txt`:

```bash
pip freeze > requirements.txt
```

Khởi tạo file `main.py` và kiểm tra ứng dụng FastAPI có thể chạy được.

### Step 2. Thiết kế Backend Folder Structure

Thiết lập cấu trúc source code ban đầu:

```text
backend/
├── app/
│   ├── __init__.py
│   ├── main.py
│   │
│   ├── api/
│   │   ├── __init__.py
│   │   └── routes/
│   │       ├── __init__.py
│   │       └── health.py
│   │
│   ├── core/
│   │   ├── __init__.py
│   │   └── config.py
│   │
│   └── schemas/
│       ├── __init__.py
│       └── health.py
│
├── tests/
│   └── test_health.py
│
├── .env
├── .env.example
├── .dockerignore
├── .gitignore
├── Dockerfile
└── requirements.txt
```

Trách nhiệm từng thành phần:

| Thành phần | Trách nhiệm |
|---|---|
| `main.py` | Khởi tạo FastAPI application |
| `api/routes/` | Khai báo các API endpoint |
| `core/config.py` | Quản lý cấu hình ứng dụng |
| `schemas/` | Định nghĩa request/response schema |
| `tests/` | Chứa các bài kiểm thử |
| `Dockerfile` | Định nghĩa cách build Backend image |
| `.env` | Lưu cấu hình môi trường local |

Không cần tạo trước các module nghiệp vụ như `buildings`, `simulation`, `spatial_analysis` nếu chưa triển khai trong ticket này.

### Step 3. Implement FastAPI Application

Tạo FastAPI application trong `app/main.py`.

Yêu cầu:

- Khởi tạo FastAPI instance.
- Thiết lập tên ứng dụng và version.
- Đăng ký API router.
- Cấu hình CORS middleware.
- Có thể truy cập Swagger UI tại `/docs`.
- Có thể truy cập OpenAPI schema tại `/openapi.json`.

CORS cần cho phép Frontend local truy cập Backend.

Ví dụ:

```python
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.api.routes.health import router as health_router

app = FastAPI(
    title="3D Map Medical API",
    version="0.1.0",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(health_router)
```

Lưu ý: Origin của Frontend nên được chuyển sang cấu hình môi trường thay vì hard-code khi hoàn thiện cấu hình.

### Step 4. Implement Health-check API

Tạo endpoint:

```http
GET /api/v1/health
```

Mục đích:

- Kiểm tra Backend có đang hoạt động hay không.
- Phục vụ Docker health check.
- Hỗ trợ kiểm tra tình trạng ứng dụng trong quá trình phát triển.

Response đề xuất:

```json
{
  "status": "healthy",
  "service": "3d-map-medical-api",
  "version": "0.1.0"
}
```

HTTP Status:

```http
200 OK
```

Tách endpoint health-check khỏi các API nghiệp vụ để dễ quản lý về sau.

**Lưu ý:** Ở ticket này, health-check chỉ xác nhận ứng dụng đang hoạt động. Chưa cần kiểm tra database nếu database chưa được tích hợp.

### Step 5. Implement Application Configuration

Tạo `app/core/config.py` để quản lý các biến môi trường.

Một số cấu hình ban đầu:

```env
APP_NAME=3D Map Medical API
APP_VERSION=0.1.0
APP_ENV=development
API_V1_PREFIX=/api/v1

HOST=0.0.0.0
PORT=8000

FRONTEND_ORIGIN=http://localhost:5173
```

Yêu cầu:

- Không hard-code cấu hình môi trường trong source code.
- Có `.env.example` để hướng dẫn cấu hình.
- Không commit secret hoặc thông tin nhạy cảm lên Git.
- Backend chạy được khi có file `.env` hợp lệ.

### Step 6. Create Dockerfile

Tạo Dockerfile để đóng gói Backend.

Các yêu cầu:

- Sử dụng Python base image.
- Thiết lập working directory.
- Cài đặt dependencies từ `requirements.txt`.
- Copy source code vào container.
- Expose port `8000`.
- Khởi chạy FastAPI bằng Uvicorn.

Ví dụ:

```dockerfile
FROM python:3.12-slim

WORKDIR /app

ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1

COPY requirements.txt .

RUN pip install --no-cache-dir -r requirements.txt

COPY ./app ./app

EXPOSE 8000

CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
```

Kiểm tra Docker image có thể build thành công:

```bash
docker build -t 3d-map-medical-backend .
```

### Step 7. Configure Docker Compose

Tạo `compose.yaml` tại thư mục root repository.

Mục tiêu:

- Khởi chạy Backend bằng một lệnh.
- Đồng bộ cấu hình môi trường.
- Map port từ host sang container.
- Có health check cho Backend.
- Dễ dàng bổ sung PostgreSQL/PostGIS trong ticket database sau này.

Cấu hình tối thiểu cho ticket này:

```yaml
services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile

    container_name: medical-backend

    ports:
      - "8000:8000"

    env_file:
      - ./backend/.env

    restart: unless-stopped

    healthcheck:
      test:
        - CMD
        - python
        - -c
        - "import urllib.request; urllib.request.urlopen('http://localhost:8000/api/v1/health')"
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
```

**Lưu ý về database:**

- Không cần thêm PostgreSQL, Redis hoặc Kafka nếu các dịch vụ này chưa thuộc phạm vi KAN-12.
- Khi triển khai ticket database, có thể bổ sung PostgreSQL/PostGIS vào cùng Compose network.
- Khi thêm database, cần cấu hình health check cho database và chỉ cho Backend phụ thuộc trạng thái `service_healthy` nếu Backend cần database ngay khi khởi động.

### Step 8. Configure Git Ignore

Cập nhật `.gitignore` tại root hoặc trong `backend` để bỏ qua các file không cần thiết:

```gitignore
# Python
__pycache__/
*.py[cod]
*.pyo

# Virtual environment
.venv/
venv/

# Environment
.env

# Testing
.pytest_cache/
.coverage
htmlcov/

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db
```

Đảm bảo `.env.example` vẫn được commit.

### Step 9. Testing & Verification

#### 9.1. Test Local Development

Chạy Backend:

```bash
cd backend

uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
```

Kiểm tra:

```http
GET http://localhost:8000/api/v1/health
```

Mở Swagger:

```text
http://localhost:8000/docs
```

#### 9.2. Test Docker Build

Tại root repository:

```bash
docker compose build
```

#### 9.3. Test Docker Compose

Khởi chạy:

```bash
docker compose up -d
```

Kiểm tra container:

```bash
docker compose ps
```

Xem logs:

```bash
docker compose logs -f backend
```

Kiểm tra API:

```bash
curl http://localhost:8000/api/v1/health
```

Dừng hệ thống:

```bash
docker compose down
```

#### 9.4. Test Health Check

Kiểm tra trạng thái container:

```bash
docker inspect medical-backend
```

Đảm bảo Docker có thể gọi endpoint health-check và nhận được trạng thái healthy.

#### 9.5. Test API Unit Test

Cài đặt:

```bash
pip install pytest httpx
```

Kiểm tra các trường hợp:

- Health endpoint trả về HTTP 200.
- Response có đầy đủ các field cần thiết.
- API router được đăng ký đúng.
- Không xảy ra lỗi khi khởi tạo FastAPI application.

## 5. Acceptance Criteria

- [ ] Có thư mục `backend/` tại root repository.
- [ ] Khởi tạo thành công FastAPI project.
- [ ] Có cấu trúc source code rõ ràng.
- [ ] Có endpoint `GET /api/v1/health`.
- [ ] Health endpoint trả về HTTP 200 và response hợp lệ.
- [ ] Có cấu hình CORS cho Frontend local.
- [ ] Có file `.env.example`.
- [ ] Không commit file `.env` chứa cấu hình local hoặc secret.
- [ ] Có Dockerfile build Backend thành công.
- [ ] Có `compose.yaml` tại root repository.
- [ ] Backend có thể khởi chạy bằng Docker Compose.
- [ ] Docker health check hoạt động.
- [ ] Swagger UI truy cập được tại `/docs`.
- [ ] Backend chạy được cả local và Docker.
- [ ] Có README hướng dẫn khởi chạy và kiểm tra API.

## 6. Expected Deliverables

Sau khi hoàn thành ticket, repository cần có cấu trúc:

```text
3D-map-medical/
├── frontend/
│
├── backend/
│   ├── app/
│   │   ├── api/
│   │   ├── core/
│   │   ├── schemas/
│   │   └── main.py
│   ├── tests/
│   ├── .env.example
│   ├── .dockerignore
│   ├── Dockerfile
│   └── requirements.txt
│
├── compose.yaml
├── .gitignore
└── README.md
```

## 7. Suggested Commit Strategy

```text
chore(backend): initialize FastAPI project

feat(backend): implement health check endpoint

chore(backend): configure environment settings and CORS

chore(docker): add backend Dockerfile

chore(docker): configure Docker Compose

test(backend): add health endpoint tests

docs: add backend setup and run instructions
```

## 8. Definition of Done

Ticket được xem là hoàn thành khi:

1. Backend FastAPI có thể chạy độc lập trên local.
2. Health-check API trả về kết quả hợp lệ.
3. Docker image build thành công.
4. Docker Compose có thể khởi chạy Backend.
5. Backend container chuyển sang trạng thái healthy.
6. Swagger UI hoạt động.
7. README có hướng dẫn để một developer khác có thể clone repository và chạy Backend theo hướng dẫn.

**Kết quả cuối cùng:** KAN-12 cung cấp nền tảng Backend và môi trường container hóa ổn định để các ticket tiếp theo có thể triển khai database, API dữ liệu GIS và mô phỏng dịch tễ.
