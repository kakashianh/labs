export interface SirSimulationRequest {
  population: number
  susceptible: number
  infected: number
  recovered: number
  beta: number
  gamma: number
  dt: number
  steps: number
}

export interface SirPoint {
  time_step: number
  time: number
  susceptible: number
  infected: number
  recovered: number
}

export interface SirSimulationResult {
  population: number
  point_count: number
  points: SirPoint[]
}

export interface SirResultPanelProps {
  result: SirSimulationResult | null
  loading: boolean
  error: string | null
}
