# GIS data preparation (KAN-6)

## Purpose

Extract 2D building footprints from the Geofabrik Vietnam OSM PBF, clip them to a verified TP.HCM administrative boundary, and export reproducible GeoJSON files for KAN-13.

The pipeline is a local data-preparation tool. It does not add a Python service, modify the Rust/Axum backend, create PostGIS tables, or import data into a database.

## Prerequisites

- GDAL/OGR with the OSM PBF driver (`ogr2ogr`, `ogrinfo`).
- `curl`, `sha256sum`, and a POSIX-compatible shell (Git Bash/WSL on Windows).
- QGIS for visual quality assurance.

Check the environment:

```bash
ogr2ogr --version
ogrinfo --formats | grep -i OSM
```

## Required inputs

1. `data/sample/source/hcm_boundary.geojson`: exactly one verified TP.HCM administrative boundary feature, exported from OSM/Overpass Turbo with its CRS metadata intact.
2. `data/sample/source/vietnam-latest.osm.pbf`: downloaded automatically by the script from Geofabrik if absent.

The boundary must be selected and verified manually. Do not choose an OSM relation only by a fuzzy name match or assume an `admin_level` value.

## Run

From the repository root:

```bash
bash scripts/gis/prepare_osm_data.sh
```

The script prints the PBF checksum and GDAL layer summaries. It writes:

- `data/sample/processed/hcm_boundary.geojson`
- `data/sample/processed/hcm_buildings.geojson`

Open both outputs in QGIS and complete the KAN-6 quality report before treating the data as ready.

## Reproducibility

The Geofabrik `latest` URL is mutable. Record the retrieval date, file size, SHA-256, GDAL version, boundary relation ID and query URL in KAN-6 metadata. The checksum identifies the exact PBF snapshot used.

Large PBF files and temporary artifacts are excluded from Git. Generated GeoJSON files should only be committed after successful extraction and validation.

## Known constraints

- GDAL's OSM driver reconstructs polygon features from OSM ways and relations; verify the resulting `multipolygons` layer and required fields with `ogrinfo` before relying on the extraction.
- The output is 2D; no building height or floor count is inferred.
- OSM coverage is not guaranteed to be complete.
