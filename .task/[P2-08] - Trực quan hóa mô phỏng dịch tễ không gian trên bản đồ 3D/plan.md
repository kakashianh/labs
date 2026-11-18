## 1. Objective

Tích hợp kết quả spatial SIR, risk classification và hotspot vào CesiumJS để người dùng quan sát diễn biến theo building và time_step.

Expected result:
- Building colors reflect selected metric/risk at a selected time_step.
- Timeline changes update map state.
- Selecting a building shows its S/I/R and infection ratio.
- Hotspot geometry is displayed.
- Legend and interaction remain understandable and responsive.

## 2. Dependencies

| Dependency | Required state |
|---|---|
| KAN-25 | Stable API for per-building states and aggregates |
| KAN-26 | Risk level and thresholds |
| KAN-27 | Hotspot geometry and stats |
| Phase 1 map | Existing Cesium buildings, selection and detail panel |

## 3. Scope

### In scope
- Map building state lookup by building ID.
- Color building by infected count, infection ratio or risk level.
- Time-step slider/timeline.
- Refresh map state when selected time_step changes.
- Building selection popup/detail panel with S/I/R, N and ratio.
- Hotspot geometry layer.
- Legend, loading/error/empty states.
- Avoid unnecessary full geometry refetch when only time_step changes.

### Out of scope
- Rebuilding the Cesium map foundation.
- 3D floor/room visualization without source geometry.
- Animating individual people.
- Editing simulation parameters in this task if simulation launch flow is not part of Jira.
- Medical advice or real-time public health alerting.

## 4. Frontend Data Flow

```text
Select Simulation Run
  -> Load run metadata / available time range
  -> Select time_step
  -> Fetch building states + risk + hotspots
  -> Join by building_id
  -> Apply style to Cesium entities
  -> Select building
  -> Show S/I/R + population + ratio
```

Data contract:
- Use stable building_id to join API state with Cesium entities.
- Never rely on array order or feature index.
- Missing state must have an explicit neutral/unknown style; do not classify missing data as Low.
- Keep raw numeric values; formatting only at UI boundary.
- If API uses pagination, fetch required pages or use a viewport/region query; do not silently render partial results as complete.
- Keep one selected simulation_id/time_step source of truth.

## 5. Visualization Rules

Threshold colors must map to named labels and have a legend; color is not the only signal.
- Low/Medium/High from KAN-26 configuration.
- Missing/unavailable data shown separately from Low.
- If choosing continuous infection ratio styling, define stable range and tooltip values.
- Hotspots shown as a separate Cesium data source/layer and toggleable if cluttered.
- Building detail panel displays S, I, R, N, ratio, risk level, simulation and time.
- Do not infer height/floors or render invented 3D geometry.

## 6. Implementation Plan

### Step 1 – Inspect Existing Cesium Entity and UI Contracts
- [x] Inspect building entity IDs/properties and selection flow.
- [x] Inspect current API client/query patterns.
- [x] Inspect existing building detail panel and map layers.
- [x] Define mapping API building_id to Cesium entity.
- [x] Define simulation/time_step state ownership.

### Step 2 – Integrate Simulation Result APIs
- [x] Add typed DTOs for per-building states, aggregates, risk and hotspots.
- [x] Add API client methods.
- [x] Handle loading, empty, error and abort/stale responses.
- [x] Handle pagination or bounded region responses.
- [x] Ensure responses correspond to selected simulation_id/time_step.

### Step 3 – Implement Time-step Controls
- [x] Add timeline/slider for available time_steps.
- [x] Show current step/time.
- [x] Fetch or select data for new time_step.
- [x] Prevent stale response from replacing newer selection.
- [x] Debounce or cancel rapid time-step requests where appropriate.
- [x] Keep selected step within run's valid range.

### Step 4 – Style Building Entities
- [x] Join state by stable building_id.
- [x] Apply risk/ratio/count style.
- [x] Define unknown/no-data style separately.
- [x] Restore base building style when simulation layer is disabled.
- [x] Avoid mutating shared source properties irreversibly.
- [x] Add accessible legend and textual risk labels.

### Step 5 – Building Details and Hotspot Layer
- [x] Extend building selection details with N, S, I, R and ratio.
- [x] Add risk level and threshold context.
- [x] Render hotspot geometry in separate layer.
- [x] Display hotspot stats on selection/hover if interaction supports it.
- [x] Add hotspot layer visibility toggle.
- [x] Cleanup Cesium primitives/data sources on unmount or run change.

### Step 6 – Tests and Manual E2E
- [x] Unit test DTO mapping and unknown-style classification.
- [ ] Test selected simulation/time_step synchronization.
- [x] Test missing state uses unknown style.
- [ ] Test stale requests do not overwrite latest time_step.
- [ ] Test building selection shows matching state.
- [ ] Test hotspot layer rendering and cleanup.
- [ ] Test legend consistency with backend thresholds.
- [x] Run frontend lint, type checks, unit tests and production build (GitHub Actions).
- [ ] Run browser E2E/manual test with a completed spatial simulation.
- [ ] Capture evidence for timeline, building details and hotspots.

