# 3D Urban Epidemic Simulation

A 3D GIS web application for exploring how infectious diseases may spread across an urban environment. The system combines building-level geospatial data, a spatially extended SIR model, and interactive 3D visualization to help users inspect infection dynamics across buildings and regions over time.

> **Project scope:** Academic simulation and visualization prototype. Model outputs are illustrative and are not intended for medical diagnosis, public-health decisions, or epidemiological forecasting.

## Overview

The application represents an urban area as a collection of georeferenced building footprints. Users can initialize an epidemic scenario, run a discrete-time SIR simulation, and inspect building-level and regional results through a CesiumJS-based 3D map.

### Key capabilities

- **3D urban visualization** — render building footprints in a CesiumJS scene and inspect individual buildings.
- **Building and region data** — query building details, regions, and nearby buildings using PostGIS.
- **Building-level SIR simulation** — initialize susceptible, infected, and recovered populations per building and simulate changes over time.
- **Spatial infection pressure** — incorporate local infection prevalence and distance-weighted neighboring-building prevalence.
- **Risk classification** — classify building-level infection risk and calculate regional aggregates.
- **Hotspot detection** — group connected high-risk buildings into spatial clusters and return their aggregate statistics and geometry.
- **API documentation** — expose the backend API contract through OpenAPI.

## Technology stack

| Layer | Technologies |
| --- | --- |
| Frontend | React, TypeScript, Vite |
| 3D GIS | CesiumJS |
| Backend | Rust, Axum, Tokio |
| Database | PostgreSQL, PostGIS |
| Cache / supporting service | Redis |
| API contract | OpenAPI |
| Local environment | Docker Compose |
| CI | GitHub Actions, Rust tests, Testcontainers, PostGIS integration checks, frontend checks |

## Architecture

```text
┌──────────────────────────────────────┐
│           React + CesiumJS           │
│  3D buildings · map interaction      │
│  simulation controls · result views  │
└──────────────────┬───────────────────┘
                   │ HTTP / JSON
                   ▼
┌──────────────────────────────────────┐
│              Rust + Axum             │
│ Building & region APIs                │
│ SIR simulation                        │
│ Spatial pressure · risk · hotspots    │
└───────────────┬──────────────┬───────┘
                │              │
                ▼              ▼
       ┌────────────────┐  ┌───────────┐
       │ PostgreSQL     │  │   Redis   │
       │ + PostGIS      │  │           │
       │ GIS & results  │  │           │
       └────────────────┘  └───────────┘
```

The backend applies database migrations at startup. Docker Compose starts PostgreSQL/PostGIS, Redis, and the Axum API; the frontend is started separately with Vite during local development.

## Getting started

### Prerequisites

- Git
- Docker Engine and Docker Compose plugin
- Node.js 22 or later and npm
- (Optional) Rust toolchain matching the backend CI toolchain for running the backend outside Docker

### 1. Clone the repository

```bash
git clone https://github.com/kakashianh/labs.git
cd labs
```

### 2. Configure environment variables

Create a root-level Compose environment file from the backend example:

```bash
cp backend/.env.example .env
```

Set a strong local value for `POSTGRES_PASSWORD` in `.env`. Do not commit this file or share real credentials.

The Compose stack uses these variables:

| Variable | Default / purpose |
| --- | --- |
| `POSTGRES_USER` | `appuser` |
| `POSTGRES_PASSWORD` | Required secret; set locally |
| `POSTGRES_DB` | `appdb` |
| `RUST_LOG` | `backend=debug,tower_http=debug` |

### 3. Start backend services

```bash
docker compose up --build -d
```

This starts:

- PostgreSQL with PostGIS on `localhost:5432`
- Redis on `localhost:6379`
- Rust/Axum API on `localhost:8000`

The backend waits for PostgreSQL and applies SQLx migrations during startup.

To follow service logs:

```bash
docker compose logs -f backend
```

To stop the services:

```bash
docker compose down
```

To remove the local database volume as well (this permanently deletes local database data):

```bash
docker compose down -v
```

### 4. Configure and run the frontend

In a second terminal:

```bash
cd frontend
cp .env.example .env
npm ci
npm run dev
```

