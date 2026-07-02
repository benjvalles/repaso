<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type LevelMode = "automatic" | "manual";
  type AppView = "loading" | "setup_pin" | "child_select" | "child_session" | "child_summary" | "adult_panel";

  type Profile = {
    id: string;
    display_name: string;
    school_year: number;
    age: number | null;
    level_mode: LevelMode;
    current_level: number;
    created_at: string;
    updated_at: string;
  };

  type LLMConfig = {
    provider: string;
    model: string;
    base_url: string;
    api_key: string;
  };

  type AppStatus = {
    guardian_pin_set: boolean;
    adult_unlocked: boolean;
    profiles: Profile[];
    llm_config: LLMConfig;
  };

  type CurrentQuestion = {
    question_id: string;
    question_text: string;
    question_number: number;
    total_questions: number;
    concept: string;
    difficulty: string;
  };

  type StartSessionResponse = {
    session_id: string;
    total_questions: number;
    first_question: CurrentQuestion | null;
  };

  type SubmitAnswerResponse = {
    is_correct: boolean;
    feedback: string;
    correct_answer: string;
    explanation_needed: boolean;
    next_question: CurrentQuestion | null;
    session_finished: boolean;
  };

  type ExplanationResponse = {
    explanation: string;
    key_points: string[];
    next_steps: string[];
    reformulated_question: string | null;
  };

  type SessionQuestion = {
    id: string;
    session_id: string;
    question_text: string;
    correct_answer: string;
    student_answer: string | null;
    concept: string;
    difficulty: string;
    is_correct: boolean | null;
    explanation: string | null;
    question_number: number;
    time_spent_secs: number | null;
    created_at: string;
    answered_at: string | null;
  };

  type Session = {
    id: string;
    profile_id: string;
    status: string;
    total_questions: number;
    questions_answered: number;
    correct_count: number;
    current_question_index: number;
    started_at: string;
    ended_at: string | null;
  };

  type SessionSummary = {
    session: Session;
    questions: SessionQuestion[];
    concepts_worked: string[];
    concepts_mastered: string[];
    concepts_to_practice: string[];
    accuracy_pct: number;
    avg_time_per_question: number;
    total_time_secs: number;
  };

  type DashboardStats = {
    total_sessions: number;
    total_questions_answered: number;
    total_correct: number;
    overall_accuracy_pct: number;
    total_time_secs: number;
    avg_time_per_question: number;
    concepts_mastered: string[];
    concepts_in_progress: string[];
    concepts_needing_practice: string[];
  };

  type ConceptStat = {
    concept: string;
    total_attempts: number;
    correct_attempts: number;
    accuracy_pct: number;
    last_practiced: string;
  };

  type EvolutionPoint = {
    session_id: string;
    started_at: string;
    accuracy_pct: number;
    questions_answered: number;
    correct_count: number;
  };

  type ExportSessionRow = {
    session_id: string;
    started_at: string;
    ended_at: string | null;
    question_number: number;
    question_text: string;
    concept: string;
    difficulty: string;
    student_answer: string | null;
    correct_answer: string;
    is_correct: boolean | null;
    time_spent_secs: number | null;
  };

  type User = {
    id: string;
    display_name: string;
    role: string;
    created_at: string;
  };

  type StudentGroup = {
    id: string;
    name: string;
    owner_user_id: string;
    created_at: string;
  };

  type TutorStudentInfo = {
    student_id: string;
    display_name: string;
    school_year: number;
    current_level: number;
    last_session: string | null;
    accuracy_pct: number;
  };

  type TutorDashboard = {
    total_students: number;
    active_assignments: number;
    reports_generated: number;
    students: TutorStudentInfo[];
  };

  type Assignment = {
    id: string;
    tutor_user_id: string;
    student_id: string;
    concept: string;
    difficulty: string;
    due_date: string | null;
    status: string;
    created_at: string;
  };

  type Report = {
    id: string;
    tutor_user_id: string;
    student_id: string;
    period: string;
    report_data: string;
    generated_at: string;
  };

  type ProfileForm = {
    id: string | null;
    display_name: string;
    school_year: number;
    age: string;
    level_mode: LevelMode;
    manual_level: number;
  };

  let view = $state<AppView>("loading");
  let status = $state<AppStatus | null>(null);
  let error = $state("");
  let notice = $state("");

  // Child session state
  let selectedProfileId = $state("");
  let sessionId = $state("");
  let currentQuestion = $state<CurrentQuestion | null>(null);
  let studentAnswer = $state("");
  let answerFeedback = $state<SubmitAnswerResponse | null>(null);
  let showExplanation = $state(false);
  let explanationData = $state<ExplanationResponse | null>(null);
  let sessionSummary = $state<SessionSummary | null>(null);
  let questionStartTime = $state(0);
  let isSubmitting = $state(false);

  // Adult panel state
  let adultTab = $state<"profiles" | "llm" | "sessions" | "dashboard" | "professional">("profiles");
  let loginPin = $state("");
  let profileForm = $state<ProfileForm>(emptyProfileForm());
  let llmForm = $state<LLMConfig>({ provider: "ollama", model: "llama3", base_url: "http://localhost:11434", api_key: "" });
  let testResult = $state("");
  let resetPhrase = $state("");
  let showReset = $state(false);
  let sessionHistory = $state<Session[]>([]);
  let historyProfileId = $state("");

  // Dashboard state
  let dashboardProfileId = $state("");
  let dashboardStats = $state<DashboardStats | null>(null);
  let conceptStats = $state<ConceptStat[]>([]);
  let evolutionData = $state<EvolutionPoint[]>([]);
  let conceptFilter = $state("");
  let filteredConceptStats = $state<ConceptStat[]>([]);
  let recentSessions = $state<Session[]>([]);
  let recentLimit = $state(5);

  // Professional layer state
  let currentUser = $state<User | null>(null);
  let users = $state<User[]>([]);
  let studentGroups = $state<StudentGroup[]>([]);
  let tutorDashboard = $state<TutorDashboard | null>(null);
  let tutorStudents = $state<Profile[]>([]);
  let assignments = $state<Assignment[]>([]);
  let reports = $state<Report[]>([]);
  let professionalTab = $state<"dashboard" | "students" | "groups" | "assignments" | "reports">("dashboard");
  let groupForm = $state({ name: "" });
  let assignmentForm = $state({ student_id: "", concept: "", difficulty: "easy", due_date: "" });
  let reportForm = $state({ student_id: "", period: "" });

  onMount(() => { refreshStatus(); });

  function emptyProfileForm(): ProfileForm {
    return { id: null, display_name: "", school_year: 1, age: "", level_mode: "automatic", manual_level: 1 };
  }

  async function refreshStatus() {
    try {
      status = await invoke<AppStatus>("get_app_status");
      llmForm = { ...status.llm_config };
      if (!status.guardian_pin_set) { view = "setup_pin"; }
      else { view = status.profiles.length > 0 ? "child_select" : "adult_panel"; }
    } catch (err) { error = msg(err); view = "child_select"; }
  }

  async function run(action: () => Promise<void>) {
    error = ""; notice = "";
    try { await action(); } catch (err) { error = msg(err); }
  }
  function msg(e: unknown) { return e instanceof Error ? e.message : String(e); }

  // === PIN ===
  async function setupPin(e: Event) {
    e.preventDefault();
    await run(async () => {
      await invoke("setup_guardian_pin", { pin: loginPin });
      loginPin = ""; notice = "PIN configurado"; await refreshStatus();
    });
  }
  async function unlockAdult(e: Event) {
    e.preventDefault();
    await run(async () => {
      const ok = await invoke<boolean>("verify_guardian_pin", { pin: loginPin });
      loginPin = "";
      if (!ok) { error = "PIN incorrecto"; return; }
      view = "adult_panel"; notice = "Zona adulta desbloqueada";
    });
  }
  async function lockAdult() {
    await run(async () => { await invoke("lock_adult_area"); view = "child_select"; });
  }
  async function resetData(e: Event) {
    e.preventDefault();
    await run(async () => {
      await invoke("reset_local_data", { confirmPhrase: resetPhrase });
      resetPhrase = ""; showReset = false; await refreshStatus();
    });
  }

  // === PROFILES ===
  async function saveProfile(e: Event) {
    e.preventDefault();
    await run(async () => {
      const p = { display_name: profileForm.display_name, school_year: Number(profileForm.school_year),
        age: profileForm.age.trim() ? Number(profileForm.age) : null,
        level_mode: profileForm.level_mode,
        manual_level: profileForm.level_mode === "manual" ? Number(profileForm.manual_level) : null };
      if (profileForm.id) { await invoke("update_profile", { request: { ...p, id: profileForm.id } }); notice = "Perfil actualizado"; }
      else { await invoke("create_profile", { request: p }); notice = "Perfil creado"; }
      profileForm = emptyProfileForm(); await refreshStatus();
    });
  }
  function editProfile(p: Profile) {
    profileForm = { id: p.id, display_name: p.display_name, school_year: p.school_year,
      age: p.age === null ? "" : String(p.age), level_mode: p.level_mode, manual_level: p.current_level };
  }
  async function deleteProfile(p: Profile) {
    if (!window.confirm(`Eliminar perfil de ${p.display_name}?`)) return;
    await run(async () => { await invoke("delete_profile", { id: p.id }); notice = "Perfil eliminado"; await refreshStatus(); });
  }
  function courseLabel(c: number) { return `${c}o Primaria`; }

  // === LLM CONFIG ===
  async function saveLLMConfig(e: Event) {
    e.preventDefault();
    await run(async () => {
      await invoke("set_llm_config", { request: llmForm });
      notice = "Configuracion LLM guardada"; await refreshStatus();
    });
  }
  async function testLLM() {
    testResult = ""; error = "";
    try {
      testResult = await invoke<string>("test_llm_connection");
    } catch (err) { error = msg(err); }
  }

  // === SESSIONS ===
  async function loadSessions() {
    if (!historyProfileId) return;
    await run(async () => { sessionHistory = await invoke<Session[]>("list_sessions", { profileId: historyProfileId }); });
  }

  // === DASHBOARD ===
  async function loadDashboard() {
    if (!dashboardProfileId) return;
    await run(async () => {
      dashboardStats = await invoke<DashboardStats>("get_dashboard_stats", { profileId: dashboardProfileId });
      conceptStats = await invoke<ConceptStat[]>("get_concept_stats", { profileId: dashboardProfileId });
      evolutionData = await invoke<EvolutionPoint[]>("get_evolution", { profileId: dashboardProfileId });
      const allSessions = await invoke<Session[]>("list_sessions", { profileId: dashboardProfileId });
      recentSessions = allSessions.filter(s => s.status === "completed").slice(0, recentLimit);
      applyConceptFilter();
    });
  }

  function applyConceptFilter() {
    filteredConceptStats = conceptFilter
      ? conceptStats.filter(c => c.concept === conceptFilter)
      : conceptStats;
  }

  function formatTime(secs: number): string {
    if (secs < 60) return `${secs}s`;
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return s > 0 ? `${m}m ${s}s` : `${m}m`;
  }

  $effect(() => { conceptFilter; applyConceptFilter(); });

  async function exportData(format: "csv" | "json") {
    if (!dashboardProfileId) return;
    await run(async () => {
      const data = await invoke<ExportSessionRow[]>("export_sessions", { profileId: dashboardProfileId });
      let content: string;
      let filename: string;
      let mimeType: string;

      if (format === "json") {
        content = JSON.stringify(data, null, 2);
        filename = `mates_export_${dashboardProfileId}.json`;
        mimeType = "application/json";
      } else {
        const headers = ["session_id","started_at","ended_at","question_number","question_text","concept","difficulty","student_answer","correct_answer","is_correct","time_spent_secs"];
        const csvRows = [headers.join(",")];
        for (const row of data) {
          csvRows.push(headers.map(h => {
            const val = (row as Record<string, unknown>)[h];
            const str = val === null || val === undefined ? "" : String(val);
            return str.includes(",") || str.includes('"') || str.includes("\n") ? `"${str.replace(/"/g, '""')}"` : str;
          }).join(","));
        }
        content = csvRows.join("\n");
        filename = `mates_export_${dashboardProfileId}.csv`;
        mimeType = "text/csv";
      }

      const blob = new Blob([content], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url; a.download = filename; a.click();
      URL.revokeObjectURL(url);
      notice = `Exportado como ${format.toUpperCase()}`;
    });
  }

  // === PROFESSIONAL LAYER ===
  async function loadProfessionalData() {
    if (!currentUser) return;
    await run(async () => {
      tutorDashboard = await invoke<TutorDashboard>("get_tutor_dashboard", { tutorUserId: currentUser!.id });
      tutorStudents = await invoke<Profile[]>("list_tutor_students", { tutorUserId: currentUser!.id });
      assignments = await invoke<Assignment[]>("list_assignments", { tutorUserId: currentUser!.id });
      reports = await invoke<Report[]>("list_reports", { tutorUserId: currentUser!.id });
      studentGroups = await invoke<StudentGroup[]>("list_student_groups", { userId: currentUser!.id });
    });
  }

  async function createGroup(e: Event) {
    e.preventDefault();
    if (!currentUser || !groupForm.name.trim()) return;
    await run(async () => {
      await invoke("create_student_group", { request: { name: groupForm.name.trim() }, userId: currentUser!.id });
      groupForm.name = "";
      notice = "Grupo creado";
      await loadProfessionalData();
    });
  }

  async function createAssignment(e: Event) {
    e.preventDefault();
    if (!currentUser || !assignmentForm.student_id || !assignmentForm.concept.trim()) return;
    await run(async () => {
      await invoke("create_assignment", {
        request: {
          student_id: assignmentForm.student_id,
          concept: assignmentForm.concept.trim(),
          difficulty: assignmentForm.difficulty,
          due_date: assignmentForm.due_date || null,
        },
        tutorUserId: currentUser!.id,
      });
      assignmentForm = { student_id: "", concept: "", difficulty: "easy", due_date: "" };
      notice = "Tarea creada";
      await loadProfessionalData();
    });
  }

  async function generateReportAction(e: Event) {
    e.preventDefault();
    if (!currentUser || !reportForm.student_id || !reportForm.period.trim()) return;
    await run(async () => {
      await invoke("generate_report", {
        request: { student_id: reportForm.student_id, period: reportForm.period.trim() },
        tutorUserId: currentUser!.id,
      });
      reportForm = { student_id: "", period: "" };
      notice = "Reporte generado";
      await loadProfessionalData();
    });
  }

  async function assignStudentToMe(studentId: string) {
    if (!currentUser) return;
    await run(async () => {
      await invoke("assign_student_to_tutor", { tutorUserId: currentUser!.id, request: { student_id: studentId } });
      notice = "Estudiante asignado";
      await loadProfessionalData();
    });
  }

  async function removeStudentFromMe(studentId: string) {
    if (!currentUser) return;
    await run(async () => {
      await invoke("remove_student_from_tutor", { tutorUserId: currentUser!.id, studentId });
      notice = "Asignacion removida";
      await loadProfessionalData();
    });
  }

  // === CHILD SESSION ===
  async function startSession(profileId: string) {
    selectedProfileId = profileId;
    await run(async () => {
      const res = await invoke<StartSessionResponse>("start_session", { profileId });
      sessionId = res.session_id;
      currentQuestion = res.first_question;
      questionStartTime = Date.now();
      view = "child_session";
      studentAnswer = ""; answerFeedback = null; showExplanation = false; explanationData = null;
    });
  }

  async function submitAnswer(e: Event) {
    e.preventDefault();
    if (!studentAnswer.trim() || isSubmitting) return;
    isSubmitting = true;
    await run(async () => {
      const timeSpent = Math.round((Date.now() - questionStartTime) / 1000);
      const res = await invoke<SubmitAnswerResponse>("submit_answer", {
        request: { session_id: sessionId, question_id: currentQuestion!.question_id, answer: studentAnswer.trim(), time_spent_secs: timeSpent }
      });
      answerFeedback = res;
      if (res.session_finished) {
        const summary = await invoke<SessionSummary>("end_session", { sessionId });
        sessionSummary = summary;
        view = "child_summary";
      }
    });
    isSubmitting = false;
  }

  async function loadExplanation() {
    if (!currentQuestion) return;
    await run(async () => {
      explanationData = await invoke<ExplanationResponse>("get_explanation", { questionId: currentQuestion!.question_id });
      showExplanation = true;
    });
  }

  function nextQuestion() {
    if (!answerFeedback?.next_question) return;
    currentQuestion = answerFeedback.next_question;
    answerFeedback = null; showExplanation = false; explanationData = null;
    studentAnswer = ""; questionStartTime = Date.now();
  }

  function goHome() { view = "child_select"; currentQuestion = null; sessionId = ""; sessionSummary = null; }
  function courseForAge(a: number) { return Math.min(6, Math.max(1, a - 5)); }
</script>

<main class="shell">
  <section class="app-frame">
    <header class="topbar">
      <div><p class="eyebrow">Mates</p><h1>Repaso de matematicas</h1></div>
      {#if view !== "child_session" && view !== "child_summary"}
        {#if status?.guardian_pin_set}
          <button class="secondary" type="button" onclick={() => view = "adult_panel"}>Zona adulta</button>
        {/if}
      {/if}
    </header>

    {#if error}<p class="alert error">{error}</p>{/if}
    {#if notice}<p class="alert notice">{notice}</p>{/if}

    {#if view === "loading"}
      <section class="card"><p>Cargando...</p></section>

    {:else if view === "setup_pin"}
      <section class="grid two-columns">
        <div class="panel intro-panel">
          <p class="eyebrow">Primer arranque</p>
          <h2>Configura el PIN adulto</h2>
          <p>Este PIN protege perfiles y configuracion. Se guarda con hash, nunca en claro.</p>
        </div>
        <form class="card" onsubmit={setupPin}>
          <label for="setup-pin">PIN de 4 a 6 digitos</label>
          <input id="setup-pin" type="password" inputmode="numeric" bind:value={loginPin} />
          <button type="submit">Guardar PIN</button>
        </form>
      </section>

    {:else if view === "child_select"}
      <section class="panel child-zone">
        <p class="eyebrow">Zona infantil</p>
        <h2>Elige tu perfil</h2>
        {#if !status || status.profiles.length === 0}
          <p class="empty">Todavia no hay perfiles.</p>
        {:else}
          <div class="profile-grid">
            {#each status.profiles as p}
              <button class="profile-card" type="button" onclick={() => startSession(p.id)}>
                <strong>{p.display_name}</strong>
                <span>{courseLabel(p.school_year)} · nivel {p.current_level}</span>
              </button>
            {/each}
          </div>
        {/if}
      </section>

    {:else if view === "child_session" && currentQuestion}
      <section class="session-zone">
        <div class="session-header">
          <button class="secondary small" type="button" onclick={goHome}>Salir</button>
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
            <form class="answer-form" onsubmit={submitAnswer}>
              <input id="answer-input" placeholder="Escribe tu respuesta..." bind:value={studentAnswer} autofocus />
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
                <button class="secondary" type="button" onclick={loadExplanation}>¿Por qué?</button>
              {/if}
              {#if answerFeedback.next_question}
                <button type="button" onclick={nextQuestion}>Siguiente pregunta →</button>
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

    {:else if view === "child_summary" && sessionSummary}
      <section class="summary-zone">
        <div class="summary-header">
          <p class="eyebrow">Sesion completada</p>
          <h2>¡Buen trabajo, {status?.profiles.find(p => p.id === selectedProfileId)?.display_name || "amigo"}!</h2>
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

        <button type="button" onclick={goHome}>Volver al inicio</button>
      </section>

    {:else if view === "adult_panel"}
      <section class="panel adult-zone">
        <div class="adult-header">
          <h2>Zona adulta</h2>
          <button class="secondary" type="button" onclick={lockAdult}>Bloquear</button>
        </div>

        <div class="tab-bar">
          <button class:active={adultTab === "profiles"} type="button" onclick={() => adultTab = "profiles"}>Perfiles</button>
          <button class:active={adultTab === "llm"} type="button" onclick={() => adultTab = "llm"}>IA / LLM</button>
          <button class:active={adultTab === "sessions"} type="button" onclick={() => { adultTab = "sessions"; if (historyProfileId) loadSessions(); }}>Historial</button>
          <button class:active={adultTab === "dashboard"} type="button" onclick={() => { adultTab = "dashboard"; if (dashboardProfileId) loadDashboard(); }}>Dashboard</button>
          <button class:active={adultTab === "professional"} type="button" onclick={() => { adultTab = "professional"; loadProfessionalData(); }}>Profesional</button>
        </div>

        {#if adultTab === "profiles"}
          <form class="card stack" onsubmit={saveProfile}>
            <h3>{profileForm.id ? "Editar perfil" : "Nuevo perfil"}</h3>
            <label>Nombre <input maxlength="40" bind:value={profileForm.display_name} /></label>
            <div class="form-grid">
              <label>Curso <select bind:value={profileForm.school_year}>{#each [1,2,3,4,5,6] as c}<option value={c}>{courseLabel(c)}</option>{/each}</select></label>
              <label>Edad <input type="number" min="6" max="12" placeholder="Opcional" bind:value={profileForm.age} /></label>
            </div>
            <div class="form-grid">
              <label>Nivel <select bind:value={profileForm.level_mode}><option value="automatic">Automatico</option><option value="manual">Manual</option></select></label>
              {#if profileForm.level_mode === "manual"}
                <label>Nivel manual <select bind:value={profileForm.manual_level}>{#each [1,2,3,4,5,6] as l}<option value={l}>{l}</option>{/each}</select></label>
              {/if}
            </div>
            <div class="row">
              <button type="submit">{profileForm.id ? "Guardar" : "Crear"}</button>
              {#if profileForm.id}<button class="secondary" type="button" onclick={() => profileForm = emptyProfileForm()}>Cancelar</button>{/if}
            </div>
          </form>

          {#if status}
            {#each status.profiles as p}
              <div class="profile-row">
                <div><strong>{p.display_name}</strong><span>{courseLabel(p.school_year)} · nivel {p.current_level}</span></div>
                <div class="row compact">
                  <button class="secondary" type="button" onclick={() => editProfile(p)}>Editar</button>
                  <button class="danger ghost" type="button" onclick={() => deleteProfile(p)}>Eliminar</button>
                </div>
              </div>
            {/each}
          {/if}

          <button class="link-button" type="button" onclick={() => showReset = !showReset}>Reset datos locales</button>
          {#if showReset}
            <form class="danger-card" onsubmit={resetData}>
              <strong>Reset completo</strong>
              <p>Borra PIN y perfiles. Escribe RESET para confirmar.</p>
              <input placeholder="RESET" bind:value={resetPhrase} />
              <button class="danger" type="submit">Borrar todo</button>
            </form>
          {/if}

        {:else if adultTab === "llm"}
          <form class="card stack" onsubmit={saveLLMConfig}>
            <h3>Configuracion LLM</h3>
            <label>Proveedor
              <select bind:value={llmForm.provider}>
                <option value="ollama">Ollama (local)</option>
                <option value="gemini">Google Gemini</option>
                <option value="openai">OpenAI Compatible</option>
              </select>
            </label>
            <label>Modelo <input bind:value={llmForm.model} placeholder="ej: llama3, gemini-1.5-flash" /></label>
            <label>URL Base <input bind:value={llmForm.base_url} placeholder="http://localhost:11434" /></label>
            {#if llmForm.provider !== "ollama"}
              <label>API Key <input type="password" bind:value={llmForm.api_key} placeholder="API key" /></label>
            {/if}
            <div class="row">
              <button type="submit">Guardar</button>
              <button class="secondary" type="button" onclick={testLLM}>Probar conexion</button>
            </div>
          </form>
          {#if testResult}<p class="alert notice">{testResult}</p>{/if}

        {:else if adultTab === "sessions"}
          <div class="card stack">
            <h3>Historial de sesiones</h3>
            <label>Perfil
              <select bind:value={historyProfileId} onchange={loadSessions}>
                <option value="">Seleccionar...</option>
                {#if status}{#each status.profiles as p}<option value={p.id}>{p.display_name}</option>{/each}{/if}
              </select>
            </label>
          </div>
          {#each sessionHistory as s}
            <div class="profile-row">
              <div>
                <strong>Sesion {s.id.slice(0, 8)}</strong>
                <span>{s.questions_answered}/{s.total_questions} · {s.correct_count} correctas · {s.status}</span>
              </div>
            </div>
          {/each}

        {:else if adultTab === "dashboard"}
          <div class="card stack">
            <h3>Dashboard de progreso</h3>
            <label>Perfil
              <select bind:value={dashboardProfileId} onchange={loadDashboard}>
                <option value="">Seleccionar...</option>
                {#if status}{#each status.profiles as p}<option value={p.id}>{p.display_name}</option>{/each}{/if}
              </select>
            </label>
          </div>

          {#if dashboardStats}
            <div class="stats-grid dashboard-stats">
              <div class="stat-card">
                <span class="stat-number">{dashboardStats.total_sessions}</span>
                <span class="stat-label">Sesiones</span>
              </div>
              <div class="stat-card">
                <span class="stat-number">{Math.round(dashboardStats.overall_accuracy_pct)}%</span>
                <span class="stat-label">Precision global</span>
              </div>
              <div class="stat-card">
                <span class="stat-number">{formatTime(dashboardStats.total_time_secs)}</span>
                <span class="stat-label">Tiempo total</span>
              </div>
              <div class="stat-card">
                <span class="stat-number">{formatTime(Math.round(dashboardStats.avg_time_per_question))}</span>
                <span class="stat-label">Promedio/pregunta</span>
              </div>
            </div>
            <div class="stats-grid dashboard-stats">
              <div class="stat-card">
                <span class="stat-number">{dashboardStats.total_questions_answered}</span>
                <span class="stat-label">Preguntas</span>
              </div>
              <div class="stat-card">
                <span class="stat-number">{dashboardStats.total_correct}</span>
                <span class="stat-label">Correctas</span>
              </div>
              <div class="stat-card">
                <span class="stat-number">{dashboardStats.concepts_mastered.length}</span>
                <span class="stat-label">Dominados</span>
              </div>
              <div class="stat-card">
                <span class="stat-number">{dashboardStats.concepts_needing_practice.length}</span>
                <span class="stat-label">Practicar</span>
              </div>
            </div>

            {#if dashboardStats.concepts_mastered.length > 0}
              <div class="card concept-mastered">
                <h3>Conceptos dominados</h3>
                <div class="concept-tags">
                  {#each dashboardStats.concepts_mastered as c}<span class="tag good">{c}</span>{/each}
                </div>
              </div>
            {/if}

            {#if dashboardStats.concepts_in_progress.length > 0}
              <div class="card concept-progress">
                <h3>En progreso</h3>
                <div class="concept-tags">
                  {#each dashboardStats.concepts_in_progress as c}<span class="tag progress">{c}</span>{/each}
                </div>
              </div>
            {/if}

            {#if dashboardStats.concepts_needing_practice.length > 0}
              <div class="card concept-practice">
                <h3>Necesitan practica</h3>
                <div class="concept-tags">
                  {#each dashboardStats.concepts_needing_practice as c}<span class="tag needs-work">{c}</span>{/each}
                </div>
              </div>
            {/if}

            {#if conceptStats.length > 0}
              <div class="card">
                <h3>Detalle por concepto</h3>
                <label class="concept-filter-label">Filtrar concepto
                  <select bind:value={conceptFilter}>
                    <option value="">Todos</option>
                    {#each conceptStats as cs}<option value={cs.concept}>{cs.concept}</option>{/each}
                  </select>
                </label>
                {#each filteredConceptStats as cs}
                  <div class="concept-detail">
                    <div class="concept-name">{cs.concept}</div>
                    <div class="concept-bar">
                      <div class="concept-fill" style="width: {cs.accuracy_pct}%"></div>
                    </div>
                    <div class="concept-accuracy">{Math.round(cs.accuracy_pct)}% ({cs.correct_attempts}/{cs.total_attempts})</div>
                  </div>
                {/each}
              </div>
            {/if}

            {#if evolutionData.length > 0}
              <div class="card">
                <h3>Evolucion por sesion</h3>
                <div class="evolution-chart">
                  {#each evolutionData as point, i}
                    <div class="evolution-bar" style="height: {point.accuracy_pct}%">
                      <span class="evolution-label">{Math.round(point.accuracy_pct)}%</span>
                      <span class="evolution-date">{new Date(point.started_at).toLocaleDateString("es-ES")}</span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            {#if recentSessions.length > 0}
              <div class="card">
                <h3>Sesiones recientes</h3>
                <div class="recent-sessions-list">
                  {#each recentSessions as s}
                    <div class="recent-session-row">
                      <div class="recent-session-date">{new Date(s.started_at).toLocaleDateString("es-ES")}</div>
                      <div class="recent-session-accuracy">
                        <div class="recent-session-bar">
                          <div class="recent-session-fill" style="width: {s.total_questions > 0 ? (s.correct_count / s.total_questions * 100) : 0}%"></div>
                        </div>
                        <span>{s.correct_count}/{s.total_questions} · {s.total_questions > 0 ? Math.round(s.correct_count / s.total_questions * 100) : 0}%</span>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            <div class="row export-buttons">
              <button class="secondary" type="button" onclick={() => exportData("csv")}>Exportar CSV</button>
              <button class="secondary" type="button" onclick={() => exportData("json")}>Exportar JSON</button>
            </div>
          {/if}

        {:else if adultTab === "professional"}
          <div class="professional-header">
            <h2>Zona Profesional</h2>
            <p class="eyebrow">Gestion de estudiantes, tareas y reportes</p>
          </div>

          <div class="tab-bar professional-subtabs">
            <button class:active={professionalTab === "dashboard"} type="button" onclick={() => professionalTab = "dashboard"}>Resumen</button>
            <button class:active={professionalTab === "students"} type="button" onclick={() => professionalTab = "students"}>Estudiantes</button>
            <button class:active={professionalTab === "groups"} type="button" onclick={() => professionalTab = "groups"}>Grupos</button>
            <button class:active={professionalTab === "assignments"} type="button" onclick={() => professionalTab = "assignments"}>Tareas</button>
            <button class:active={professionalTab === "reports"} type="button" onclick={() => professionalTab = "reports"}>Reportes</button>
          </div>

          {#if professionalTab === "dashboard"}
            {#if tutorDashboard}
              <div class="stats-grid professional-stats">
                <div class="stat-card">
                  <span class="stat-number">{tutorDashboard.total_students}</span>
                  <span class="stat-label">Estudiantes</span>
                </div>
                <div class="stat-card">
                  <span class="stat-number">{tutorDashboard.active_assignments}</span>
                  <span class="stat-label">Tareas activas</span>
                </div>
                <div class="stat-card">
                  <span class="stat-number">{tutorDashboard.reports_generated}</span>
                  <span class="stat-label">Reportes</span>
                </div>
              </div>

              {#if tutorDashboard.students.length > 0}
                <div class="card">
                  <h3>Mis estudiantes</h3>
                  {#each tutorDashboard.students as student}
                    <div class="professional-student-row">
                      <div class="student-info">
                        <strong>{student.display_name}</strong>
                        <span>{courseLabel(student.school_year)} · nivel {student.current_level}</span>
                      </div>
                      <div class="student-stats">
                        <span class="accuracy-badge">{Math.round(student.accuracy_pct)}% precision</span>
                        {#if student.last_session}
                          <span class="last-session">Ultima sesion: {new Date(student.last_session).toLocaleDateString("es-ES")}</span>
                        {/if}
                      </div>
                      <button class="secondary small" type="button" onclick={() => removeStudentFromMe(student.student_id)}>Remover</button>
                    </div>
                  {/each}
                </div>
              {/if}
            {/if}

          {:else if professionalTab === "students"}
            <div class="card">
              <h3>Estudiantes asignados</h3>
              {#if tutorStudents.length === 0}
                <p class="empty">No tienes estudiantes asignados</p>
              {:else}
                {#each tutorStudents as student}
                  <div class="profile-row">
                    <div>
                      <strong>{student.display_name}</strong>
                      <span>{courseLabel(student.school_year)} · nivel {student.current_level}</span>
                    </div>
                    <button class="secondary small" type="button" onclick={() => removeStudentFromMe(student.id)}>Remover</button>
                  </div>
                {/each}
              {/if}
            </div>

            <div class="card">
              <h3>Asignar nuevo estudiante</h3>
              <p class="empty">Selecciona un estudiante existente para asignarlo a tu cuenta</p>
              {#if status}
                {#each status.profiles as p}
                  <div class="profile-row">
                    <div>
                      <strong>{p.display_name}</strong>
                      <span>{courseLabel(p.school_year)} · nivel {p.current_level}</span>
                    </div>
                    <button class="secondary small" type="button" onclick={() => assignStudentToMe(p.id)}>Asignar</button>
                  </div>
                {/each}
              {/if}
            </div>

          {:else if professionalTab === "groups"}
            <div class="card stack">
              <h3>Crear grupo</h3>
              <form onsubmit={createGroup}>
                <label>Nombre del grupo <input maxlength="40" placeholder="Ej: Grupo A, Clase 3B" bind:value={groupForm.name} /></label>
                <button type="submit" disabled={!groupForm.name.trim()}>Crear grupo</button>
              </form>
            </div>

            {#if studentGroups.length > 0}
              <div class="card">
                <h3>Mis grupos</h3>
                {#each studentGroups as group}
                  <div class="profile-row">
                    <div>
                      <strong>{group.name}</strong>
                      <span>Creado: {new Date(group.created_at).toLocaleDateString("es-ES")}</span>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}

          {:else if professionalTab === "assignments"}
            <div class="card stack">
              <h3>Crear tarea</h3>
              <form onsubmit={createAssignment}>
                <label>Estudiante
                  <select bind:value={assignmentForm.student_id}>
                    <option value="">Seleccionar...</option>
                    {#each tutorStudents as s}<option value={s.id}>{s.display_name}</option>{/each}
                  </select>
                </label>
                <label>Concepto <input maxlength="60" placeholder="Ej: Sumas con carry" bind:value={assignmentForm.concept} /></label>
                <div class="form-grid">
                  <label>Dificultad
                    <select bind:value={assignmentForm.difficulty}>
                      <option value="easy">Facil</option>
                      <option value="medium">Media</option>
                      <option value="hard">Dificil</option>
                    </select>
                  </label>
                  <label>Fecha limite <input type="date" bind:value={assignmentForm.due_date} /></label>
                </div>
                <button type="submit" disabled={!assignmentForm.student_id || !assignmentForm.concept.trim()}>Crear tarea</button>
              </form>
            </div>

            {#if assignments.length > 0}
              <div class="card">
                <h3>Tareas activas</h3>
                {#each assignments as a}
                  <div class="profile-row">
                    <div>
                      <strong>{a.concept}</strong>
                      <span>Estudiante: {tutorStudents.find(s => s.id === a.student_id)?.display_name || a.student_id} · {a.difficulty} · {a.status}</span>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}

          {:else if professionalTab === "reports"}
            <div class="card stack">
              <h3>Generar reporte</h3>
              <form onsubmit={generateReportAction}>
                <label>Estudiante
                  <select bind:value={reportForm.student_id}>
                    <option value="">Seleccionar...</option>
                    {#each tutorStudents as s}<option value={s.id}>{s.display_name}</option>{/each}
                  </select>
                </label>
                <label>Periodo <input maxlength="30" placeholder="Ej: Enero 2024, Trimestre 1" bind:value={reportForm.period} /></label>
                <button type="submit" disabled={!reportForm.student_id || !reportForm.period.trim()}>Generar reporte</button>
              </form>
            </div>

            {#if reports.length > 0}
              <div class="card">
                <h3>Reportes generados</h3>
                {#each reports as r}
                  <div class="profile-row">
                    <div>
                      <strong>{r.period}</strong>
                      <span>Estudiante: {tutorStudents.find(s => s.id === r.student_id)?.display_name || r.student_id} · {new Date(r.generated_at).toLocaleDateString("es-ES")}</span>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          {/if}
        {/if}
      </section>
    {/if}
  </section>
</main>
