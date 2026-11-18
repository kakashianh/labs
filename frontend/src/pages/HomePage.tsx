import { useEffect, useRef, useState, type FormEvent } from 'react'
import CesiumViewer from '../components/map/CesiumViewer'
import SirResultPanel from '../components/simulation/SirResultPanel'
import { isSirSimulationResult } from '../services/sir-result'
import { simulateSir, validateSirRequest } from '../services/sir-api'
import type { SirSimulationRequest, SirSimulationResult } from '../types/sir'
import '../styles/sir-result.css'

type SirFormValues = Record<keyof SirSimulationRequest, string>

const INITIAL_VALUES: SirFormValues = {
  population: '1000',
  susceptible: '990',
  infected: '10',
  recovered: '0',
  beta: '0.3',
  gamma: '0.1',
  dt: '0.1',
  steps: '100',
}

const FIELDS: {
  key: keyof SirSimulationRequest
  label: string
  min: number
  step: number | 'any'
  max?: number
}[] = [
  { key: 'population', label: 'Population', min: 0.000001, step: 'any' },
  { key: 'susceptible', label: 'Susceptible (S)', min: 0, step: 'any' },
  { key: 'infected', label: 'Infected (I)', min: 0, step: 'any' },
  { key: 'recovered', label: 'Recovered (R)', min: 0, step: 'any' },
  { key: 'beta', label: 'Beta (β)', min: 0, step: 'any' },
  { key: 'gamma', label: 'Gamma (γ)', min: 0, step: 'any' },
  { key: 'dt', label: 'Time step (dt)', min: 0.000001, step: 'any' },
  { key: 'steps', label: 'Số bước (steps)', min: 1, max: 100000, step: 1 },
]

export default function HomePage() {
  const [configCollapsed, setConfigCollapsed] = useState(false)
  const [formValues, setFormValues] = useState<SirFormValues>(INITIAL_VALUES)
  const [result, setResult] = useState<SirSimulationResult | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const requestController = useRef<AbortController | null>(null)

  useEffect(() => () => requestController.current?.abort(), [])

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    const request = Object.fromEntries(
      Object.entries(formValues).map(([key, value]) => [key, Number(value)]),
    ) as unknown as SirSimulationRequest

    const validationError = validateSirRequest(request)
    if (validationError) {
      requestController.current?.abort()
      setResult(null)
      setLoading(false)
      setError(validationError)
      return
    }

    requestController.current?.abort()
    const controller = new AbortController()
    requestController.current = controller
    setResult(null)
    setError(null)
    setLoading(true)

    void simulateSir(request, controller.signal)
      .then((response) => {
        if (
          !isSirSimulationResult(response) ||
          response.population !== request.population ||
          response.point_count !== request.steps + 1
        ) {
          throw new Error('Backend trả về kết quả SIR không khớp request hoặc sai contract.')
        }
        setResult(response)
      })
      .catch((requestError: unknown) => {
        if (controller.signal.aborted) return
        setError(
          requestError instanceof Error
            ? requestError.message
            : 'Không thể chạy mô phỏng SIR.',
        )
      })
      .finally(() => {
        if (!controller.signal.aborted) {
          setLoading(false)
          requestController.current = null
        }
      })
  }

  return (
    <main className="home-page">
      <CesiumViewer />

      <form className={`sir-config-panel${configCollapsed ? " is-collapsed" : ""}`} onSubmit={handleSubmit}>
        <header className="sir-config-header">
          <div>
            <p className="sir-eyebrow">SIMULATION SETUP</p>
            <h2>Cấu hình mô phỏng SIR</h2>
          </div>
          <div className="sir-config-header-actions">
            <span className="sir-config-method">SIR · Euler</span>
            <button type="button" className="sir-panel-toggle" aria-expanded={!configCollapsed} aria-label={configCollapsed ? 'Hiện cấu hình mô phỏng SIR' : 'Ẩn cấu hình mô phỏng SIR'} onClick={() => setConfigCollapsed((value) => !value)}>
              {configCollapsed ? 'Hiện' : 'Ẩn'}
            </button>
          </div>
        </header>

        {!configCollapsed && <div className="sir-config-body">
        <div className="sir-config-fields">
          {FIELDS.map((field) => (
            <label className="sir-config-field" key={field.key} htmlFor={`sir-${field.key}`}>
              <span>{field.label}</span>
              <input
                id={`sir-${field.key}`}
                name={field.key}
                type="number"
                min={field.min}
                max={field.max}
                step={field.step}
                value={formValues[field.key]}
                required
                onChange={(event) =>
                  setFormValues((current) => ({
                    ...current,
                    [field.key]: event.currentTarget.value,
                  }))
                }
              />
            </label>
          ))}
        </div>

        <p className="sir-config-hint">
          S + I + R phải bằng Population. Beta, Gamma và dt dùng cùng một đơn vị thời gian.
        </p>
        {error && (
          <p className="sir-config-error" role="alert">{error}</p>
        )}
        <button className="sir-config-submit" type="submit" disabled={loading}>
          {loading ? 'Đang mô phỏng…' : 'Chạy mô phỏng'}
        </button>
        </div>}
      </form>

      <SirResultPanel result={result} loading={loading} error={error} />
    </main>
  )
}
