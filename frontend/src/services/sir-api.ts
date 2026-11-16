import { postJson } from './api-client.ts'
import type { SirSimulationRequest, SirSimulationResult } from '../types/sir'

export function validateSirRequest(request: SirSimulationRequest): string | null {
  const values = Object.values(request)
  if (!values.every(Number.isFinite)) return 'Tất cả tham số phải là số hữu hạn.'
  if (request.population <= 0) return 'Population phải lớn hơn 0.'
  if (
    request.susceptible < 0 ||
    request.infected < 0 ||
    request.recovered < 0
  ) {
    return 'S, I và R không được âm.'
  }

  const initialTotal = request.susceptible + request.infected + request.recovered
  const tolerance = 1e-9 + 1e-9 * request.population
  if (Math.abs(initialTotal - request.population) > tolerance) {
    return 'Tổng Susceptible + Infected + Recovered phải bằng Population.'
  }

  if (request.beta < 0 || request.gamma < 0) {
    return 'Beta và Gamma phải lớn hơn hoặc bằng 0.'
  }
  if (request.dt <= 0) return 'dt phải lớn hơn 0.'
  if (!Number.isInteger(request.steps) || request.steps < 1 || request.steps > 100_000) {
    return 'Steps phải là số nguyên từ 1 đến 100.000.'
  }
  if (!Number.isFinite(request.dt * request.steps) || request.dt * request.steps > 10_000) {
    return 'Tổng thời lượng dt × steps không được vượt quá 10.000 đơn vị thời gian.'
  }

  return null
}

export function simulateSir(
  request: SirSimulationRequest,
  signal?: AbortSignal,
): Promise<SirSimulationResult> {
  return postJson<SirSimulationRequest, SirSimulationResult>(
    '/api/v1/simulations/sir',
    request,
    signal,
  )
}