## 7. Acceptance Criteria

- [ ] 3D map reflects correct building epidemiological state from API.
- [ ] Changing time_step updates building colors and metrics.
- [ ] Building selection displays correct S/I/R and infection ratio.
- [ ] Hotspots are visible and identifiable.
- [ ] Legend explains Low/Medium/High and missing data.
- [ ] No invented building height/floors or individual movement.
- [ ] Loading/error/empty states do not crash map.
- [ ] Tests and E2E evidence cover core interactions.

## 8. Deliverables

1. API integration and typed frontend models.
2. Time-step control.
3. Cesium building thematic styling.
4. Building epidemiological detail panel.
5. Hotspot layer and legend.
6. Frontend tests/E2E evidence and user documentation.

## 9. Definition of Done

KAN-28 hoàn thành khi CesiumJS thể hiện được spatial SIR theo building/time_step, người dùng xem được S/I/R khi chọn building, hotspot hiển thị đúng geometry và legend phân biệt rõ cả dữ liệu thiếu.

## 10. Decisions and Detailed Integration Contracts

### 10.1 Simulation Run Selection and Metadata

- KAN-28 does not introduce a simulation-history/list API.
- The initial `simulation_id` is supplied by the existing simulation initialization/run flow.
- Load run metadata from:
  `GET /api/v1/simulations/{simulation_id}`.
- Only a run with `status = completed` can enter the visualization flow.
- Use `time_step_count` from the run summary. Valid steps are integer values from `0` through `time_step_count - 1`.
- Do not infer calendar dates or clock times from simulation time.
- Show the selected time as `Step {time_step} · t = {time}`, where `time` comes from the result API and uses the simulation's configured time unit.

### 10.2 Risk Classification Snapshot

- Read risk thresholds from KAN-26 response data; do not duplicate default threshold constants in the frontend.
- KAN-27 hotspot requests and KAN-26 risk results must use the same completed `simulation_id` and selected `time_step`.
- Do not POST `risk-classification` on every timeline change.
- The POST endpoint is a configuration/write operation. Invoke it only when the user explicitly requests applying or changing risk thresholds; then refresh risk and hotspot data.
- Preserve the threshold snapshot returned by the backend and use it consistently for the legend, building classification and hotspot context.

### 10.3 API Loading, Pagination and Request Consistency

- Fetch KAN-25 building results, KAN-26 risk results and KAN-27 hotspots for the same simulation and time step.
- KAN-25 and KAN-26 paginated endpoints use `page_size <= 500`; fetch all required pages using `pagination.total_pages` before treating the dataset as complete.
- Do not silently render only the first page as the complete simulation.
- Aggregate and hotspot endpoints are not paginated; use their complete responses.
- Join all building-related data using `building_id`, never array position or Cesium feature index.
- Keep one source of truth for selected `simulation_id` and `time_step`.
- Cancel in-flight requests where supported, or ignore responses whose selection key no longer matches the current selection.
- Apply a new map state atomically after all required data for that selection is ready; do not mix responses from different runs or time steps.
- Keep the previous stable map state or show an explicit loading state while the next selection is loading. Never present a partially fetched dataset as complete.

### 10.4 Building Color Metric and Legend

- Default building color mode: `risk_level` (Low, Medium, High, Unclassified).
- Provide `infection_ratio` as the alternative continuous color mode.
- `infected count` is shown in tooltip/detail panel, not as a separate color mode in KAN-28.
- Risk colors and labels must be driven by KAN-26 classifications and its saved threshold snapshot; do not hardcode risk thresholds in frontend logic.
- Unclassified, missing state and unavailable data must use a distinct neutral/unknown style, never the Low style.
- For continuous infection-ratio styling, define a stable documented visual range and show the exact numeric ratio in the detail panel/tooltip.
- Legend must identify the active metric and explain unknown/missing data. Color must not be the only signal; retain text labels.

### 10.5 Hotspot Interaction and Layer Lifecycle

- Render hotspot geometry in a separate Cesium data source/layer from building entities.
- Provide independent visibility toggles for building thematic styling and hotspot layer.
- Selecting/hovering a hotspot (where supported by the existing interaction model) shows `cluster_id`, member count, population total, infected total, infection ratio and aggregate risk level.
- Use `building_ids` from the hotspot response to highlight or select member buildings; do not infer membership from geometry intersection.
- Clear or replace hotspot and thematic layers when simulation changes, and remove Cesium data sources/primitives/listeners on unmount.
- Preserve the base building appearance when the simulation layer is disabled.

### 10.6 Error and Boundary Handling

- Handle unknown simulation, non-completed run, invalid time step, empty result set, empty hotspot set, missing region/building data, and API failures as explicit UI states.
- Do not convert absent or excluded population into zero population or Low risk.
- A valid time step with no hotspot should show an empty hotspot state while still rendering available building results.
- Do not update Cesium with stale API responses after the user changes simulation or time step.

