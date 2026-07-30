<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

<div class="card stack">
  <h3>Crear tarea</h3>
  <form onsubmit={(e) => appState.createAssignment(e)}>
    <label>Estudiante
      <select bind:value={appState.assignmentForm.student_id}>
        <option value="">Seleccionar...</option>
        {#each appState.tutorStudents as s}<option value={s.id}>{s.display_name}</option>{/each}
      </select>
    </label>
    <label>Concepto <input maxlength="60" placeholder="Ej: Sumas con carry" bind:value={appState.assignmentForm.concept} /></label>
    <div class="form-grid">
      <label>Dificultad
        <select bind:value={appState.assignmentForm.difficulty}>
          <option value="easy">Facil</option>
          <option value="medium">Media</option>
          <option value="hard">Dificil</option>
        </select>
      </label>
      <label>Fecha limite <input type="date" bind:value={appState.assignmentForm.due_date} /></label>
    </div>
    <button type="submit" disabled={!appState.assignmentForm.student_id || !appState.assignmentForm.concept.trim()}>Crear tarea</button>
  </form>
</div>

{#if appState.assignments.length > 0}
  <div class="card">
    <h3>Tareas activas</h3>
    {#each appState.assignments as a}
      <div class="profile-row">
        <div>
          <strong>{a.concept}</strong>
          <span>Estudiante: {appState.tutorStudents.find(s => s.id === a.student_id)?.display_name || a.student_id} · {a.difficulty} · {a.status}</span>
        </div>
      </div>
    {/each}
  </div>
{/if}
