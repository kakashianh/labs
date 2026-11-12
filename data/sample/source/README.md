# OSM source data – Ben Thanh sample area

This directory stores reproducible source inputs for KAN-6.

## 1. Study-area boundary

The approved study area is an approximately 1 km × 1 km rectangular area centered on Ben Thanh Market.

- Center: 10.77257° N, 106.69802° E.
- WGS 84 bounds: west 106.693378, south 10.768079, east 106.702662, north 10.777061.
- Boundary method: locally defined rectangular area of interest (AOI), not an administrative boundary.
- Center reference: [OpenStreetMap way 39514795](https://www.openstreetmap.org/way/39514795).

The GitHub Actions workflow creates `hcm_boundary.geojson` from these fixed coordinates. No historical administrative relation or Overpass attic query is required.

## 2. Vietnam OSM extract

The workflow and local processing script use the Geofabrik Vietnam PBF:

- URL: https://download.geofabrik.de/asia/vietnam-latest.osm.pbf
- Local path: `data/sample/source/vietnam-latest.osm.pbf`
- The PBF is intentionally excluded from Git because of its size.

Record retrieval date, file size, and SHA-256 in generated metadata. Geofabrik's `latest` URL is mutable, so the checksum identifies the exact source snapshot used.

**Temporal distinction:** the AOI is a fixed sample polygon. Building data comes from the latest Vietnam extract available when the workflow runs; it is not a historical building snapshot.

## 3. License and attribution

OpenStreetMap data is available under the Open Database License (ODbL). Attribute © OpenStreetMap contributors and retain source URLs, retrieval dates, and checksums in project documentation and derived-data metadata.

See https://www.openstreetmap.org/copyright and https://www.geofabrik.de/data/download.html.

Do not commit source files if their applicable terms prohibit redistribution.