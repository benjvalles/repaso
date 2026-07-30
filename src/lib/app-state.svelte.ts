import { invoke } from "@tauri-apps/api/core"
import type {
  AppView, AppStatus, Profile, LLMConfig, ProfileForm,
  CurrentQuestion, StartSessionResponse, SubmitAnswerResponse, ExplanationResponse,
  Session, SessionSummary,
  DashboardStats, ConceptStat, EvolutionPoint, ExportSessionRow,
  User, StudentGroup, TutorDashboard,
  Assignment, Report, CloudStatus, SyncResult,
} from "./types"
import { emptyProfileForm, courseLabel, courseForAge, formatTime, msg } from "./helpers"

class AppState {
  // =========== VIEW STATE ===========
  view = $state<AppView>("loading")
  status = $state<AppStatus | null>(null)
  error = $state("")
  notice = $state("")
  pendingServerRequests = $state(0)
  isWaitingForServer = $derived(this.pendingServerRequests > 0)

  // =========== CHILD SESSION STATE ===========
  selectedProfileId = $state("")
  sessionId = $state("")
  currentQuestion = $state<CurrentQuestion | null>(null)
  studentAnswer = $state("")
  answerFeedback = $state<SubmitAnswerResponse | null>(null)
  showExplanation = $state(false)
  explanationData = $state<ExplanationResponse | null>(null)
  sessionSummary = $state<SessionSummary | null>(null)
  questionStartTime = $state(0)
  isSubmitting = $state(false)

  // =========== ADULT PANEL STATE ===========
  adultTab = $state<"profiles" | "llm" | "sessions" | "dashboard" | "cloud" | "professional">("profiles")
  loginPin = $state("")
  profileForm = $state<ProfileForm>({
    id: null, display_name: "", school_year: 1, age: "", level_mode: "automatic", manual_level: 1, manual_prompt: "",
  })
  llmForm = $state<LLMConfig>({
    provider: "ollama", model: "llama3", base_url: "http://localhost:11434", api_key: "",
  })
  testResult = $state("")
  resetPhrase = $state("")
  showReset = $state(false)
  sessionHistory = $state<Session[]>([])
  historyProfileId = $state("")
  detailSessionSummary = $state<SessionSummary | null>(null)

  // =========== DELETED PROFILES STATE ===========
  deletedProfiles = $state<Profile[]>([])

  // =========== DELETED SESSIONS STATE ===========
  deletedSessions = $state<Session[]>([])

  // =========== DASHBOARD STATE ===========
  dashboardProfileId = $state("")
  dashboardStats = $state<DashboardStats | null>(null)
  conceptStats = $state<ConceptStat[]>([])
  evolutionData = $state<EvolutionPoint[]>([])
  conceptFilter = $state("")
  filteredConceptStats = $derived(
    this.conceptFilter
      ? this.conceptStats.filter(c => c.concept === this.conceptFilter)
      : this.conceptStats
  )
  recentSessions = $state<Session[]>([])
  recentLimit = $state(5)

  // =========== PROFESSIONAL LAYER STATE ===========
  currentUser = $state<User | null>(null)
  users = $state<User[]>([])
  studentGroups = $state<StudentGroup[]>([])
  tutorDashboard = $state<TutorDashboard | null>(null)
  tutorStudents = $state<Profile[]>([])
  assignments = $state<Assignment[]>([])
  reports = $state<Report[]>([])
  professionalTab = $state<"dashboard" | "students" | "groups" | "assignments" | "reports">("dashboard")
  groupForm = $state({ name: "" })
  assignmentForm = $state({ student_id: "", concept: "", difficulty: "easy", due_date: "" })
  reportForm = $state({ student_id: "", period: "" })

  // =========== CLOUD / BASEROW STATE ===========
  cloudStatus = $state<CloudStatus>({ connected: false, user_name: null, email: null, last_sync: null, auto_login: false, email_verified: false })
  cloudForm = $state({
    mode: 'login' as 'login' | 'register',
    name: '',
    email: '',
    password: '',
    confirmPassword: '',
    consent: false,
  })

