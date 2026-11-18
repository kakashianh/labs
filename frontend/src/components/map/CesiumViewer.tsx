import { useEffect, useRef, useState } from 'react'
import * as Cesium from 'cesium'
import 'cesium/Build/Cesium/Widgets/widgets.css'
import BuildingInfoPanel from './BuildingInfoPanel'
import { fetchBuildingDetail, fetchAllBuildings, fetchRegions } from '../../services/gis-api'
import { buildingsToFeatureCollection, isSelectedEntityRemoved } from '../../services/gis-data'
import type { Building } from '../../types/gis'
import SpatialSimulationControls, { type SpatialColorMetric } from '../simulation/SpatialSimulationControls'
import type { SpatialSimulationFrame } from '../../services/spatial-simulation.ts'

type LoadState = 'loading' | 'ready' | 'empty' | 'error'

const DEFAULT_BUILDING_FILL = Cesium.Color.CORNFLOWERBLUE.withAlpha(0.35)
const SELECTED_BUILDING_FILL = Cesium.Color.ORANGE.withAlpha(0.75)
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i

const BASEMAP_URL =
  'https://services.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}'
const BASEMAP_CREDIT =
  'Sources: Esri, DigitalGlobe, GeoEye, i-cubed, USDA FSA, USGS, AEX, Getmapping, Aerogrid, IGN, IGP, swisstopo, and the GIS User Community'

