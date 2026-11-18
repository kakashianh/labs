import assert from 'node:assert/strict'
import test from 'node:test'
import { buildSpatialInitializationPayload } from './spatial-simulation-launch.ts'

const validBuilding = {
  building_id: 'building-a',
  population: 100,
  initial_infected: 5,
  initial_recovered: 2,
}

test('builds the exact initialize API payload with request_id and manual population', () => {
  assert.deepEqual(
    buildSpatialInitializationPayload([validBuilding], 'request-1'),
    {
      request_id: 'request-1',
      buildings: [validBuilding],
    },
  )
})

test('rejects an empty building selection', () => {
  assert.throws(
    () => buildSpatialInitializationPayload([], 'request-1'),
    /Chọn ít nhất một building/,
  )
})

test('rejects missing, non-finite, negative and over-population values', () => {
  assert.throws(
    () => buildSpatialInitializationPayload([{ ...validBuilding, population: Number.NaN }], 'request-1'),
    /Nhập đầy đủ/,
  )
  assert.throws(
    () => buildSpatialInitializationPayload([{ ...validBuilding, initial_infected: -1 }], 'request-1'),
    /không hợp lệ/,
  )
  assert.throws(
    () => buildSpatialInitializationPayload([{ ...validBuilding, initial_recovered: 96 }], 'request-1'),
    /vượt quá population/,
  )
})

test('rejects duplicate building IDs and empty request IDs', () => {
  assert.throws(
    () => buildSpatialInitializationPayload([validBuilding, validBuilding], 'request-1'),
    /chọn trùng/,
  )
  assert.throws(
    () => buildSpatialInitializationPayload([validBuilding], '  '),
    /request_id/,
  )
})


test('rejects more than the backend-supported 500 buildings', () => {
  const buildings = Array.from({ length: 501 }, (_, index) => ({
    ...validBuilding,
    building_id: `building-${index}`,
  }))
  assert.throws(
    () => buildSpatialInitializationPayload(buildings, 'request-1'),
    /tối đa 500 building/,
  )
})
