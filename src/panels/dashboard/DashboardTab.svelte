<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"

  import ProfileSelector from "./ProfileSelector.svelte"
  import StatsGrid from "./StatsGrid.svelte"
  import ConceptStatus from "./ConceptStatus.svelte"
  import ConceptDetail from "./ConceptDetail.svelte"
  import EvolutionChart from "./EvolutionChart.svelte"
  import RecentSessions from "./RecentSessions.svelte"
  import ExportButtons from "./ExportButtons.svelte"
</script>

<ProfileSelector />
{#if appState.dashboardStats}
  <StatsGrid />
  <ConceptStatus />
  <ConceptDetail />
  <EvolutionChart />
  <RecentSessions />
  <ExportButtons />
{/if}
