import { EntityBrowser } from '../components/EntityBrowser'

export function EntityGraphPage() {
  return (
    <div className="p-6">
      <EntityBrowser mode="graph" />
    </div>
  )
}