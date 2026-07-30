<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

<div class="card stack">
  <h3>Dashboard de progreso</h3>
  <label>Perfil
    <select bind:value={appState.dashboardProfileId} onchange={() => appState.loadDashboard()}>
      <option value="">Seleccionar...</option>
      {#if appState.status}{#each appState.status.profiles as p}<option value={p.id}>{p.display_name}</option>{/each}{/if}
    </select>
  </label>
</div>
