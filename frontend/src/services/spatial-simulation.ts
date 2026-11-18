import { getJson } from './api-client.ts'

export interface SpatialSimulationSummary {
  simulation_id: string
  status: string
  population_source: string
  eligible_building_count: number
  excluded_building_count: number
  result_point_count: number
  time_step_count: number
  completed_at: string | null
}

export interface SpatialResultsPage {
  simulation_id: string
  time_step: number
  time: number
  data: SpatialBuildingResult[]
  pagination: SpatialPagination
}

export interface SpatialBuildingResult {
  building_id: string
  region_id: string
  population: number
  susceptible: number
  infected: number
  recovered: number
}

export interface SpatialRiskThresholds {
  version: string
  source: string
  risk_low_max: number
  risk_medium_max: number
}

export type SpatialRiskLevel = 'low' | 'medium' | 'high' | 'unclassified'

export interface SpatialBuildingRisk {
  building_id: string
  region_id: string
  population: number | null
  infected: number
  infection_ratio: number | null
  risk_level: SpatialRiskLevel
  unclassified_reason: string | null
}

export interface SpatialRiskPage {
  simulation_id: string
  time_step: number
  thresholds: SpatialRiskThresholds
  data: SpatialBuildingRisk[]
  pagination: SpatialPagination
}

export interface SpatialPagination {
  page: number
  page_size: number
  total: number
  total_pages: number
}

export interface SpatialHotspot {
  cluster_id: string
  simulation_id: string
  time_step: number
  region_id: string
  member_count: number
  population_total: number
  infected_total: number
  infection_ratio: number
  risk_level: SpatialRiskLevel
  building_ids: string[]
  geometry: {
    type: 'MultiPolygon'
    coordinates: number[][][][]
  }
}

export interface SpatialHotspotsResponse {
  simulation_id: string
  time_step: number
  region_id: string | null
  configuration: {
    algorithm: string
    algorithm_version: string
    minimum_risk_level: 'medium' | 'high'
    risk_thresholds: SpatialRiskThresholds
    radius_m: number
    include_singletons: boolean
  }
  hotspot_count: number
  hotspots: SpatialHotspot[]
}

export interface SpatialBuildingView {
  building_id: string
  region_id: string
  result: SpatialBuildingResult | null
  risk: SpatialBuildingRisk
}

export interface SpatialSimulationFrame {
  summary: SpatialSimulationSummary
  time_step: number
  time: number
  thresholds: SpatialRiskThresholds
  buildings: SpatialBuildingView[]
  hotspots: SpatialHotspot[]
}

const PAGE_SIZE = 500

function queryPath(
  simulationId: string,
  endpoint: 'results' | 'risk-results' | 'hotspots',
  timeStep: number,
  page?: number,
): string {
  const params = new URLSearchParams({ time_step: String(timeStep) })
  if (page !== undefined) {
    params.set('page', String(page))
    params.set('page_size', String(PAGE_SIZE))
  }
  return `/api/v1/simulations/${encodeURIComponent(simulationId)}/${endpoint}?${params.toString()}`
}

async function fetchAllResultPages(
  simulationId: string,
  timeStep: number,
  signal?: AbortSignal,
): Promise<SpatialResultsPage> {
  const first = await getJson<SpatialResultsPage>(
    queryPath(simulationId, 'results', timeStep, 1),
    signal,
  )
  const pages = await Promise.all(
    Array.from({ length: Math.max(0, first.pagination.total_pages - 1) }, (_, index) =>
      getJson<SpatialResultsPage>(
        queryPath(simulationId, 'results', timeStep, index + 2),
        signal,
      ),
    ),
  )
  return { ...first, data: [...first.data, ...pages.flatMap((page) => page.data)] }
}

async function fetchAllRiskPages(
  simulationId: string,
  timeStep: number,
  signal?: AbortSignal,
): Promise<SpatialRiskPage> {
  const first = await getJson<SpatialRiskPage>(
    queryPath(simulationId, 'risk-results', timeStep, 1),
    signal,
  )
  const pages = await Promise.all(
    Array.from({ length: Math.max(0, first.pagination.total_pages - 1) }, (_, index) =>
      getJson<SpatialRiskPage>(
        queryPath(simulationId, 'risk-results', timeStep, index + 2),
        signal,
      ),
    ),
  )
  return { ...first, data: [...first.data, ...pages.flatMap((page) => page.data)] }
}

export async function fetchSpatialSimulationSummary(
  simulationId: string,
  signal?: AbortSignal,
): Promise<SpatialSimulationSummary> {
  return getJson<SpatialSimulationSummary>(
    `/api/v1/simulations/${encodeURIComponent(simulationId)}`,
    signal,
  )
}

export async function fetchSpatialSimulationFrame(
  simulationId: string,
  timeStep: number,
  signal?: AbortSignal,
): Promise<SpatialSimulationFrame> {
  const summary = await fetchSpatialSimulationSummary(simulationId, signal)
  if (summary.status !== 'completed') {
    throw new Error(`Simulation chưa hoàn tất (status: ${summary.status}).`)
  }
  if (
    !Number.isInteger(timeStep) ||
    timeStep < 0 ||
    timeStep >= summary.time_step_count
  ) {
    throw new Error('Time step nằm ngoài phạm vi của simulation.')
  }

  const [results, risks, hotspots] = await Promise.all([
    fetchAllResultPages(simulationId, timeStep, signal),
    fetchAllRiskPages(simulationId, timeStep, signal),
    getJson<SpatialHotspotsResponse>(
      queryPath(simulationId, 'hotspots', timeStep),
      signal,
    ),
  ])

  if (
    results.simulation_id !== simulationId ||
    risks.simulation_id !== simulationId ||
    hotspots.simulation_id !== simulationId ||
    results.time_step !== timeStep ||
    risks.time_step !== timeStep ||
    hotspots.time_step !== timeStep
  ) {
    throw new Error('Backend trả về dữ liệu không khớp simulation/time step đang chọn.')
  }

  const buildings = mergeSpatialBuildingData(results.data, risks.data)

  return {
    summary,
    time_step: timeStep,
    time: results.time,
    thresholds: risks.thresholds,
    buildings,
    hotspots: hotspots.hotspots,
  }
}

export function getBuildingDisplayRisk(
  building: SpatialBuildingView,
): SpatialRiskLevel {
  if (!building.result || building.risk.risk_level === 'unclassified') {
    return 'unclassified'
  }
  return building.risk.risk_level
}

/** Joins S/I/R values and risk classification by stable building UUID. */
export function mergeSpatialBuildingData(
  results: SpatialBuildingResult[],
  risks: SpatialBuildingRisk[],
): SpatialBuildingView[] {
  const resultsById = new Map(results.map((item) => [item.building_id, item]))
  return risks.map((risk) => ({
    building_id: risk.building_id,
    region_id: risk.region_id,
    result: resultsById.get(risk.building_id) ?? null,
    risk,
  }))
}
