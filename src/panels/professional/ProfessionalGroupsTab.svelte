<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

<div class="card stack">
  <h3>Crear grupo</h3>
  <form onsubmit={(e) => appState.createGroup(e)}>
    <label>Nombre del grupo <input maxlength="40" placeholder="Ej: Grupo A, Clase 3B" bind:value={appState.groupForm.name} /></label>
    <button type="submit" disabled={!appState.groupForm.name.trim()}>Crear grupo</button>
  </form>
</div>

{#if appState.studentGroups.length > 0}
  <div class="card">
    <h3>Mis grupos</h3>
    {#each appState.studentGroups as group}
      <div class="profile-row">
        <div>
          <strong>{group.name}</strong>
          <span>Creado: {new Date(group.created_at).toLocaleDateString("es-ES")}</span>
        </div>
      </div>
    {/each}
  </div>
{/if}