  // =========== HELPERS ===========
  emptyProfileForm = emptyProfileForm
  courseLabel = courseLabel
  courseForAge = courseForAge
  formatTime = formatTime

  // =========== SERVER UTILITIES ===========
  /**
   * Ejecuta una acción incrementando un contador de peticiones pendientes,
   * lo que activa el spinner global. Espera 250ms antes de ejecutar para
   * evitar parpadeos en operaciones rápidas.
   * @param action - Función asíncrona a ejecutar
   */
  withServerWait = async (action: () => Promise<void>) => {
    console.log("[spinner] start", this.pendingServerRequests)
    this.pendingServerRequests++
    await new Promise((resolve) => setTimeout(resolve, 250))
    try {
      await action()
    } finally {
      this.pendingServerRequests--
      console.log("[spinner] end", this.pendingServerRequests)
    }
  }

  /**
   * Envuelve withServerWait con gestión de errores: limpia error/notice
   * previos y captura cualquier excepción en `this.error`.
   * @param action - Función asíncrona a ejecutar
   */
  run = async (action: () => Promise<void>) => {
    this.error = ""
    this.notice = ""
    await this.withServerWait(async () => {
      try {
        await action()
      } catch (err) {
        this.error = msg(err)
      }
    })
  }

  // =========== PIN ===========
  /**
   * Recarga el estado completo de la app desde el backend: verifica si el
   * PIN está configurado, lista los perfiles, y redirige a la vista adecuada.
   */
  refreshStatus = async () => {
    await this.withServerWait(async () => {
      try {
        this.status = await invoke<AppStatus>("get_app_status")
        this.cloudStatus = { ...this.status.cloud_status }
        this.llmForm = { ...this.status.llm_config }
        if (!this.status.guardian_pin_set) {
          this.view = "setup_pin"
        } else if (this.status.profiles.length === 0 && this.status.adult_unlocked) {
          this.view = "adult_panel"
        } else {
          this.view = "child_select"
        }
      } catch (err) {
        this.error = msg(err)
        this.view = "child_select"
      }
    })
  }

  /**
   * Establece el PIN de guardián por primera vez.
   * @param pin - PIN de 4-6 dígitos (opcional; usa `loginPin` del store si no se pasa)
   */
  setupPin = async (pin?: string) => {
    const value = pin ?? this.loginPin
    await this.run(async () => {
      await invoke("setup_guardian_pin", { pin: value })
      this.loginPin = ""
      this.notice = "PIN configurado"
      await this.refreshStatus()
    })
  }

  /**
   * Verifica el PIN ingresado y desbloquea el panel adulto si es correcto.
   * @param pin - PIN a verificar (opcional; usa `loginPin` del store si no se pasa)
   */
  unlockAdult = async (pin?: string) => {
    const value = pin ?? this.loginPin
    await this.run(async () => {
      const ok = await invoke<boolean>("verify_guardian_pin", { pin: value })
      this.loginPin = ""
      if (!ok) {
        this.error = "PIN incorrecto"
        return
      }
      this.view = "adult_panel"
      this.notice = "Zona adulta desbloqueada"
    })
  }

  /** Bloquea el panel adulto y vuelve a la selección de perfiles */
  lockAdult = async () => {
    await this.run(async () => {
      await invoke("lock_adult_area")
      this.view = "child_select"
    })
  }

  /**
   * Borra todos los datos locales (PIN, perfiles, sesiones) tras confirmar
   * con la frase "RESET".
   * @param e - Evento del formulario
   */
  resetData = async (e: Event) => {
    e.preventDefault()
    await this.run(async () => {
      await invoke("reset_local_data", { confirmPhrase: this.resetPhrase })
      this.resetPhrase = ""
      this.showReset = false
      await this.refreshStatus()
    })
  }

