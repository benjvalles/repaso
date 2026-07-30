<script lang="ts">
  import type { SessionSummary } from "./lib/types"

  /**
   * @property sessionSummary - Resumen completo de la sesión completada
   * @property childName - Nombre del niño para el saludo personalizado
   * @property onGoHome - Callback para volver a la selección de perfiles
   */
  let {
    sessionSummary,
    childName = "amigo",
    onGoHome = () => {},
  }: {
    sessionSummary: SessionSummary
    childName: string
    onGoHome: () => void
  } = $props()
</script>

<section class="summary-zone">
  <div class="summary-header">
    <p class="eyebrow">Sesion completada</p>
    <h2>¡Buen trabajo, {childName}!</h2>
  </div>

  <div class="stats-grid">
    <div class="stat-card">
      <span class="stat-number">{sessionSummary.session.correct_count}</span>
      <span class="stat-label">Correctas</span>
    </div>
    <div class="stat-card">
      <span class="stat-number">{sessionSummary.session.total_questions - sessionSummary.session.correct_count}</span>
      <span class="stat-label">Por practicar</span>
    </div>
    <div class="stat-card">
      <span class="stat-number">{Math.round(sessionSummary.accuracy_pct)}%</span>
      <span class="stat-label">Precision</span>
    </div>
    <div class="stat-card">
      <span class="stat-number">{sessionSummary.total_time_secs}s</span>
      <span class="stat-label">Tiempo total</span>
    </div>
  </div>

  {#if sessionSummary.concepts_mastered.length > 0}
    <div class="concepts-section mastered">
      <h3>Conceptos que dominas</h3>
      <div class="concept-tags">{#each sessionSummary.concepts_mastered as c}<span class="tag good">{c}</span>{/each}</div>
    </div>
  {/if}

  {#if sessionSummary.concepts_to_practice.length > 0}
    <div class="concepts-section practice">
      <h3>Conceptos para seguir practicando</h3>
      <div class="concept-tags">{#each sessionSummary.concepts_to_practice as c}<span class="tag needs-work">{c}</span>{/each}</div>
    </div>
  {/if}

  <div class="questions-review">
    <h3>Resumen por pregunta</h3>
    {#each sessionSummary.questions as q}
      <div class="review-item {q.is_correct ? 'correct' : 'incorrect'}">
        <span class="review-num">#{q.question_number}</span>
        <div class="review-content">
          <p class="review-q">{q.question_text}</p>
          <p class="review-a">Tu respuesta: <strong>{q.student_answer || "sin respuesta"}</strong>
            {#if !q.is_correct} → Correcta: <strong>{q.correct_answer}</strong>{/if}
          </p>
        </div>
        <span class="review-icon">{q.is_correct ? "✓" : "✗"}</span>
      </div>
    {/each}
  </div>

  <button type="button" onclick={() => onGoHome()}>Volver al inicio</button>
</section>
