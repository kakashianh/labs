import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import { viteStaticCopy } from 'vite-plugin-static-copy'

// In Codespaces, docker-compose publishes the Rust API on port 8000 of the
// workspace. Proxy API requests through Vite so the browser uses one origin.
export default defineConfig({
  plugins: [
    react(),
    viteStaticCopy({
      targets: [
        {
          src: 'node_modules/cesium/Build/Cesium/Workers',
          dest: 'cesium',
          rename: { stripBase: 4 },
        },
        {
          src: 'node_modules/cesium/Build/Cesium/Assets',
          dest: 'cesium',
          rename: { stripBase: 4 },
        },
        {
          src: 'node_modules/cesium/Build/Cesium/Widgets',
          dest: 'cesium',
          rename: { stripBase: 4 },
        },
        {
          src: 'node_modules/cesium/Build/Cesium/ThirdParty',
          dest: 'cesium',
          rename: { stripBase: 4 },
        },
      ],
    }),
  ],
  define: {
    CESIUM_BASE_URL: JSON.stringify('/cesium/'),
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8000',
        changeOrigin: true,
      },
    },
  },
})
