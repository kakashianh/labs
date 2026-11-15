# SIR simulation model

The backend implements a deterministic, aggregate SIR model using explicit Euler integration. It is a mathematical demonstration engine, not a clinical or epidemiological forecasting service.

## Variables and units

- population, susceptible, infected and recovered are aggregate counts represented as f64; fractional expected counts are valid.
- beta (β) and gamma (γ) are non-negative rates in inverse time units, not probabilities.
- dt is the time interval in the same unit used by beta and gamma.
- steps is the number of integration intervals.
- Results contain the initial state at step 0 and one point after every interval; time = time_step × dt.
- The recovered initial population is named recovered in code to avoid confusion with the basic reproduction number R₀.

## Equations

\[
\frac{dS}{dt}=-\beta\frac{SI}{N},\quad
\frac{dI}{dt}=\beta\frac{SI}{N}-\gamma I,\quad
\frac{dR}{dt}=\gamma I
\]

Both flows are calculated from the state at the beginning of each step:

\[
F_{infection}=\min(S_t,\beta\frac{S_t I_t}{N}\Delta t),\quad
F_{recovery}=\min(I_t,\gamma I_t\Delta t)
\]

\[
S_{t+1}=S_t-F_{infection},\quad
I_{t+1}=I_t+F_{infection}-F_{recovery},\quad
R_{t+1}=R_t+F_{recovery}
\]

Clamping a flow prevents it from exceeding the compartment available at the beginning of the step. It is a numerical boundary guard, not a substitute for selecting a sufficiently small dt.

## Validation and numerical policy

- population must be finite and > 0; S/I/R must be finite and >= 0.
- Initial S + I + R must match N within 1e-9 + 1e-9 × N.
- beta and gamma must be finite and >= 0; no artificial upper bound of 1 is applied because they are rates.
- dt must be finite and > 0; steps must be 1..=100,000; total duration dt × steps must be finite and <= 10,000 caller-defined time units.
- Every step checks finite values, non-negative compartments and population conservation using the same tolerance. Only tiny negative round-off within tolerance is clamped to zero; larger violations return an error.
- No rounding is performed during simulation.

## Rust usage

Call backend::simulation::sir::simulate(SirInput { ... }) from domain/application code. The module has no Axum request/response or database dependency. HTTP exposure belongs to the later API integration task.

## Limitations

This is a homogeneous, closed-population SIR model. It does not model building-level movement, spatial contacts, births/deaths, vaccination, isolation or real-world parameter fitting. Results are not real-world health advice or forecasts.