  // =========== PROFILES ===========
  /**
   * Crea o actualiza un perfil según si `profileForm.id` está presente.
   * @param e - Evento del formulario
   */
  saveProfile = async (e: Event) => {
    e.preventDefault()
    await this.run(async () => {
      const p = {
        display_name: this.profileForm.display_name,
        school_year: Number(this.profileForm.school_year),
        age: this.profileForm.age?.toInt?.() || null,
        level_mode: this.profileForm.level_mode,
        manual_level: this.profileForm.level_mode === "manual" ? Number(this.profileForm.manual_level) : null,
        manual_prompt: this.profileForm.level_mode === "manual" ? this.profileForm.manual_prompt : null,
      }
      if (this.profileForm.id) {
        await invoke("update_profile", { request: { ...p, id: this.profileForm.id } })
        this.notice = "Perfil actualizado"
      } else {
        await invoke("create_profile", { request: p })
        this.notice = "Perfil creado"
      }
      this.profileForm = this.emptyProfileForm()
      if (this.cloudStatus.connected && this.cloudStatus.email_verified) {
        const result = await invoke<SyncResult>("sync_all_data")
        const errs = result.errors.length > 0 ? ` Errores: ${result.errors.join(", ")}` : ""
        this.notice += ` y sincronizado (config: ${result.config_synced}, perfiles: ${result.profiles_synced}, sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced})${errs}`
      }
      await this.refreshStatus()
    })
  }

  /**
   * Carga los datos de un perfil en el formulario de edición.
   * @param p - Perfil a editar
   */
  editProfile = (p: Profile) => {
    this.profileForm = {
      id: p.id, display_name: p.display_name, school_year: p.school_year,
      age: p.age === null ? "" : String(p.age), level_mode: p.level_mode, manual_level: p.current_level, manual_prompt: p.manual_prompt ?? "",
    }
  }

  /**
   * Elimina un perfil tras confirmación del usuario.
   * Sincroniza automaticamente con la nube si hay sesion activa.
   * @param p - Perfil a eliminar
   */
  deleteProfile = async (p: Profile) => {
    if (!window.confirm(`Eliminar perfil de ${p.display_name}?`)) return
    await this.run(async () => {
      await invoke("delete_profile", { id: p.id })
      if (this.cloudStatus.connected && this.cloudStatus.email_verified) {
        const result = await invoke<SyncResult>("sync_all_data")
        const errs = result.errors.length > 0 ? ` Errores: ${result.errors.join(", ")}` : ""
        this.notice = `Perfil eliminado y sincronizado (config: ${result.config_synced}, perfiles: ${result.profiles_synced}, sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced})${errs}`
      } else {
        this.notice = "Perfil eliminado"
      }
      await this.refreshStatus()
      await this.loadDeletedProfiles()
    })
  }

  /**
   * Carga la lista de perfiles eliminados (soft-delete) desde el backend.
   */
  loadDeletedProfiles = async () => {
    try {
      this.deletedProfiles = await invoke<Profile[]>("list_deleted_profiles")
    } catch {
      this.deletedProfiles = []
    }
  }

  /**
   * Recupera un perfil eliminado y todos sus datos asociados.
   * Sincroniza automaticamente con la nube si hay sesion activa.
   * @param p - Perfil a recuperar
   */
  recoverProfile = async (p: Profile) => {
    await this.run(async () => {
      await invoke("recover_profile", { id: p.id })
      if (this.cloudStatus.connected && this.cloudStatus.email_verified) {
        const result = await invoke<SyncResult>("sync_all_data")
        const errs = result.errors.length > 0 ? ` Errores: ${result.errors.join(", ")}` : ""
        this.notice = `Perfil recuperado y sincronizado (config: ${result.config_synced}, perfiles: ${result.profiles_synced}, sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced})${errs}`
      } else {
        this.notice = "Perfil recuperado"
      }
      await this.refreshStatus()
      await this.loadDeletedProfiles()
    })
  }

