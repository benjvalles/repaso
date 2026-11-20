import type { ProfileForm, SubjectPrompts, Subject } from "./types"

/** Devuelve un mapa de contextos pedagógicos vacío (uno por profesor) */
export const emptyPrompts = (): SubjectPrompts => ({
  mates: "", lengua_espanola: "", lengua_inglesa: "", naturales: "", personalizado: "",
})

/** Etiqueta legible de una asignatura ("profesor") */
export const subjectLabel = (s: Subject): string => ({
  mates: "Matemáticas",
  lengua_espanola: "Lengua Española",
  lengua_inglesa: "Lengua Inglesa",
  naturales: "Naturales",
  personalizado: "Personalizado",
})[s]

/** Devuelve un objeto ProfileForm vacío para resetear el formulario de perfiles */
export const emptyProfileForm = (): ProfileForm => ({
  id: null, display_name: "", school_year: 1, age: "", level_mode: "automatic", manual_level: 1, manual_prompt: "", prompts: emptyPrompts(),
})

/**
 * Convierte un número de curso a su etiqueta en español (ej: 3 → "3o Primaria")
 * @param c - Número de curso (1-6)
 */
export const courseLabel = (c: number) => `${c}o Primaria`

/**
 * Calcula el curso recomendado según la edad del niño
 * @param a - Edad en años
 */
export const courseForAge = (a: number) => Math.min(6, Math.max(1, a - 5))

/**
 * Da formato humano a un número de segundos (ej: 125 → "2m 5s")
 * @param secs - Segundos totales
 */
export const formatTime = (secs: number): string => {
  if (secs < 60) return `${secs}s`
  const m = Math.floor(secs / 60)
  const s = secs % 60
  return s > 0 ? `${m}m ${s}s` : `${m}m`
}

/** Convierte un error desconocido a string legible */
export const msg = (e: unknown) => (e instanceof Error ? e.message : String(e))
