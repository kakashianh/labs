import type {
  Building,
  BuildingListResponse,
  GeoJsonFeatureCollection,
} from '../types/gis'

export const BUILDINGS_PAGE_SIZE = 500

type BuildingPageFetcher = (page: number, pageSize: number) => Promise<BuildingListResponse>

/**
 * Loads every page reported by the API. The current dataset size is deliberately
 * not used as a completion condition: the API pagination metadata is authoritative.
 */
export async function fetchAllBuildingPages(
  fetchPage: BuildingPageFetcher,
  pageSize = BUILDINGS_PAGE_SIZE,
): Promise<Building[]> {
  const firstPage = await fetchPage(1, pageSize)
  const buildings = [...firstPage.data]

  for (let page = 2; page <= firstPage.pagination.total_pages; page += 1) {
    const response = await fetchPage(page, pageSize)
    buildings.push(...response.data)
  }

  return buildings
}

function isValidMultiPolygon(geometry: Building['geometry']): boolean {
  return (
    geometry?.type === 'MultiPolygon' &&
    Array.isArray(geometry.coordinates) &&
    geometry.coordinates.length > 0 &&
    geometry.coordinates.every(
      (polygon) =>
        Array.isArray(polygon) &&
        polygon.length > 0 &&
        polygon.every(
          (ring) =>
            Array.isArray(ring) &&
            ring.length >= 4 &&
            ring.every(
              (position) =>
                Array.isArray(position) &&
                position.length >= 2 &&
                Number.isFinite(position[0]) &&
                Number.isFinite(position[1]),
            ),
        ),
    )
  )
}

/**
 * The API returns a Geometry inside each DTO, while Cesium's GeoJSON loader
 * expects Features/FeatureCollections. Keep the backend UUID as Feature.id.
 */
export function buildingsToFeatureCollection(
  buildings: Building[],
): { featureCollection: GeoJsonFeatureCollection; skippedCount: number } {
  const features: GeoJsonFeatureCollection['features'] = []
  let skippedCount = 0

  for (const building of buildings) {
    if (!building.geometry || !isValidMultiPolygon(building.geometry)) {
      skippedCount += 1
      continue
    }

    features.push({
      type: 'Feature',
      id: building.id,
      geometry: building.geometry,
      properties: {
        id: building.id,
        buildingId: building.id,
        regionId: building.region_id,
        externalId: building.external_id,
        name: building.name,
        buildingType: building.building_type,
      },
    })
  }

  return {
    featureCollection: {
      type: 'FeatureCollection',
      features,
    },
    skippedCount,
  }
}