  /**
   * Elimina una sesión individual (soft-delete).
   * Sincroniza automáticamente con la nube si hay sesión activa.
   * @param sessionId - ID de la sesión a eliminar
   */
  deleteSession = async (sessionId: string) => {
    this.error = ""
    this.notice = ""
    try {
      await invoke("delete_session", { id: sessionId })
      if (this.cloudStatus.connected && this.cloudStatus.email_verified) {
        const result = await invoke<SyncResult>("sync_all_data")
        const errs = result.errors.length > 0 ? ` Errores: ${result.errors.join(", ")}` : ""
        this.notice = `Sesión eliminada y sincronizada (sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced})${errs}`
      } else {
        this.notice = "Sesión eliminada"
      }
      if (this.historyProfileId) {
        this.sessionHistory = await invoke<Session[]>("list_sessions", { profileId: this.historyProfileId })
        this.deletedSessions = await invoke<Session[]>("list_deleted_sessions", { profileId: this.historyProfileId })
      }
    } catch (err) {
      this.error = msg(err)
    }
  }

  /**
   * Recupera una sesión eliminada (soft-delete).
   * Sincroniza automáticamente con la nube si hay sesión activa.
   * @param sessionId - ID de la sesión a recuperar
   */
  recoverSession = async (sessionId: string) => {
    this.error = ""
    this.notice = ""
    try {
      await invoke("recover_session", { id: sessionId })
      if (this.cloudStatus.connected && this.cloudStatus.email_verified) {
        const result = await invoke<SyncResult>("sync_all_data")
        const errs = result.errors.length > 0 ? ` Errores: ${result.errors.join(", ")}` : ""
        this.notice = `Sesión recuperada y sincronizada (sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced})${errs}`
      } else {
        this.notice = "Sesión recuperada"
      }
      if (this.historyProfileId) {
        this.sessionHistory = await invoke<Session[]>("list_sessions", { profileId: this.historyProfileId })
        this.deletedSessions = await invoke<Session[]>("list_deleted_sessions", { profileId: this.historyProfileId })
      }
    } catch (err) {
      this.error = msg(err)
    }
  }

  /**
   * Elimina permanentemente sesiones que llevan más de un mes en soft-delete.
   * Se ejecuta al iniciar la app. No sincroniza con la nube.
   */
  purgeOldSessions = async () => {
    try {
      await invoke<number>("purge_old_sessions")
    } catch {
      // silencioso — la purga no debe interferir con el arranque
    }
  }

  // =========== LLM ===========
  /**
   * Guarda la configuración del proveedor LLM en el backend.
   * @param e - Evento del formulario
   */
  saveLLMConfig = async (e: Event) => {
    e.preventDefault()
    await this.run(async () => {
      await invoke("set_llm_config", { request: this.llmForm })
      this.notice = "Configuración LLM guardada"
      await this.refreshStatus()
    })
  }

  /**
   * Prueba la conexión con el proveedor LLM configurado.
   * Almacena el resultado en `testResult` o el error en `error`.
   */
  testLLM = async () => {
    this.testResult = ""
    this.error = ""
    await this.withServerWait(async () => {
      try {
        this.testResult = await invoke<string>("test_llm_connection")
      } catch (err) {
        this.error = msg(err)
      }
    })
  }

  // =========== SESSIONS ===========
  /**
   * Carga el historial de sesiones del perfil seleccionado en `historyProfileId`.
   */
  loadSessions = async () => {
    if (!this.historyProfileId) return
    await this.run(async () => {
      this.sessionHistory = await invoke<Session[]>("list_sessions", { profileId: this.historyProfileId })
    })
  }

  /**
   * Carga las sesiones eliminadas (soft-delete) del perfil seleccionado en `historyProfileId`.
   */
  loadDeletedSessions = async () => {
    if (!this.historyProfileId) return
    try {
      this.deletedSessions = await invoke<Session[]>("list_deleted_sessions", { profileId: this.historyProfileId })
    } catch {
      this.deletedSessions = []
    }
  }

  /**
   * Carga el resumen detallado de una sesión (preguntas, respuestas, estadísticas).
   * @param sessionId - ID de la sesión a cargar.
   */
  loadSessionDetail = async (sessionId: string) => {
    await this.run(async () => {
      this.detailSessionSummary = await invoke<SessionSummary>("get_session_summary", { sessionId })
    })
  }

