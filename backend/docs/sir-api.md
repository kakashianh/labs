# SIR simulation API

## Endpoint

`POST /api/v1/simulations/sir`

Content type: `application/json`

The endpoint runs the aggregate SIR engine synchronously. It does not read or write PostgreSQL/Redis and does not create a background job.

## Request

All fields are required:

| Field | Type | Meaning |
|---|---|---|
| `population` | number | Total population, finite and greater than zero |
| `susceptible` | number | Initial susceptible population |
| `infected` | number | Initial infected population |
| `recovered` | number | Initial recovered population |
| `beta` | number | Transmission rate |
| `gamma` | number | Recovery rate |
| `dt` | number | Integration interval |
| `steps` | integer | Number of integration intervals (1–100,000) |

The initial S + I + R total must equal population within the domain tolerance. All compartment values and rates must be finite and non-negative. `dt` must be finite and greater than zero; `dt × steps` must not exceed 10,000 caller-defined time units.

Example:

~~~bash
curl -X POST http://localhost:8000/api/v1/simulations/sir \
  -H "Content-Type: application/json" \
  -d '{
    "population": 10000,
    "susceptible": 9990,
    "infected": 10,
    "recovered": 0,
    "beta": 0.3,
    "gamma": 0.1,
    "dt": 1.0,
    "steps": 100
  }'
~~~

## Units and time semantics

- `beta` and `gamma` are rates in inverse time units, not probabilities.
- `dt` uses the same time unit as the rates.
- The caller defines the unit (for example, day); the API does not infer it.
- `time = time_step × dt`.
- S, I and R are aggregate expected counts represented as floating-point values; the API does not round each step.

## Response

The response contains the initial state and one point after every integration interval. Therefore, `point_count = steps + 1`.

~~~json
{
  "population": 10000.0,
  "point_count": 2,
  "points": [
    {
      "time_step": 0,
      "time": 0.0,
      "susceptible": 9990.0,
      "infected": 10.0,
      "recovered": 0.0
    },
    {
      "time_step": 1,
      "time": 1.0,
      "susceptible": 9987.003,
      "infected": 11.997,
      "recovered": 1.0
    }
  ]
}
~~~

The example response above is shortened to two points for readability. A real response returns every requested point.

## Errors

All errors use the existing Backend envelope:

~~~json
{ "error": "..." }
~~~

| HTTP status | Meaning | Example |
|---|---|---|
| 400 | Malformed JSON, missing/wrong-type fields, or invalid SIR parameters | `{"error":"invalid JSON request body"}` |
| 500 | Numerical instability or unexpected internal failure | `{"error":"Internal server error"}` |

Domain validation errors are returned as HTTP 400. Numerical instability is treated as an internal error and its implementation details are not exposed.

## Scope and limitations

This endpoint runs one homogeneous, closed-population SIR simulation. It does not persist scenarios/history, model building-level infection, spatial contacts or hotspots, or estimate epidemiological parameters from real-world data. The engine's mathematical assumptions and numerical policy are documented in [sir-model.md](sir-model.md).
