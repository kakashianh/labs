import { useEffect, useRef } from 'react'
import * as Cesium from 'cesium'
import 'cesium/Build/Cesium/Widgets/widgets.css'

/**
 * CesiumViewer — khởi tạo Cesium Viewer hiển thị bản đồ nền OSM.
 * Viewer được tạo trong useEffect và destroy khi component unmount,
 * an toàn với React StrictMode (mount / unmount / mount lại).
 */
export default function CesiumViewer() {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewerRef = useRef<Cesium.Viewer | null>(null)

  useEffect(() => {
    const container = containerRef.current
    if (!container) {
      return
    }

    // Tránh khởi tạo nhiều Viewer trên cùng một DOM element.
    if (viewerRef.current) {
      return
    }

    const ionToken = import.meta.env.VITE_CESIUM_ION_TOKEN as
      | string
      | undefined

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
          url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
          credit: '© OpenStreetMap contributors',
        }),
      ),
    })

    viewerRef.current = viewer

    // Camera ban đầu: khu vực Ba Đình, Hà Nội.
    viewer.camera.setView({
      destination: Cesium.Cartesian3.fromDegrees(105.8342, 21.0278, 8000),
      orientation: {
        heading: 0,
        pitch: Cesium.Math.toRadians(-55),
        roll: 0,
      },
    })

    return () => {
      viewer.destroy()
      viewerRef.current = null
    }
  }, [])

  return (
    <div
      ref={containerRef}
      className="cesium-map-container"
      data-testid="cesium-viewer"
    />
  )
}
