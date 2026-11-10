# KAN-6 – Dataset metadata

> Status: preparation template. Fill the pending values only after retrieving and inspecting the actual OSM source files.

## Study area

| Property | Value |
|---|---|
| Name | Thành phố Hồ Chí Minh, Việt Nam |
| Boundary source | OpenStreetMap administrative relation |
| OSM relation ID | Pending boundary verification |
| Boundary name / admin_level | Pending boundary verification |
| Retrieval date | Pending |
| Overpass query / source URL | Pending |
| Geometry | MultiPolygon expected; confirm from source |
| Output file | `data/sample/processed/hcm_boundary.geojson` |

## Building footprints

| Property | Value |
|---|---|
| Source | OpenStreetMap via Geofabrik Vietnam extract |
| Download URL | https://download.geofabrik.de/asia/vietnam-latest.osm.pbf |
| Retrieval date | Pending |
| Source file | `data/sample/source/vietnam-latest.osm.pbf` (not committed) |
| SHA-256 | Pending |
| Source CRS | Inspect PBF metadata; do not assume |
| Output CRS | EPSG:4326, GeoJSON RFC 7946 |
| Selection | OSM features with `building IS NOT NULL` |
| Spatial filter | Clip to verified HCMC boundary |
| Geometry | MultiPolygon, 2D |
| Output file | `data/sample/processed/hcm_buildings.geojson` |

## Coordinate reference system

- Source CRS: pending inspection of boundary and PBF using GDAL/OGR.
- Output CRS: WGS 84 longitude/latitude (EPSG:4326), emitted as RFC 7946 GeoJSON.
- Any reprojection must be performed by GDAL using the CRS declared by the source. Never use CRS assignment as a substitute for coordinate transformation.

## Attribute contract

See [data_dictionary.md](./data_dictionary.md).

## License and attribution

OpenStreetMap contributors, © OpenStreetMap contributors, data available under ODbL. Include attribution in reports and downstream product credits. Source URL, retrieval date, and checksum must identify the exact snapshot used.

- Copyright: https://www.openstreetmap.org/copyright
- Geofabrik: https://download.geofabrik.de/asia/vietnam.html

## Processing provenance

| Item | Value |
|---|---|
| Script | `scripts/gis/prepare_osm_data.sh` |
| GDAL version | Pending |
| Boundary input checksum | Pending |
| PBF checksum | Pending |
| Execution date | Pending |
| Output feature counts | Pending; populate from actual run |
