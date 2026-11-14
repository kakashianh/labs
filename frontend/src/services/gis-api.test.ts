import assert from 'node:assert/strict'
import test from 'node:test'
import { ApiRequestError } from './api-client.ts'
import { fetchBuildingDetail } from './gis-api.ts'

const sampleBuilding = {
  id: '11111111-1111-4111-8111-111111111111',
  region_id: '22222222-2222-4222-8222-222222222222',
  external_id: 'way/123',
  name: null,
  building_type: null,
  geometry: { type: 'MultiPolygon' as const, coordinates: [] },
}

test('fetchBuildingDetail unwraps the data envelope and preserves nullable fields', async () => {
  const originalFetch = globalThis.fetch
  let requestedUrl = ''
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    requestedUrl = String(input)
    return new Response(JSON.stringify({ data: sampleBuilding }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    })
  }) as typeof fetch

  try {
    const result = await fetchBuildingDetail(sampleBuilding.id)
    assert.ok(requestedUrl.endsWith(`/api/v1/buildings/${sampleBuilding.id}`))
    assert.deepEqual(result, sampleBuilding)
    assert.equal(result.name, null)
    assert.equal(result.building_type, null)
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('fetchBuildingDetail surfaces 400 invalid UUID responses', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = (async () =>
    new Response(JSON.stringify({ error: 'Invalid UUID' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    })) as typeof fetch

  try {
    await assert.rejects(
      fetchBuildingDetail('not-a-uuid'),
      (error: unknown) => error instanceof ApiRequestError && error.status === 400,
    )
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('fetchBuildingDetail surfaces 404 when the building does not exist', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = (async () =>
    new Response(JSON.stringify({ error: 'Building not found' }), {
      status: 404,
      headers: { 'Content-Type': 'application/json' },
    })) as typeof fetch

  try {
    await assert.rejects(
      fetchBuildingDetail(sampleBuilding.id),
      (error: unknown) => error instanceof ApiRequestError && error.status === 404,
    )
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('fetchBuildingDetail propagates network errors', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = (async () => {
    throw new TypeError('Network unavailable')
  }) as typeof fetch

  try {
    await assert.rejects(fetchBuildingDetail(sampleBuilding.id), /Network unavailable/)
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('fetchBuildingDetail forwards AbortSignal and rejects after cancellation', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = (async (_input: URL | RequestInfo, init?: RequestInit) =>
    new Promise<Response>((_resolve, reject) => {
      init?.signal?.addEventListener('abort', () => {
        reject(new DOMException('The operation was aborted', 'AbortError'))
      })
    })) as typeof fetch

  const controller = new AbortController()
  try {
    const request = fetchBuildingDetail(sampleBuilding.id, controller.signal)
    controller.abort()
    await assert.rejects(request, (error: unknown) =>
      error instanceof Error && error.name === 'AbortError',
    )
  } finally {
    globalThis.fetch = originalFetch
  }
})
