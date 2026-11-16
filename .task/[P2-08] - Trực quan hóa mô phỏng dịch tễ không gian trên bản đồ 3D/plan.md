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
- [ ] Inspect building entity IDs/properties and selection flow.
- [ ] Inspect current API client/query patterns.
- [ ] Inspect existing building detail panel and map layers.
- [ ] Define mapping API building_id to Cesium entity.
- [ ] Define simulation/time_step state ownership.

### Step 2 – Integrate Simulation Result APIs
- [ ] Add typed DTOs for per-building states, aggregates, risk and hotspots.
- [ ] Add API client methods.
- [ ] Handle loading, empty, error and abort/stale responses.
- [ ] Handle pagination or bounded region responses.
- [ ] Ensure responses correspond to selected simulation_id/time_step.

### Step 3 – Implement Time-step Controls
- [ ] Add timeline/slider for available time_steps.
- [ ] Show current step/time.
- [ ] Fetch or select data for new time_step.
- [ ] Prevent stale response from replacing newer selection.
- [ ] Debounce or cancel rapid time-step requests where appropriate.
- [ ] Keep selected step within run's valid range.

### Step 4 – Style Building Entities
- [ ] Join state by stable building_id.
- [ ] Apply risk/ratio/count style.
- [ ] Define unknown/no-data style separately.
- [ ] Restore base building style when simulation layer is disabled.
- [ ] Avoid mutating shared source properties irreversibly.
- [ ] Add accessible legend and textual risk labels.

### Step 5 – Building Details and Hotspot Layer
- [ ] Extend building selection details with N, S, I, R and ratio.
- [ ] Add risk level and threshold context.
- [ ] Render hotspot geometry in separate layer.
- [ ] Display hotspot stats on selection/hover if interaction supports it.
- [ ] Add hotspot layer visibility toggle.
- [ ] Cleanup Cesium primitives/data sources on unmount or run change.

### Step 6 – Tests and Manual E2E
- [ ] Unit test DTO mapping and style classification.
- [ ] Test selected simulation/time_step synchronization.
- [ ] Test missing state uses unknown style.
- [ ] Test stale requests do not overwrite latest time_step.
- [ ] Test building selection shows matching state.
- [ ] Test hotspot layer rendering and cleanup.
- [ ] Test legend consistency with backend thresholds.
- [ ] Run frontend lint, type checks, unit tests and production build.
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
