-- KAN-22: Persist spatial SIR runs, per-building initial states and exclusions.
CREATE TABLE simulation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL UNIQUE,
    request_payload JSONB NOT NULL,
    population_source VARCHAR(32) NOT NULL DEFAULT 'manual',
    status VARCHAR(24) NOT NULL DEFAULT 'initialized',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT simulation_runs_population_source_check CHECK (population_source = 'manual'),
    CONSTRAINT simulation_runs_status_check CHECK (status IN ('initialized'))
);

CREATE TABLE simulation_building_states (
    simulation_id UUID NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE RESTRICT,
    population DOUBLE PRECISION NOT NULL CHECK (population > 0 AND population < 'Infinity'::float8),
    initial_s DOUBLE PRECISION NOT NULL CHECK (initial_s >= 0 AND initial_s < 'Infinity'::float8),
    initial_i DOUBLE PRECISION NOT NULL CHECK (initial_i >= 0 AND initial_i < 'Infinity'::float8),
    initial_r DOUBLE PRECISION NOT NULL CHECK (initial_r >= 0 AND initial_r < 'Infinity'::float8),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (simulation_id, building_id),
    CONSTRAINT simulation_building_states_conservation_check
        CHECK (abs((initial_s + initial_i + initial_r) - population)
            <= 1e-9 + 1e-9 * population)
);

CREATE TABLE simulation_building_exclusions (
    simulation_id UUID NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE RESTRICT,
    reason_code VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (simulation_id, building_id),
    CONSTRAINT simulation_building_exclusions_reason_check
        CHECK (reason_code IN ('missing_population'))
);

CREATE INDEX simulation_building_states_building_idx
    ON simulation_building_states (building_id, simulation_id);