  /**
   * Cierra la vista de detalle de sesión.
   */
  closeSessionDetail = () => {
    this.detailSessionSummary = null
  }

  // =========== DASHBOARD ===========
  /**
   * Carga todas las estadísticas del dashboard para el perfil seleccionado:
   * stats globales, por concepto, evolución y sesiones recientes.
   */
  loadDashboard = async () => {
    if (!this.dashboardProfileId) return
    await this.run(async () => {
      this.dashboardStats = await invoke<DashboardStats>("get_dashboard_stats", { profileId: this.dashboardProfileId })
      this.conceptStats = await invoke<ConceptStat[]>("get_concept_stats", { profileId: this.dashboardProfileId })
      this.evolutionData = await invoke<EvolutionPoint[]>("get_evolution", { profileId: this.dashboardProfileId })
      const allSessions = await invoke<Session[]>("list_sessions", { profileId: this.dashboardProfileId })
      this.recentSessions = allSessions.filter(s => s.status === "completed").slice(0, this.recentLimit)
    })
  }

  /**
   * Exporta las sesiones del perfil del dashboard en CSV o JSON,
   * descargando el archivo al navegador.
   * @param format - Formato de exportación ("csv" | "json")
   */
  exportData = async (format: "csv" | "json") => {
    if (!this.dashboardProfileId) return
    await this.run(async () => {
      const data = await invoke<ExportSessionRow[]>("export_sessions", { profileId: this.dashboardProfileId })
      let content: string
      let filename: string
      let mimeType: string
      if (format === "json") {
        content = JSON.stringify(data, null, 2)
        filename = `mates_export_${this.dashboardProfileId}.json`
        mimeType = "application/json"
      } else {
        const headers = ["session_id","started_at","ended_at","question_number","question_text","concept","difficulty","student_answer","correct_answer","is_correct","time_spent_secs"]
        const csvRows = [headers.join(",")]
        for (const row of data) {
          csvRows.push(
            headers.map(h => {
              const val = (row as Record<string, unknown>)[h]
              const str = val === null || val === undefined ? "" : String(val)
              return str.includes(",") || str.includes('"') || str.includes("\n")
                ? `"${str.replace(/"/g, '""')}"`
                : str
            }).join(",")
          )
        }
        content = csvRows.join("\n")
        filename = `mates_export_${this.dashboardProfileId}.csv`
        mimeType = "text/csv"
      }
      const blob = new Blob([content], { type: mimeType })
      const url = URL.createObjectURL(blob)
      const a = document.createElement("a")
      a.href = url
      a.download = filename
      a.click()
      URL.revokeObjectURL(url)
      this.notice = `Exportado como ${format.toUpperCase()}`
    })
  }

  // =========== PROFESSIONAL LAYER ===========
  /**
   * Carga los datos de la capa profesional: dashboard del tutor,
   * estudiantes asignados, tareas, reportes y grupos.
   */
  loadProfessionalData = async () => {
    if (!this.currentUser) return
    await this.run(async () => {
      this.tutorDashboard = await invoke<TutorDashboard>("get_tutor_dashboard", { tutorUserId: this.currentUser!.id })
      this.tutorStudents = await invoke<Profile[]>("list_tutor_students", { tutorUserId: this.currentUser!.id })
      this.assignments = await invoke<Assignment[]>("list_assignments", { tutorUserId: this.currentUser!.id })
      this.reports = await invoke<Report[]>("list_reports", { tutorUserId: this.currentUser!.id })
      this.studentGroups = await invoke<StudentGroup[]>("list_student_groups", { userId: this.currentUser!.id })
    })
  }

