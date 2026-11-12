# KAN-6 – Data quality report

> Status: NOT RUN. This report must be completed from the actual generated files. Do not replace pending values with estimates.
>
> Temporal scope: boundary is the historical OSM state at `2025-06-30T23:59:59Z` (UTC), relation `1973756`. Building footprints are from the Geofabrik Vietnam snapshot downloaded during the workflow run; they are not historical building records from June 2025.

## Input inventory

| Check | Result |
|---|---|
| HCMC boundary source and relation ID | OSM relation 1973756; historical cutoff 2025-06-30T23:59:59Z; run verification pending |
| Boundary geometry / feature count | Pending workflow run |
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
| Null required identifiers | Pending | Pending |
| Duplicate identifiers | N/A | Pending |
| Features outside study boundary | N/A | Pending |

## Issues and remediation

| Issue | Count | Action | Outcome |
|---|---:|---|---|
| Pending inspection | — | Run the KAN-6 extraction and validation workflow | Pending |

## Visual QA

- [ ] Boundary corresponds to the intended TP.HCM administrative extent.
- [ ] Building footprints render in QGIS.
- [ ] No obvious clipping artifacts or widespread geometry distortion.
- [ ] Representative central and peripheral areas inspected.
- [ ] Any gaps or uneven OSM coverage documented.

## Temporal interpretation

- The administrative boundary represents the OSM database state at 2025-06-30T23:59:59Z.
- Building footprints represent the Geofabrik Vietnam extract downloaded at workflow execution time, clipped to the historical boundary. Do not describe these footprints as a June 2025 building snapshot.

## Known limitations

- OSM is community-maintained; completeness and attribute coverage vary by location.
- A missing building in OSM must not be interpreted as proof that no building exists.
- Building classification is based on the source `building=*` tag and may be generic or absent.
- This dataset contains 2D footprints only; it is not a 3D building model or a population dataset.

## Sign-off

- Prepared by: Pending
- Validation date: Pending
- Handoff to KAN-13: Pending
