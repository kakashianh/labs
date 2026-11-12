# KAN-6 – Dataset metadata

> Status: template. Runtime values must be populated from a successful workflow run; do not invent checksums or feature counts.

## Study area

| Property | Value |
|---|---|
| Name | Ben Thanh Market sample area, District 1, Ho Chi Minh City |
| Area | Approximately 1 km² (about 1 km × 1 km) |
| Center | 10.77257° N, 106.69802° E |
| WGS 84 bounds | 106.693378–106.702662° E, 10.768079–10.777061° N |
| Boundary method | Locally defined rectangular AOI; not an administrative boundary |
| Center reference | https://www.openstreetmap.org/way/39514795 |
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
| Output CRS | EPSG:4326, GeoJSON RFC 7946 |
| Selection | OSM features with `building IS NOT NULL` |
| Spatial filter | Clip to Ben Thanh sample AOI |
| Temporal meaning | Latest Geofabrik snapshot at workflow execution; not historical building state |
| Geometry | MultiPolygon, 2D |
| Output file | `data/sample/processed/hcm_buildings.geojson` |

## Coordinate reference system

- AOI and output: WGS 84 longitude/latitude (EPSG:4326).
- For area/distance analysis, reproject to a suitable local projected CRS or use geodesic calculations.

## Attribute contract

See [data_dictionary.md](./data_dictionary.md).

## License and attribution

OpenStreetMap contributors, © OpenStreetMap contributors, data available under ODbL. Include attribution in reports and downstream product credits. Source URLs, retrieval date, and checksums identify the exact source snapshot.

- Copyright: https://www.openstreetmap.org/copyright
- Geofabrik: https://download.geofabrik.de/asia/vietnam.html

## Processing provenance

| Item | Value |
|---|---|
| Script | `scripts/gis/prepare_osm_data.sh` |
| Workflow | `.github/workflows/kan-6-prepare-osm.yml` |
| AOI coordinates | Fixed bounds documented above |
| GDAL version | Pending |
| AOI input checksum | Pending |
| PBF checksum | Pending |
| Execution date | Pending |
| Boundary/building feature counts | Pending; populate from actual run |
| Output checksums | Pending |
