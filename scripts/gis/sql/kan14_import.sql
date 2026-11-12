\set ON_ERROR_STOP on
BEGIN;

WITH region_upsert AS (
    INSERT INTO public.regions (
        code,
        name,
        description,
        boundary_method,
        source_reference,
        boundary
    )
    SELECT
        'hcm-ben-thanh-1km2',
        LEFT(name::text, 255),
        area_note::text,
        boundary_method::text,
        source_reference::text,
        ST_Multi(geom)::geometry(MultiPolygon, 4326)
    FROM public.kan14_stage_regions
    ON CONFLICT (code) DO UPDATE SET
        name = EXCLUDED.name,
        description = EXCLUDED.description,
        boundary_method = EXCLUDED.boundary_method,
        source_reference = EXCLUDED.source_reference,
        boundary = EXCLUDED.boundary,
        updated_at = NOW()
    RETURNING id
),
building_upsert AS (
    INSERT INTO public.buildings (
        external_id,
        osm_type,
        osm_id,
        osm_way_id,
        region_id,
        name,
        building_type,
        footprint
    )
    SELECT
        b.building_id::text,
        b.osm_type::text,
        NULLIF(BTRIM(b.osm_id::text), '')::BIGINT,
        NULLIF(BTRIM(b.osm_way_id::text), '')::BIGINT,
        r.id,
        NULLIF(b.name::text, ''),
        NULLIF(b.building::text, ''),
        ST_Multi(b.geom)::geometry(MultiPolygon, 4326)
    FROM public.kan14_stage_buildings b
    CROSS JOIN region_upsert r
    ON CONFLICT (external_id) DO UPDATE SET
        osm_type = EXCLUDED.osm_type,
        osm_id = EXCLUDED.osm_id,
        osm_way_id = EXCLUDED.osm_way_id,
        region_id = EXCLUDED.region_id,
        name = EXCLUDED.name,
        building_type = EXCLUDED.building_type,
        footprint = EXCLUDED.footprint,
        updated_at = NOW()
    RETURNING (xmax = 0) AS inserted
)
SELECT
    COUNT(*) FILTER (WHERE inserted) AS inserted_buildings,
    COUNT(*) FILTER (WHERE NOT inserted) AS updated_buildings
FROM building_upsert;

COMMIT;

SELECT
    r.code,
    r.id AS region_id,
    COUNT(b.id) AS buildings_in_region
FROM public.regions r
LEFT JOIN public.buildings b ON b.region_id = r.id
WHERE r.code = 'hcm-ben-thanh-1km2'
GROUP BY r.code, r.id;
