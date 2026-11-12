\set ON_ERROR_STOP on

DO $kan14_verify$
DECLARE
    v_region_id UUID;
    v_expected BIGINT;
    v_actual BIGINT;
BEGIN
    SELECT id INTO v_region_id
    FROM public.regions
    WHERE code = 'hcm-ben-thanh-1km2';

    IF v_region_id IS NULL THEN
        RAISE EXCEPTION 'Imported region hcm-ben-thanh-1km2 was not found.';
    END IF;

    SELECT COUNT(*) INTO v_expected FROM public.kan14_stage_buildings;
    SELECT COUNT(*) INTO v_actual
    FROM public.buildings
    WHERE region_id = v_region_id;

    IF v_actual <> v_expected THEN
        RAISE EXCEPTION 'Building count mismatch: expected %, found %.', v_expected, v_actual;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.kan14_stage_buildings s
        LEFT JOIN public.buildings b ON b.external_id = s.building_id::text
        WHERE b.id IS NULL
           OR b.region_id IS DISTINCT FROM v_region_id
           OR b.osm_type IS DISTINCT FROM s.osm_type::text
           OR b.osm_id IS DISTINCT FROM NULLIF(BTRIM(s.osm_id::text), '')::BIGINT
           OR b.osm_way_id IS DISTINCT FROM NULLIF(BTRIM(s.osm_way_id::text), '')::BIGINT
           OR b.name IS DISTINCT FROM NULLIF(s.name::text, '')
           OR b.building_type IS DISTINCT FROM NULLIF(s.building::text, '')
           OR NOT ST_Equals(b.footprint, ST_Multi(s.geom))
    ) THEN
        RAISE EXCEPTION 'Imported building attributes or geometry differ from the staged source.';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.buildings b
        WHERE b.region_id = v_region_id
          AND (b.footprint IS NULL OR ST_IsEmpty(b.footprint)
               OR NOT ST_IsValid(b.footprint)
               OR ST_SRID(b.footprint) <> 4326
               OR GeometryType(b.footprint) <> 'MULTIPOLYGON')
    ) THEN
        RAISE EXCEPTION 'Invalid building geometry found after import.';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.buildings b
        WHERE b.region_id = v_region_id
          AND NOT EXISTS (
              SELECT 1 FROM public.regions r
              WHERE r.id = b.region_id
          )
    ) THEN
        RAISE EXCEPTION 'Orphan building region_id found.';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.buildings b
        JOIN public.regions r ON r.id = b.region_id
        WHERE b.region_id = v_region_id
          AND NOT ST_Intersects(r.boundary, b.footprint)
    ) THEN
        RAISE EXCEPTION 'A building does not intersect its assigned region boundary.';
    END IF;
END;
$kan14_verify$;

SELECT
    r.code,
    COUNT(b.id) AS building_count,
    COUNT(*) FILTER (WHERE b.name IS NULL) AS buildings_without_name,
    COUNT(*) FILTER (WHERE b.building_type IS NULL) AS buildings_without_type,
    COUNT(*) FILTER (WHERE NOT ST_IsValid(b.footprint)) AS invalid_footprints,
    COUNT(*) FILTER (WHERE ST_SRID(b.footprint) <> 4326) AS wrong_srid
FROM public.regions r
JOIN public.buildings b ON b.region_id = r.id
WHERE r.code = 'hcm-ben-thanh-1km2'
GROUP BY r.code;

SELECT ST_Extent(boundary) AS region_extent
FROM public.regions
WHERE code = 'hcm-ben-thanh-1km2';

EXPLAIN ANALYZE
SELECT b.id
FROM public.buildings b
JOIN public.regions r ON r.id = b.region_id
WHERE r.code = 'hcm-ben-thanh-1km2'
  AND ST_Intersects(r.boundary, b.footprint);
