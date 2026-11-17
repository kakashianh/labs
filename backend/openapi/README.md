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


## Building neighbors

GET /api/v1/buildings/{building_id}/neighbors?radius_m=250&limit=100 returns nearby buildings using minimum footprint-to-footprint distance, not centroid distance.

| Parameter | Default | Constraint |
|---|---:|---|
| building_id | — | Internal building UUID |
| radius_m | Required | Finite, greater than 0 and at most 5,000 m |
| limit | 100 | Integer from 1 to 500 |

The source footprint is geometry(MultiPolygon, 4326). Both ST_DWithin and ST_Distance cast footprints to PostGIS geography, so the predicate and returned distance_m use the same metric semantics. The center building is excluded from its own neighbor list. Results are ordered by distance ascending and then UUID ascending. A building exactly on the radius boundary is included because ST_DWithin is inclusive.

The response includes excluded_invalid_geometry_count, the total number of invalid, empty, or non-EPSG:4326 footprints in public.buildings. Such records are excluded from neighbor candidates. A center building with such a footprint returns HTTP 400. A missing center returns HTTP 404. No neighbors is HTTP 200 with data: [].

The repository also exposes db::building_neighbors::build_building_adjacency for a batch of UUIDs. It returns each undirected pair once, ordered by UUID, plus excluded_invalid_geometry_count and missing_building_count so simulation setup can reject or report incomplete input instead of silently treating it as a complete graph.

Example:

    curl "http://localhost:8000/api/v1/buildings/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/neighbors?radius_m=100&limit=100"


### Spatial index and performance notes

The base buildings_footprint_gist_idx is a GiST index on geometry and is not applicable to ST_DWithin(footprint::geography, ...). Migration 20261116000000_add_building_footprint_geography_gist_index.sql adds a partial GiST expression index on (footprint::geography), limited to valid, non-empty EPSG:4326 footprints. Neighbor lookup and batch adjacency use LATERAL candidate scans so PostgreSQL can apply that expression index to the candidate side.

Live-data review on 2026-11-16 found 431 buildings, all EPSG:4326 and valid/non-empty. Before the geography expression index, the materialized all-pairs adjacency query used sequential scans and took about 5.44 seconds for 21,387 edges at a 250 m radius. The existing geometry GiST index was not used after geography casting. The new migration is part of this branch and has not been applied to the live Supabase database. Re-run EXPLAIN (ANALYZE, BUFFERS) after deployment against the live dataset to verify planner choice and latency; the integration test forces index scans only to prove the index is usable, not that PostgreSQL should always choose it for tiny tables.
