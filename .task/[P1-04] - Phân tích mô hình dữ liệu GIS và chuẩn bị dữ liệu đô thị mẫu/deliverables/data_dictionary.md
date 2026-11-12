# KAN-6 – Data dictionary

## Layer: hcm_boundary

| Field | Type | Nullable | Description |
|---|---|---:|---|
| name | string | No | Ben Thanh sample study area |
| area_note | string | No | Approximate 1 km × 1 km AOI |
| center_lat | number | No | AOI center latitude, WGS 84 |
| center_lon | number | No | AOI center longitude, WGS 84 |
| boundary_method | string | No | Locally defined rectangle; not administrative boundary |
| source_reference | string | No | OSM reference for Ben Thanh Market location |

- Geometry: Polygon
- CRS: EPSG:4326 / RFC 7946
- Center: 10.77257° N, 106.69802° E

## Layer: hcm_buildings

| Field | Logical type | Nullable | Description |
|---|---|---:|---|
| building_id | string | No | Stable source namespace ID: `way/{osm_way_id}` or `relation/{osm_id}` |
| osm_type | string | No | `way` or `relation` |
| osm_id | integer/string | Yes | OSM relation ID, if applicable |
| osm_way_id | integer/string | Yes | OSM closed-way ID, if applicable |
| name | string | Yes | OSM `name` tag |
| building | string | Yes | OSM `building` tag |
| geometry | MultiPolygon | No | 2D building footprint |

- Source: Geofabrik Vietnam OSM PBF, latest snapshot at workflow execution.
- Selection: features with `building IS NOT NULL`.
- Spatial operation: clip to `hcm_boundary`.
- Output CRS: EPSG:4326 / RFC 7946.

## Important interpretation

The study boundary is a small, locally defined sample polygon around Ben Thanh Market. It does not represent a ward, district, or city administrative boundary. Building footprints are 2D OSM data and do not provide verified height, floor count, population, occupancy, or epidemiological attributes.