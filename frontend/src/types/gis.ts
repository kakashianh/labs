export interface RegionListItem {
  id: string
  code: string
  name: string
  description: string | null
}

export interface RegionListResponse {
  data: RegionListItem[]
}

export interface MultiPolygonGeometry {
  type: 'MultiPolygon'
  coordinates: number[][][][]
}

export interface Building {
  id: string
  region_id: string
  external_id: string
  name: string | null
  building_type: string | null
  geometry: MultiPolygonGeometry
}

export interface BuildingDetailResponse {
  data: Building
}

export interface Pagination {
  page: number
  page_size: number
  total: number
  total_pages: number
}

export interface BuildingListResponse {
  data: Building[]
  pagination: Pagination
}

export interface ApiErrorResponse {
  error: string
}

export interface GeoJsonFeature {
  type: 'Feature'
  id: string
  geometry: MultiPolygonGeometry
  properties: {
    id: string
    buildingId: string
    regionId: string
    externalId: string
    name: string | null
    buildingType: string | null
  }
}

export interface GeoJsonFeatureCollection {
  type: 'FeatureCollection'
  features: GeoJsonFeature[]
}
