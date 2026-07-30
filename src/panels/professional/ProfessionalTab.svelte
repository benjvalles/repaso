<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"

  import ProfessionalDashboardTab from "./ProfessionalDashboardTab.svelte"
  import ProfessionalStudentsTab from "./ProfessionalStudentsTab.svelte"
  import ProfessionalGroupsTab from "./ProfessionalGroupsTab.svelte"
  import ProfessionalAssignmentsTab from "./ProfessionalAssignmentsTab.svelte"
  import ProfessionalReportsTab from "./ProfessionalReportsTab.svelte"
</script>

<div class="professional-header">
  <h2>Zona Profesional</h2>
  <p class="eyebrow">Gestion de estudiantes, tareas y reportes</p>
</div>

<div class="tab-bar professional-subtabs">
  <button class:active={appState.professionalTab === "dashboard"} type="button" onclick={() => appState.professionalTab = "dashboard"}>Resumen</button>
  <button class:active={appState.professionalTab === "students"} type="button" onclick={() => appState.professionalTab = "students"}>Estudiantes</button>
  <button class:active={appState.professionalTab === "groups"} type="button" onclick={() => appState.professionalTab = "groups"}>Grupos</button>
  <button class:active={appState.professionalTab === "assignments"} type="button" onclick={() => appState.professionalTab = "assignments"}>Tareas</button>
  <button class:active={appState.professionalTab === "reports"} type="button" onclick={() => appState.professionalTab = "reports"}>Reportes</button>
</div>

{#if appState.professionalTab === "dashboard"}
  <ProfessionalDashboardTab />
{:else if appState.professionalTab === "students"}
  <ProfessionalStudentsTab />
{:else if appState.professionalTab === "groups"}
  <ProfessionalGroupsTab />
{:else if appState.professionalTab === "assignments"}
  <ProfessionalAssignmentsTab />
{:else if appState.professionalTab === "reports"}
  <ProfessionalReportsTab />
{/if}
