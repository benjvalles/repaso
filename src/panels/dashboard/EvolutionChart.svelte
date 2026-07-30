<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

{#if appState.evolutionData.length > 0}
  <div class="card">
    <h3>Evolucion por sesion</h3>
    <div class="evolution-chart">
      {#each appState.evolutionData as point, i}
        <div class="evolution-bar" style="height: {point.accuracy_pct}%">
          <span class="evolution-label">{Math.round(point.accuracy_pct)}%</span>
          <span class="evolution-date">{new Date(point.started_at).toLocaleDateString("es-ES")}</span>
        </div>
      {/each}
    </div>
  </div>
{/if}
