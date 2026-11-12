# KAN-6 – Data quality report

> Status: NOT RUN. Complete this report from actual generated files. Do not replace pending values with estimates.

## Study area

- Area: approximately 1 km² around Ben Thanh Market.
- Center: 10.77257° N, 106.69802° E.
- Bounds: 106.693378–106.702662° E, 10.768079–10.777061° N.
- Boundary: locally defined sample AOI, not an administrative boundary.

## Input inventory

| Check | Result |
|---|---|
| AOI source and method | Fixed rectangular AOI; workflow verification pending |
| AOI geometry / feature count | Pending workflow run |
| Vietnam PBF retrieval date and SHA-256 | Pending |
| GDAL version | Pending |
| Source CRS and extent | Pending |

## Output inventory

| Metric | Boundary | Buildings |
|---|---:|---:|
| Feature count | Pending | Pending |
| Geometry type | Pending | Pending |
| CRS | Pending | Pending |
| Empty geometries | Pending | Pending |
| Invalid geometries | Pending | Pending |
| Null required identifiers | N/A | Pending |
| Duplicate identifiers | N/A | Pending |
| Features outside study area | N/A | Pending |

## Issues and remediation

| Issue | Count | Action | Outcome |
|---|---:|---|---|
| Pending inspection | — | Run KAN-6 extraction and validation workflow | Pending |

## Visual QA

- [ ] AOI is centered around Ben Thanh Market and covers approximately 1 km².
- [ ] Building footprints render in QGIS.
- [ ] No obvious clipping artifacts or widespread geometry distortion.
- [ ] Representative blocks inside the AOI inspected.
- [ ] Any gaps or uneven OSM coverage documented.

## Temporal interpretation

The AOI is a fixed sample polygon. Building footprints represent the Geofabrik Vietnam extract downloaded at workflow execution time; they are not a historical building snapshot.

## Known limitations

- OSM is community-maintained; completeness and attribute coverage vary by location.
- A missing building in OSM must not be interpreted as proof that no building exists.
- Building classification is based on the source `building=*` tag and may be generic or absent.
- This dataset contains 2D footprints only; it is not a 3D building model or a population dataset.

## Sign-off

- Prepared by: Pending
- Validation date: Pending
- Handoff to KAN-13: Pending
