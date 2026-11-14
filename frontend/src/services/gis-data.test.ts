import assert from 'node:assert/strict'
import test from 'node:test'
import {
  buildingsToFeatureCollection,
  fetchAllBuildingPages,
} from './gis-data.ts'
import type { Building, BuildingListResponse } from '../types/gis.ts'

const geometry: Building['geometry'] = {
  type: 'MultiPolygon',
  coordinates: [
    [
      [
        [106.695, 10.765],
        [106.696, 10.765],
        [106.696, 10.766],
        [106.695, 10.765],
      ],
    ],
  ],
}

function building(id: string): Building {
  return {
    id,
    region_id: 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    external_id: 'way/123',
    name: null,
    building_type: 'residential',
    geometry,
  }
}

function page(
  data: Building[],
  currentPage: number,
  totalPages: number,
): BuildingListResponse {
  return {
    data,
    pagination: {
      page: currentPage,
      page_size: 2,
      total: totalPages === 0 ? 0 : (totalPages - 1) * 2 + data.length,
      total_pages: totalPages,
    },
  }
}

test('fetches every page instead of assuming the first page is complete', async () => {
  const requested: number[] = []
  const result = await fetchAllBuildingPages(async (currentPage) => {
    requested.push(currentPage)
    if (currentPage === 1) return page([building('uuid-1'), building('uuid-2')], 1, 3)
    if (currentPage === 2) return page([building('uuid-3'), building('uuid-4')], 2, 3)
    return page([building('uuid-5')], 3, 3)
  }, 2)

  assert.deepEqual(requested, [1, 2, 3])
  assert.deepEqual(result.map((item) => item.id), [
    'uuid-1',
    'uuid-2',
    'uuid-3',
    'uuid-4',
    'uuid-5',
  ])
})

test('returns an empty list when the API reports zero pages', async () => {
  let requested = 0
  const result = await fetchAllBuildingPages(async () => {
    requested += 1
    return page([], 1, 0)
  })

  assert.deepEqual(result, [])
  assert.equal(requested, 1)
})

test('supports a final partial page', async () => {
  const result = await fetchAllBuildingPages(async (currentPage) => {
    if (currentPage === 1) return page([building('uuid-1'), building('uuid-2')], 1, 2)
    return page([building('uuid-3')], 2, 2)
  }, 2)

  assert.equal(result.length, 3)
  assert.equal(result[2].id, 'uuid-3')
})

test('converts API geometries to Features and preserves UUID/properties', () => {
  const result = buildingsToFeatureCollection([building('building-uuid')])

  assert.equal(result.skippedCount, 0)
  assert.equal(result.featureCollection.type, 'FeatureCollection')
  assert.equal(result.featureCollection.features[0].id, 'building-uuid')
  assert.equal(result.featureCollection.features[0].properties.buildingId, 'building-uuid')
  assert.equal(result.featureCollection.features[0].properties.externalId, 'way/123')
  assert.equal(result.featureCollection.features[0].geometry.type, 'MultiPolygon')
})

test('skips malformed geometry without losing the remaining buildings', () => {
  const malformed = {
    ...building('bad-geometry'),
    geometry: { type: 'MultiPolygon', coordinates: [] },
  } as Building

  const result = buildingsToFeatureCollection([malformed, building('valid-uuid')])

  assert.equal(result.skippedCount, 1)
  assert.deepEqual(result.featureCollection.features.map((feature) => feature.id), [
    'valid-uuid',
  ])
})
