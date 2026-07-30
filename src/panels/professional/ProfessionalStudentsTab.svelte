<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

<div class="card">
  <h3>Estudiantes asignados</h3>
  {#if appState.tutorStudents.length === 0}
    <p class="empty">No tienes estudiantes asignados</p>
  {:else}
    {#each appState.tutorStudents as student}
      <div class="profile-row">
        <div>
          <strong>{student.display_name}</strong>
          <span>{appState.courseLabel(student.school_year)} · nivel {student.current_level}</span>
        </div>
        <button class="secondary small" type="button" onclick={() => appState.removeStudentFromMe(student.id)}>Remover</button>
      </div>
    {/each}
  {/if}
</div>

<div class="card">
  <h3>Asignar nuevo estudiante</h3>
  <p class="empty">Selecciona un estudiante existente para asignarlo a tu cuenta</p>
  {#if appState.status}
    {#each appState.status.profiles as p}
      <div class="profile-row">
        <div>
          <strong>{p.display_name}</strong>
          <span>{appState.courseLabel(p.school_year)} · nivel {p.current_level}</span>
        </div>
        <button class="secondary small" type="button" onclick={() => appState.assignStudentToMe(p.id)}>Asignar</button>
      </div>
    {/each}
  {/if}
</div>
