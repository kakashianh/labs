# 3D Map Medical – Frontend

React + TypeScript + Vite frontend with CesiumJS.

## Requirements

- Node.js 22.17 or newer (Cesium engine requirement).
- npm.
- Rust + Axum backend and PostGIS for live GIS data.

## Local setup

```bash
cd frontend
npm ci
cp .env.example .env.local
npm run dev
```

Set `VITE_API_BASE_URL` in `.env.local` to the browser-accessible Axum API URL. The default is `http://localhost:8000`.

Start the backend and PostGIS from the repository root in another terminal:

```bash
docker compose up --build
```

The frontend calls only these backend endpoints:

- `GET /api/v1/regions`
- `GET /api/v1/buildings?page={page}&page_size={page_size}`

The building service requests pages using the API's `pagination.total_pages` metadata (up to the supported page size of 500). It does not assume the current sample size is fixed. API Geometry values are converted to GeoJSON Features before loading them into Cesium. The API UUID is retained as the feature/entity ID.

The map shows ground-level building footprints. No building height is fabricated because the current dataset does not provide verified height/floor values. Click a footprint to highlight it and display its UUID; the detailed information panel belongs to the next map-interaction task.

## Verification

```bash
npm run lint
npm test
npm run build
```

Unit tests cover multi-page aggregation, empty data, a final partial page, GeoJSON FeatureCollection conversion, UUID/property preservation, and malformed geometry handling.

## Configuration and security

- `VITE_API_BASE_URL` must point to the Axum API reachable from the browser.
- `VITE_CESIUM_ION_TOKEN` is optional; the OSM basemap does not require it.
- Never put PostgreSQL passwords, Supabase service-role keys, or other server credentials in Vite environment variables. Vite variables are bundled into browser assets.
