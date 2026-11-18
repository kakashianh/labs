import assert from 'node:assert/strict'
import test from 'node:test'
import {
  getBuildingDisplayRisk,
  mergeSpatialBuildingData,
  type SpatialBuildingResult,
  type SpatialBuildingRisk,
} from './spatial-simulation.ts'

const result: SpatialBuildingResult = {
  building_id: 'building-a',
  region_id: 'region-a',
  population: 100,
  susceptible: 80,
  infected: 15,
  recovered: 5,
}

function risk(
  buildingId: string,
  level: SpatialBuildingRisk['risk_level'],
  population: number | null = 100,
): SpatialBuildingRisk {
  return {
    building_id: buildingId,
    region_id: 'region-a',
    population,
    infected: population === null ? 0 : 15,
    infection_ratio: population === null ? null : 0.15,
    risk_level: level,
    unclassified_reason: population === null ? 'missing_population' : null,
  }
}

test('joins result and risk records by building UUID, not array position', () => {
  const joined = mergeSpatialBuildingData(
    [result],
    [risk('building-b', 'high'), risk('building-a', 'medium')],
  )

  assert.equal(joined.length, 2)
  assert.equal(joined[0].building_id, 'building-b')
  assert.equal(joined[0].result, null)
  assert.equal(joined[1].result?.infected, 15)
  assert.equal(joined[1].risk.risk_level, 'medium')
})

test('keeps missing-population buildings unclassified instead of treating them as Low', () => {
  const [missing] = mergeSpatialBuildingData([], [risk('building-x', 'unclassified', null)])

  assert.equal(getBuildingDisplayRisk(missing), 'unclassified')
  assert.equal(missing.result, null)
  assert.equal(missing.risk.infection_ratio, null)
})

test('preserves backend risk levels for buildings with complete SIR data', () => {
  const [high] = mergeSpatialBuildingData([result], [risk('building-a', 'high')])

  assert.equal(getBuildingDisplayRisk(high), 'high')
  assert.equal(high.result?.susceptible, 80)
  assert.equal(high.result?.recovered, 5)
})
