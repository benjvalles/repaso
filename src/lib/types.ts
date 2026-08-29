type LevelMode = "automatic" | "manual"
type AppView = "loading" | "setup_pin" | "child_select" | "child_session" | "child_summary" | "child_chat" | "adult_unlock" | "adult_panel"

type Profile = {
  id: string
  display_name: string
  school_year: number
  age: number | null
  level_mode: LevelMode
  current_level: number
  manual_prompt: string | null
  created_at: string
  updated_at: string
}

type LLMConfig = {
  provider: string
  model: string
  base_url: string
  api_key: string
}

type AppStatus = {
  guardian_pin_set: boolean
  adult_unlocked: boolean
  profiles: Profile[]
  llm_config: LLMConfig
  cloud_status: CloudStatus
}

type CurrentQuestion = {
  question_id: string
  question_text: string
  question_number: number
  total_questions: number
  concept: string
  difficulty: string
}

type StartSessionResponse = {
  session_id: string
  total_questions: number
  first_question: CurrentQuestion | null
}

type SubmitAnswerResponse = {
  is_correct: boolean
  feedback: string
  correct_answer: string
  explanation_needed: boolean
  next_question: CurrentQuestion | null
  session_finished: boolean
}

type ExplanationResponse = {
  explanation: string
  key_points: string[]
  next_steps: string[]
  reformulated_question: string | null
}

type SessionQuestion = {
  id: string
  session_id: string
  question_text: string
  correct_answer: string
  student_answer: string | null
  concept: string
  difficulty: string
  is_correct: boolean | null
  explanation: string | null
  question_number: number
  time_spent_secs: number | null
  created_at: string
  answered_at: string | null
}

type Session = {
  id: string
  profile_id: string
  status: string
  total_questions: number
  questions_answered: number
  correct_count: number
  current_question_index: number
  started_at: string
  ended_at: string | null
}

type SessionSummary = {
  session: Session
  questions: SessionQuestion[]
  concepts_worked: string[]
  concepts_mastered: string[]
  concepts_to_practice: string[]
  accuracy_pct: number
  avg_time_per_question: number
  total_time_secs: number
}

type DashboardStats = {
  total_sessions: number
  total_questions_answered: number
  total_correct: number
  overall_accuracy_pct: number
  total_time_secs: number
  avg_time_per_question: number
  concepts_mastered: string[]
  concepts_in_progress: string[]
  concepts_needing_practice: string[]
}

type ConceptStat = {
  concept: string
  total_attempts: number
  correct_attempts: number
  accuracy_pct: number
  last_practiced: string
}

type EvolutionPoint = {
  session_id: string
  started_at: string
  accuracy_pct: number
  questions_answered: number
  correct_count: number
}

type ExportSessionRow = {
  session_id: string
  started_at: string
  ended_at: string | null
  question_number: number
  question_text: string
  concept: string
  difficulty: string
  student_answer: string | null
  correct_answer: string
  is_correct: boolean | null
  time_spent_secs: number | null
}

type User = {
  id: string
  display_name: string
  role: string
  created_at: string
}

type StudentGroup = {
  id: string
  name: string
  owner_user_id: string
  created_at: string
}

type TutorStudentInfo = {
  student_id: string
  display_name: string
  school_year: number
  current_level: number
  last_session: string | null
  accuracy_pct: number
}

type TutorDashboard = {
  total_students: number
  active_assignments: number
  reports_generated: number
  students: TutorStudentInfo[]
}

type Assignment = {
  id: string
  tutor_user_id: string
  student_id: string
  concept: string
  difficulty: string
  due_date: string | null
  status: string
  created_at: string
}

type Report = {
  id: string
  tutor_user_id: string
  student_id: string
  period: string
  report_data: string
  generated_at: string
}

type ProfileForm = {
  id: string | null
  display_name: string
  school_year: number
  age: string
  level_mode: LevelMode
  manual_level: number
  manual_prompt: string
}

type CloudStatus = {
  connected: boolean
  user_name: string | null
  email: string | null
  last_sync: string | null
  auto_login: boolean
  email_verified: boolean
}

type SyncResult = {
  config_synced: number
  profiles_synced: number
  sessions_synced: number
  session_questions_synced: number
  errors: string[]
}

export type {
  LevelMode, AppView,
  Profile, LLMConfig, AppStatus,
  CurrentQuestion, StartSessionResponse, SubmitAnswerResponse, ExplanationResponse,
  SessionQuestion, Session, SessionSummary,
  DashboardStats, ConceptStat, EvolutionPoint, ExportSessionRow,
  User, StudentGroup, TutorStudentInfo, TutorDashboard,
  Assignment, Report, ProfileForm,
  CloudStatus, SyncResult,
}