### 10.7 Additional Verification Cases

- [ ] Reject or explain a run whose status is not `completed`.
- [ ] Verify time-step lower/upper bounds and display backend-provided `time`.
- [ ] Verify pagination loads every required page, including more than 500 buildings.
- [ ] Verify risk threshold snapshot is used by legend and risk styling without repeated POST calls.
- [ ] Verify default `risk_level` mode and alternate `infection_ratio` mode.
- [ ] Verify infected count appears in detail/tooltip and is not a separate color mode.
- [x] Verify Unclassified/missing data never appears as Low (unit test + map style guard).
- [ ] Verify hotspot selection displays backend statistics and highlights members by `building_ids`.
- [ ] Verify building and hotspot layers can be toggled independently.
- [ ] Verify changing run removes old styles, hotspot geometry and event handlers.
- [ ] Verify empty hotspot response does not hide valid building results.
- [ ] Verify stale requests cannot overwrite the currently selected simulation/time step.



## 11. Implementation Status

### 11.1 Cesium Basemap Failure Follow-up (2026-11-18)

- Browser smoke test found the Cesium canvas initialized but the map surface appeared blank; the test did not capture browser console/network logs, so the precise runtime cause is not yet confirmed.
- Existing branch commit `ba595a8` added an ArcGIS imagery source with OpenStreetMap fallback.
- Follow-up inspection found the imagery error handler checked `timesRetried` but did not set Cesium's `TileProviderError.retry` flag. Cesium therefore had no explicit opt-in to retry transient tile failures, so the fallback threshold could remain unreachable.
- Updated the handler to opt into two retries and then switch to OpenStreetMap, while retaining the globe base color and warning state.
- [ ] Verify ArcGIS imagery success path in browser.
- [ ] Simulate/observe ArcGIS tile failure and verify OSM fallback activates.
- [ ] Verify the globe and 431 building footprints remain visible when imagery is unavailable.
- [ ] Inspect browser console and network responses to identify any independent WebGL, Cesium asset, or GeoJSON rendering errors.
- [ ] Re-run the complete spatial simulation E2E after loading a real completed `simulation_id`.

### 11.2 Browser Smoke-test Follow-up (2026-11-18)

- Made the Building thematic layer and Hotspot layer controls visible before a run is loaded; controls remain disabled until a completed simulation is loaded, with an explanation shown to the user. This avoids interpreting their absence on the initial screen as a missing feature.
- Added Cesium `scene.renderError` and WebGL context lost/restored handling to surface renderer failures in the map status instead of leaving an unexplained black canvas.
- Kept the ArcGIS imagery retry + OpenStreetMap fallback from the previous fix.
- [ ] Verify both layer controls are visible on initial load and become interactive after loading a completed run.
- [ ] Verify renderer diagnostics are clear in the browser and no WebGL/render errors remain.
- [ ] Verify the Cesium globe/building footprints render in the Codespaces browser after the latest changes.


- Frontend implementation is on `feat/p2-08-3d-epidemic-visualization`.
- The map now loads completed spatial runs by `simulation_id`, supports time-step navigation, risk/ratio styling, S/I/R building details, aggregate summaries and a separate clickable hotspot layer.
- Frontend unit tests, lint and production build have passed in GitHub Actions. Backend formatting, Clippy, Rust tests and Testcontainers/PostGIS integration tests have also passed in the current workflow; backend Docker image build is the remaining CI job at the time of this update.
- Browser-based manual E2E and evidence capture are intentionally left unchecked until the feature is tested in Codespaces with a real completed spatial simulation.


### 11.3 Follow-up After TinyFish Read-only Diagnostic (2026-11-18)

- TinyFish confirmed that the page shell and Cesium DOM elements load, and the API status label reports 1 region / 431 buildings; however, the map surface remains black. The automation could not inspect DevTools console or Network responses, so it did not establish whether this is a WebGL runtime limitation, Cesium asset/tile failure, or an application rendering defect.
- TinyFish observed both layer checkboxes checked and reported no disabled state on the preview. The source code already gates them with `disabled={!summary}`; this mismatch may mean the Codespaces preview is serving an older workspace build. The controls now initialize unchecked, are reset when a new run starts, and are enabled visually only after a completed run and first frame load successfully.
- Added a guarded Cesium Viewer constructor. If Cesium/WebGL initialization throws, the map status now reports the initialization error instead of leaving an unexplained blank surface.
- [ ] Pull/reload the latest feature-branch commit in the Codespaces workspace and verify the initial layer controls are unchecked and disabled.
- [ ] Recheck the map after the Codespaces workspace is running the latest source; if it remains black, capture actual browser DevTools console and Network evidence before changing imagery/asset configuration.
- [ ] Confirm the guarded initialization error state by checking the real browser runtime; do not infer the cause from the TinyFish headless rendering alone.
- [ ] Run the completed spatial simulation E2E with a real `simulation_id`; no run was started during the read-only diagnostic.