export default function CesiumViewer() {
  const [loadState, setLoadState] = useState<LoadState>('loading')
  const [regionCount, setRegionCount] = useState(0)
  const [buildingCount, setBuildingCount] = useState(0)
  const [skippedCount, setSkippedCount] = useState(0)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [selectedBuildingId, setSelectedBuildingId] = useState<string | null>(null)
  const [buildingDetail, setBuildingDetail] = useState<Building | null>(null)
  const [detailLoading, setDetailLoading] = useState(false)
  const [detailError, setDetailError] = useState<string | null>(null)
  const [spatialFrame, setSpatialFrame] = useState<SpatialSimulationFrame | null>(null)
  const [spatialMetric, setSpatialMetric] = useState<SpatialColorMetric>('risk_level')
  const [buildingLayerVisible, setBuildingLayerVisible] = useState(true)
  const [hotspotLayerVisible, setHotspotLayerVisible] = useState(true)
  const [selectedHotspotId, setSelectedHotspotId] = useState<string | null>(null)
  const viewerRef = useRef<Cesium.Viewer | null>(null)
  const buildingDataSourceRef = useRef<Cesium.GeoJsonDataSource | null>(null)
  const hotspotDataSourceRef = useRef<Cesium.GeoJsonDataSource | null>(null)
  const hotspotLoadVersionRef = useRef(0)
  const clearSelectionRef = useRef<(() => void) | null>(null)
  const selectedBuildingIdRef = useRef<string | null>(null)

  useEffect(() => {
    if (!selectedBuildingId) {
      setBuildingDetail(null)
      setDetailLoading(false)
      setDetailError(null)
      return
    }

    const controller = new AbortController()
    const requestedId = selectedBuildingId
    setBuildingDetail(null)
    setDetailLoading(true)
    setDetailError(null)

    void fetchBuildingDetail(requestedId, controller.signal)
      .then((detail) => {
        if (
          !controller.signal.aborted &&
          selectedBuildingIdRef.current === requestedId &&
          detail.id === requestedId
        ) {
          setBuildingDetail(detail)
        }
      })
      .catch((error: unknown) => {
        if (
          controller.signal.aborted ||
          selectedBuildingIdRef.current !== requestedId
        ) return
        if (error instanceof Error && error.name === 'AbortError') return
        setDetailError(
          error instanceof Error ? error.message : 'Không thể tải thông tin công trình.',
        )
      })
      .finally(() => {
        if (!controller.signal.aborted && selectedBuildingIdRef.current === requestedId) {
          setDetailLoading(false)
        }
      })

    return () => controller.abort()
  }, [selectedBuildingId])

  useEffect(() => {
    const container = document.querySelector<HTMLDivElement>('[data-testid="cesium-viewer"]')
    if (!container) return

    const controller = new AbortController()
    const ionToken = import.meta.env.VITE_CESIUM_ION_TOKEN as string | undefined

    if (ionToken) Cesium.Ion.defaultAccessToken = ionToken

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

    viewerRef.current = viewer

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
    let selectedOriginalMaterial: Cesium.MaterialProperty | undefined
    let cleanupDataSourceListeners: (() => void) | null = null

    const restoreSelectedStyle = () => {
      if (selectedEntity?.polygon) {
        selectedEntity.polygon.material =
          selectedOriginalMaterial ?? new Cesium.ColorMaterialProperty(DEFAULT_BUILDING_FILL)
      }
      selectedEntity = undefined
      selectedOriginalMaterial = undefined
    }

    const clearSelection = () => {
      restoreSelectedStyle()
      selectedBuildingIdRef.current = null
      setSelectedBuildingId(null)
    }
    clearSelectionRef.current = clearSelection

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
        buildingDataSourceRef.current = dataSource
        if (controller.signal.aborted || viewer.isDestroyed()) return

        // Keep React selection state in sync if a selected Cesium entity or its
        // entire data source is removed while the viewer remains mounted.
        const handleEntitiesChanged = (
          _collection: Cesium.EntityCollection,
          _added: Cesium.Entity[],
          removed: Cesium.Entity[],
        ) => {
          if (isSelectedEntityRemoved(selectedEntity, removed)) clearSelection()
        }
        const handleDataSourceRemoved = (
          _collection: Cesium.DataSourceCollection,
          removedDataSource: Cesium.DataSource,
        ) => {
          if (removedDataSource === dataSource) clearSelection()
        }

        dataSource.entities.collectionChanged.addEventListener(handleEntitiesChanged)
        viewer.dataSources.dataSourceRemoved.addEventListener(handleDataSourceRemoved)
        cleanupDataSourceListeners = () => {
          dataSource.entities.collectionChanged.removeEventListener(handleEntitiesChanged)
          viewer.dataSources.dataSourceRemoved.removeEventListener(handleDataSourceRemoved)
        }

        clickHandler.setInputAction((movement: Cesium.ScreenSpaceEventHandler.PositionedEvent) => {
          const picked = viewer.scene.pick(movement.position)
          const entity =
            picked?.id instanceof Cesium.Entity ? (picked.id as Cesium.Entity) : undefined

          // Empty globe/map space clears the active selection. A non-building
          // entity is ignored so it cannot accidentally replace the panel state.
          if (!entity) {
            clearSelection()
            setSelectedHotspotId(null)
            return
          }

          const clusterId = entity.properties?.clusterId?.getValue(Cesium.JulianDate.now())
          if (typeof clusterId === 'string') {
            setSelectedHotspotId(clusterId)
            clearSelection()
            return
          }
          if (!entity.polygon) return
          setSelectedHotspotId(null)

          const propertyId = entity.properties?.buildingId?.getValue(Cesium.JulianDate.now())
          if (typeof propertyId !== 'string' || !UUID_PATTERN.test(propertyId)) return

          if (selectedEntity !== entity) {
            restoreSelectedStyle()
            selectedEntity = entity
            selectedOriginalMaterial = entity.polygon.material
            entity.polygon.material = new Cesium.ColorMaterialProperty(SELECTED_BUILDING_FILL)
          }
          selectedBuildingIdRef.current = propertyId
          setSelectedBuildingId(propertyId)
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
      cleanupDataSourceListeners?.()
      clearSelectionRef.current = null
      buildingDataSourceRef.current = null
      hotspotDataSourceRef.current = null
      viewerRef.current = null
      hotspotLoadVersionRef.current += 1
      clickHandler.destroy()
      if (!viewer.isDestroyed()) viewer.destroy()
    }
  }, [])


  useEffect(() => {
    const viewer = viewerRef.current
    const dataSource = buildingDataSourceRef.current
    if (!viewer || viewer.isDestroyed() || !dataSource) return

    const byBuildingId = new Map(
      (spatialFrame?.buildings ?? []).map((item) => [item.building_id, item]),
    )
    const selectedHotspot = spatialFrame?.hotspots.find(
      (item) => item.cluster_id === selectedHotspotId,
    )
    const highlightedIds = new Set(selectedHotspot?.building_ids ?? [])

    for (const entity of dataSource.entities.values) {
      if (!entity.polygon) continue
      const propertyId = entity.properties?.buildingId?.getValue(Cesium.JulianDate.now())
      const buildingId = typeof propertyId === 'string' ? propertyId : entity.id
      const item = byBuildingId.get(buildingId)
      let fill = DEFAULT_BUILDING_FILL

      if (spatialFrame && buildingLayerVisible && item) {
        if (spatialMetric === 'risk_level') {
          switch (item.risk.risk_level) {
            case 'low': fill = Cesium.Color.fromCssColorString('#22c55e').withAlpha(0.72); break
            case 'medium': fill = Cesium.Color.fromCssColorString('#f59e0b').withAlpha(0.78); break
            case 'high': fill = Cesium.Color.fromCssColorString('#ef4444').withAlpha(0.82); break
            default: fill = Cesium.Color.fromCssColorString('#94a3b8').withAlpha(0.55)
          }
        } else if (item.risk.infection_ratio !== null && item.result) {
          const ratio = Math.max(0, Math.min(1, item.risk.infection_ratio))
          fill = Cesium.Color.fromHsl(0.62 * (1 - ratio), 0.82, 0.5, 0.78)
        } else {
          fill = Cesium.Color.fromCssColorString('#94a3b8').withAlpha(0.55)
        }
      }

      if (buildingId === selectedBuildingId) fill = SELECTED_BUILDING_FILL
      entity.polygon.material = new Cesium.ColorMaterialProperty(fill)
      entity.polygon.outline = highlightedIds.has(buildingId)
      entity.polygon.outlineColor = Cesium.Color.YELLOW
    }
  }, [spatialFrame, spatialMetric, buildingLayerVisible, selectedHotspotId, selectedBuildingId, loadState])

  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer || viewer.isDestroyed()) return
    const version = ++hotspotLoadVersionRef.current
    const previous = hotspotDataSourceRef.current
    hotspotDataSourceRef.current = null
    if (previous) void viewer.dataSources.remove(previous, true)

    if (!spatialFrame || !hotspotLayerVisible || spatialFrame.hotspots.length === 0) return

    const featureCollection = {
      type: 'FeatureCollection' as const,
      features: spatialFrame.hotspots.map((hotspot) => ({
        type: 'Feature' as const,
        id: hotspot.cluster_id,
        geometry: hotspot.geometry,
        properties: {
          clusterId: hotspot.cluster_id,
          memberCount: hotspot.member_count,
          populationTotal: hotspot.population_total,
          infectedTotal: hotspot.infected_total,
          infectionRatio: hotspot.infection_ratio,
          riskLevel: hotspot.risk_level,
          buildingIds: hotspot.building_ids.join(','),
        },
      })),
    }

    void Cesium.GeoJsonDataSource.load(featureCollection, {
      clampToGround: true,
      stroke: Cesium.Color.WHITE,
      strokeWidth: 3,
      fill: Cesium.Color.fromCssColorString('#a855f7').withAlpha(0.28),
    }).then(async (dataSource) => {
      if (version !== hotspotLoadVersionRef.current || viewer.isDestroyed()) {
        dataSource.entities.removeAll()
        return
      }
      await viewer.dataSources.add(dataSource)
      if (version !== hotspotLoadVersionRef.current || viewer.isDestroyed()) {
        if (!viewer.isDestroyed()) void viewer.dataSources.remove(dataSource, true)
        return
      }
      hotspotDataSourceRef.current = dataSource
    }).catch((error: unknown) => {
      if (version === hotspotLoadVersionRef.current) {
        setErrorMessage(error instanceof Error ? error.message : 'Không thể hiển thị hotspot.')
      }
    })

    return () => {
      hotspotLoadVersionRef.current += 1
    }
  }, [spatialFrame, hotspotLayerVisible, loadState])

  const closeBuildingPanel = () => clearSelectionRef.current?.()

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
      <SpatialSimulationControls
        onFrameChange={(frame) => { setSpatialFrame(frame); setSelectedHotspotId(null) }}
        onMetricChange={setSpatialMetric}
        onLayerVisibilityChange={(buildings, hotspots) => { setBuildingLayerVisible(buildings); setHotspotLayerVisible(hotspots) }}
      />
      {selectedHotspotId && spatialFrame && (() => {
        const hotspot = spatialFrame.hotspots.find((item) => item.cluster_id === selectedHotspotId)
        return hotspot ? (
          <aside className="spatial-hotspot-detail" aria-label="Thông tin hotspot">
            <button type="button" onClick={() => setSelectedHotspotId(null)} aria-label="Đóng hotspot">×</button>
            <h3>Hotspot · {hotspot.risk_level.toUpperCase()}</h3>
            <dl>
              <div><dt>Cluster ID</dt><dd>{hotspot.cluster_id}</dd></div>
              <div><dt>Buildings</dt><dd>{hotspot.member_count}</dd></div>
              <div><dt>Population</dt><dd>{hotspot.population_total}</dd></div>
              <div><dt>Infected</dt><dd>{hotspot.infected_total}</dd></div>
              <div><dt>Infection ratio</dt><dd>{(hotspot.infection_ratio * 100).toFixed(2)}%</dd></div>
              <div><dt>Members</dt><dd>{hotspot.building_ids.join(', ')}</dd></div>
            </dl>
          </aside>
        ) : null
      })()}
      {selectedBuildingId && (
        <BuildingInfoPanel
          building={buildingDetail}
          loading={detailLoading}
          error={detailError}
          selectedBuildingId={selectedBuildingId}
          onClose={closeBuildingPanel}
          spatialData={spatialFrame?.buildings.find((item) => item.building_id === selectedBuildingId) ?? null}
          simulationTime={spatialFrame ? { simulationId: spatialFrame.summary.simulation_id, timeStep: spatialFrame.time_step, time: spatialFrame.time } : null}
        />
      )}
    </div>
  )
}
