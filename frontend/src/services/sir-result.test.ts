import assert from 'node:assert/strict'
import test from 'node:test'
import { formatSirValue, isSirSimulationResult } from './sir-result.ts'

const validResult = {
  population: 1000,
  point_count: 2,
  points: [
    { time_step: 0, time: 0, susceptible: 990, infected: 10, recovered: 0 },
    { time_step: 1, time: 0.1, susceptible: 989.7, infected: 10.2, recovered: 0.1 },
  ],
}

test('accepts a valid SIR API response', () => {
  assert.equal(isSirSimulationResult(validResult), true)
})

test('rejects point_count that differs from points length', () => {
  assert.equal(isSirSimulationResult({ ...validResult, point_count: 3 }), false)
})

test('rejects non-sequential time_step values', () => {
  const points = [...validResult.points]
  points[1] = { ...points[1], time_step: 3 }
  assert.equal(isSirSimulationResult({ ...validResult, points }), false)
})

test('rejects non-finite or negative compartment values', () => {
  const points = [...validResult.points]
  points[1] = { ...points[1], infected: Number.NaN }
  assert.equal(isSirSimulationResult({ ...validResult, points }), false)

  points[1] = { ...points[1], infected: -1 }
  assert.equal(isSirSimulationResult({ ...validResult, points }), false)
})

test('rejects empty and malformed responses', () => {
  assert.equal(isSirSimulationResult(null), false)
  assert.equal(isSirSimulationResult({ ...validResult, points: [], point_count: 0 }), false)
  assert.equal(isSirSimulationResult({ ...validResult, population: 0 }), false)
})

test('formats values only for display', () => {
  assert.equal(formatSirValue(1234.567), '1.234,57')
})
