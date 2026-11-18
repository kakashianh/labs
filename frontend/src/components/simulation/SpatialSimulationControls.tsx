import { useEffect, useRef, useState } from 'react'
import SpatialSimulationLaunch from './SpatialSimulationLaunch'
import {
  fetchSpatialSimulationFrame,
  fetchSpatialSimulationSummary,
  type SpatialSimulationFrame,
  type SpatialSimulationSummary,
} from '../../services/spatial-simulation.ts'

export type SpatialColorMetric = 'risk_level' | 'infection_ratio'

interface SpatialSimulationControlsProps {
  selectedBuildingIds: string[]
  selectionMode: boolean
  onSelectionModeChange: (enabled: boolean) => void
  onClearSelection: () => void
  onFrameChange: (frame: SpatialSimulationFrame | null) => void
  onMetricChange: (metric: SpatialColorMetric) => void
  onLayerVisibilityChange: (buildingVisible: boolean, hotspotVisible: boolean) => void
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat('vi-VN', { maximumFractionDigits: 4 }).format(value)
}

export default function SpatialSimulationControls({
  selectedBuildingIds,
  selectionMode,
  onSelectionModeChange,
  onClearSelection,
  onFrameChange,
  onMetricChange,
  onLayerVisibilityChange,
}: SpatialSimulationControlsProps) {
  const [collapsed, setCollapsed] = useState(false)
  const [simulationId, setSimulationId] = useState('')
  const [summary, setSummary] = useState<SpatialSimulationSummary | null>(null)
  const [frame, setFrame] = useState<SpatialSimulationFrame | null>(null)
  const [timeStep, setTimeStep] = useState(0)
  const [timeValue, setTimeValue] = useState<number | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [metric, setMetric] = useState<SpatialColorMetric>('risk_level')
  const [buildingVisible, setBuildingVisible] = useState(false)
  const [hotspotVisible, setHotspotVisible] = useState(false)
  const requestController = useRef<AbortController | null>(null)
  const requestVersion = useRef(0)

  useEffect(() => () => {
    requestController.current?.abort()
    requestVersion.current += 1
  }, [])

  const loadStep = async (id: string, step: number) => {
    requestController.current?.abort()
    const controller = new AbortController()
    requestController.current = controller
    const version = ++requestVersion.current
    setLoading(true)
    setError(null)

    try {
      const frame = await fetchSpatialSimulationFrame(id, step, controller.signal)
      if (controller.signal.aborted || version !== requestVersion.current) return
      setSummary(frame.summary)
      setTimeStep(step)
      setTimeValue(frame.time)
      setFrame(frame)
      onFrameChange(frame)
    } catch (requestError) {
      if (controller.signal.aborted || version !== requestVersion.current) return
      setError(requestError instanceof Error ? requestError.message : 'Không thể tải spatial simulation.')
    } finally {
      if (!controller.signal.aborted && version === requestVersion.current) setLoading(false)
    }
  }

  const handleLoad = async (simulationIdOverride?: string) => {
    const id = simulationIdOverride ?? simulationId.trim()
    if (simulationIdOverride) setSimulationId(simulationIdOverride)
    if (!id) {
      setError('Nhập simulation ID được trả về từ luồng khởi tạo mô phỏng.')
      return
    }

    requestController.current?.abort()
    const controller = new AbortController()
    requestController.current = controller
    const version = ++requestVersion.current
    setLoading(true)
    setError(null)
    setSummary(null)
    setTimeValue(null)
    setFrame(null)
    setBuildingVisible(false)
    setHotspotVisible(false)
    onFrameChange(null)
    onLayerVisibilityChange(false, false)

    try {
      const nextSummary = await fetchSpatialSimulationSummary(id, controller.signal)
      if (controller.signal.aborted || version !== requestVersion.current) return
      if (nextSummary.status !== 'completed') {
        throw new Error(`Simulation chưa hoàn tất (status: ${nextSummary.status}).`)
      }
      if (nextSummary.time_step_count < 1) {
        throw new Error('Simulation không có time step để hiển thị.')
      }
      const frame = await fetchSpatialSimulationFrame(id, 0, controller.signal)
      if (controller.signal.aborted || version !== requestVersion.current) return
      setSummary(nextSummary)
      setTimeStep(0)
      setTimeValue(frame.time)
      setFrame(frame)
      setBuildingVisible(true)
      setHotspotVisible(true)
      onFrameChange(frame)
      onLayerVisibilityChange(true, true)
    } catch (requestError) {
      if (controller.signal.aborted || version !== requestVersion.current) return
      setError(requestError instanceof Error ? requestError.message : 'Không thể tải spatial simulation.')
    } finally {
      if (!controller.signal.aborted && version === requestVersion.current) setLoading(false)
    }
  }

  const handleStepChange = (step: number) => {
    void loadStep(simulationId.trim(), step)
  }

  const handleMetricChange = (value: SpatialColorMetric) => {
    setMetric(value)
    onMetricChange(value)
  }

  const handleBuildingVisibility = (checked: boolean) => {
    setBuildingVisible(checked)
    onLayerVisibilityChange(checked, hotspotVisible)
  }

  const handleHotspotVisibility = (checked: boolean) => {
    setHotspotVisible(checked)
    onLayerVisibilityChange(buildingVisible, checked)
  }

  return (
    <section className={`spatial-simulation-panel${collapsed ? " is-collapsed" : ""}`} aria-label="Spatial SIR visualization">
      <header className="spatial-panel-header">
        <div>
          <p className="spatial-eyebrow">PHASE 2 · KAN-28</p>
          <h2>Spatial SIR · 3D</h2>
        </div>
        <div className="spatial-panel-header-actions">
          <span className="spatial-status">{summary ? summary.status : 'Chưa chọn run'}</span>
          <button type="button" className="spatial-panel-toggle" aria-expanded={!collapsed} aria-label={collapsed ? 'Hiện Spatial SIR' : 'Ẩn Spatial SIR'} onClick={() => setCollapsed((value) => !value)}>
            {collapsed ? 'Hiện' : 'Ẩn'}
          </button>
        </div>
      </header>

      {!collapsed && <div className="spatial-panel-body">
      <SpatialSimulationLaunch
        selectedBuildingIds={selectedBuildingIds}
        selectionMode={selectionMode}
        onSelectionModeChange={onSelectionModeChange}
        onClearSelection={onClearSelection}
        disabled={loading}
        onRunCreated={async (id) => { await handleLoad(id) }}
      />

      <label className="spatial-field">
        <span>Simulation ID</span>
        <div className="spatial-load-row">
          <input
            value={simulationId}
            onChange={(event) => setSimulationId(event.currentTarget.value)}
            placeholder="UUID từ bước initialize"
            aria-label="Simulation ID"
            spellCheck={false}
          />
          <button type="button" onClick={() => void handleLoad()} disabled={loading}>
            {loading ? 'Đang tải…' : 'Tải run'}
          </button>
        </div>
      </label>

      {error && <p className="spatial-error" role="alert">{error}</p>}

      {summary && (
        <>
          <div className="spatial-summary">
            <span>{summary.eligible_building_count} eligible</span>
            <span>{summary.excluded_building_count} excluded</span>
            <span>{summary.time_step_count} steps</span>
            {frame && <span>Population {formatNumber(frame.aggregate.population)}</span>}
            {frame && <span>Infected {formatNumber(frame.aggregate.infected)}</span>}
            {frame && <span>Ratio {frame.aggregate.population > 0 ? formatNumber((frame.aggregate.infected / frame.aggregate.population) * 100) : '—'}%</span>}
          </div>
          {frame && (
            <div className="spatial-risk-counts" aria-label="Số lượng công trình theo mức rủi ro">
              {(['low', 'medium', 'high', 'unclassified'] as const).map((level) => {
                const count = frame.riskAggregate.data.reduce(
                  (total, region) => total + (region.buckets.find((bucket) => bucket.risk_level === level)?.building_count ?? 0),
                  0,
                )
                return <span key={level}>{level}: <strong>{count}</strong></span>
              })}
            </div>
          )}

          <label className="spatial-field spatial-timeline">
            <span>Time step <strong>{timeStep} / {summary.time_step_count - 1}</strong></span>
            <input
              type="range"
              min={0}
              max={Math.max(0, summary.time_step_count - 1)}
              step={1}
              value={timeStep}
              disabled={loading || summary.time_step_count <= 1}
              onChange={(event) => handleStepChange(Number(event.currentTarget.value))}
            />
            <small>{timeValue === null ? '—' : `t = ${formatNumber(timeValue)} (đơn vị mô phỏng)`}</small>
          </label>

          <label className="spatial-field">
            <span>Chế độ tô màu</span>
            <select
              value={metric}
              onChange={(event) => handleMetricChange(event.currentTarget.value as SpatialColorMetric)}
            >
              <option value="risk_level">Risk level (mặc định)</option>
              <option value="infection_ratio">Infection ratio</option>
            </select>
          </label>


          <section className="spatial-risk-legend" aria-label="Chú giải bản đồ">
            <h3>{metric === 'risk_level' ? 'Risk level · threshold snapshot' : 'Infection ratio · 0% → 100%'}</h3>
            {metric === 'risk_level' ? (
              <>
                <div className="spatial-risk-legend-item"><i className="spatial-risk-swatch" style={{ background: '#22c55e' }} /> Low ≤ {formatNumber((frame?.thresholds.risk_low_max ?? 0) * 100)}%</div>
                <div className="spatial-risk-legend-item"><i className="spatial-risk-swatch" style={{ background: '#f59e0b' }} /> Medium &gt; {formatNumber((frame?.thresholds.risk_low_max ?? 0) * 100)}% and ≤ {formatNumber((frame?.thresholds.risk_medium_max ?? 0) * 100)}%</div>
                <div className="spatial-risk-legend-item"><i className="spatial-risk-swatch" style={{ background: '#ef4444' }} /> High above medium threshold</div>
                <div className="spatial-risk-legend-item"><i className="spatial-risk-swatch" style={{ background: '#94a3b8' }} /> Unclassified / no data</div>
                {frame && <small>Source: {frame.thresholds.source} · {frame.thresholds.version}</small>}
              </>
            ) : (
              <>
                <div className="spatial-ratio-gradient" />
                <div className="spatial-risk-legend-item">Blue = lower ratio · Red = higher ratio</div>
                <div className="spatial-risk-legend-item"><i className="spatial-risk-swatch" style={{ background: '#94a3b8' }} /> Unclassified / no data</div>
              </>
            )}
          </section>

          {frame && frame.hotspots.length === 0 && <p className="spatial-loading">Không có hotspot tại time step này.</p>}

          {loading && <p className="spatial-loading" aria-live="polite">Đang tải đầy đủ kết quả và hotspot cho step {timeStep}…</p>}
        </>
      )}

      <section className="spatial-layer-toggles" aria-label="Điều khiển lớp bản đồ">
        <h3>Lớp bản đồ</h3>
        <label>
          <input
            type="checkbox"
            checked={buildingVisible}
            disabled={!summary}
            onChange={(event) => handleBuildingVisibility(event.currentTarget.checked)}
          />
          Building thematic layer
        </label>
        <label>
          <input
            type="checkbox"
            checked={hotspotVisible}
            disabled={!summary}
            onChange={(event) => handleHotspotVisibility(event.currentTarget.checked)}
          />
          Hotspot layer
        </label>
        {!summary && <small>Tải một simulation đã completed để bật/tắt các lớp dữ liệu.</small>}
      </section>
      </div>}
    </section>
  )
}
