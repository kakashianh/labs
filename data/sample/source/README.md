# OSM source data – HCMC

This directory stores reproducible source inputs for KAN-6.

## 1. City boundary

Obtain the administrative boundary from OpenStreetMap using [Overpass Turbo](https://overpass-turbo.eu/).

1. Search for administrative relations whose name identifies Ho Chi Minh City.
2. Verify the selected relation visually and inspect `boundary=administrative`, `name`, `admin_level`, and relation identity.
3. Ensure the result is the intended city-level boundary, not a ward, district, metropolitan-area approximation, or similarly named feature.
4. Export exactly the selected boundary feature as GeoJSON.
5. Save it as `hcm_boundary.geojson` in this directory.
6. Record the relation ID, Overpass query URL, retrieval date, and attribution in the KAN-6 `metadata.md`.

Do not guess or hard-code a relation ID/admin_level before verifying the result. The processing script expects a single boundary layer/feature with a readable CRS.

## 2. Vietnam OSM extract

The processing script uses the Geofabrik Vietnam PBF:

- URL: https://download.geofabrik.de/asia/vietnam-latest.osm.pbf
- Local path: `data/sample/source/vietnam-latest.osm.pbf`
- The PBF is intentionally excluded from Git because of its size. Download it when running the pipeline.

Record retrieval date, file size, and SHA-256 in `metadata.md`. Geofabrik extracts are snapshots and can change over time; use the checksum to identify the exact input used.

## 3. License and attribution

OpenStreetMap data is available under the Open Database License (ODbL). Attribute © OpenStreetMap contributors and retain the source URL and retrieval date in project documentation and derived-data metadata.

See https://www.openstreetmap.org/copyright and https://www.geofabrik.de/data/download.html.

Do not commit source files if their applicable terms prohibit redistribution.
