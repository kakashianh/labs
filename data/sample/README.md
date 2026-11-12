# KAN-6 sample GIS data

This folder contains reproducible source references and outputs for a compact urban GIS sample around Ben Thanh Market, District 1, Ho Chi Minh City.

## Agreed study area

- Area: approximately **1 km² (about 1 km × 1 km)** centered on Ben Thanh Market.
- Center: 10.77257° N, 106.69802° E.
- AOI bounds (WGS 84): 106.693378–106.702662° E, 10.768079–10.777061° N.
- Boundary: a locally defined rectangular polygon for the coursework sample, **not an administrative boundary**.
- Center reference: OpenStreetMap way [39514795](https://www.openstreetmap.org/way/39514795).
- Building footprints: Geofabrik Vietnam OSM extract downloaded when the workflow runs and clipped to this AOI. They are current-at-download, not a historical building snapshot.

## Run in GitHub Actions

1. Open the repository and select branch `task/p1-04-gis-data-model-sample-data`.
2. Open **Actions** and select **KAN-6 - Prepare Ben Thanh sample GIS data**.
3. Select **Run workflow**, confirm `ben-thanh-1km2`, and start the run.
4. A successful run uploads artifact `kan-6-ben-thanh-osm-data`.
5. Download the artifact. It contains the AOI, processed GeoJSON layers, checksums, provenance, generated metadata, and automated quality summary.

The workflow creates the AOI polygon, downloads the Geofabrik Vietnam PBF, clips it with Osmium, and processes OSM building footprints with GDAL/OGR. After validation, it commits the source boundary, processed GeoJSON layers, and lightweight metadata/quality summary when each file is at most 50 MiB. Larger outputs remain in the workflow artifact.

## Data and licensing notes

- Output format: GeoJSON RFC 7946 / WGS 84 longitude-latitude (EPSG:4326).
- Building layer contains 2D OSM building footprints only. Height, floors, population, occupancy, and epidemiological attributes are not inferred.
- Attribution: © OpenStreetMap contributors, ODbL.
- OSM coverage may be incomplete; automated QA does not replace visual inspection in QGIS.

Do not commit the large source PBF or temporary clipped PBF to Git.