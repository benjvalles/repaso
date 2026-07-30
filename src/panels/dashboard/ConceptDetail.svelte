<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

{#if appState.conceptStats.length > 0}
  <div class="card">
    <h3>Detalle por concepto</h3>
    <label class="concept-filter-label">Filtrar concepto
      <select bind:value={appState.conceptFilter}>
        <option value="">Todos</option>
        {#each appState.conceptStats as cs}<option value={cs.concept}>{cs.concept}</option>{/each}
      </select>
    </label>
    {#each appState.filteredConceptStats as cs}
      <div class="concept-detail">
        <div class="concept-name">{cs.concept}</div>
        <div class="concept-bar">
          <div class="concept-fill" style="width: {cs.accuracy_pct}%"></div>
        </div>
        <div class="concept-accuracy">{Math.round(cs.accuracy_pct)}% ({cs.correct_attempts}/{cs.total_attempts})</div>
      </div>
    {/each}
  </div>
{/if}
