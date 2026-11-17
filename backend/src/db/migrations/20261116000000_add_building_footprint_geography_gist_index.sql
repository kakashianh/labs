-- P2-01: metric spatial index for EPSG:4326 footprint neighbor queries.
-- The existing geometry GiST index cannot serve ST_DWithin(footprint::geography, ...).
-- Invalid/empty/non-4326 records are excluded by the partial-index predicate.
CREATE INDEX buildings_footprint_geography_gist_idx
    ON public.buildings
    USING GIST ((footprint::geography))
    WHERE ST_SRID(footprint) = 4326
      AND NOT ST_IsEmpty(footprint)
      AND ST_IsValid(footprint);
