# Local infrastructure (Docker Compose)

PP-11 provisions local development services only. It does not define PostgreSQL schemas, JanusGraph graph schema, Milvus collections, or production deployment.

## Prerequisites

- Docker Engine/Desktop with Docker Compose V2 (`docker compose version`).
- GNU Make (optional; commands can be run directly).
- For optional profiles, allocate additional RAM and disk. Graph and vector services are intentionally opt-in and resource intensive.

Copy the safe template and edit the local values:

```bash
cp .env.example .env
```

Never commit `.env`, paste its secrets into issues/chat, or reuse these development credentials outside a local machine. Published ports bind to `127.0.0.1` by default.

## Profiles

| Profile | Services | Host endpoints |
|---|---|---|
| Default | PostgreSQL, MinIO, one-shot MinIO bucket initializer | PostgreSQL `127.0.0.1:5432`; MinIO API `:9000`, Console `:9001` |
| `graph` | Cassandra, Elasticsearch, JanusGraph | CQL `:9042`; Elasticsearch `:9200`; Gremlin Server `:8182` |
| `vector` | etcd, Milvus standalone | Milvus gRPC `:19530`; health/metrics `:9091` |

Default startup does not activate graph or vector services:

```bash
docker compose up -d
docker compose --profile graph up -d
docker compose --profile vector up -d
docker compose --profile graph --profile vector up -d
```

Profiles can be started independently. Milvus reuses the core MinIO service and its dedicated `protein3d-milvus` bucket; application artifacts use `protein3d-artifacts`. Do not point both workloads at the same bucket/prefix.

## Lifecycle commands

| Command | Purpose |
|---|---|
| `make infra-up` | Start default PostgreSQL + MinIO |
| `make infra-up-graph` | Start core + graph profile |
| `make infra-up-vector` | Start core + vector profile |
| `make infra-up-all` | Start core + both optional profiles |
| `make infra-down` | Stop containers; preserve named volumes |
| `make infra-status` | Show service state |
| `make infra-logs` | Follow logs |
| `make infra-config` | Render/validate Compose configuration |
| `make infra-reset` | **Destructive:** remove all project containers and named volumes |

Equivalent Docker Compose commands:

```bash
docker compose ps
docker compose logs -f
docker compose down
docker compose --profile graph --profile vector config --quiet
```

`infra-reset` runs `docker compose --profile graph --profile vector down --volumes --remove-orphans`. It permanently deletes local PostgreSQL, MinIO, Cassandra, Elasticsearch, JanusGraph, etcd, and Milvus data. Normal `infra-down` never deletes volumes.

## Readiness and connectivity

- PostgreSQL: `docker compose exec postgres pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"`.
- MinIO: `docker compose exec minio mc ready local`; the initializer creates both project buckets.
- JanusGraph: wait for its health check after Cassandra and Elasticsearch become healthy; connect via Gremlin Server at `ws://127.0.0.1:8182/gremlin`.
- Milvus: wait for its health check; connect using the SDK at `http://127.0.0.1:19530`.

Use `docker compose ps` and `docker compose logs <service>` to diagnose startup. First startup of Cassandra, Elasticsearch, JanusGraph, or Milvus can take several minutes. Elasticsearch may require host `vm.max_map_count` tuning on Linux; see the official Elasticsearch Docker documentation. Milvus standalone has substantial memory and fast-disk requirements; see the official [Milvus prerequisites](https://milvus.io/docs/prerequisite-docker.md).

## Version and compatibility decisions

- PostgreSQL `17.6-alpine`; MinIO `RELEASE.2025-09-07T16-13-09Z`.
- Graph: JanusGraph `1.2.0`, Cassandra `5.0.9`, Elasticsearch `9.5.4`. These versions follow JanusGraph 1.2.0's published tested compatibility matrix (Cassandra 5.0.x and Elasticsearch 9.5.x). JanusGraph uses the documented `cql-es` template.
- Vector: Milvus `2.6.26` standalone and etcd `3.5.18`; MinIO is shared with the core profile using a dedicated bucket.

Compatibility is based on the selected upstream release documentation and must be rechecked when upgrading any pinned image. These local settings disable Elasticsearch security for loopback-only development; do not reuse them in production.

## Ownership boundaries

- PP-11 owns Compose services, profiles, ports, volumes, health checks, lifecycle commands, and this operational guide.
- PP-12 owns PostgreSQL schema, migrations, and typed application settings.
- PP-26–PP-29 own JanusGraph logical schema and graph operations.
- PP-30–PP-33 own embedding generation, Milvus collection/index schema, and vector search.
- PP-10 remains the owner of logical data contracts.