Open the Vite URL printed in the terminal (normally [http://localhost:5173](http://localhost:5173)).

The frontend environment file supports:

| Variable | Description |
| --- | --- |
| `VITE_API_BASE_URL` | Backend API base URL; local default is `http://localhost:8000` |
| `VITE_CESIUM_ION_TOKEN` | Optional Cesium Ion token. The OpenStreetMap basemap can be used without it. |

### Development commands

Run these from the `frontend` directory:

```bash
npm run dev       # Start Vite development server
npm run lint      # Run Oxlint
npm test          # Run frontend service tests
npm run build     # Type-check and create a production build
npm run preview   # Preview the production build
```

Run Rust checks from the `backend` directory:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib --all-features
```

API integration tests use Testcontainers and require a working Docker environment:

```bash
cargo test --test buildings_api --test region_building_details_api --test building_neighbors_api --test spatial_sir_initialization_api --test spatial_sir_results_api --test spatial_risk_api --test spatial_hotspots_api -- --nocapture
```

## API

The Axum service listens on port `8000`. The API is organized around these capabilities:

| Capability | Description |
| --- | --- |
| Health | Service health checks |
| Buildings | Building list, details, and nearby-building queries |
| Regions | Region data and building-region relationships |
| Simulations | SIR simulation setup and execution |
| Simulation results | Building-level time-step results and regional aggregates |
| Risk classification | Building risk levels and regional risk summaries |
| Hotspots | Connected clusters of candidate buildings and their spatial geometry |

The OpenAPI specification is maintained in `backend/openapi/openapi.json`. Use that file as the source of truth for request schemas, response schemas, and endpoint details.

## Spatial and simulation model

The project uses a building-level SIR model:

- **S (Susceptible):** population not yet infected.
- **I (Infected):** currently infected population.
- **R (Recovered):** recovered population.

Spatial influence is modeled using nearby building footprints and distance in meters. The simulation uses a discrete time step and applies infection and recovery updates from the same beginning-of-step state.

Important modeling assumptions:

- Building population is supplied as simulation input; it is **not inferred** from building footprint area or number of floors.
- Neighbor influence is distance-weighted and controlled by simulation parameters.
- Risk categories are configurable visualization thresholds, not clinical or public-health standards.
- Hotspots are connected components of qualifying buildings under the configured spatial adjacency rule. They are descriptive clusters, not statistically significant disease clusters.

These assumptions should be considered when interpreting any map, chart, or aggregate result.

## Repository structure

```text
.
├── .github/
│   └── workflows/       # CI and deployment workflows
├── .task/               # Project plans and implementation tasks
├── backend/
│   ├── deployment/      # Backend Dockerfile
│   ├── migrations/      # SQLx database migrations
│   ├── openapi/          # OpenAPI specification
│   └── src/
│       ├── routes/       # Axum HTTP routes
│       └── simulation/   # SIR and spatial simulation domain logic
├── data/                 # Sample GIS data and related assets
├── frontend/
│   └── src/
│       ├── components/   # Reusable UI components
│       ├── pages/        # Application pages
│       ├── services/     # API clients and service tests
│       └── styles/       # Application styles
├── scripts/              # Development and data utilities
└── docker-compose.yml
```

## Continuous integration

GitHub Actions validates changes with:

- Rust formatting, Clippy, unit tests, and backend Docker image build
- Axum API integration tests backed by Testcontainers/PostGIS
- Docker Compose startup and PostGIS schema/migration verification
- Frontend linting, unit tests, and production build

Deployment to Render is a manual workflow and requires the corresponding deploy-hook secret to be configured in GitHub.

## Security notes

- Never commit `.env` files, database passwords, API tokens, or deployment secrets.
- Keep database credentials on the backend; do not expose them through `VITE_*` variables.
- Use a dedicated secret for each environment (local development, Codespaces, CI, and deployment).
- The permissive CORS configuration is intended for development and should be restricted before exposing the API to production traffic.

## Project status

The repository is developed in phases. Implementation plans and task-level progress are tracked in [`.task/`](.task/). The default branch is `project/3D-map-medical`.

## License

No license file is currently defined in this repository. Unless a license is added, all rights remain with the repository owner and reuse should not be assumed.
