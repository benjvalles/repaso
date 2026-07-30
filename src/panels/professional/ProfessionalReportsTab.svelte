<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

<div class="card stack">
  <h3>Generar reporte</h3>
  <form onsubmit={(e) => appState.generateReportAction(e)}>
    <label>Estudiante
      <select bind:value={appState.reportForm.student_id}>
        <option value="">Seleccionar...</option>
        {#each appState.tutorStudents as s}<option value={s.id}>{s.display_name}</option>{/each}
      </select>
    </label>
    <label>Periodo <input maxlength="30" placeholder="Ej: Enero 2024, Trimestre 1" bind:value={appState.reportForm.period} /></label>
    <button type="submit" disabled={!appState.reportForm.student_id || !appState.reportForm.period.trim()}>Generar reporte</button>
  </form>
</div>

{#if appState.reports.length > 0}
  <div class="card">
    <h3>Reportes generados</h3>
    {#each appState.reports as r}
      <div class="profile-row">
        <div>
          <strong>{r.period}</strong>
          <span>Estudiante: {appState.tutorStudents.find(s => s.id === r.student_id)?.display_name || r.student_id} · {new Date(r.generated_at).toLocaleDateString("es-ES")}</span>
        </div>
      </div>
    {/each}
  </div>
{/if}
