import { useEffect, useMemo, useState } from 'react'
import type { SirPoint, SirResultPanelProps } from '../../types/sir'
import { formatSirValue, isSirSimulationResult } from '../../services/sir-result'

const CHART_WIDTH = 760
const CHART_HEIGHT = 250
const CHART_PADDING = { top: 18, right: 18, bottom: 34, left: 48 }

type SeriesKey = 'susceptible' | 'infected' | 'recovered'

const SERIES: { key: SeriesKey; label: string; color: string; dash?: string }[] = [
  { key: 'susceptible', label: 'Susceptible (S)', color: '#38bdf8' },
  { key: 'infected', label: 'Infected (I)', color: '#fb7185', dash: '7 5' },
  { key: 'recovered', label: 'Recovered (R)', color: '#34d399', dash: '3 4' },
]

function SirResultChart({
  points,
  population,
  selectedTimeStep,
  onSelect,
}: {
  points: SirPoint[]
  population: number
  selectedTimeStep: number
  onSelect: (timeStep: number) => void
}) {
  const plotWidth = CHART_WIDTH - CHART_PADDING.left - CHART_PADDING.right
  const plotHeight = CHART_HEIGHT - CHART_PADDING.top - CHART_PADDING.bottom
  const maxTimeStep = Math.max(points.length - 1, 1)
  const yMaximum = Math.max(population, 1)

  const x = (timeStep: number) =>
    CHART_PADDING.left + (timeStep / maxTimeStep) * plotWidth
  const y = (value: number) =>
    CHART_PADDING.top + plotHeight - (value / yMaximum) * plotHeight

  const selected = points[selectedTimeStep]

  return (
    <div className="sir-chart-scroll">
      <svg
        className="sir-chart"
        viewBox={`0 0 ${CHART_WIDTH} ${CHART_HEIGHT}`}
        role="img"
        aria-label="Biểu đồ số lượng Susceptible, Infected và Recovered theo time step"
      >
        {[0, 0.25, 0.5, 0.75, 1].map((fraction) => {
          const value = yMaximum * fraction
          const yPosition = y(value)
          return (
            <g key={fraction}>
              <line
                x1={CHART_PADDING.left}
                x2={CHART_WIDTH - CHART_PADDING.right}
                y1={yPosition}
                y2={yPosition}
                className="sir-chart-grid"
              />
              <text x={CHART_PADDING.left - 9} y={yPosition + 4} textAnchor="end" className="sir-chart-axis-label">
                {formatSirValue(value)}
              </text>
            </g>
          )
        })}
        <line
          x1={CHART_PADDING.left}
          x2={CHART_WIDTH - CHART_PADDING.right}
          y1={CHART_HEIGHT - CHART_PADDING.bottom}
          y2={CHART_HEIGHT - CHART_PADDING.bottom}
          className="sir-chart-axis"
        />
        {SERIES.map((series) => (
          <polyline
            key={series.key}
            points={points.map((point) => `${x(point.time_step)},${y(point[series.key])}`).join(' ')}
            fill="none"
            stroke={series.color}
            strokeWidth="2.8"
            strokeDasharray={series.dash}
            strokeLinejoin="round"
            strokeLinecap="round"
          />
        ))}
        {selected && (
          <g>
            <line
              x1={x(selected.time_step)}
              x2={x(selected.time_step)}
              y1={CHART_PADDING.top}
              y2={CHART_HEIGHT - CHART_PADDING.bottom}
              className="sir-chart-cursor"
            />
            {SERIES.map((series) => (
              <circle
                key={series.key}
                cx={x(selected.time_step)}
                cy={y(selected[series.key])}
                r="5"
                fill={series.color}
                stroke="#0f172a"
                strokeWidth="2"
              />
            ))}
          </g>
        )}
        <text x={CHART_PADDING.left} y={CHART_HEIGHT - 8} className="sir-chart-axis-label">0</text>
        <text
          x={CHART_WIDTH - CHART_PADDING.right}
          y={CHART_HEIGHT - 8}
          textAnchor="end"
          className="sir-chart-axis-label"
        >
          {points.length - 1}
        </text>
        <text
          x={CHART_WIDTH / 2}
          y={CHART_HEIGHT - 8}
          textAnchor="middle"
          className="sir-chart-axis-title"
        >
          Time step
        </text>
      </svg>
      <div className="sir-chart-legend" aria-label="Chú giải biểu đồ">
        {SERIES.map((series) => (
          <span key={series.key}>
            <i style={{ backgroundColor: series.color }} />
            {series.label}
          </span>
        ))}
      </div>
      <div className="sir-chart-point-buttons" aria-label="Chọn time step trên biểu đồ">
        {points.length <= 30 && points.map((point) => (
          <button
            key={point.time_step}
            type="button"
            className={point.time_step === selectedTimeStep ? 'is-selected' : ''}
            aria-label={`Chọn time step ${point.time_step}`}
            aria-pressed={point.time_step === selectedTimeStep}
            onClick={() => onSelect(point.time_step)}
          />
        ))}
      </div>
    </div>
  )
}