  /**
   * Crea un nuevo grupo de estudiantes.
   * @param e - Evento del formulario
   */
  createGroup = async (e: Event) => {
    e.preventDefault()
    if (!this.currentUser || !this.groupForm.name.trim()) return
    await this.run(async () => {
      await invoke("create_student_group", { request: { name: this.groupForm.name.trim() }, userId: this.currentUser!.id })
      this.groupForm.name = ""
      this.notice = "Grupo creado"
      await this.loadProfessionalData()
    })
  }

  /**
   * Crea una nueva tarea para un estudiante.
   * @param e - Evento del formulario
   */
  createAssignment = async (e: Event) => {
    e.preventDefault()
    if (!this.currentUser || !this.assignmentForm.student_id || !this.assignmentForm.concept.trim()) return
    await this.run(async () => {
      await invoke("create_assignment", {
        request: {
          student_id: this.assignmentForm.student_id,
          concept: this.assignmentForm.concept.trim(),
          difficulty: this.assignmentForm.difficulty,
          due_date: this.assignmentForm.due_date || null,
        },
        tutorUserId: this.currentUser!.id,
      })
      this.assignmentForm = { student_id: "", concept: "", difficulty: "easy", due_date: "" }
      this.notice = "Tarea creada"
      await this.loadProfessionalData()
    })
  }

  /**
   * Genera un reporte de progreso para un estudiante en un período dado.
   * @param e - Evento del formulario
   */
  generateReportAction = async (e: Event) => {
    e.preventDefault()
    if (!this.currentUser || !this.reportForm.student_id || !this.reportForm.period.trim()) return
    await this.run(async () => {
      await invoke("generate_report", {
        request: { student_id: this.reportForm.student_id, period: this.reportForm.period.trim() },
        tutorUserId: this.currentUser!.id,
      })
      this.reportForm = { student_id: "", period: "" }
      this.notice = "Reporte generado"
      await this.loadProfessionalData()
    })
  }

  /**
   * Asigna un estudiante al tutor actual.
   * @param studentId - ID del estudiante a asignar
   */
  assignStudentToMe = async (studentId: string) => {
    if (!this.currentUser) return
    await this.run(async () => {
      await invoke("assign_student_to_tutor", { tutorUserId: this.currentUser!.id, request: { student_id: studentId } })
      this.notice = "Estudiante asignado"
      await this.loadProfessionalData()
    })
  }

  /**
   * Remueve la asignación de un estudiante del tutor actual.
   * @param studentId - ID del estudiante a remover
   */
  removeStudentFromMe = async (studentId: string) => {
    if (!this.currentUser) return
    await this.run(async () => {
      await invoke("remove_student_from_tutor", { tutorUserId: this.currentUser!.id, studentId })
      this.notice = "Asignacion removida"
      await this.loadProfessionalData()
    })
  }

  // =========== CLOUD / BASEROW ===========
  /**
   * Carga el estado de la nube (sesion activa o no) desde el backend.
   */
  loadCloudStatus = async () => {
    await this.withServerWait(async () => {
      try {
        this.cloudStatus = await invoke<CloudStatus>("get_cloud_status")
      } catch (err) {
        this.cloudStatus = { connected: false, user_name: null, email: null, last_sync: null, auto_login: false, email_verified: false }
        console.warn("No se pudo obtener estado de la nube:", err)
      }
    })
  }

  /**
   * Registra una nueva cuenta en Baserow.
   * @param e - Evento del formulario
   */
  registerCloudAccount = async (e: Event) => {
    e.preventDefault()
    const f = this.cloudForm
    if (f.name.length < 2) { this.error = "El nombre debe tener al menos 2 caracteres"; return }
    if (!f.email.includes('@')) { this.error = "Email invalido"; return }
    if (f.password.length < 8) { this.error = "La contrasena debe tener al menos 8 caracteres"; return }
    if (f.password !== f.confirmPassword) { this.error = "Las contrasenas no coinciden"; return }
    if (!f.consent) { this.error = "Debes aceptar el consentimiento de privacidad"; return }

    await this.run(async () => {
      await invoke("register_account", {
        request: { name: f.name, email: f.email, password: f.password, consent: f.consent },
      })
      this.cloudForm = { mode: 'login', name: '', email: '', password: '', confirmPassword: '', consent: false }
      this.cloudStatus.auto_login = true
      await this.loadCloudStatus()
      this.notice = "Cuenta creada. Revisa tu email para verificar la cuenta antes de sincronizar"
    })
  }

