import assert from 'node:assert/strict'
import test from 'node:test'
import { ApiRequestError } from './api-client.ts'
import { simulateSir, validateSirRequest } from './sir-api.ts'
import type { SirSimulationRequest, SirSimulationResult } from '../types/sir.ts'

const request: SirSimulationRequest = {
  population: 1000,
  susceptible: 990,
  infected: 10,
  recovered: 0,
  beta: 0.3,
  gamma: 0.1,
  dt: 0.1,
  steps: 2,
}

const responseBody: SirSimulationResult = {
  population: 1000,
  point_count: 3,
  points: [
    { time_step: 0, time: 0, susceptible: 990, infected: 10, recovered: 0 },
    { time_step: 1, time: 0.1, susceptible: 989.7, infected: 10.2, recovered: 0.1 },
    { time_step: 2, time: 0.2, susceptible: 989.39, infected: 10.4, recovered: 0.21 },
  ],
}

test('simulateSir posts the exact request and returns the API response', async () => {
  const originalFetch = globalThis.fetch
  let requestedUrl = ''
  let requestInit: RequestInit | undefined
  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    requestedUrl = String(input)
    requestInit = init
    return new Response(JSON.stringify(responseBody), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    })
  }) as typeof fetch

  try {
    const result = await simulateSir(request)
    assert.ok(requestedUrl.endsWith('/api/v1/simulations/sir'))
    assert.equal(requestInit?.method, 'POST')
    assert.equal(new Headers(requestInit?.headers).get('content-type'), 'application/json')
    assert.deepEqual(JSON.parse(String(requestInit?.body)), request)
    assert.deepEqual(result, responseBody)
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('simulateSir surfaces Backend validation errors', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = (async () =>
    new Response(JSON.stringify({ error: 'invalid SIR parameters' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    })) as typeof fetch

  try {
    await assert.rejects(
      simulateSir(request),
      (error: unknown) =>
        error instanceof ApiRequestError &&
        error.status === 400 &&
        error.message === 'invalid SIR parameters',
    )
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('simulateSir forwards request cancellation', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = (async (_input: URL | RequestInfo, init?: RequestInit) =>
    new Promise<Response>((_resolve, reject) => {
      init?.signal?.addEventListener('abort', () => {
        reject(new DOMException('The operation was aborted', 'AbortError'))
      })
    })) as typeof fetch

  const controller = new AbortController()
  try {
    const pending = simulateSir(request, controller.signal)
    controller.abort()
    await assert.rejects(
      pending,
      (error: unknown) => error instanceof Error && error.name === 'AbortError',
    )
  } finally {
    globalThis.fetch = originalFetch
  }
})


test('validateSirRequest accepts valid parameters', () => {
  assert.equal(validateSirRequest(request), null)
})

test('validateSirRequest rejects a mismatched initial population', () => {
  const message = validateSirRequest({ ...request, susceptible: 989 })
  assert.ok(message)
  assert.match(message, /tổng Susceptible/i)
})

test('validateSirRequest rejects invalid rates, time steps and duration', () => {
  const invalidRequests = [
    [{ ...request, beta: -0.1 }, /Beta/],
    [{ ...request, dt: 0 }, /dt/],
    [{ ...request, steps: 1.5 }, /Steps/],
    [{ ...request, dt: 101, steps: 100 }, /10.000/],
  ] as const

  for (const [invalidRequest, expectedMessage] of invalidRequests) {
    const message = validateSirRequest(invalidRequest)
    assert.ok(message)
    assert.match(message, expectedMessage)
  }
})
