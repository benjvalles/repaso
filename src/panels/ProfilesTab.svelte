<script lang="ts">
  import { appState } from "../lib/app-state.svelte"
</script>

<form class="card stack" onsubmit={(e) => appState.saveProfile(e)}>
  <h3>{appState.profileForm.id ? "Editar perfil" : "Nuevo perfil"}</h3>
  <label>Nombre <input maxlength="40" bind:value={appState.profileForm.display_name} /></label>
  <div class="form-grid">
    <label>Curso <select bind:value={appState.profileForm.school_year}>{#each [1,2,3,4,5,6] as c}<option value={c}>{appState.courseLabel(c)}</option>{/each}</select></label>
    <label>Edad <input type="number" min="6" max="12" placeholder="Opcional" bind:value={appState.profileForm.age} /></label>
  </div>
  <div class="form-grid">
    <label>Nivel <select bind:value={appState.profileForm.level_mode}><option value="automatic">Automatico</option><option value="manual">Manual</option></select></label>
    {#if appState.profileForm.level_mode === "manual"}
      <label>Nivel manual <select bind:value={appState.profileForm.manual_level}>{#each [1,2,3,4,5,6] as l}<option value={l}>{l}</option>{/each}</select></label>
    {/if}
  </div>
  {#if appState.profileForm.level_mode === "manual"}
    <label>Contexto pedagógico para la IA <textarea maxlength="1000" rows="4" bind:value={appState.profileForm.manual_prompt} placeholder="Ej: Tiene dificultades con multiplicaciones. Prioriza tablas y explica trucos para memorizarlas."></textarea></label>
    <p class="muted">No escribas nombres ni datos personales. Describe solo necesidades pedagógicas.</p>
  {/if}
  <div class="row">
    <button type="submit">{appState.profileForm.id ? "Guardar" : "Crear"}</button>
    {#if appState.profileForm.id}<button class="secondary" type="button" onclick={() => appState.profileForm = appState.emptyProfileForm()}>Cancelar</button>{/if}
  </div>
</form>

{#if appState.status}
  {#each appState.status.profiles as p (p.id)}
    <div class="profile-row">
      <div><strong>{p.display_name}</strong><span>{appState.courseLabel(p.school_year)} · nivel {p.current_level}</span></div>
      <div class="row compact">
        <button class="secondary" type="button" onclick={() => appState.editProfile(p)}>Editar</button>
        <button class="danger ghost" type="button" onclick={() => appState.deleteProfile(p)}>Eliminar</button>
      </div>
    </div>
  {/each}
{/if}

{#if appState.deletedProfiles.length > 0}
  <details class="deleted-profiles">
    <summary>Perfiles eliminados ({appState.deletedProfiles.length})</summary>
    {#each appState.deletedProfiles as p (p.id)}
      <div class="profile-row muted">
        <div><strong>{p.display_name}</strong><span>— Eliminado</span></div>
        <button class="secondary" type="button" onclick={() => appState.recoverProfile(p)}>Recuperar</button>
      </div>
    {/each}
  </details>
{/if}

<button class="link-button" type="button" onclick={() => appState.showReset = !appState.showReset}>Reset datos locales</button>
{#if appState.showReset}
  <form class="danger-card" onsubmit={(e) => appState.resetData(e)}>
    <strong>Reset completo</strong>
    <p>Borra PIN y perfiles. Escribe RESET para confirmar.</p>
    <input placeholder="RESET" bind:value={appState.resetPhrase} />
    <button class="danger" type="submit">Borrar todo</button>
  </form>
{/if}
