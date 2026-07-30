<script lang="ts">
  import type { DashboardStats } from "../../lib/types"
  import { appState } from "../../lib/app-state.svelte"

  let ds: DashboardStats | null = $derived(appState.dashboardStats)
</script>

{#if ds && ds.concepts_mastered.length > 0}
  <div class="card concept-mastered">
    <h3>Conceptos dominados</h3>
    <div class="concept-tags">
      {#each ds.concepts_mastered as c}<span class="tag good">{c}</span>{/each}
    </div>
  </div>
{/if}

{#if ds && ds.concepts_in_progress.length > 0}
  <div class="card concept-progress">
    <h3>En progreso</h3>
    <div class="concept-tags">
      {#each ds.concepts_in_progress as c}<span class="tag progress">{c}</span>{/each}
    </div>
  </div>
{/if}

{#if ds && ds.concepts_needing_practice.length > 0}
  <div class="card concept-practice">
    <h3>Necesitan practica</h3>
    <div class="concept-tags">
      {#each ds.concepts_needing_practice as c}<span class="tag needs-work">{c}</span>{/each}
    </div>
  </div>
{/if}
