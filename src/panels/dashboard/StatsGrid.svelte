<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

{#if appState.dashboardStats}
<div class="stats-grid dashboard-stats">
  <div class="stat-card">
    <span class="stat-number">{appState.dashboardStats.total_sessions}</span>
    <span class="stat-label">Sesiones</span>
  </div>
  <div class="stat-card">
    <span class="stat-number">{Math.round(appState.dashboardStats.overall_accuracy_pct)}%</span>
    <span class="stat-label">Precision global</span>
  </div>
  <div class="stat-card">
    <span class="stat-number">{appState.formatTime(appState.dashboardStats.total_time_secs)}</span>
    <span class="stat-label">Tiempo total</span>
  </div>
  <div class="stat-card">
    <span class="stat-number">{appState.formatTime(Math.round(appState.dashboardStats.avg_time_per_question))}</span>
    <span class="stat-label">Promedio/pregunta</span>
  </div>
</div>
<div class="stats-grid dashboard-stats">
  <div class="stat-card">
    <span class="stat-number">{appState.dashboardStats.total_questions_answered}</span>
    <span class="stat-label">Preguntas</span>
  </div>
  <div class="stat-card">
    <span class="stat-number">{appState.dashboardStats.total_correct}</span>
    <span class="stat-label">Correctas</span>
  </div>
  <div class="stat-card">
    <span class="stat-number">{appState.dashboardStats.concepts_mastered.length}</span>
    <span class="stat-label">Dominados</span>
  </div>
  <div class="stat-card">
    <span class="stat-number">{appState.dashboardStats.concepts_needing_practice.length}</span>
    <span class="stat-label">Practicar</span>
  </div>
</div>
{/if}
