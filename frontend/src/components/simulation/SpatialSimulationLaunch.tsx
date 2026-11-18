import { useEffect, useState } from 'react'
import {
  initializeAndRunSpatialSimulation,
  type SpatialLaunchBuildingInput,
  type SpatialSirRunConfig,
} from '../../services/spatial-simulation-launch.ts'

interface SpatialSimulationLaunchProps {
  selectedBuildingIds: string[]
  selectionMode: boolean
  onSelectionModeChange: (enabled: boolean) => void
  onClearSelection: () => void
  onRunCreated: (simulationId: string) => Promise<void>
  disabled?: boolean
}

interface BuildingDraft {
  population: string
  initial_infected: string
  initial_recovered: string
}

const EMPTY_DRAFT: BuildingDraft = {
  population: '',
  initial_infected: '0',
  initial_recovered: '0',
}

export default function SpatialSimulationLaunch({
  selectedBuildingIds,
  selectionMode,
  onSelectionModeChange,
  onClearSelection,
  onRunCreated,
  disabled = false,
}: SpatialSimulationLaunchProps) {
  const [drafts, setDrafts] = useState<Record<string, BuildingDraft>>({})
  const [config, setConfig] = useState<SpatialSirRunConfig>({
    radius_m: 100,
    beta: 0.3,
    gamma: 0.1,
    dt: 0.1,
    steps: 10,
  })
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [createdId, setCreatedId] = useState<string | null>(null)

  useEffect(() => {
    setDrafts((current) => {
      const next: Record<string, BuildingDraft> = {}
      for (const id of selectedBuildingIds) next[id] = current[id] ?? { ...EMPTY_DRAFT }
      return next
    })
  }, [selectedBuildingIds])

  const updateBuilding = (id: string, field: keyof BuildingDraft, value: string) => {
    setDrafts((current) => ({
      ...current,
      [id]: { ...(current[id] ?? EMPTY_DRAFT), [field]: value },
    }))
  }

  const updateConfig = (field: keyof SpatialSirRunConfig, value: string) => {
    setConfig((current) => ({
      ...current,
      [field]: value === '' ? Number.NaN : Number(value),
    }))
  }

  const handleCreate = async () => {
    setError(null)
    setCreatedId(null)

    const buildings: SpatialLaunchBuildingInput[] = selectedBuildingIds.map((building_id) => {
      const draft = drafts[building_id] ?? EMPTY_DRAFT
      return {
        building_id,
        population: draft.population.trim() === '' ? Number.NaN : Number(draft.population),
        initial_infected: draft.initial_infected.trim() === '' ? Number.NaN : Number(draft.initial_infected),
        initial_recovered: draft.initial_recovered.trim() === '' ? Number.NaN : Number(draft.initial_recovered),
      }
    })

    if (
      !Number.isFinite(config.radius_m) || config.radius_m <= 0 || config.radius_m > 5000 ||
      !Number.isFinite(config.beta) || config.beta < 0 ||
      !Number.isFinite(config.gamma) || config.gamma < 0 ||
      !Number.isFinite(config.dt) || config.dt <= 0 ||
      !Number.isInteger(config.steps) || config.steps < 1
    ) {
      setError('Tham số SIR không hợp lệ: radius 0–5000m, beta/gamma không âm, dt > 0 và steps là số nguyên dương.')
      return
    }

    setSubmitting(true)
    try {
      const { initialized } = await initializeAndRunSpatialSimulation(buildings, config)
      setCreatedId(initialized.simulation_id)
      await onRunCreated(initialized.simulation_id)
      onSelectionModeChange(false)
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : 'Không thể khởi tạo và chạy spatial simulation.')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <section className="spatial-launch" aria-label="Khởi tạo spatial SIR mới">
      <h3>Tạo mô phỏng mới</h3>
      <p>Chọn building trực tiếp trên bản đồ, sau đó nhập dân số và trạng thái S/I/R ban đầu.</p>
      <div className="spatial-launch-actions">
        <button
          type="button"
          className={selectionMode ? 'is-active' : ''}
          onClick={() => onSelectionModeChange(!selectionMode)}
          disabled={disabled || submitting}
        >
          {selectionMode ? 'Đang chọn trên bản đồ · Xong' : 'Chọn building trên bản đồ'}
        </button>
        <button
          type="button"
          className="spatial-launch-secondary"
          onClick={onClearSelection}
          disabled={disabled || submitting || selectedBuildingIds.length === 0}
        >
          Bỏ chọn
        </button>
      </div>
      {selectionMode && <small className="spatial-launch-hint">Click các footprint trên bản đồ để thêm/bỏ building khỏi danh sách.</small>}
      <div className="spatial-launch-count">{selectedBuildingIds.length} building được chọn</div>

      {selectedBuildingIds.map((id) => {
        const draft = drafts[id] ?? EMPTY_DRAFT
        return (
          <fieldset className="spatial-launch-building" key={id}>
            <legend>{id}</legend>
            <label>
              Population
              <input
                type="number"
                min="1"
                step="any"
                value={draft.population}
                onChange={(event) => updateBuilding(id, 'population', event.currentTarget.value)}
                placeholder="Nhập dân số"
                disabled={submitting}
              />
            </label>
            <label>
              Initial infected
              <input
                type="number"
                min="0"
                step="any"
                value={draft.initial_infected}
                onChange={(event) => updateBuilding(id, 'initial_infected', event.currentTarget.value)}
                disabled={submitting}
              />
            </label>
            <label>
              Initial recovered
              <input
                type="number"
                min="0"
                step="any"
                value={draft.initial_recovered}
                onChange={(event) => updateBuilding(id, 'initial_recovered', event.currentTarget.value)}
                disabled={submitting}
              />
            </label>
          </fieldset>
        )
      })}

      <div className="spatial-launch-config">
        <label>Radius (m)<input type="number" min="0.01" max="5000" step="any" value={config.radius_m} onChange={(event) => updateConfig('radius_m', event.currentTarget.value)} disabled={submitting} /></label>
        <label>Beta<input type="number" min="0" step="any" value={config.beta} onChange={(event) => updateConfig('beta', event.currentTarget.value)} disabled={submitting} /></label>
        <label>Gamma<input type="number" min="0" step="any" value={config.gamma} onChange={(event) => updateConfig('gamma', event.currentTarget.value)} disabled={submitting} /></label>
        <label>dt<input type="number" min="0.0001" step="any" value={config.dt} onChange={(event) => updateConfig('dt', event.currentTarget.value)} disabled={submitting} /></label>
        <label>Steps<input type="number" min="1" step="1" value={config.steps} onChange={(event) => updateConfig('steps', event.currentTarget.value)} disabled={submitting} /></label>
      </div>

      <button
        type="button"
        className="spatial-launch-submit"
        onClick={() => void handleCreate()}
        disabled={disabled || submitting || selectedBuildingIds.length === 0}
      >
        {submitting ? 'Đang khởi tạo và chạy…' : 'Khởi tạo & chạy mô phỏng'}
      </button>
      {error && <p className="spatial-error" role="alert">{error}</p>}
      {createdId && <p className="spatial-launch-success">Run đã tạo: <code>{createdId}</code></p>}
    </section>
  )
}
