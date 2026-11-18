import type { Building } from '../../types/gis'
import type { SpatialBuildingView } from '../../services/spatial-simulation.ts'

interface BuildingInfoPanelProps {
  building: Building | null
  loading: boolean
  error: string | null
  selectedBuildingId: string
  onClose: () => void
  spatialData?: SpatialBuildingView | null
  simulationTime?: { simulationId: string; timeStep: number; time: number } | null
}

function displayValue(value: string | null | undefined): string | null {
  if (typeof value !== 'string' || value.trim().length === 0) return null
  return value
}

export default function BuildingInfoPanel({
  building,
  loading,
  error,
  selectedBuildingId,
  onClose,
  spatialData = null,
  simulationTime = null,
}: BuildingInfoPanelProps) {
  const name = displayValue(building?.name)
  const buildingType = displayValue(building?.building_type)

  return (
    <aside
      className="building-info-panel"
      aria-label="Thông tin công trình được chọn"
      aria-live="polite"
      onPointerDown={(event) => event.stopPropagation()}
      onClick={(event) => event.stopPropagation()}
    >
      <header className="building-info-header">
        <div>
          <p className="building-info-eyebrow">THÔNG TIN CÔNG TRÌNH</p>
          <h2>{name ?? 'Công trình'}</h2>
        </div>
        <button
          className="building-info-close"
          type="button"
          aria-label="Đóng thông tin công trình"
          onClick={onClose}
        >
          ×
        </button>
      </header>

      {loading && <p className="building-info-state">Đang tải thông tin công trình…</p>}

      {!loading && error && (
        <div className="building-info-state building-info-error" role="alert">
          <p>{error}</p>
          <p className="building-info-hint">Bạn có thể đóng bảng thông tin và chọn công trình khác.</p>
        </div>
      )}

      {!loading && !error && building && (
        <dl className="building-info-fields">
          <div>
            <dt>Building ID</dt>
            <dd>{building.id}</dd>
          </div>
          <div>
            <dt>Region ID</dt>
            <dd>{building.region_id}</dd>
          </div>
          <div>
            <dt>Mã nguồn dữ liệu</dt>
            <dd>{building.external_id}</dd>
          </div>
          <div>
            <dt>Loại công trình</dt>
            <dd>{buildingType ?? 'Chưa có dữ liệu'}</dd>
          </div>
        </dl>
      )}


      {spatialData && (
        <section className="building-spatial-state" aria-label="Trạng thái dịch tễ">
          <h3>Spatial SIR · {spatialData.risk.risk_level.toUpperCase()}</h3>
          {simulationTime && (
            <p className="building-spatial-context">
              Step {simulationTime.timeStep} · t = {simulationTime.time} · {simulationTime.simulationId}
            </p>
          )}
          <dl className="building-info-fields">
            <div><dt>Population (N)</dt><dd>{spatialData.result?.population ?? 'Không có dữ liệu'}</dd></div>
            <div><dt>Susceptible (S)</dt><dd>{spatialData.result?.susceptible ?? 'Không có dữ liệu'}</dd></div>
            <div><dt>Infected (I)</dt><dd>{spatialData.risk.infected}</dd></div>
            <div><dt>Recovered (R)</dt><dd>{spatialData.result?.recovered ?? 'Không có dữ liệu'}</dd></div>
            <div><dt>Infection ratio</dt><dd>{spatialData.risk.infection_ratio === null ? 'Unclassified' : `${(spatialData.risk.infection_ratio * 100).toFixed(2)}%`}</dd></div>
            <div><dt>Risk</dt><dd>{spatialData.risk.risk_level}</dd></div>
          </dl>
          {spatialData.risk.unclassified_reason && (
            <p className="building-info-hint">Lý do: {spatialData.risk.unclassified_reason}</p>
          )}
        </section>
      )}

      {!loading && !error && !building && (
        <p className="building-info-state">
          Không tìm thấy thông tin cho công trình <code>{selectedBuildingId}</code>.
        </p>
      )}
    </aside>
  )
}
