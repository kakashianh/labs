# GIS data preparation (KAN-6)

## Purpose

Extract 2D building footprints from the Geofabrik Vietnam OSM PBF, clip them to the approved historical TP.HCM administrative boundary, and export GeoJSON files for KAN-13.

The pipeline is a local data-preparation tool. It does not add a Python service, modify the Rust/Axum backend, create PostGIS tables, or import data into a database.

## Study boundary and temporal scope

- Boundary: OSM relation `1973756`.
- Historical cutoff: `2025-06-30T23:59:59Z`.
- The GitHub Actions workflow retrieves the relation from Overpass attic data at this exact cutoff and records its version/provenance.
- The building footprints are extracted from the Geofabrik Vietnam snapshot downloaded at workflow execution time. They are current-at-download, not historical-at-cutoff, and are clipped to the historical boundary.

Do not replace the historical boundary with the current geometry of relation 1973756.

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

1. `data/sample/source/hcm_boundary.geojson`: exactly one historical TP.HCM administrative boundary feature. Obtain it from the workflow artifact or reproduce the documented Overpass attic query.
2. `data/sample/source/vietnam-latest.osm.pbf`: downloaded automatically by the script from Geofabrik if absent.

The boundary must be the approved historical snapshot, not the current relation geometry.

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

The Geofabrik `latest` URL is mutable. Record the retrieval date, file size, SHA-256, GDAL version, boundary relation ID, historical cutoff, relation version, and Overpass query/endpoint in KAN-6 metadata. Checksums identify the exact inputs used.

Large PBF files and temporary artifacts are excluded from Git. Generated GeoJSON files should only be committed after successful extraction and validation; the preferred handoff is the workflow artifact.

## Known constraints

- Overpass attic support and public endpoint availability may vary. The workflow tries multiple public instances and fails rather than silently falling back to current relation geometry.
- GDAL's OSM driver reconstructs polygon features from OSM ways and relations; verify the `multipolygons` layer and required fields with `ogrinfo` before relying on extraction.
- The output is 2D; no building height or floor count is inferred.
- OSM coverage is not guaranteed to be complete.
