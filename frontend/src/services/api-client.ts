const configuredBaseUrl = import.meta.env?.VITE_API_BASE_URL?.trim()

// In development, call the same-origin Vite proxy. In production, use the
// explicitly configured API origin.
export const API_BASE_URL = import.meta.env?.DEV
  ? ''
  : (configuredBaseUrl || 'http://localhost:8000').replace(/\/+$/, '')

export class ApiRequestError extends Error {
  public readonly status: number

  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiRequestError'
    this.status = status
  }
}

async function readErrorMessage(response: Response): Promise<string> {
  let message = \`API request failed with status \${response.status}\`

  try {
    const body: unknown = await response.json()
    if (
      typeof body === 'object' &&
      body !== null &&
      'error' in body &&
      typeof body.error === 'string'
    ) {
      message = body.error
    }
  } catch {
    // Keep the status-based message when the response is not JSON.
  }

  return message
}

export async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  const response = await fetch(\`\${API_BASE_URL}\${path}\`, {
    method: 'GET',
    headers: { Accept: 'application/json' },
    signal,
  })

  if (!response.ok) {
    throw new ApiRequestError(await readErrorMessage(response), response.status)
  }

  return (await response.json()) as T
}

export async function postJson<TRequest, TResponse>(
  path: string,
  body: TRequest,
  signal?: AbortSignal,
): Promise<TResponse> {
  const response = await fetch(\`\${API_BASE_URL}\${path}\`, {
    method: 'POST',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
    signal,
  })

  if (!response.ok) {
    throw new ApiRequestError(await readErrorMessage(response), response.status)
  }

  return (await response.json()) as TResponse
}
