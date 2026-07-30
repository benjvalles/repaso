<script lang="ts">
  import { appState } from "../lib/app-state.svelte"

  import ProfilesTab from "./ProfilesTab.svelte"
  import LlmTab from "./LlmTab.svelte"
  import SessionsTab from "./SessionsTab.svelte"
  import DashboardTab from "./dashboard/DashboardTab.svelte"
  import ProfessionalTab from "./professional/ProfessionalTab.svelte"
  import CloudTab from "./CloudTab.svelte"
</script>

<section class="panel adult-zone">
  <div class="adult-header">
    <h2>Zona adulta</h2>
    <button class="secondary" type="button" onclick={() => appState.lockAdult()}>Bloquear</button>
  </div>

  <div class="tab-bar">
    <button class:active={appState.adultTab === "profiles"} type="button" onclick={() => { appState.adultTab = "profiles"; appState.loadDeletedProfiles() }}>Perfiles</button>
    <button class:active={appState.adultTab === "llm"} type="button" onclick={() => appState.adultTab = "llm"}>IA / LLM</button>
    <button class:active={appState.adultTab === "sessions"} type="button" onclick={() => { appState.adultTab = "sessions"; if (appState.historyProfileId) { appState.loadSessions(); appState.loadDeletedSessions() } }}>Historial</button>
    <button class:active={appState.adultTab === "dashboard"} type="button" onclick={() => { appState.adultTab = "dashboard"; if (appState.dashboardProfileId) appState.loadDashboard() }}>Dashboard</button>
    <button class:active={appState.adultTab === "cloud"} type="button" onclick={() => { appState.adultTab = "cloud"; appState.loadCloudStatus() }}>Nube</button>
    <button class:active={appState.adultTab === "professional"} type="button" onclick={() => { appState.adultTab = "professional"; appState.loadProfessionalData() }}>Profesional</button>
  </div>

  {#if appState.adultTab === "profiles"}
    <ProfilesTab />
  {:else if appState.adultTab === "llm"}
    <LlmTab />
  {:else if appState.adultTab === "sessions"}
    <SessionsTab />
  {:else if appState.adultTab === "dashboard"}
    <DashboardTab />
  {:else if appState.adultTab === "cloud"}
    <CloudTab />
  {:else if appState.adultTab === "professional"}
    <ProfessionalTab />
  {/if}
</section>
