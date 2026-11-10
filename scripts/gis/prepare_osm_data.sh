#!/usr/bin/env bash
set -Eeuo pipefail

# KAN-6: Prepare HCMC 2D building footprints from OSM.
# Run from any directory; paths are resolved relative to the repository root.

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
SOURCE_DIR="${REPO_ROOT}/data/sample/source"
OUTPUT_DIR="${REPO_ROOT}/data/sample/processed"

PBF_URL="${OSM_PBF_URL:-https://download.geofabrik.de/asia/vietnam-latest.osm.pbf}"
PBF_FILE="${OSM_PBF_FILE:-${SOURCE_DIR}/vietnam-latest.osm.pbf}"
BOUNDARY_FILE="${HCM_BOUNDARY_FILE:-${SOURCE_DIR}/hcm_boundary.geojson}"
BOUNDARY_OUTPUT="${OUTPUT_DIR}/hcm_boundary.geojson"
BUILDINGS_OUTPUT="${OUTPUT_DIR}/hcm_buildings.geojson"

log() { printf '[KAN-6] %s\n' "$*"; }
fail() { printf '[KAN-6][ERROR] %s\n' "$*" >&2; exit 1; }

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "Required command not found: $1"
}

require_command ogr2ogr
require_command ogrinfo
require_command curl
require_command sha256sum

mkdir -p "$SOURCE_DIR" "$OUTPUT_DIR"

[[ -s "$BOUNDARY_FILE" ]] || fail "Missing boundary input: $BOUNDARY_FILE. Export and verify the HCMC administrative boundary first (see data/sample/source/README.md)."

if [[ ! -s "$PBF_FILE" ]]; then
  log "Vietnam PBF not found; downloading Geofabrik extract."
  curl --fail --location --retry 3 --output "$PBF_FILE" "$PBF_URL"
fi
[[ -s "$PBF_FILE" ]] || fail "OSM PBF download is empty: $PBF_FILE"

log "GDAL version: $(ogr2ogr --version)"
log "PBF SHA-256: $(sha256sum "$PBF_FILE")"
log "Inspecting boundary input."
ogrinfo -ro -so "$BOUNDARY_FILE"

# Normalize the boundary to RFC 7946 GeoJSON. Source CRS must be declared/readable;
# GDAL performs a coordinate transformation rather than merely assigning an SRID.
log "Normalizing HCMC boundary to EPSG:4326."
BOUNDARY_TMP="${BOUNDARY_OUTPUT}.tmp.geojson"
rm -f "$BOUNDARY_TMP"
ogr2ogr \
  -f GeoJSON "$BOUNDARY_TMP" "$BOUNDARY_FILE" \
  -t_srs EPSG:4326 \
  -makevalid \
  -nlt PROMOTE_TO_MULTI \
  -lco RFC7946=YES \
  -lco COORDINATE_PRECISION=7
mv "$BOUNDARY_TMP" "$BOUNDARY_OUTPUT"

# Confirm the expected OSM layer and attributes before extraction.
log "Inspecting OSM multipolygons layer."
ogrinfo -ro -so "$PBF_FILE" multipolygons

# OSM multipolygons exposes osm_id for relations and osm_way_id for closed ways.
# Prefix the selected ID with its OSM object namespace to avoid cross-type collisions.
# Keep both source ID columns for traceability. Clip against the verified boundary.
log "Extracting building=* footprints and clipping to HCMC."
BUILDINGS_TMP="${BUILDINGS_OUTPUT}.tmp.geojson"
rm -f "$BUILDINGS_TMP"
ogr2ogr \
  -f GeoJSON "$BUILDINGS_TMP" "$PBF_FILE" multipolygons \
  -dialect SQLite \
  -sql "SELECT
    CASE
      WHEN osm_way_id IS NOT NULL THEN 'way/' || CAST(osm_way_id AS TEXT)
      WHEN osm_id IS NOT NULL THEN 'relation/' || CAST(osm_id AS TEXT)
      ELSE NULL
    END AS building_id,
    CASE
      WHEN osm_way_id IS NOT NULL THEN 'way'
      WHEN osm_id IS NOT NULL THEN 'relation'
      ELSE NULL
    END AS osm_type,
    osm_id,
    osm_way_id,
    name,
    building,
    geometry
    FROM multipolygons
    WHERE building IS NOT NULL
      AND (osm_way_id IS NOT NULL OR osm_id IS NOT NULL)" \
  -clipsrc "$BOUNDARY_OUTPUT" \
  -clipsrclayer hcm_boundary \
  -t_srs EPSG:4326 \
  -makevalid \
  -nlt PROMOTE_TO_MULTI \
  -lco RFC7946=YES \
  -lco COORDINATE_PRECISION=7
mv "$BUILDINGS_TMP" "$BUILDINGS_OUTPUT"

log "Checking that both output layers can be read."
ogrinfo -ro -so "$BOUNDARY_OUTPUT"
ogrinfo -ro -so "$BUILDINGS_OUTPUT"

log "Output checksums:"
sha256sum "$BOUNDARY_OUTPUT" "$BUILDINGS_OUTPUT"

log "Extraction completed. Open both GeoJSON files in QGIS and complete:"
log ".task/[P1-04] - Phân tích mô hình dữ liệu GIS và chuẩn bị dữ liệu đô thị mẫu/deliverables/data_quality_report.md"
log "Do not mark KAN-6 complete until IDs, geometry validity, boundary coverage and visual QA have been checked."
