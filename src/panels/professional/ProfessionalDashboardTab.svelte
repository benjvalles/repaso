<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

{#if appState.tutorDashboard}
  <div class="stats-grid professional-stats">
    <div class="stat-card">
      <span class="stat-number">{appState.tutorDashboard.total_students}</span>
      <span class="stat-label">Estudiantes</span>
    </div>
    <div class="stat-card">
      <span class="stat-number">{appState.tutorDashboard.active_assignments}</span>
      <span class="stat-label">Tareas activas</span>
    </div>
    <div class="stat-card">
      <span class="stat-number">{appState.tutorDashboard.reports_generated}</span>
      <span class="stat-label">Reportes</span>
    </div>
  </div>

  {#if appState.tutorDashboard.students.length > 0}
    <div class="card">
      <h3>Mis estudiantes</h3>
      {#each appState.tutorDashboard.students as student}
        <div class="professional-student-row">
          <div class="student-info">
            <strong>{student.display_name}</strong>
            <span>{appState.courseLabel(student.school_year)} · nivel {student.current_level}</span>
          </div>
          <div class="student-stats">
            <span class="accuracy-badge">{Math.round(student.accuracy_pct)}% precision</span>
            {#if student.last_session}
              <span class="last-session">Ultima sesion: {new Date(student.last_session).toLocaleDateString("es-ES")}</span>
            {/if}
          </div>
          <button class="secondary small" type="button" onclick={() => appState.removeStudentFromMe(student.student_id)}>Remover</button>
        </div>
      {/each}
    </div>
  {/if}
{/if}
