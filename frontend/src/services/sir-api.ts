import { postJson } from './api-client.ts'
import type { SirSimulationRequest, SirSimulationResult } from '../types/sir'

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
