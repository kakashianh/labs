# OSM source data – HCMC

This directory stores reproducible source inputs for KAN-6.

## 1. Historical city boundary

The approved study boundary is OpenStreetMap relation `1973756`, reconstructed from the OSM database state at **`2025-06-30T23:59:59Z`**.

The GitHub Actions workflow ``.github/workflows/kan-6-prepare-osm.yml`` retrieves the historical relation using Overpass attic data and writes `hcm_boundary.geojson`. It records the resolved relation version, version timestamp, Overpass endpoint, and boundary checksum in the artifact.

For manual retrieval, use Overpass Turbo with this query:

```overpassql
[out:json][timeout:180][date:"2025-06-30T23:59:59Z"];
relation(1973756);
out meta;
>;
out skel qt;
```

Convert the returned OSM JSON to a GeoJSON feature for relation 1973756, preserving its tags and geometry. Do not query the current relation without the `date` setting: its geometry may have changed since the historical cutoff. Verify `boundary=administrative`, `admin_level=4`, relation identity, and the city-level extent. Record the relation version and timestamp.

## 2. Vietnam OSM extract

The workflow and local processing script use the Geofabrik Vietnam PBF:

- URL: https://download.geofabrik.de/asia/vietnam-latest.osm.pbf
- Local path: `data/sample/source/vietnam-latest.osm.pbf`
- The PBF is intentionally excluded from Git because of its size.

Record retrieval date, file size, and SHA-256 in generated metadata. Geofabrik's `latest` URL is mutable, so the checksum identifies the exact source snapshot used.

**Temporal distinction:** only the administrative boundary is historical at the agreed cutoff. The downloaded building data is the latest Vietnam extract available when the workflow runs, spatially clipped to the historical boundary.

## 3. License and attribution

OpenStreetMap data is available under the Open Database License (ODbL). Attribute © OpenStreetMap contributors and retain source URLs, retrieval dates, and checksums in project documentation and derived-data metadata.

See https://www.openstreetmap.org/copyright and https://www.geofabrik.de/data/download.html.

Do not commit source files if their applicable terms prohibit redistribution.
