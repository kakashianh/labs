import { useEffect, useState } from 'react'
import * as Cesium from 'cesium'
import 'cesium/Build/Cesium/Widgets/widgets.css'
import { fetchAllBuildings, fetchRegions } from '../../services/gis-api'
import { buildingsToFeatureCollection } from '../../services/gis-data'

type LoadState = 'loading' | 'ready' | 'empty' | 'error'

const DEFAULT_BUILDING_FILL = Cesium.Color.CORNFLOWERBLUE.withAlpha(0.35)
const SELECTED_BUILDING_FILL = Cesium.Color.ORANGE.withAlpha(0.75)

// Esri World Imagery is used instead of the OSM Foundation standard tile server.
// Keep the provider's required source attribution visible in the Cesium viewer.
const BASEMAP_URL =
  'https://services.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}'
const BASEMAP_CREDIT =
  'Sources: Esri, DigitalGlobe, GeoEye, i-cubed, USDA FSA, USGS, AEX, Getmapping, Aerogrid, IGN, IGP, swisstopo, and the GIS User Community'

/**
 * CesiumViewer displays an Esri imagery basemap and the building footprints
 * served by the Rust API. Data loading and viewer resources share one effect so
 * StrictMode cleanup aborts pending requests and destroys every Cesium resource.
 */
export default function CesiumViewer() {
  const [loadState, setLoadState] = useState<LoadState>('loading')
  const [regionCount, setRegionCount] = useState(0)
  const [buildingCount, setBuildingCount] = useState(0)
  const [skippedCount, setSkippedCount] = useState(0)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [selectedBuilding, setSelectedBuilding] = useState<string | null>(null)

  useEffect(() => {
    const container = document.querySelector<HTMLDivElement>('[data-testid="cesium-viewer"]')
    if (!container) return

    const controller = new AbortController()
    const ionToken = import.meta.env.VITE_CESIUM_ION_TOKEN as string | undefined

    if (ionToken) {
      Cesium.Ion.defaultAccessToken = ionToken
    }

    const viewer = new Cesium.Viewer(container, {
      animation: false,
      timeline: false,
      geocoder: false,
      baseLayerPicker: false,
      homeButton: false,
      sceneModePicker: false,
      navigationHelpButton: false,
      fullscreenButton: false,
      infoBox: false,
      selectionIndicator: false,
      baseLayer: new Cesium.ImageryLayer(
        new Cesium.UrlTemplateImageryProvider({
          url: BASEMAP_URL,
          credit: new Cesium.Credit(BASEMAP_CREDIT),
        }),
      ),
    })

    viewer.camera.setView({
      destination: Cesium.Cartesian3.fromDegrees(105.8342, 21.0278, 8000),
      orientation: {
        heading: 0,
        pitch: Cesium.Math.toRadians(-55),
        roll: 0,
      },
    })

    const clickHandler = new Cesium.ScreenSpaceEventHandler(viewer.scene.canvas)
    let selectedEntity: Cesium.Entity | undefined

    async function loadBuildingData() {
      try {
        const [regionsResponse, buildings] = await Promise.all([
          fetchRegions(controller.signal),
          fetchAllBuildings(undefined, controller.signal),
        ])

        if (controller.signal.aborted || viewer.isDestroyed()) return

        setRegionCount(regionsResponse.data.length)
        setBuildingCount(buildings.length)

        const { featureCollection, skippedCount: invalidGeometryCount } =
          buildingsToFeatureCollection(buildings)
        setSkippedCount(invalidGeometryCount)

        if (featureCollection.features.length === 0) {
          setLoadState('empty')
          return
        }

        const dataSource = await Cesium.GeoJsonDataSource.load(
          featureCollection as Parameters<typeof Cesium.GeoJsonDataSource.load>[0],
          {
            clampToGround: true,
            stroke: Cesium.Color.STEELBLUE,
            strokeWidth: 1.5,
            fill: DEFAULT_BUILDING_FILL,
          },
        )

        if (controller.signal.aborted || viewer.isDestroyed()) return

        await viewer.dataSources.add(dataSource)
        if (controller.signal.aborted || viewer.isDestroyed()) return

        clickHandler.setInputAction((movement: Cesium.ScreenSpaceEventHandler.PositionedEvent) => {
          const picked = viewer.scene.pick(movement.position)
          const entity =
            picked?.id instanceof Cesium.Entity ? (picked.id as Cesium.Entity) : undefined

          if (!entity?.polygon) {
            const previousPolygon = selectedEntity?.polygon
            if (previousPolygon) {
              previousPolygon.material =
                new Cesium.ColorMaterialProperty(DEFAULT_BUILDING_FILL)
            }
            selectedEntity = undefined
            setSelectedBuilding(null)
            return
          }

          const previousPolygon = selectedEntity?.polygon
          if (previousPolygon) {
            previousPolygon.material =
              new Cesium.ColorMaterialProperty(DEFAULT_BUILDING_FILL)
          }

          const selectedPolygon = entity.polygon
          selectedEntity = entity
          selectedPolygon.material =
            new Cesium.ColorMaterialProperty(SELECTED_BUILDING_FILL)

          const propertyId = entity.properties?.buildingId?.getValue(Cesium.JulianDate.now())
          setSelectedBuilding(
            typeof propertyId === 'string' ? propertyId : String(entity.id),
          )
        }, Cesium.ScreenSpaceEventType.LEFT_CLICK)

        setLoadState('ready')
        void viewer.flyTo(dataSource).catch(() => {
          // A camera flight failure should not hide successfully loaded features.
        })
      } catch (error) {
        if (controller.signal.aborted) return
        setLoadState('error')
        setErrorMessage(
          error instanceof Error ? error.message : 'Không thể tải dữ liệu GIS từ Backend.',
        )
      }
    }

    void loadBuildingData()

    return () => {
      controller.abort()
      clickHandler.destroy()
      if (!viewer.isDestroyed()) {
        viewer.destroy()
      }
    }
  }, [])

  return (
    <div className="map-shell">
      <div
        className="cesium-map-container"
        data-testid="cesium-viewer"
        aria-label="Bản đồ 3D công trình"
      />
      <section className="map-status" aria-live="polite">
        {loadState === 'loading' && <p>Đang tải khu vực và dữ liệu công trình…</p>}
        {loadState === 'ready' && (
          <>
            <p>
              {regionCount} khu vực · {buildingCount - skippedCount} công trình hiển thị
            </p>
            {skippedCount > 0 && (
              <p className="map-status-warning">
                Bỏ qua {skippedCount} công trình có geometry không hợp lệ.
              </p>
            )}
            {selectedBuilding && (
              <p className="map-status-selection">
                Công trình được chọn: <code>{selectedBuilding}</code>
              </p>
            )}
          </>
        )}
        {loadState === 'empty' && (
          <p>
            Không có công trình hợp lệ để hiển thị
            {buildingCount > 0 ? ` (đã bỏ qua ${skippedCount} geometry lỗi)` : ''}.
          </p>
        )}
        {loadState === 'error' && (
          <p className="map-status-error">
            Không thể tải dữ liệu GIS: {errorMessage}
            <br />
            Kiểm tra Backend và VITE_API_BASE_URL.
          </p>
        )}
      </section>
    </div>
  )
}