  /**
   * Inicia sesion en Baserow.
   * @param e - Evento del formulario
   */
  loginCloudAccount = async (e: Event) => {
    e.preventDefault()
    const f = this.cloudForm
    if (!f.email.includes('@')) { this.error = "Email invalido"; return }
    if (!f.password) { this.error = "La contrasena no puede estar vacia"; return }

    await this.run(async () => {
      await invoke("login_account", { request: { email: f.email, password: f.password } })
      this.cloudForm = { mode: 'login', name: '', email: '', password: '', confirmPassword: '', consent: false }
      this.cloudStatus.auto_login = true
      await this.loadCloudStatus()
      if (this.cloudStatus.email_verified) {
        const result = await invoke<SyncResult>("sync_all_data")
        await this.refreshStatus()
        if (result.errors.length > 0) {
          this.notice = `Sesion iniciada. Sincronizado con errores: ${result.errors.join(", ")}`
        } else {
          this.notice = `Sesion iniciada. Sincronizado correctamente (config: ${result.config_synced}, perfiles: ${result.profiles_synced}, sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced})`
        }
      } else {
        this.notice = "Sesion iniciada. Debes verificar tu email antes de sincronizar"
      }
    })
  }

  /**
   * Cierra sesion en Baserow.
   */
  logoutCloudAccount = async () => {
    await this.run(async () => {
      await invoke("logout_account")
      this.notice = "Sesion cerrada"
      await this.loadCloudStatus()
    })
  }

  /**
   * Ejecuta sincronizacion manual de todos los datos.
   */
  syncNow = async () => {
    await this.run(async () => {
      const result = await invoke<SyncResult>("sync_all_data")
      if (result.errors.length > 0) {
        this.notice = `Sincronizado (config: ${result.config_synced}, perfiles: ${result.profiles_synced}, sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced}). Errores: ${result.errors.join(", ")}`
      } else {
        this.notice = `Sincronizado correctamente (config: ${result.config_synced}, perfiles: ${result.profiles_synced}, sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced})`
      }
      await this.refreshStatus()
    })
  }

  /**
   * Fuerza sincronización desde la nube: sube datos locales que no existen
   * en remoto y sobrescribe todo lo local con los datos remotos.
   */
  forceSyncFromCloud = async () => {
    await this.run(async () => {
      const result = await invoke<SyncResult>("force_sync_from_cloud")
      if (result.errors.length > 0) {
        this.notice = `Forzado desde nube (config: ${result.config_synced}, perfiles: ${result.profiles_synced}, sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced}). Errores: ${result.errors.join(", ")}`
      } else {
        this.notice = `Forzado desde nube correctamente (config: ${result.config_synced}, perfiles: ${result.profiles_synced}, sesiones: ${result.sessions_synced}, preguntas: ${result.session_questions_synced})`
      }
      await this.refreshStatus()
    })
  }

  /**
   * Activa o desactiva el auto-login en la nube.
   * @param enabled - true para activar, false para desactivar
   */
  setAutoLogin = async (enabled: boolean) => {
    await invoke("set_cloud_auto_login", { enabled })
    this.cloudStatus.auto_login = enabled
  }

  /**
   * Envia el codigo de verificacion al backend para validar el email.
   * @param code - Codigo de 6 digitos recibido por email
   */
  verifyEmailCode = async (code: string) => {
    await this.run(async () => {
      await invoke("verify_email_code", { code })
      this.cloudStatus.email_verified = true
      this.notice = "Email verificado correctamente"
    })
  }

  /**
   * Solicita el reenvio del codigo de verificacion al email de la cuenta.
   */
  resendVerificationCode = async () => {
    await this.run(async () => {
      await invoke("resend_verification_code")
      this.notice = "Codigo reenviado. Revisa tu email"
    })
  }

