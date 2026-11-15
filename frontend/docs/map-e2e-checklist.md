# P1-11 – Map end-to-end verification checklist

This checklist validates the integrated P1-09/P1-10 workflow. It is intentionally separate from the unit-test suite: a passing unit test does not prove that the live Backend, PostGIS, browser, Cesium and API configuration work together.

## 1. Test environment

Record these values before a manual run:

| Item | Value |
|---|---|
| Branch / commit | `feat/p1-11-map-e2e-integration` / fill in commit SHA |
| Browser and version | fill in |
| OS / runtime | fill in |
| Backend URL | `http://localhost:8000` (or Codespaces forwarded URL) |
| Frontend URL | `http://localhost:5173` (or forwarded URL) |
| Dataset snapshot | record actual API `pagination.total`; do not assume a fixed count |
| CI run | add GitHub Actions run URL |

Do not commit local `.env` files, database passwords, API credentials or Cesium secrets.

## 2. Start the application

1. Configure `POSTGRES_PASSWORD` in the local shell or an ignored environment file. Keep the secret out of source control and chat.
2. Start the repository services from the repository root:

   ```bash
   docker compose up --build -d
   ```

3. Check the API health endpoint:

   ```bash
   curl --fail http://localhost:8000/health
   ```

4. Check the list endpoints and record the live pagination metadata:

   ```bash
   curl --fail 'http://localhost:8000/api/v1/regions'
   curl --fail 'http://localhost:8000/api/v1/buildings?page=1&page_size=100'
   ```

5. Start the frontend in a second terminal:

   ```bash
   cd frontend
   npm ci
   npm run dev -- --host 0.0.0.0
   ```

6. Open the frontend URL in a browser. In Codespaces, use the forwarded port URL and make sure `VITE_API_BASE_URL` is browser-reachable if the Vite proxy is not being used.

## 3. Automated checks

Run from `frontend/`:

```bash
npm run lint
npm test
npm run build
```

The unit suite covers:
- API pagination across multiple pages, including a final partial page.
- A valid empty result (`total_pages: 0`).
- GeoJSON FeatureCollection conversion and preservation of building UUID, region UUID and coordinate order.
- Invalid geometry filtering without dropping valid records.
- Removal-event predicate for a selected entity.
- Detail response envelope, nullable fields, HTTP 400/404, network failure and cancellation.

Record the CI run and its individual job outcomes. Do not mark browser-only scenarios as passed based on unit tests.

## 4. Browser integration scenarios

Mark a scenario only after executing it in the browser. Add a short observation or screenshot reference in the Evidence column.

| ID | Scenario | Expected result | Status | Evidence / observation |
|---|---|---|---|---|
| MAP-01 | Open the map with Backend and PostGIS available | Region/building requests succeed; map reaches ready state and footprints render | [ ] Pending | |
| MAP-02 | Inspect building list response and pagination | All pages are requested through `total_pages`; no hard-coded dataset count | [ ] Pending | |
| MAP-03 | Inspect a rendered entity | Cesium entity keeps the source building UUID and region UUID | [ ] Pending | |
| MAP-04 | Select a building | Highlight appears; detail request path uses the same UUID; panel matches response data | [ ] Pending | |
| MAP-05 | Select a second building quickly | Highlight and panel follow the latest selection; no stale detail is shown | [ ] Pending | |
| MAP-06 | Click empty map space / close the panel | Selection, panel and highlight are cleared | [ ] Pending | |
| MAP-07 | Navigate while the panel is open | Zoom, pan and rotate/tilt remain usable | [ ] Pending | |
| MAP-08 | Return an empty building list | Empty state appears; this is not shown as a request failure | [ ] Pending | |
| MAP-09 | Fail the building-list request / stop Backend | Error state appears; no false empty state and no Cesium crash | [ ] Pending | |
| MAP-10 | Delay or fail a detail request | Loading/error state is clear; old detail is not attributed to the new selection | [ ] Pending | |
| MAP-11 | Remove the selected entity from its data source | Selection, panel and highlight clear; no stale detail remains | [ ] Pending | |
| MAP-12 | Remove the active data source | Selection is cleared and no removed entity remains highlighted | [ ] Pending | |
| MAP-13 | Reload the page | Exactly one active viewer/data source is created; no duplicate footprints or handlers | [ ] Pending | |
| MAP-14 | Inspect geometry placement | Footprints appear in the expected study area; GeoJSON longitude/latitude order is preserved | [ ] Pending | |
| MAP-15 | Inspect nullable attributes | Missing name/type values use safe fallback text; no runtime error | [ ] Pending | |

### How to exercise error and lifecycle scenarios

- For MAP-08, use a controlled API fixture or browser request interception to return HTTP 200 with `data: []` and `pagination.total_pages: 0`.
- For MAP-09, block the `/api/v1/buildings` request in DevTools or stop the Backend, then reload the page.
- For MAP-10, throttle or intercept the detail endpoint; select building A then B and verify that A's delayed response cannot replace B's detail.
- For MAP-11/MAP-12, use a temporary browser-debug action or a local development breakpoint to call `dataSource.entities.remove(selectedEntity)` and `viewer.dataSources.remove(dataSource, true)`. Do not add a production-only UI control solely for this test.
- For MAP-13, reload several times and inspect the Cesium viewer/data-source state and browser console.

## 5. Browser and network evidence

For each manual run, record:
- Browser console errors and warnings.
- HTTP method, path, status and response shape for regions, building pages and building detail.
- Actual `pagination.total`, `total_pages` and number of rendered valid features.
- One selected building UUID and the corresponding detail response UUID.
- Any geometry skipped by validation and the reason.
- Any deviations, known limitations or untested scenarios.

## 6. Exit criteria

P1-11 browser verification is complete only when:
- MAP-01 through MAP-15 have an explicit pass/fail result.
- Any failed scenario has a linked issue or a fix and a rerun result.
- Frontend lint, unit tests and production build pass in CI.
- Backend/PostGIS integration jobs pass when applicable.
- The actual dataset count and environment are recorded; no fixed count is assumed.
- No manual scenario is marked passed without execution evidence.
