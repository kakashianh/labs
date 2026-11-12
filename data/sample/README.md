# KAN-6 sample GIS data

This folder contains reproducible source references and outputs for the historical TP.HCM 2D OSM dataset.

## Agreed study boundary

- Boundary: OSM administrative relation `1973756` (Ho Chi Minh City before the 2025-07-01 administrative change).
- Historical cutoff: `2025-06-30T23:59:59Z`.
- The workflow queries Overpass attic data at that exact timestamp. It must not use the current relation geometry as a substitute.
- The building footprints come from the Geofabrik Vietnam extract downloaded at workflow execution time, then are clipped to the historical boundary. **The buildings are not a historical snapshot as of June 2025.**

## Run in GitHub Actions

1. Open the repository and select branch `task/p1-04-gis-data-model-sample-data`.
2. Open **Actions** and select **KAN-6 - Prepare HCMC OSM sample data**.
3. Select **Run workflow**, confirm `old-hcmc`, and start the run.
4. A successful run uploads artifact `kan-6-hcmc-old-osm-data`.
5. Download the artifact from the completed run. It contains GeoJSON layers, checksums, boundary provenance, generated metadata, and an automated quality summary.

The workflow resolves relation 1973756 from historical OSM state at the agreed cutoff, verifies its administrative tags and approximate geographic extent, downloads the Geofabrik Vietnam PBF, clips it with Osmium, and processes buildings with GDAL/OGR. After validation, it commits the source boundary, processed GeoJSON layers, and lightweight metadata/quality summary to the selected branch when each file is at most 50 MiB. Larger outputs remain available in the workflow artifact instead.

> GitHub only allows manually dispatching a workflow when its workflow file is present on the repository's default branch. If this workflow exists only on the task branch and does not appear in Actions, it must first be reviewed and merged to the default branch; then select this task branch as the run ref. Do not merge the entire task branch unless intended.

## Data and licensing notes

- Output format: GeoJSON RFC 7946 / WGS 84 longitude-latitude (EPSG:4326).
- Building layer contains 2D OSM building footprints only. Height, floors, population, occupancy, and epidemiological attributes are not inferred.
- Attribution: © OpenStreetMap contributors, ODbL.
- OSM is community-maintained and may be incomplete; automated QA does not replace visual inspection in QGIS.

Do not commit the large source PBF or temporary clipped PBF to Git. The workflow commits validated boundary/building GeoJSON and lightweight provenance when each file is at most 50 MiB; otherwise it preserves the complete output in the workflow artifact for KAN-13/KAN-14.