function SirCurrentMetrics({ point }: { point: SirPoint }) {
  const metrics = [
    { key: 'susceptible', label: 'Susceptible (S)', value: point.susceptible, className: 'sir-metric-s' },
    { key: 'infected', label: 'Infected (I)', value: point.infected, className: 'sir-metric-i' },
    { key: 'recovered', label: 'Recovered (R)', value: point.recovered, className: 'sir-metric-r' },
  ]

  return (
    <section className="sir-metrics" aria-label="Chỉ số tại time step đang chọn">
      {metrics.map((metric) => (
        <article className={`sir-metric ${metric.className}`} key={metric.key}>
          <span>{metric.label}</span>
          <strong>{formatSirValue(metric.value)}</strong>
        </article>
      ))}
      <article className="sir-metric sir-metric-time">
        <span>Thời gian</span>
        <strong>{formatSirValue(point.time)}</strong>
        <small>Time step {point.time_step}</small>
      </article>
    </section>
  )
}

function SirPlaybackControls({
  pointCount,
  selectedTimeStep,
  playing,
  onSelect,
  onPlay,
  onPause,
  onReset,
}: {
  pointCount: number
  selectedTimeStep: number
  playing: boolean
  onSelect: (timeStep: number) => void
  onPlay: () => void
  onPause: () => void
  onReset: () => void
}) {
  const lastIndex = pointCount - 1
  const canPlay = pointCount > 1 && selectedTimeStep < lastIndex

  return (
    <section className="sir-playback" aria-label="Điều khiển phát lại mô phỏng">
      <label htmlFor="sir-time-step">
        <span>Time step</span>
        <output>{selectedTimeStep} / {lastIndex}</output>
      </label>
      <input
        id="sir-time-step"
        type="range"
        min={0}
        max={lastIndex}
        step={1}
        value={selectedTimeStep}
        onChange={(event) => onSelect(Number(event.currentTarget.value))}
        disabled={pointCount <= 1}
      />
      <div className="sir-playback-actions">
        {playing ? (
          <button type="button" onClick={onPause}>Pause</button>
        ) : (
          <button type="button" onClick={onPlay} disabled={!canPlay}>Play</button>
        )}
        <button type="button" className="sir-button-secondary" onClick={onReset} disabled={selectedTimeStep === 0 && !playing}>
          Reset
        </button>
      </div>
    </section>
  )
}

export default function SirResultPanel({ result, loading, error }: SirResultPanelProps) {
  const validResult = useMemo(
    () => (result && isSirSimulationResult(result) ? result : null),
    [result],
  )
  const [selectedTimeStep, setSelectedTimeStep] = useState(0)
  const [playing, setPlaying] = useState(false)

  useEffect(() => {
    setSelectedTimeStep(0)
    setPlaying(false)
  }, [result])

  useEffect(() => {
    setPlaying(false)
  }, [loading, error])

  useEffect(() => {
    if (!playing || loading || error || !validResult || validResult.point_count <= 1) return

    const lastIndex = validResult.point_count - 1
    const timer = window.setInterval(() => {
      setSelectedTimeStep((current) => Math.min(current + 1, lastIndex))
    }, 500)

    return () => window.clearInterval(timer)
  }, [playing, loading, error, validResult])

  const point = validResult?.points[selectedTimeStep]
  const isAtEnd = Boolean(validResult && selectedTimeStep >= validResult.point_count - 1)

  useEffect(() => {
    if (playing && isAtEnd) setPlaying(false)
  }, [playing, isAtEnd])

  if (loading) {
    return <section className="sir-result-panel" aria-live="polite"><p className="sir-state">Đang tải kết quả mô phỏng SIR…</p></section>
  }

  if (error) {
    return <section className="sir-result-panel" role="alert"><p className="sir-state sir-state-error">{error}</p></section>
  }

  if (!result) {
    return (
      <section className="sir-result-panel">
        <header className="sir-panel-header">
          <div><p className="sir-eyebrow">SIMULATION</p><h2>Kết quả mô phỏng SIR</h2></div>
          <span className="sir-status">Chưa có dữ liệu</span>
        </header>
        <p className="sir-state">Chạy một mô phỏng SIR để xem diễn biến Susceptible, Infected và Recovered tại đây.</p>
      </section>
    )
  }

  if (!validResult || !point) {
    return (
      <section className="sir-result-panel" role="alert">
        <header className="sir-panel-header"><h2>Kết quả mô phỏng SIR</h2></header>
        <p className="sir-state sir-state-error">Response SIR không hợp lệ hoặc không khớp contract của Backend.</p>
      </section>
    )
  }

  return (
    <section className="sir-result-panel">
      <header className="sir-panel-header">
        <div><p className="sir-eyebrow">SIMULATION RESULT</p><h2>Kết quả mô phỏng SIR</h2></div>
        <span className="sir-status">{validResult.point_count} điểm dữ liệu</span>
      </header>
      <SirResultChart
        points={validResult.points}
        population={validResult.population}
        selectedTimeStep={selectedTimeStep}
        onSelect={setSelectedTimeStep}
      />
      <SirCurrentMetrics point={point} />
      <SirPlaybackControls
        pointCount={validResult.point_count}
        selectedTimeStep={selectedTimeStep}
        playing={playing}
        onSelect={(timeStep) => { setPlaying(false); setSelectedTimeStep(timeStep) }}
        onPlay={() => setPlaying(true)}
        onPause={() => setPlaying(false)}
        onReset={() => { setPlaying(false); setSelectedTimeStep(0) }}
      />
    </section>
  )
}
