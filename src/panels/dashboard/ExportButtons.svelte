<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

<div class="row export-buttons">
  <button class="secondary" type="button" onclick={() => appState.exportData("csv")}>Exportar CSV</button>
  <button class="secondary" type="button" onclick={() => appState.exportData("json")}>Exportar JSON</button>
</div>
