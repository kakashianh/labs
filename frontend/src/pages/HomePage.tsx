import CesiumViewer from '../components/map/CesiumViewer'
import SirResultPanel from '../components/simulation/SirResultPanel'
import '../styles/sir-result.css'

export default function HomePage() {
  return (
    <main className="home-page">
      <CesiumViewer />
      <SirResultPanel result={null} loading={false} error={null} />
    </main>
  )
}
