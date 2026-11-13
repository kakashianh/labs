#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
BOUNDARY_FILE="$ROOT_DIR/data/sample/processed/hcm_boundary.geojson"
BUILDINGS_FILE="$ROOT_DIR/data/sample/processed/hcm_buildings.geojson"
PREFLIGHT_SQL="$ROOT_DIR/scripts/gis/sql/kan14_preflight.sql"
IMPORT_SQL="$ROOT_DIR/scripts/gis/sql/kan14_import.sql"
VERIFY_SQL="$ROOT_DIR/scripts/gis/sql/kan14_verify.sql"

MODE="${1:-}"
if [[ -z "$MODE" ]]; then
  echo "Usage: $0 <validate|apply> [--yes] [--allow-count-change]" >&2
  exit 2
fi
shift

AUTO_APPROVE=0
ALLOW_COUNT_CHANGE=0
for arg in "$@"; do
  case "$arg" in
    --yes) AUTO_APPROVE=1 ;;
    --allow-count-change) ALLOW_COUNT_CHANGE=1 ;;
    *)
      echo "Unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

if [[ "$MODE" != "validate" && "$MODE" != "apply" ]]; then
  echo "Mode must be 'validate' or 'apply'." >&2
  exit 2
fi

POSTGRES_HOST="${POSTGRES_HOST:-127.0.0.1}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_USER="${POSTGRES_USER:-appuser}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-apppassword}"
POSTGRES_DB="${POSTGRES_DB:-appdb}"
EXPECTED_BUILDINGS="${EXPECTED_BUILDINGS:-431}"

die() {
  echo "ERROR: $*" >&2
  exit 1
}

for command_name in docker ogr2ogr ogrinfo; do
  command -v "$command_name" >/dev/null 2>&1 || die "Required command not found: $command_name"
done

[[ -f "$BOUNDARY_FILE" ]] || die "Missing input: $BOUNDARY_FILE"
[[ -f "$BUILDINGS_FILE" ]] || die "Missing input: $BUILDINGS_FILE"
[[ -f "$PREFLIGHT_SQL" && -f "$IMPORT_SQL" && -f "$VERIFY_SQL" ]] || die "KAN-14 SQL files are missing."

ogrinfo --formats | grep -q "PostgreSQL" || die "GDAL was built without the PostgreSQL driver. Install GDAL with PostgreSQL support."

docker compose exec -T postgres pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB" >/dev/null 2>&1 \
  || die "PostGIS is not ready. Start it with: docker compose up -d --build postgres backend"

export PGPASSWORD="$POSTGRES_PASSWORD"
PG_CONNECTION="PG:host=$POSTGRES_HOST port=$POSTGRES_PORT dbname=$POSTGRES_DB user=$POSTGRES_USER"

run_sql_file() {
  cat "$1" | docker compose exec -T postgres psql -X -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB"
}

run_scalar() {
  docker compose exec -T postgres psql -X -A -t -v ON_ERROR_STOP=1 \
    -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "$1" | tr -d '\r' | xargs
}

echo "== KAN-14: load source files into PostGIS staging tables =="
ogr2ogr -f PostgreSQL "$PG_CONNECTION" "$BOUNDARY_FILE" \
  -nln kan14_stage_regions -overwrite \
  -lco SCHEMA=public -lco GEOMETRY_NAME=geom -lco FID=ogc_fid \
  -nlt PROMOTE_TO_MULTI -dim XY -t_srs EPSG:4326

ogr2ogr -f PostgreSQL "$PG_CONNECTION" "$BUILDINGS_FILE" \
  -nln kan14_stage_buildings -overwrite \
  -lco SCHEMA=public -lco GEOMETRY_NAME=geom -lco FID=ogc_fid \
  -nlt PROMOTE_TO_MULTI -dim XY -t_srs EPSG:4326

region_count="$(run_scalar "SELECT COUNT(*) FROM public.kan14_stage_regions")"
building_count="$(run_scalar "SELECT COUNT(*) FROM public.kan14_stage_buildings")"

[[ "$region_count" == "1" ]] || die "Expected exactly 1 region feature; found $region_count."
if [[ "$building_count" != "$EXPECTED_BUILDINGS" && "$ALLOW_COUNT_CHANGE" -ne 1 ]]; then
  die "Expected $EXPECTED_BUILDINGS buildings from the approved KAN-6 snapshot; found $building_count. Review the new snapshot, then rerun with --allow-count-change only if approved."
fi

echo "Staging snapshot: regions=$region_count, buildings=$building_count"
echo "== KAN-14: automated pre-import validation =="
run_sql_file "$PREFLIGHT_SQL"

if [[ "$MODE" == "validate" ]]; then
  cat <<EOF

Pre-import validation passed. No records were written to regions/buildings.
Human review required before Gate 1 is approved:
  - Confirm the two source GeoJSON files and snapshot counts.
  - Review the field mapping documented in scripts/gis/README.md.
  - Confirm the stable region code: hcm-ben-thanh-1km2.
Then run: bash scripts/gis/import_sample_to_postgis.sh apply
EOF
  exit 0
fi

if [[ "$AUTO_APPROVE" -ne 1 ]]; then
  [[ -t 0 ]] || die "Non-interactive apply requires explicit --yes after human review of the validate output."
  echo
  echo "This will upsert the approved sample data into public.regions and public.buildings."
  read -r -p "Type IMPORT to approve Gate 1 and continue: " approval
  [[ "$approval" == "IMPORT" ]] || die "Import cancelled; no target rows were changed."
fi

echo "== KAN-14: transactional import/upsert =="
run_sql_file "$IMPORT_SQL"

echo "== KAN-14: post-import validation =="
run_sql_file "$VERIFY_SQL"
count_before_reimport="$(run_scalar "SELECT COUNT(*) FROM public.buildings b JOIN public.regions r ON r.id=b.region_id WHERE r.code='hcm-ben-thanh-1km2'")"

echo "== KAN-14: idempotency check (run the same upsert again) =="
run_sql_file "$IMPORT_SQL"
run_sql_file "$VERIFY_SQL"
count_after_reimport="$(run_scalar "SELECT COUNT(*) FROM public.buildings b JOIN public.regions r ON r.id=b.region_id WHERE r.code='hcm-ben-thanh-1km2'")"

[[ "$count_before_reimport" == "$count_after_reimport" ]] \
  || die "Building count changed after re-import ($count_before_reimport -> $count_after_reimport)."

cat <<EOF

KAN-14 automated import checks passed.
  Region code: hcm-ben-thanh-1km2
  Regions in staging: $region_count
  Buildings in staging: $building_count
  Buildings after re-import: $count_after_reimport

Human Gate 3 is still required:
  Open public.regions.boundary and public.buildings.footprint in QGIS.
  Confirm the Ben Thanh AOI, footprint placement, outliers and alignment
  against the KAN-6 GeoJSON before accepting KAN-14.
EOF
