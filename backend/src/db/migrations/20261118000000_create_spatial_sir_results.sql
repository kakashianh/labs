-- KAN-24: Persist completed building-level SIR time series atomically.
ALTER TABLE public.simulation_runs
    DROP CONSTRAINT simulation_runs_status_check;

ALTER TABLE public.simulation_runs
    ADD CONSTRAINT simulation_runs_status_check
    CHECK (status IN ('initialized', 'completed'));

ALTER TABLE public.simulation_runs
    ADD COLUMN simulation_config JSONB,
    ADD COLUMN completed_at TIMESTAMPTZ;

CREATE TABLE public.simulation_building_results (
    simulation_id UUID NOT NULL REFERENCES public.simulation_runs(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES public.buildings(id) ON DELETE RESTRICT,
    time_step INTEGER NOT NULL CHECK (time_step >= 0),
    time DOUBLE PRECISION NOT NULL CHECK (time >= 0 AND time < 'Infinity'::float8),
    susceptible DOUBLE PRECISION NOT NULL CHECK (susceptible >= 0 AND susceptible < 'Infinity'::float8),
    infected DOUBLE PRECISION NOT NULL CHECK (infected >= 0 AND infected < 'Infinity'::float8),
    recovered DOUBLE PRECISION NOT NULL CHECK (recovered >= 0 AND recovered < 'Infinity'::float8),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (simulation_id, building_id, time_step)
);

CREATE INDEX simulation_building_results_time_idx
    ON public.simulation_building_results (simulation_id, time_step, building_id);

CREATE INDEX simulation_building_results_building_idx
    ON public.simulation_building_results (building_id, simulation_id);
