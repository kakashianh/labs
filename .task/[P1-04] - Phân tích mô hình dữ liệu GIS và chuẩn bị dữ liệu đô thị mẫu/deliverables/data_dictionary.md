# KAN-6 – Data dictionary

This dictionary defines the proposed handoff contract. Confirm actual field names and types against the generated GeoJSON before KAN-13 uses it.

## Layer: `hcm_boundary`

| Field | Logical type | Nullable | Description |
|---|---|---:|---|
| `osm_id` | string/integer | No | OSM relation ID of the selected administrative boundary |
| `name` | string | Yes | OSM `name` tag |
| `admin_level` | string | Yes | OSM administrative level tag |
| `boundary` | string | Yes | Expected administrative boundary tag |
| `geometry` | MultiPolygon | No | Study-area boundary |

## Layer: `hcm_buildings`

| Field | Logical type | Nullable | Description |
|---|---|---:|---|
| `building_id` | string | No | Stable source-derived identifier: `way/{id}` or `relation/{id}` |
| `osm_id` | string/integer | Yes | OSM relation ID; populated for relation features |
| `osm_way_id` | string/integer | Yes | OSM closed-way ID; populated for way features |
| `osm_type` | string | No | `way` or `relation`, inferred from populated source ID |
| `name` | string | Yes | OSM `name` tag |
| `building` | string | No | OSM `building=*` classification |
| `geometry` | MultiPolygon | No | 2D building footprint |

## Identifier rules

- Do not use GeoJSON feature array position or an auto-generated FID as a durable ID.
- Prefix OSM IDs with their object type because OSM node/way/relation IDs belong to different namespaces.
- Preserve original OSM ID fields to allow tracing back to source objects.
- If an expected source ID is missing, stop the pipeline and investigate rather than silently generating a random ID.

## Geometry and CRS

- Building footprint output is 2D MultiPolygon.
- GeoJSON RFC 7946 output coordinates are WGS 84 longitude/latitude (EPSG:4326).
- Do not calculate metric area/distance directly from degree coordinates.
- No height, floor count, population, occupancy, or epidemic attributes are part of KAN-6.
