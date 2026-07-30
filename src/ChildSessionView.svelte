<script lang="ts">
  import type { CurrentQuestion, SubmitAnswerResponse, ExplanationResponse } from "./lib/types"

  let studentAnswer = $state("")
  let answerInput: HTMLTextAreaElement | undefined = $state(undefined)

  /**
   * @property currentQuestion - Pregunta activa de la sesión
   * @property answerFeedback - Feedback tras responder (null mientras no se haya respondido)
   * @property showExplanation - Controla la visibilidad de la explicación
   * @property explanationData - Datos de la explicación cargada
   * @property isSubmitting - Indica si se está enviando una respuesta
   * @property onSubmit - Callback con la respuesta escrita
   * @property onLoadExplanation - Callback para cargar la explicación
   * @property onNextQuestion - Callback para avanzar a la siguiente pregunta
   * @property onGoHome - Callback para salir de la sesión
   */
  let {
    currentQuestion,
    answerFeedback = null as SubmitAnswerResponse | null,
    showExplanation = false,
    explanationData = null as ExplanationResponse | null,
    isSubmitting = false,
    onSubmit = (_answer: string) => {},
    onLoadExplanation = () => {},
    onNextQuestion = () => {},
    onGoHome = () => {},
  }: {
    currentQuestion: CurrentQuestion
    answerFeedback: SubmitAnswerResponse | null
    showExplanation: boolean
    explanationData: ExplanationResponse | null
    isSubmitting: boolean
    onSubmit: (answer: string) => void
    onLoadExplanation: () => void
    onNextQuestion: () => void
    onGoHome: () => void
  } = $props()

  function handleSubmit(e: Event) {
    e.preventDefault()
    if (!studentAnswer.trim() || isSubmitting) return
    onSubmit(studentAnswer)
    studentAnswer = ""
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === "Enter") {
      e.preventDefault()
      handleSubmit(e)
    }
  }

  $effect(() => { currentQuestion; answerInput?.focus() })
</script>

<section class="session-zone">
  <div class="session-header">
    <button class="secondary small" type="button" onclick={() => onGoHome()}>Salir</button>
    <div class="progress-bar">
      <div class="progress-fill" style="width: {currentQuestion.question_number / currentQuestion.total_questions * 100}%"></div>
    </div>
    <span class="progress-text">{currentQuestion.question_number} / {currentQuestion.total_questions}</span>
  </div>

  <div class="question-card">
    <div class="question-meta">
      <span class="badge concept">{currentQuestion.concept}</span>
      <span class="badge difficulty {currentQuestion.difficulty}">{currentQuestion.difficulty}</span>
    </div>
    <p class="question-text">{currentQuestion.question_text}</p>

    {#if !answerFeedback}
      <form class="answer-form" onsubmit={handleSubmit}>
        <textarea id="answer-input" placeholder="Escribe tu respuesta..." bind:value={studentAnswer} bind:this={answerInput} onkeydown={handleKeydown}></textarea>
        <button type="submit" disabled={!studentAnswer.trim() || isSubmitting}>
          {isSubmitting ? "Enviando..." : "Responder"}
        </button>
      </form>
    {:else}
      <div class="feedback-card {answerFeedback.is_correct ? 'correct' : 'incorrect'}">
        <p class="feedback-title">{answerFeedback.is_correct ? "¡Correcto!" : "Incorrecto"}</p>
        <p class="feedback-text">{answerFeedback.feedback}</p>
        {#if !answerFeedback.is_correct}
          <p class="feedback-correct">Respuesta correcta: <strong>{answerFeedback.correct_answer}</strong></p>
        {/if}
      </div>

      <div class="feedback-actions">
        {#if !answerFeedback.is_correct && !showExplanation}
          <button class="secondary" type="button" onclick={() => onLoadExplanation()}>¿Por qué?</button>
        {/if}
        {#if answerFeedback.next_question}
          <button type="button" onclick={() => onNextQuestion()}>Siguiente pregunta →</button>
        {/if}
      </div>

      {#if showExplanation && explanationData}
        <div class="explanation-card">
          <h3>Explicación</h3>
          <p>{explanationData.explanation}</p>
          {#if explanationData.key_points.length > 0}
            <div class="key-points">
              <strong>Puntos clave:</strong>
              <ul>{#each explanationData.key_points as kp}<li>{kp}</li>{/each}</ul>
            </div>
          {/if}
          {#if explanationData.next_steps.length > 0}
            <div class="next-steps">
              <strong>Siguientes pasos:</strong>
              <ul>{#each explanationData.next_steps as ns}<li>{ns}</li>{/each}</ul>
            </div>
          {/if}
        </div>
      {/if}
    {/if}
  </div>
</section>
