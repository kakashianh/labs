# Spatial building-level SIR simulation (KAN-24)

## Model

The engine uses synchronous explicit Euler integration. At the beginning of each timestep it snapshots every eligible building's S/I/R state. KAN-23 computes infection pressure from that immutable snapshot:

- local prevalence: `I_i / N_i`
- neighbor prevalence: distance-weighted mean of `I_j / N_j`, using `max(0, 1 - distance_m / radius_m)`
- infection pressure: `lambda_i = beta * (local_prevalence + neighbor_prevalence)` (alpha defaults to 1)

For each building, both flows use the same beginning-of-step snapshot:

- infection = `min(S_i, lambda_i * S_i * dt)`
- recovery = `min(I_i, gamma * I_i * dt)`
- next S = `S_i - infection`
- next I = `I_i + infection - recovery`
- next R = `R_i + recovery`

The next snapshot is swapped in only after every building passes finite, non-negative, and population-conservation checks. Population is manually supplied during initialization; it is never inferred from footprint geometry or building attributes.

## Execution and persistence

1. Initialize a run with `POST /api/v1/simulations/initialize`.
2. Execute it with `POST /api/v1/simulations/{simulation_id}/run`.
3. Inspect lifecycle and persisted counts with `GET /api/v1/simulations/{simulation_id}`.
4. Detailed time-series retrieval belongs to KAN-25.

The full result set, including timestep zero, is inserted in batches inside one PostgreSQL transaction. The run is marked `completed` in the same transaction. A failed computation or insert cannot leave a completed run with partial results. Repeating the same completed run with the same configuration returns an idempotent replay; a different configuration returns HTTP 409.

## Resource limits

- Maximum 500 eligible buildings.
- Maximum 250,000 persisted result rows per run, where rows = buildings × (steps + 1).
- Existing Phase 1 maximum of 100,000 steps and duration of 10,000 caller-defined time units still apply.
- Neighbor radius is greater than zero and at most 5,000 meters.
- Persistence batches contain at most 500 rows.

These limits keep the synchronous HTTP execution bounded. The Phase 2 reference dataset of 431 buildings fits the building-count limit; benchmark timing and memory remain a CI/manual verification item.

## Interpretation

This is an academic building-level approximation, not an individually calibrated epidemiological model or medical guidance. Neighbor relationships use minimum footprint-to-footprint distance in meters.
