import { getJson } from './api-client'
import { fetchAllBuildingPages } from './gis-data'
import type {
  Building,
  BuildingListResponse,
  RegionListResponse,
} from '../types/gis'

export function fetchRegions(signal?: AbortSignal) {
  return getJson<RegionListResponse>('/api/v1/regions', signal)
}

export function fetchBuildingPage(
  page: number,
  pageSize: number,
  regionId?: string,
  signal?: AbortSignal,
): Promise<BuildingListResponse> {
  const params = new URLSearchParams({
    page: String(page),
    page_size: String(pageSize),
  })

  if (regionId) {
    params.set('region_id', regionId)
  }

  return getJson<BuildingListResponse>(`/api/v1/buildings?${params.toString()}`, signal)
}

export function fetchAllBuildings(
  regionId?: string,
  signal?: AbortSignal,
): Promise<Building[]> {
  return fetchAllBuildingPages(
    (page, pageSize) => fetchBuildingPage(page, pageSize, regionId, signal),
  )
}
