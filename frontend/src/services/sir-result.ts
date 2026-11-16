import type { SirSimulationResult } from '../types/sir'

export function isSirSimulationResult(value: unknown): value is SirSimulationResult {
  if (typeof value !== 'object' || value === null) return false

  const candidate = value as Partial<SirSimulationResult>
  if (
    typeof candidate.population !== 'number' ||
    !Number.isFinite(candidate.population) ||
    candidate.population <= 0 ||
    !Number.isInteger(candidate.point_count) ||
    !Array.isArray(candidate.points) ||
    candidate.point_count !== candidate.points.length ||
    candidate.points.length === 0
  ) {
    return false
  }

  return candidate.points.every((point, index) => {
    if (typeof point !== 'object' || point === null) return false

    return (
      Number.isInteger(point.time_step) &&
      point.time_step === index &&
      Number.isFinite(point.time) &&
      point.time >= 0 &&
      Number.isFinite(point.susceptible) &&
      point.susceptible >= 0 &&
      Number.isFinite(point.infected) &&
      point.infected >= 0 &&
      Number.isFinite(point.recovered) &&
      point.recovered >= 0
    )
  })
}

export function formatSirValue(value: number): string {
  return new Intl.NumberFormat('vi-VN', {
    maximumFractionDigits: 2,
  }).format(value)
}
