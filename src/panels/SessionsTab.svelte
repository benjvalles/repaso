<script lang="ts">
  import { onMount } from "svelte"
  import { tick } from "svelte"
  import { appState } from "../lib/app-state.svelte"

  let deletingSessionId = $state("")

  onMount(() => {
    if (appState.historyProfileId) {
      appState.loadSessions()
      appState.loadDeletedSessions()
    }
  })
</script>

<div class="card stack">
  <h3>Historial de sesiones</h3>
  <label>Perfil
    <select bind:value={appState.historyProfileId} onchange={() => { appState.loadSessions(); appState.loadDeletedSessions() }}>
      <option value="">Seleccionar...</option>
      {#if appState.status}{#each appState.status.profiles as p}<option value={p.id}>{p.display_name}</option>{/each}{/if}
    </select>
  </label>
</div>
{#each appState.sessionHistory as s}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus -->
  <div class="profile-row" role="button" tabindex="0" onclick={() => appState.loadSessionDetail(s.id)}>
    <div class="session-info">
      <strong>Sesión {s.id.slice(0, 8)}</strong>
      <span>{new Date(s.started_at).toLocaleString("es-ES")}</span>
      <span>{s.questions_answered}/{s.total_questions} · {s.correct_count} correctas · {s.status}</span>
    </div>
    <button class="small danger" type="button" disabled={deletingSessionId === s.id} onclick={async (e) => {
      e.stopPropagation()
      if (!window.confirm("¿Eliminar esta sesión?")) return
      deletingSessionId = s.id
      await tick()
      await appState.deleteSession(s.id)
      deletingSessionId = ""
    }}>
      {#if deletingSessionId === s.id}
        <span class="btn-spinner"></span>
      {:else}
        Eliminar
      {/if}
    </button>
  </div>
{/each}

{#if appState.deletedSessions.length > 0}
  <details class="deleted-profiles">
    <summary>Sesiones eliminadas ({appState.deletedSessions.length})</summary>
    {#each appState.deletedSessions as s (s.id)}
      <div class="profile-row muted">
        <div class="session-info">
          <strong>Sesión {s.id.slice(0, 8)}</strong>
          <span>{new Date(s.started_at).toLocaleString("es-ES")}</span>
          <span>{s.questions_answered}/{s.total_questions} · {s.correct_count} correctas · {s.status} — Eliminada</span>
        </div>
        <button class="secondary" type="button" onclick={() => appState.recoverSession(s.id)}>Recuperar</button>
      </div>
    {/each}
  </details>
{/if}

{#if appState.detailSessionSummary}
  {@const summary = appState.detailSessionSummary}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions, a11y_no_noninteractive_element_interactions -->
  <div class="modal-overlay" role="dialog" tabindex="0" onkeydown={(e) => e.key === "Escape" && appState.closeSessionDetail()} onclick={() => appState.closeSessionDetail()}>
      <div class="modal-content" role="document" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h3>Detalle de sesión</h3>
        <button class="secondary small" type="button" onclick={() => appState.closeSessionDetail()}>Cerrar</button>
      </div>

      <div class="session-info">
        <span>Estado: {summary.session.status}</span>
        <span>Inicio: {new Date(summary.session.started_at).toLocaleString("es-ES")}</span>
        {#if summary.session.ended_at}
          <span>Fin: {new Date(summary.session.ended_at).toLocaleString("es-ES")}</span>
        {/if}
        <span>{summary.session.questions_answered}/{summary.session.total_questions} preguntas · {summary.session.correct_count} correctas · {summary.accuracy_pct.toFixed(0)}% acierto</span>
        {#if summary.total_time_secs > 0}
          <span>Tiempo total: {Math.floor(summary.total_time_secs / 60)}m {summary.total_time_secs % 60}s</span>
        {/if}
      </div>

      <div class="modal-concepts">
        {#if summary.concepts_mastered.length}
          <div class="concepts-section mastered">
            <h3>Conceptos dominados</h3>
            <div class="concept-tags">
              {#each summary.concepts_mastered as c}
                <span class="tag good">{c}</span>
              {/each}
            </div>
          </div>
        {/if}
        {#if summary.concepts_to_practice.length}
          <div class="concepts-section practice">
            <h3>Conceptos a practicar</h3>
            <div class="concept-tags">
              {#each summary.concepts_to_practice as c}
                <span class="tag needs-work">{c}</span>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <div class="questions-list">
        <h3>Preguntas</h3>
        {#each summary.questions as q}
          <div class="review-item" class:correct={q.is_correct === true} class:incorrect={q.is_correct === false}>
            <span class="review-num">#{q.question_number}</span>
            <div class="review-content">
              <div class="review-q">{q.question_text}</div>
              <div class="review-a">
                {#if q.student_answer !== null}
                  <span>Respuesta: <strong>{q.student_answer}</strong></span>
                {:else}
                  <span class="empty">Sin responder</span>
                {/if}
                <span> · Correcta: <strong>{q.correct_answer}</strong></span>
                {#if q.time_spent_secs !== null}
                  <span> · {q.time_spent_secs}s</span>
                {/if}
              </div>
              <div class="review-tags">
                <span class="badge concept">{q.concept}</span>
                <span class="badge difficulty {q.difficulty}">{q.difficulty}</span>
              </div>
              {#if q.explanation}
                <div class="explanation-card">
                  <p>{q.explanation}</p>
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0,0,0,0.3);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 40px 20px;
    z-index: 200;
    overflow-y: auto;
  }
  .modal-content {
    background: #fff;
    border-radius: 24px;
    max-width: 750px;
    width: 100%;
    padding: 24px;
    box-shadow: 0 20px 60px rgba(0,0,0,0.2);
    margin-top: 20px;
  }
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }
  .modal-header h3 {
    margin: 0;
  }
  .session-info {
    display: grid;
    gap: 6px;
    padding: 14px;
    background: #f7efe4;
    border-radius: 14px;
    margin-bottom: 16px;
    font-size: 0.9rem;
  }
  .modal-concepts {
    margin-bottom: 16px;
  }
  .modal-concepts .concepts-section {
    margin-bottom: 10px;
  }
  .questions-list h3 {
    margin-bottom: 10px;
  }
  .review-tags {
    display: flex;
    gap: 6px;
    margin-top: 6px;
  }
  .profile-row {
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  .profile-row:hover {
    background: rgba(47, 125, 128, 0.06);
  }
  .profile-row .session-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .profile-row .session-info strong {
    font-size: 0.95rem;
  }
  .profile-row .session-info span {
    font-size: 0.8rem;
    color: #555;
  }
  .profile-row button.small {
    flex-shrink: 0;
  }
  .btn-spinner {
    display: inline-block;
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: btn-spin 0.6s linear infinite;
  }
  @keyframes btn-spin { to { transform: rotate(360deg); } }
</style>
