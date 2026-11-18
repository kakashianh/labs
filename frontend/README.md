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

## Spatial SIR visualization (KAN-28)

1. Initialize a spatial simulation and execute its run using the existing Phase 2 flow.
2. Copy the returned `simulation_id` into the **Spatial SIR · 3D** panel on the map.
3. The panel checks `GET /api/v1/simulations/{simulation_id}` and only visualizes runs with `status = completed`.
4. Use the time-step slider to load all pages from the building-results and risk-results APIs, together with SIR/risk aggregates and hotspot geometry for the same time step.
5. The default building style is the backend's `risk_level` classification. `infection_ratio` is available as an alternative. The legend reads its thresholds from the saved KAN-26 risk snapshot.
6. Click a building to inspect S/I/R/N, infection ratio and risk. Click a hotspot polygon to inspect its cluster statistics and highlight member buildings.
7. Building thematic styling and hotspot geometry can be toggled independently.

The timeline displays the backend-provided simulation time and does not convert it to a real-world date. Missing or unclassified data is rendered in neutral grey, never as Low risk. Changing time steps cancels/ignores stale requests so an older response cannot replace the currently selected frame.
