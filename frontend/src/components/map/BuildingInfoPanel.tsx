import type { Building } from '../../types/gis'

interface BuildingInfoPanelProps {
  building: Building | null
  loading: boolean
  error: string | null
  selectedBuildingId: string
  onClose: () => void
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

      {!loading && !error && !building && (
        <p className="building-info-state">
          Không tìm thấy thông tin cho công trình <code>{selectedBuildingId}</code>.
        </p>
      )}
    </aside>
  )
}
