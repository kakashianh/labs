import { postJson } from './api-client.ts'

export interface SpatialLaunchBuildingInput {
  building_id: string
  population: number
  initial_infected: number
  initial_recovered: number
}

export interface SpatialSirRunConfig {
  radius_m: number
  beta: number
  gamma: number
  dt: number
  steps: number
}

export interface SpatialInitializationResponse {
  simulation_id: string
  status: string
  population_source: string
  idempotent_replay: boolean
  eligible: Array<{
    building_id: string
    population: number
    initial_s: number
    initial_i: number
    initial_r: number
  }>
  excluded: Array<{ building_id: string; reason_code: string }>
}

export interface SpatialRunResponse {
  simulation_id: string
  status: string
  building_count: number
  point_count: number
  time_step_count: number
  excluded_invalid_geometry_count: number
  idempotent_replay: boolean
}

export function buildSpatialInitializationPayload(
  buildings: SpatialLaunchBuildingInput[],
  requestId: string,
) {
  if (!requestId.trim()) throw new Error('request_id không được để trống.')
  if (buildings.length === 0) throw new Error('Chọn ít nhất một building trên bản đồ.')
  if (buildings.length > 500) throw new Error('Spatial SIR hiện hỗ trợ tối đa 500 building trong một run.')

  const seen = new Set<string>()
  const normalized = buildings.map((building) => {
    if (seen.has(building.building_id)) {
      throw new Error(`Building bị chọn trùng: ${building.building_id}`)
    }
    seen.add(building.building_id)

    const { population, initial_infected, initial_recovered } = building
    if (![population, initial_infected, initial_recovered].every(Number.isFinite)) {
      throw new Error(`Nhập đầy đủ dân số và S/I/R ban đầu cho building ${building.building_id}.`)
    }
    if (population <= 0 || initial_infected < 0 || initial_recovered < 0) {
      throw new Error(`Dữ liệu dân số và ca bệnh không hợp lệ tại building ${building.building_id}.`)
    }
    if (initial_infected + initial_recovered > population) {
      throw new Error(`Initial infected + recovered vượt quá population tại building ${building.building_id}.`)
    }

    return {
      building_id: building.building_id,
      population,
      initial_infected,
      initial_recovered,
    }
  })

  return { request_id: requestId, buildings: normalized }
}

export async function initializeAndRunSpatialSimulation(
  buildings: SpatialLaunchBuildingInput[],
  config: SpatialSirRunConfig,
  signal?: AbortSignal,
): Promise<{ initialized: SpatialInitializationResponse; run: SpatialRunResponse }> {
  const payload = buildSpatialInitializationPayload(buildings, crypto.randomUUID())
  const initialized = await postJson<
    typeof payload,
    SpatialInitializationResponse
  >('/api/v1/simulations/initialize', payload, signal)

  const run = await postJson<SpatialSirRunConfig, SpatialRunResponse>(
    `/api/v1/simulations/${encodeURIComponent(initialized.simulation_id)}/run`,
    config,
    signal,
  )

  if (run.simulation_id !== initialized.simulation_id) {
    throw new Error('Backend trả về simulation_id không khớp giữa initialize và run.')
  }
  if (run.status !== 'completed') {
    throw new Error(`Simulation chưa hoàn tất (status: ${run.status}).`)
  }

  return { initialized, run }
}
