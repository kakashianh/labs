-- KAN-13: PostGIS base schema for the Ben Thanh study area.
-- Source contract: both hcm_boundary and hcm_buildings are MultiPolygon, EPSG:4326.
-- Internal identifiers use UUID; OSM identifiers remain traceable through external_id.

CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE regions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(64) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    boundary_method VARCHAR(64) NOT NULL,
    source_reference TEXT,
    boundary geometry(MultiPolygon, 4326) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE buildings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(128) NOT NULL UNIQUE,
    osm_type VARCHAR(16) NOT NULL,
    osm_id BIGINT,
    osm_way_id BIGINT,
    region_id UUID NOT NULL REFERENCES regions(id) ON DELETE RESTRICT,
    name TEXT,
    building_type TEXT,
    footprint geometry(MultiPolygon, 4326) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT buildings_osm_type_check
        CHECK (osm_type IN ('way', 'relation')),
    CONSTRAINT buildings_osm_identifier_check
        CHECK (
            (osm_type = 'way' AND osm_way_id IS NOT NULL AND osm_id IS NULL)
            OR
            (osm_type = 'relation' AND osm_id IS NOT NULL AND osm_way_id IS NULL)
        )
);

CREATE INDEX regions_boundary_gist_idx
    ON regions USING GIST (boundary);

CREATE INDEX buildings_footprint_gist_idx
    ON buildings USING GIST (footprint);

CREATE INDEX buildings_region_id_idx
    ON buildings (region_id);
