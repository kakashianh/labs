# API documentation

KAN-15 publishes the OpenAPI 3.0.3 contract and Swagger UI from the Axum server.

- OpenAPI JSON: `GET /api-docs/openapi.json`
- Swagger UI: `GET /docs`
- Building list: `GET /api/v1/buildings`

Run the backend from the repository root:

```bash
cp backend/.env.example backend/.env
docker compose up --build
```

Open `http://localhost:8000/docs` in a browser. Swagger UI loads its browser assets from the Swagger UI CDN and reads the local OpenAPI JSON endpoint.

## Building list query

| Parameter | Default | Constraint |
|---|---:|---|
| `page` | 1 | Positive integer |
| `page_size` | 100 | 1–500 |
| `region_id` | — | UUID from `public.regions.id` |

The result is ordered by `id ASC`. `total` is calculated after applying the region filter. An empty result, including a page beyond the last page, returns HTTP 200 with `data: []`; `total_pages` is 0 when `total` is 0.

## Tests

Run unit tests:

```bash
cd backend
cargo test --lib
```

Run the PostGIS integration test (requires Docker):

```bash
cargo test --test buildings_api -- --nocapture
```

The integration test starts an isolated `postgis/postgis:17-3.5` container with Testcontainers, applies the backend SQLx migrations, seeds deterministic fixtures, and exercises the Axum HTTP route. It does not connect to Supabase and requires no Supabase secrets.
