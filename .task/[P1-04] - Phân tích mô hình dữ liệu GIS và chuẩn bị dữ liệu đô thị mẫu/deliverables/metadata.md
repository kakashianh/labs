# KAN-6 – Dataset metadata

> Status: template. Runtime values must be populated from the successful workflow artifact; do not invent relation versions, dates, checksums, or feature counts.

## Study area

| Property | Value |
|---|---|
| Name | Thành phố Hồ Chí Minh, Việt Nam (địa giới trước 01/07/2025) |
| Boundary source | OpenStreetMap historical administrative relation |
| OSM relation ID | 1973756 |
| Historical database cutoff | 2025-06-30T23:59:59Z |
| Boundary relation version | Pending workflow run |
| Boundary version timestamp | Pending workflow run |
| Boundary retrieval endpoint | Pending workflow run |
| Geometry | MultiPolygon expected; confirm from generated file |
| Output file | `data/sample/processed/hcm_boundary.geojson` |

## Building footprints

| Property | Value |
|---|---|
| Source | OpenStreetMap via Geofabrik Vietnam extract |
| Download URL | https://download.geofabrik.de/asia/vietnam-latest.osm.pbf |
| Retrieval date | Pending workflow run |
| Source file | `data/sample/source/vietnam-latest.osm.pbf` (not committed) |
| File size | Pending workflow run |
| SHA-256 | Pending workflow run |
| Source CRS | Inspect PBF metadata; do not assume |
| Output CRS | EPSG:4326, GeoJSON RFC 7946 |
| Selection | OSM features with `building IS NOT NULL` |
| Spatial filter | Clip to the historical HCMC boundary |
| Temporal meaning | Latest Geofabrik snapshot at workflow execution; not historical building state |
| Geometry | MultiPolygon, 2D |
| Output file | `data/sample/processed/hcm_buildings.geojson` |

## Coordinate reference system

- Source CRS: pending inspection of boundary and PBF using GDAL/OGR.
- Output CRS: WGS 84 longitude/latitude (EPSG:4326), emitted as RFC 7946 GeoJSON.
- Any reprojection must be performed by GDAL using the CRS declared by the source. Never use CRS assignment as a substitute for coordinate transformation.

## Attribute contract

See [data_dictionary.md](./data_dictionary.md).

## License and attribution

OpenStreetMap contributors, © OpenStreetMap contributors, data available under ODbL. Include attribution in reports and downstream product credits. Source URLs, retrieval date, and checksums must identify the exact snapshots used.

- Copyright: https://www.openstreetmap.org/copyright
- Geofabrik: https://download.geofabrik.de/asia/vietnam.html

## Processing provenance

| Item | Value |
|---|---|
| Script | `scripts/gis/prepare_osm_data.sh` |
| Workflow | `.github/workflows/kan-6-prepare-osm.yml` |
| Boundary cutoff | 2025-06-30T23:59:59Z |
| Boundary version / timestamp | Pending |
| Overpass endpoint | Pending |
| GDAL version | Pending |
| Boundary input checksum | Pending |
| PBF checksum | Pending |
| Execution date | Pending |
| Output feature counts | Pending; populate from actual run |