  /**
   * Elimina la cuenta de nube permanentemente.
   * Pide confirmacion antes de proceder.
   */
  deleteCloudAccount = async () => {
    if (!window.confirm("Eliminar la cuenta de nube permanentemente? Esta accion no se puede deshacer.")) return
    await this.run(async () => {
      await invoke("delete_cloud_account")
      this.cloudStatus = { connected: false, user_name: null, email: null, last_sync: null, auto_login: false, email_verified: false }
      this.notice = "Cuenta de nube eliminada"
    })
  }

  /**
   * Cambia el email de la cuenta en la nube.
   * @param newEmail - Nuevo correo electronico
   */
  changeCloudEmail = async (newEmail: string) => {
    await this.run(async () => {
      await invoke("change_cloud_email", { newEmail })
      this.cloudStatus.email = newEmail
      this.cloudStatus.email_verified = false
      this.notice = "Email actualizado. Revisa tu nuevo email para verificar la cuenta"
    })
  }

  // =========== CHILD SESSION ===========
  /**
   * Inicia una nueva sesión de práctica para un perfil.
   * Transiciona inmediatamente a child_session con un estado de carga inline
   * para evitar que el overlay del spinner bloquee la UI durante la llamada LLM.
   * @param profileId - ID del perfil infantil
   */
  startSession = async (profileId: string) => {
    this.selectedProfileId = profileId
    this.currentQuestion = null
    this.studentAnswer = ""
    this.answerFeedback = null
    this.showExplanation = false
    this.explanationData = null
    this.view = "child_session"
    try {
      const res = await invoke<StartSessionResponse>("start_session", { profileId })
      this.sessionId = res.session_id
      this.currentQuestion = res.first_question
      this.questionStartTime = Date.now()
    } catch (err) {
      this.error = msg(err)
      this.view = "child_select"
    }
  }

  /**
   * Envía la respuesta del estudiante a la pregunta actual.
   * Lee la respuesta de `studentAnswer`, que debe establecerse antes de llamar.
   * Si la sesión finaliza, obtiene el resumen y cambia a la vista de resumen.
   */
  submitAnswer = async () => {
    if (!this.studentAnswer.trim() || this.isSubmitting) return
    this.isSubmitting = true
    await this.run(async () => {
      const timeSpent = Math.round((Date.now() - this.questionStartTime) / 1000)
      const res = await invoke<SubmitAnswerResponse>("submit_answer", {
        request: {
          session_id: this.sessionId,
          question_id: this.currentQuestion!.question_id,
          answer: this.studentAnswer.trim(),
          time_spent_secs: timeSpent,
        },
      })
      this.answerFeedback = res
      if (res.session_finished) {
        const summary = await invoke<SessionSummary>("end_session", { sessionId: this.sessionId })
        this.sessionSummary = summary
        this.view = "child_summary"
      }
    })
    this.isSubmitting = false
  }

  /**
   * Carga la explicación de la pregunta actual desde el backend.
   */
  loadExplanation = async () => {
    if (!this.currentQuestion) return
    try {
      this.explanationData = await invoke<ExplanationResponse>("get_explanation", { questionId: this.currentQuestion!.question_id })
      this.showExplanation = true
    } catch (err) {
      this.error = msg(err)
    }
  }

  /**
   * Avanza a la siguiente pregunta. Requiere que `answerFeedback.next_question`
   * esté disponible (sesión no finalizada).
   */
  nextQuestion = () => {
    if (!this.answerFeedback?.next_question) return
    this.currentQuestion = this.answerFeedback.next_question
    this.answerFeedback = null
    this.showExplanation = false
    this.explanationData = null
    this.studentAnswer = ""
    this.questionStartTime = Date.now()
  }

  /** Vuelve a la pantalla de selección de perfiles y limpia el estado de sesión */
  goHome = () => {
    this.view = "child_select"
    this.currentQuestion = null
    this.sessionId = ""
    this.sessionSummary = null
  }
}

export const appState = new AppState()
