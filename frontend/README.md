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
- `GET /api/v1/buildings/{building_id}` (only after a building is selected)

The building service requests pages using the API's `pagination.total_pages` metadata (up to the supported page size of 500). It does not assume the current sample size is fixed. API Geometry values are converted to GeoJSON Features before loading them into Cesium. The API UUID is retained as the feature/entity ID and the region UUID is preserved in entity properties.

The map shows ground-level building footprints. No building height is fabricated because the current dataset does not provide verified height/floor values. Click a building footprint to highlight it and open the information panel. The panel fetches the selected building by its API UUID and displays the returned identifiers and available attributes; nullable name/type values use safe fallbacks. Clicking empty map space or closing the panel clears the selection. If the selected entity or its data source is removed while the viewer is mounted, the selection and panel are cleared as well.

## Verification

```bash
npm run lint
npm test
npm run build
```

Unit tests cover multi-page aggregation, empty data, a final partial page, GeoJSON FeatureCollection conversion, UUID/region-property/coordinate preservation, malformed geometry, selected-entity removal detection, building-detail response unwrapping, nullable fields, HTTP 400/404 responses, cancellation and network failures.

For the repeatable browser-level workflow (including live API checks, empty/error states, data-source lifecycle and camera interaction), follow [P1-11 map E2E verification checklist](./docs/map-e2e-checklist.md).

## Configuration and security

- `VITE_API_BASE_URL` must point to the Axum API reachable from the browser.
- `VITE_CESIUM_ION_TOKEN` is optional; the OSM basemap does not require it.
- Never put PostgreSQL passwords, Supabase service-role keys, or other server credentials in Vite environment variables. Vite variables are bundled into browser assets.
