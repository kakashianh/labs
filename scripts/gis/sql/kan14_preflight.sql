\set ON_ERROR_STOP on

DO $kan14_preflight$
BEGIN
    IF to_regclass('public.regions') IS NULL OR to_regclass('public.buildings') IS NULL THEN
        RAISE EXCEPTION 'KAN-13 schema is missing. Start the backend so SQLx migrations can run.';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'postgis') THEN
        RAISE EXCEPTION 'PostGIS extension is not enabled.';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM geometry_columns
        WHERE f_table_schema = 'public' AND f_table_name = 'regions'
          AND f_geometry_column = 'boundary' AND type = 'MULTIPOLYGON' AND srid = 4326
    ) OR NOT EXISTS (
        SELECT 1 FROM geometry_columns
        WHERE f_table_schema = 'public' AND f_table_name = 'buildings'
          AND f_geometry_column = 'footprint' AND type = 'MULTIPOLYGON' AND srid = 4326
    ) THEN
        RAISE EXCEPTION 'KAN-13 geometry columns must be MultiPolygon / EPSG:4326.';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname='public' AND indexname='regions_boundary_gist_idx')
       OR NOT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname='public' AND indexname='buildings_footprint_gist_idx')
       OR NOT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname='public' AND indexname='buildings_region_id_idx') THEN
        RAISE EXCEPTION 'One or more KAN-13 spatial/region indexes are missing.';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema='public' AND table_name='buildings' AND column_name='building_type'
    ) THEN
        RAISE EXCEPTION 'KAN-13 target column buildings.building_type is missing.';
    END IF;

    IF (SELECT COUNT(*) FROM public.kan14_stage_regions) <> 1 THEN
        RAISE EXCEPTION 'Expected exactly one staged region feature.';
    END IF;

    IF EXISTS (
        SELECT 1 FROM public.kan14_stage_regions
        WHERE geom IS NULL OR ST_IsEmpty(geom) OR NOT ST_IsValid(geom)
           OR ST_SRID(geom) <> 4326 OR GeometryType(geom) <> 'MULTIPOLYGON'
           OR ST_NDims(geom) <> 2
    ) THEN
        RAISE EXCEPTION 'Boundary geometry must be valid, non-empty, 2D MultiPolygon / EPSG:4326.';
    END IF;

    IF EXISTS (
        SELECT 1 FROM public.kan14_stage_regions
        WHERE NULLIF(BTRIM(name::text), '') IS NULL
           OR NULLIF(BTRIM(boundary_method::text), '') IS NULL
           OR LENGTH(boundary_method::text) > 64
           OR NULLIF(BTRIM(source_reference::text), '') IS NULL
    ) THEN
        RAISE EXCEPTION 'Region attributes are missing or boundary_method exceeds the KAN-13 VARCHAR(64) limit.';
    END IF;

    IF EXISTS (
        SELECT 1 FROM public.kan14_stage_buildings
        WHERE geom IS NULL OR ST_IsEmpty(geom) OR NOT ST_IsValid(geom)
           OR ST_SRID(geom) <> 4326 OR GeometryType(geom) <> 'MULTIPOLYGON'
           OR ST_NDims(geom) <> 2
    ) THEN
        RAISE EXCEPTION 'Building geometry must be valid, non-empty, 2D MultiPolygon / EPSG:4326.';
    END IF;

    IF EXISTS (
        SELECT 1 FROM public.kan14_stage_buildings
        WHERE NULLIF(BTRIM(building_id::text), '') IS NULL
    ) THEN
        RAISE EXCEPTION 'One or more buildings have a null/blank building_id.';
    END IF;

    IF EXISTS (
        SELECT building_id
        FROM public.kan14_stage_buildings
        GROUP BY building_id
        HAVING COUNT(*) > 1
    ) THEN
        RAISE EXCEPTION 'Duplicate building_id values found in the source snapshot.';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.kan14_stage_buildings
        WHERE NOT (
            (osm_type::text = 'way'
             AND NULLIF(BTRIM(osm_way_id::text), '') ~ '^[0-9]+$'
             AND NULLIF(BTRIM(osm_id::text), '') IS NULL)
            OR
            (osm_type::text = 'relation'
             AND NULLIF(BTRIM(osm_id::text), '') ~ '^[0-9]+$'
             AND NULLIF(BTRIM(osm_way_id::text), '') IS NULL)
        )
    ) THEN
        RAISE EXCEPTION 'OSM identifiers are inconsistent with osm_type.';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.kan14_stage_buildings
        WHERE building_id::text IS DISTINCT FROM
            CASE osm_type::text
                WHEN 'way' THEN 'way/' || BTRIM(osm_way_id::text)
                WHEN 'relation' THEN 'relation/' || BTRIM(osm_id::text)
                ELSE NULL
            END
    ) THEN
        RAISE EXCEPTION 'building_id namespace does not match osm_type and OSM identifier.';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.kan14_stage_regions r
        CROSS JOIN public.kan14_stage_buildings b
        WHERE NOT ST_Intersects(r.geom, b.geom)
    ) THEN
        RAISE EXCEPTION 'At least one building footprint does not intersect the approved AOI.';
    END IF;
END;
$kan14_preflight$;

SELECT
    (SELECT COUNT(*) FROM public.kan14_stage_regions) AS staged_regions,
    (SELECT COUNT(*) FROM public.kan14_stage_buildings) AS staged_buildings,
    (SELECT COUNT(*) FROM public.kan14_stage_buildings WHERE name IS NULL) AS buildings_without_name,
    (SELECT COUNT(*) FROM public.kan14_stage_buildings WHERE building IS NULL) AS buildings_without_type,
    (SELECT COUNT(*) FROM public.kan14_stage_buildings WHERE osm_type = 'way') AS osm_ways,
    (SELECT COUNT(*) FROM public.kan14_stage_buildings WHERE osm_type = 'relation') AS osm_relations;

SELECT
    GeometryType(geom) AS geometry_type,
    ST_SRID(geom) AS srid,
    ST_Extent(geom) AS extent
FROM public.kan14_stage_regions
GROUP BY GeometryType(geom), ST_SRID(geom);
