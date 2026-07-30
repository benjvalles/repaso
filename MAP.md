# MAPA DEL PROYECTO MATES

## Estructura de directorios

```
mates/
├── index.html                    # Entrypoint HTML
├── package.json                  # Dependencias y scripts Node
├── svelte.config.js              # Config Svelte (vitePreprocess)
├── tsconfig.json                 # TypeScript strict config
├── vite.config.js                # Vite + Tauri (puerto 1420)
├── pnpm-lock.yaml / package-lock.json
├── .env / .env.example           # Variables de entorno (gitignored)
├── AGENTS.md                     # Instrucciones para la IA
├── CONTEXT.md                    # Especificación completa del proyecto
├── MAP.md                        # ← Este archivo
├── README.md                     # Documentación del proyecto
├── CLAUDE.md                     # Instrucciones Claude legacy
├── AIRTABLE_TABLAS.md            # Documentación tablas Airtable
├── .editorconfig                 # Configuración editor
├── .gitignore                    # Ignorados git
│
├── src/                          # ← FRONTEND (Svelte 5 + TS)
│   ├── main.ts                   # Entrypoint: mount App
│   ├── App.svelte                # Componente raíz (148 líneas)
│   ├── app.css                   # Estilos globales
│   ├── prototypes.ts             # Extensiones prototype (Number, String)
│   │
│   ├── ChildSelectView.svelte    # Selección de perfil infantil
│   ├── ChildSessionView.svelte   # Sesión de práctica (preguntas)
│   ├── ChildSummaryView.svelte   # Resumen post-sesión
│   ├── SetupPinView.svelte       # Configuración inicial del PIN
│   ├── AdultUnlockView.svelte    # Desbloqueo de zona adulta
│   │
│   ├── lib/
│   │   ├── types.ts              # Tipos/ interfaces compartidos
│   │   ├── helpers.ts            # Funciones helper
│   │   └── app-state.svelte.ts   # Estado global (clase AppState)
│   │
│   ├── panels/
│   │   ├── AdultPanelView.svelte       # Contenedor zona adulta (tabs)
│   │   ├── ProfilesTab.svelte          # CRUD perfiles
│   │   ├── LlmTab.svelte               # Config LLM
│   │   ├── SessionsTab.svelte          # Historial de sesiones
│   │   ├── CloudTab.svelte             # Nube / Baserow
│   │   │
│   │   ├── dashboard/
│   │   │   ├── DashboardTab.svelte      # Layout dashboard
│   │   │   ├── ProfileSelector.svelte   # Selector de perfil
│   │   │   ├── StatsGrid.svelte         # Estadísticas globales
│   │   │   ├── ConceptStatus.svelte     # Estado de conceptos
│   │   │   ├── ConceptDetail.svelte     # Detalle por concepto
│   │   │   ├── EvolutionChart.svelte    # Gráfico evolución
│   │   │   ├── RecentSessions.svelte    # Sesiones recientes
│   │   │   └── ExportButtons.svelte     # Export CSV/JSON
│   │   │
│   │   └── professional/
│   │       ├── ProfessionalTab.svelte         # Contenedor profesional
│   │       ├── ProfessionalDashboardTab.svelte# Resumen tutor
│   │       ├── ProfessionalStudentsTab.svelte # Estudiantes asignados
│   │       ├── ProfessionalGroupsTab.svelte   # Grupos
│   │       ├── ProfessionalAssignmentsTab.svelte # Tareas
│   │       └── ProfessionalReportsTab.svelte  # Reportes
│   │
│   └── routes/ (vacío)
│
├── src-tauri/                    # ← BACKEND (Rust + SQLite)
│   ├── Cargo.toml                # Dependencias Rust (rusqlite bundled, argon2, reqwest)
│   ├── tauri.conf.json           # Configuración Tauri v2
│   ├── build.rs                  # Build script (Tauri)
│   ├── capabilities/             # Permisos Tauri
│   ├── icons/                    # Iconos de la app
│   ├── gen/                      # Código generado
│   └── src/
│       ├── main.rs               # Entrypoint → llama a mates_lib::run()
│       ├── lib.rs                # Estado AppState + comandos Tauri + setup
│       ├── helpers.rs            # Funciones helper DB, validación, evaluación
│       ├── models.rs             # Structs, enums, constantes, traits
│       ├── email.rs              # Cliente Brevo (email transaccional)
│       │
│       ├── cloud/
│       │   ├── mod.rs            # CloudSession, CloudStatus
│       │   ├── baserow.rs        # Cliente HTTP Baserow
│       │   ├── commands.rs       # Comandos cloud (registro, login, sync, etc.)
│       │   └── sync.rs           # Sincronización bidireccional SQLite ↔ Baserow
│       │
│       └── llm/
│           ├── mod.rs            # LLMQuestion, LLMExplanation, LLMProviderEnum
│           ├── common.rs         # Prompts, parsing, chat_request, logs
│           ├── commands.rs       # build_provider(), load_llm_config()
│           ├── ollama.rs         # Proveedor Ollama
│           ├── gemini.rs         # Proveedor Google Gemini
│           └── openai_compatible.rs # Proveedor OpenAI-compatible
│
├── docs/
│   └── doc-API-Baserow.md        # Documentación API Baserow
│
├── scripts/
│   └── bump-version.mjs          # Script para incrementar versión
│
├── specs/                        # Especificaciones OpenSpec
│   └── CLOUD.md                  # Especificación cloud (Baserow)
├── agents_docs/
│   └── baserow-api.md            # Documentación técnica Baserow
├── static/                       # Assets estáticos
│   ├── favicon.png               # Favicon
│   ├── svelte.svg                # Logo Svelte
│   ├── tauri.svg                 # Logo Tauri
│   └── vite.svg                  # Logo Vite
└── .github/workflows/            # CI/CD
    └── build-desktop.yml         # Build Linux, macOS, Windows
```

---

## FRONTEND — `src/`

### `src/main.ts` (10 líneas)
| Símbolo | Tipo | Línea | Descripción |
|---------|------|-------|-------------|
| `app` | `const` | 6 | Mount de App Svelte en `#app` |

### `src/prototypes.ts` (87 líneas)
| Símbolo | Tipo | Línea | Descripción |
|---------|------|-------|-------------|
| `String.prototype.getIdNumber` | método | 3 | Extrae número de un string |
| `String.prototype.toInt` | método | 4 | Convierte string a entero |
| `String.prototype.toFloat<T>` | método | 5 | Convierte string a float con control de decimales |
| `Number.prototype.toInt` | método | 8 | Convierte número a entero |
| `Number.prototype.toFloat<T>` | método | 9 | Formatea número con decimales |
| `Number.DECIMAL_SEP` | prop | 85 | Separador decimal: `','` |
| `Number.GROUP_SEP` | prop | 86 | Separador miles: `'.'` |
| `Number.LOCALE` | prop | 87 | Locale por defecto: `'es-ES'` |
| `roundToPrecision` | función | 25 | Redondeo con aritmética de enteros (evita drift flotante) |

### `src/app.css` (241 líneas)
Estilos globales: botones, formularios, layout, perfil, sesión, dashboard, profesional, spinner, responsive.

### `src/App.svelte` (157 líneas)
| Símbolo | Tipo | Línea | Descripción |
|---------|------|-------|-------------|
| `onMount` | hook | 21 | Inicializa status, auto-login cloud con await sync si email verificado, purga sesiones antiguas |
| Indicador nube | SVG | 42 | Icono nube + punto verde/rojo/gris en barra superior |
| Vista `"loading"` | bloque | 65 | Pantalla de carga |
| Vista `"setup_pin"` | bloque | 68 | SetupPinView |
| Vista `"child_select"` | bloque | 71 | ChildSelectView |
| Vista `"child_session"` | bloque | 78 | ChildSessionView |
| Vista `"child_summary"` | bloque | 102 | ChildSummaryView |
| Vista `"adult_unlock"` | bloque | 109 | AdultUnlockView |
| Vista `"adult_panel"` | bloque | 115 | AdultPanelView |

### `src/ChildSelectView.svelte` (31 líneas)
| Prop | Tipo | Descripción |
|------|------|-------------|
| `profiles` | `Profile[]` | Lista de perfiles |
| `onStartSession` | callback | Al pulsar perfil |
| `courseLabel` | función | Formatea curso |

### `src/ChildSessionView.svelte` (118 líneas)
| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `handleSubmit` | 40 | Envía respuesta |
| `handleKeydown` | 47 | Ctrl+Enter para enviar |
| `$effect` | 54 | Auto-focus en textarea al cambiar pregunta |

### `src/ChildSummaryView.svelte` (76 líneas)
| Prop | Descripción |
|------|-------------|
| `sessionSummary` | Resumen de sesión |
| `childName` | Nombre del niño |
| `onGoHome` | Volver a inicio |

### `src/SetupPinView.svelte` (29 líneas)
| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `handleSubmit` | 11 | Envía PIN al configurar |

### `src/AdultUnlockView.svelte` (35 líneas)
| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `handleSubmit` | 14 | Verifica PIN |
| `$effect` | 20 | Auto-focus en input PIN |

---

### `src/lib/types.ts` — Todos los tipos:
| Tipo | Línea | Descripción |
|------|-------|-------------|
| `LevelMode` | 1 | `"automatic" \| "manual"` |
| `AppView` | 2 | `"loading" \| "setup_pin" \| "child_select" \| "child_session" \| "child_summary" \| "adult_unlock" \| "adult_panel"` |
| `Profile` | 4 | Perfil de estudiante |
| `LLMConfig` | 16 | Configuración proveedor LLM |
| `AppStatus` | 23 | Estado completo de la app |
| `CurrentQuestion` | 31 | Pregunta activa en sesión |
| `StartSessionResponse` | 40 | Respuesta al iniciar sesión |
| `SubmitAnswerResponse` | 46 | Resultado de evaluar respuesta |
| `ExplanationResponse` | 55 | Explicación generada |
| `SessionQuestion` | 62 | Pregunta de sesión |
| `Session` | 78 | Sesión de práctica |
| `SessionSummary` | 90 | Resumen de sesión |
| `DashboardStats` | 101 | Estadísticas dashboard |
| `ConceptStat` | 113 | Estadística por concepto |
| `EvolutionPoint` | 121 | Punto de evolución |
| `ExportSessionRow` | 129 | Fila exportación CSV/JSON |
| `User` | 143 | Usuario profesional |
| `StudentGroup` | 150 | Grupo de estudiantes |
| `TutorStudentInfo` | 157 | Info estudiante para tutor |
| `TutorDashboard` | 166 | Panel de control tutor |
| `Assignment` | 173 | Tarea asignada |
| `Report` | 184 | Reporte generado |
| `ProfileForm` | 193 | Formulario de perfil |
| `CloudStatus` | 203 | Estado conexión nube |
| `SyncResult` | 212 | Resultado sincronización |

### `src/lib/helpers.ts` (32 líneas)
| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `emptyProfileForm()` | 4 | Crea ProfileForm vacío |
| `courseLabel(c)` | 12 | `3 → "3o Primaria"` |
| `courseForAge(a)` | 18 | Edad → curso recomendado |
| `formatTime(secs)` | 24 | `125 → "2m 5s"` |
| `msg(e)` | 32 | Error desconocido → string |

### `src/lib/app-state.svelte.ts` (837 líneas) — Clase `AppState`
#### Estado reactivo
| Prop | Línea | Descripción |
|------|-------|-------------|
| `view` | 14 | Vista actual |
| `status` | 15 | Estado App desde backend |
| `error` / `notice` | 16-17 | Mensajes UI |
| `pendingServerRequests` | 18 | Contador spinner |
| `isWaitingForServer` | 19 | Deriva si hay requests pendientes |
| `selectedProfileId` | 22 | Perfil seleccionado |
| `sessionId` | 23 | Sesión activa |
| `currentQuestion` | 24 | Pregunta actual |
| `studentAnswer` | 25 | Respuesta del alumno |
| `answerFeedback` | 26 | Feedback tras responder |
| `showExplanation` / `explanationData` | 27-28 | Estado explicación |
| `sessionSummary` | 29 | Resumen post-sesión |
| `questionStartTime` | 30 | Timestamp inicio pregunta |
| `isSubmitting` | 31 | Flag enviando respuesta |
| `adultTab` | 34 | Tab activo zona adulta |
| `loginPin` | 35 | PIN de login |
| `profileForm` | 36 | Formulario perfil |
| `llmForm` | 39 | Formulario LLM |
| `testResult` | 42 | Resultado test conexión |
| `resetPhrase` / `showReset` | 43-44 | Frase confirmación RESET |
| `sessionHistory` | 45 | Historial sesiones |
| `historyProfileId` | 46 | Perfil seleccionado en historial |
| `detailSessionSummary` | 47 | Resumen detallado de sesión para el modal |
| `deletedProfiles` | 49 | Perfiles eliminados (soft-delete) |
| `dashboardProfileId` | 52 | Perfil seleccionado en dashboard |
| `dashboardStats` / `conceptStats` / `evolutionData` | 53-55 | Datos dashboard |
| `conceptFilter` / `filteredConceptStats` | 56-61 | Filtro por concepto |
| `recentSessions` / `recentLimit` | 62-63 | Sesiones recientes |
| `currentUser` / `users` / `tutorStudents` etc. | 66-76 | Estado profesional |
| `professionalTab` | 73 | Pestana activa profesional |
| `groupForm` / `assignmentForm` / `reportForm` | 74-76 | Formularios profesional |
| `cloudStatus` | 79 | Estado nube |
| `cloudForm` | 80 | Formulario nube |

#### Métodos principales
| Método | Línea | Descripción |
|--------|-------|-------------|
| `withServerWait()` | 102 | Ejecuta acción con spinner |
| `run()` | 119 | Wrapper con manejo de errores |
| `refreshStatus()` | 136 | Recarga estado completo backend |
| `setupPin()` | 160 | Configura PIN guardián |
| `unlockAdult()` | 174 | Verifica PIN y desbloquea |
| `lockAdult()` | 189 | Bloquea zona adulta |
| `resetData()` | 201 | Borra todos los datos (confirma RESET) |
| `saveProfile()` | 216 | Crea o actualiza perfil |
| `editProfile()` | 243 | Carga perfil en formulario |
| `deleteProfile()` | 255 | Elimina perfil |
| `loadDeletedProfiles()` | 275 | Carga perfiles eliminados |
| `recoverProfile()` | 288 | Recupera perfil eliminado |
| `deleteSession()` | 313 | Elimina sesión individual (soft-delete, sin spinner global) |
| `recoverSession()` | 333 | Recupera sesión eliminada |
| `purgeOldSessions()` | 345 | Purga sesiones eliminadas > 30 días (silencioso, al arranque) |
| `saveLLMConfig()` | 358 | Guarda config LLM |
| `testLLM()` | 322 | Prueba conexión LLM |
| `loadSessions()` | 338 | Carga historial sesiones |
| `loadSessionDetail()` | 346 | Carga resumen detallado de una sesión |
| `closeSessionDetail()` | 355 | Limpia el detalle de sesión |
| `loadDashboard()` | 350 | Carga stats dashboard |
| `exportData()` | 366 | Exporta CSV/JSON |
| `loadProfessionalData()` | 411 | Carga datos profesionales |
| `createGroup()` | 426 | Crea grupo |
| `createAssignment()` | 441 | Crea tarea |
| `generateReportAction()` | 464 | Genera reporte |
| `assignStudentToMe()` | 482 | Asigna estudiante a tutor |
| `removeStudentFromMe()` | 495 | Remueve asignación |
| `loadCloudStatus()` | 508 | Carga estado nube |
| `registerCloudAccount()` | 523 | Registro en Baserow |
| `loginCloudAccount()` | 547 | Login en Baserow |
| `logoutCloudAccount()` | 574 | Logout nube |
| `syncNow()` | 585 | Sincronización manual |
| `setAutoLogin()` | 601 | Activa/desactiva auto-login |
| `verifyEmailCode()` | 610 | Verifica código email |
| `resendVerificationCode()` | 621 | Reenvía código de verificación |
| `deleteCloudAccount()` | 632 | Elimina cuenta de nube |
| `changeCloudEmail()` | 645 | Cambia email de la cuenta |
| `startSession()` | 659 | Inicia sesión práctica |
| `submitAnswer()` | 679 | Envía respuesta |
| `loadExplanation()` | 705 | Carga explicación LLM |
| `nextQuestion()` | 717 | Avanza a siguiente pregunta |
| `goHome()` | 728 | Vuelve a child_select |

---

### `src/panels/AdultPanelView.svelte` (40 líneas)
| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| Tabs: perfiles, LLM, historial, dashboard, nube, profesional | 18-24 | Navegación de tabs |

### `src/panels/ProfilesTab.svelte` (60 líneas)
CRUD perfiles + reset datos.

### `src/panels/LlmTab.svelte` (24 líneas)
Formulario configuración LLM + test conexión.

### `src/panels/SessionsTab.svelte` (212 líneas)
Selector perfil + listado historial sesiones + botón eliminar por sesión (soft-delete con confirmación y spinner inline) + modal detalle de sesión.

### `src/panels/CloudTab.svelte` (287 líneas)
| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `toggleMode()` | 7 | Alterna login/registro |

### `src/panels/dashboard/DashboardTab.svelte` (21 líneas)
Layout: ProfileSelector, StatsGrid, ConceptStatus, ConceptDetail, EvolutionChart, RecentSessions, ExportButtons.

### `src/panels/dashboard/StatsGrid.svelte` (42 líneas)
Muestra 8 tarjetas de estadísticas globales.

### `src/panels/dashboard/ConceptStatus.svelte` (33 líneas)
Conceptos dominados, en progreso, necesitan práctica.

### `src/panels/dashboard/ConceptDetail.svelte` (24 líneas)
Barras de precisión por concepto + filtro.

### `src/panels/dashboard/EvolutionChart.svelte` (17 líneas)
Gráfico de evolución por sesión.

### `src/panels/dashboard/RecentSessions.svelte` (22 líneas)
Lista sesiones recientes con barras de precisión.

### `src/panels/dashboard/ExportButtons.svelte` (8 líneas)
Botones exportar CSV y JSON.

### `src/panels/dashboard/ProfileSelector.svelte` (13 líneas)
Selector de perfil para dashboard.

### `src/panels/professional/ProfessionalTab.svelte` (34 líneas)
Contenedor con subtabs: Resumen, Estudiantes, Grupos, Tareas, Reportes.

### `src/panels/professional/ProfessionalDashboardTab.svelte` (41 líneas)
Stats tutor (estudiantes, tareas activas, reportes) + lista estudiantes.

### `src/panels/professional/ProfessionalStudentsTab.svelte` (36 líneas)
Estudiantes asignados + asignar nuevo.

### `src/panels/professional/ProfessionalGroupsTab.svelte` (25 líneas)
Crear grupo + listar grupos.

### `src/panels/professional/ProfessionalAssignmentsTab.svelte` (41 líneas)
Crear tarea + listar tareas activas.

### `src/panels/professional/ProfessionalReportsTab.svelte` (31 líneas)
Generar reporte + listar reportes.

---

## BACKEND — `src-tauri/src/`

### `src-tauri/src/main.rs` (6 líneas)
| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `main()` | 4 | Llama a `mates_lib::run()` |

### `src-tauri/src/lib.rs` (1769 líneas)

#### Struct `AppState` (línea 28)
Campos: `db` (Mutex\<Connection\>), `adult_unlocked`, `llm_provider`, `llm_config`, `locale`, `baserow_client`, `cloud_session`, `email_client`.

#### Comandos Tauri registrados (línea 1528)

Los comandos marcados con * están en `lib.rs`; el resto en `cloud/commands.rs` o `email.rs`.

##### lib.rs (45 comandos)
| Comando | Línea | Descripción |
|---------|-------|-------------|
| `get_app_status` | 47 | Estado general de la app |
| `setup_guardian_pin` | 93 | Configura PIN (hash Argon2) |
| `verify_guardian_pin` | 111 | Verifica PIN |
| `lock_adult_area` | 132 | Bloquea zona adulta |
| `set_locale` | 142 | Establece locale (es-ES) |
| `reset_local_data` | 154 | Borra todos los datos locales |
| `list_profiles` | 172 | Lista perfiles |
| `create_profile` | 185 | Crea perfil |
| `update_profile` | 211 | Actualiza perfil |
| `delete_profile` | 237 | Elimina perfil |
| `recover_profile` | 262 | Recupera perfil eliminado |
| `list_deleted_profiles` | 364 | Lista perfiles eliminados |
| `delete_session` | 295 | Elimina sesión individual (soft-delete) |
| `recover_session` | 317 | Recupera sesión eliminada |
| `purge_old_sessions` | 335 | Purga sesiones eliminadas > 30 días |
| `get_llm_config` | 380 | Lee config LLM |
| `set_llm_config` | 394 | Guarda config LLM |
| `test_llm_connection` | 424 | Prueba conexión LLM |
| `start_session` | 445 | Inicia sesión (10 preguntas) |
| `generate_question` | 477 | Genera pregunta con LLM |
| `submit_answer` | 503 | Evalúa respuesta |
| `get_explanation` | 564 | Explicación + reformulación |
| `end_session` | 641 | Finaliza sesión y genera resumen |
| `get_dashboard_stats` | 707 | Estadísticas dashboard |
| `get_concept_stats` | 786 | Stats por concepto |
| `get_evolution` | 800 | Evolución temporal |
| `export_sessions` | 839 | Exporta sesiones (CSV) |
| `list_sessions` | 939 | Lista sesiones de perfil |
| `get_session_summary` | 884 | Obtiene resumen de sesión con preguntas sin modificar estado |
| `create_user` | 978 | Crea usuario profesional |
| `login_user` | 1006 | Login usuario profesional |
| `list_users` | 1036 | Lista usuarios |
| `create_student_group` | 1068 | Crea grupo |
| `list_student_groups` | 1090 | Lista grupos |
| `add_student_to_group` | 1119 | Agrega estudiante a grupo |
| `remove_student_from_group` | 1135 | Remueve estudiante de grupo |
| `list_group_students` | 1153 | Lista estudiantes de grupo |
| `assign_student_to_tutor` | 1197 | Asigna estudiante a tutor |
| `remove_student_from_tutor` | 1216 | Remueve asignación |
| `list_tutor_students` | 1234 | Lista estudiantes del tutor |
| `create_assignment` | 1278 | Crea tarea |
| `list_assignments` | 1309 | Lista tareas |
| `generate_report` | 1347 | Genera reporte |
| `list_reports` | 1384 | Lista reportes |
| `get_tutor_dashboard` | 1419 | Panel de control tutor |

##### cloud/commands.rs (11 comandos)
| Comando | Línea | Descripción |
|---------|-------|-------------|
| `register_account` | 46 | Registro en Baserow + email verificación |
| `login_account` | 184 | Login con email + password |
| `logout_account` | 265 | Cierra sesión |
| `sync_all_data` | 294 | Sincroniza todos los datos |
| `force_sync_from_cloud` | 332 | Fuerza sobrescritura local con datos remotos |
| `get_cloud_status` | 381 | Estado conexión |
| `set_cloud_auto_login` | 422 | Activa/desactiva auto-login |
| `verify_email_code` | 446 | Verifica código email |
| `resend_verification_code` | 500 | Reenvía código verificación |
| `delete_cloud_account` | 560 | Elimina cuenta de nube |
| `change_cloud_email` | 613 | Cambia email de la cuenta |

##### email.rs (5 comandos)
| Comando | Línea | Descripción |
|---------|-------|-------------|
| `send_transac_email` | 360 | Envía email transaccional |
| `list_transac_emails` | 374 | Lista emails enviados |
| `get_email_content` | 388 | Obtiene contenido de email |
| `get_email_status` | 402 | Estado de un email |
| `delete_scheduled_email` | 417 | Elimina email programado |

#### Funciones helper internas
| Función | Línea | Descripción |
|---------|-------|-------------|
| `generate_question_for_session()` | 1354 | Genera pregunta LLM y guarda en DB |
| Tests | 1424 | Tests unitarios de `evaluate_answer_local` |
| `run()` | 1457 | Punto de entrada Tauri (setup DB, LLM, cloud, handlers) |

### `src-tauri/src/models.rs` (441 líneas)

#### Constantes
| Constante | Línea | Descripción |
|-----------|-------|-------------|
| `PIN_SETTING_KEY` | 6 | `"guardian_pin_hash"` |
| `LLM_PROVIDER_KEY` | 8 | `"llm_provider"` |
| `LLM_MODEL_KEY` | 10 | `"llm_model"` |
| `LLM_BASE_URL_KEY` | 12 | `"llm_base_url"` |
| `LLM_API_KEY_KEY` | 14 | `"llm_api_key"` |
| `CLOUD_SESSION_KEY` | 16 | `"cloud_session_user_id"` |
| `CLOUD_LAST_SYNC_KEY` | 18 | `"cloud_last_sync"` |
| `CLOUD_AUTO_LOGIN_KEY` | 20 | `"cloud_auto_login"` |
| `CLOUD_USER_NAME_KEY` | 22 | `"cloud_session_user_name"` |
| `CLOUD_EMAIL_KEY` | 24 | `"cloud_session_email"` |
| `CLOUD_VERIFICATION_CODE_KEY` | 26 | `"cloud_verification_code"` |
| `CLOUD_EMAIL_VERIFIED_KEY` | 28 | `"cloud_email_verified"` |
| `CLOUD_BASEROW_TOKEN_KEY` | 30 | `"baserow_api_token"` |
| `CLOUD_BASEROW_DB_ID_KEY` | 32 | `"baserow_database_id"` |

#### Structs principales
| Struct | Línea | Descripción |
|--------|-------|-------------|
| `LLMConfig` | 32 | Configuración proveedor LLM |
| `AppStatus` | 53 | Estado app (PIN, perfiles, LLM, cloud) |
| `Profile` | 63 | Perfil estudiante |
| `CreateProfileRequest` | 77 | Crear perfil (input) |
| `UpdateProfileRequest` | 88 | Actualizar perfil (input) |
| `Session` | 127 | Sesión de práctica |
| `SessionQuestion` | 141 | Pregunta de sesión |
| `SessionSummary` | 161 | Resumen sesión completada |
| `StartSessionResponse` | 174 | Respuesta inicio sesión |
| `CurrentQuestion` | 182 | Pregunta actual |
| `SubmitAnswerRequest` | 193 | Envío respuesta (input) |
| `SubmitAnswerResponse` | 202 | Resultado evaluación |
| `ExplanationResponse` | 213 | Explicación generada |
| `DashboardStats` | 229 | Estadísticas dashboard |
| `ConceptStat` | 244 | Estadística por concepto |
| `EvolutionPoint` | 255 | Punto evolución |
| `ExportSessionRow` | 266 | Fila exportación |
| `User` | 282 | Usuario profesional |
| `StudentGroup` | 305 | Grupo estudiantes |
| `TutorStudent` | 320 | Relación tutor-estudiante |
| `Assignment` | 335 | Tarea asignada |
| `Report` | 357 | Reporte generado |
| `TutorDashboard` | 375 | Panel tutor |
| `TutorStudentInfo` | 384 | Info estudiante (tutor) |
| `LLMConfigRequest` | 395 | Config LLM (input) |
| `RegisterRequest` | 420 | Registro cloud (input) |
| `CloudLoginRequest` | 434 | Login cloud (input) |

#### Enums y Traits
| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `LevelMode` | 101 | `Automatic / Manual` |
| `LevelMode::as_str()` | 108 | Serializa a string |
| `LevelMode::from_db()` | 116 | Deserializa desde DB |
| `HasAdultUnlocked` | 407 | Trait para acceso a `adult_unlocked` |

### `src-tauri/src/helpers.rs` (902 líneas)

| Función | Línea | Descripción |
|---------|-------|-------------|
| `setup_database()` | 19 | Crea todas las tablas SQLite |
| `ensure_profile_manual_prompt_column()` | 141 | Migración columna manual_prompt |
| `get_setting()` | 172 | Lee setting de app_settings |
| `set_setting()` | 190 | Guarda/actualiza setting |
| `profile_from_row()` | 210 | Construye Profile desde fila SQL |
| `get_profile_by_id()` | 238 | Busca perfil por ID |
| `list_profiles_from_db()` | 259 | Lista todos los perfiles |
| `get_session_by_id()` | 286 | Busca sesión por ID |
| `get_question_by_id()` | 318 | Busca pregunta por ID |
| `list_questions_for_session()` | 356 | Lista preguntas de sesión |
| `require_adult_unlocked()` | 399 | Verifica zona adulta desbloqueada |
| `validate_pin()` | 417 | Valida formato PIN (4-6 dígitos) |
| `hash_pin()` | 434 | Hash Argon2 de PIN |
| `verify_pin()` | 454 | Verifica PIN contra hash |
| `validate_profile_input()` | 474 | Valida campos de perfil |
| `resolve_manual_prompt()` | 521 | Normaliza contexto pedagógico |
| `manual_prompt_for_profile()` | 538 | Obtiene prompt manual del perfil |
| `resolve_current_level()` | 557 | Resuelve nivel (auto/manual) |
| `get_weakest_concept()` | 575 | Concepto con peor rendimiento |
| `get_default_concept_for_year()` | 603 | Concepto por defecto según curso |
| `evaluate_answer_local()` | 625 | Evaluación local de respuesta |
| `expected_numeric_result()` | 684 | Extrae resultado numérico esperado |
| `final_numeric_result()` | 707 | Último número en respuesta alumno |
| `extract_numbers()` | 720 | Extrae todos los números de un texto |
| `generate_default_explanation()` | 762 | Explicación genérica (fallback) |
| `get_concept_stats_for_profile()` | 780 | Estadísticas por concepto desde DB |

### `src-tauri/src/email.rs` (429 líneas)

| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `EmailClient` | 9 | Cliente HTTP Brevo |
| `EmailClient::from_env()` | 15 | Construye desde BREVO_API_KEY |
| `EmailClient::headers()` | 26 | Headers HTTP con API key |
| `EmailClient::send_transac_email()` | 40 | POST /smtp/email |
| `EmailClient::list_transac_emails()` | 68 | GET /smtp/emails |
| `EmailClient::get_email_content()` | 96 | GET /smtp/emails/{uuid} |
| `EmailClient::get_email_status()` | 120 | GET /smtp/emailStatus/{id} |
| `EmailClient::delete_scheduled_email()` | 149 | DELETE /smtp/email/{id} |
| `format_error()` | 175 | Formatea errores HTTP Brevo |

#### Schemas Request/Response
| Struct | Línea |
|--------|-------|
| `SendEmailRequest` | 192 |
| `Sender` | 213 |
| `Recipient` | 221 |
| `Attachment` | 228 |
| `MessageVersion` | 236 |
| `EmailListFilters` | 248 |
| `StatusFilters` | 262 |
| `SendEmailResponse` | 277 |
| `EmailListResponse` | 284 |
| `EmailSummary` | 291 |
| `EmailContentResponse` | 307 |
| `EmailEvent` | 321 |
| `EmailStatusResponse` | 328 |
| `BatchStatus` | 343 |

#### Comandos Tauri (email)
| Comando | Línea |
|---------|-------|
| `send_transac_email` | 360 |
| `list_transac_emails` | 374 |
| `get_email_content` | 388 |
| `get_email_status` | 402 |
| `delete_scheduled_email` | 417 |

---

### `src-tauri/src/cloud/mod.rs` (30 líneas)

| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `CloudSession` | 15 | Sesión activa en la nube |
| `CloudStatus` | 23 | Estado de conexión |

### `src-tauri/src/cloud/baserow.rs` (160 líneas)

| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `retry()` | 6 | Reintento con backoff (3 intentos) |
| `BaserowClient` | 29 | Cliente HTTP Baserow |
| `BaserowClient::new()` | 35 | Constructor |
| `BaserowClient::headers()` | 43 | Headers con Token |
| `BaserowClient::url()` | 52 | URL tabla |
| `BaserowClient::url_row()` | 57 | URL fila |
| `BaserowClient::list_rows()` | 64 | Lista filas (paginado) |
| `BaserowClient::create_row()` | 102 | Crea fila |
| `BaserowClient::update_row()` | 120 | Actualiza fila (PATCH) |
| `BaserowClient::find_account_by_email()` | 138 | Busca cuenta por email |

### `src-tauri/src/cloud/commands.rs` (668 líneas)

| Comando Tauri | Línea | Descripción |
|---------------|-------|-------------|
| `register_account` | 46 | Registro en Baserow + email verificación |
| `login_account` | 184 | Login con email + password |
| `logout_account` | 265 | Cierra sesión |
| `sync_all_data` | 294 | Sincroniza todos los datos |
| `get_cloud_status` | 343 | Estado conexión |
| `set_cloud_auto_login` | 378 | Activa/desactiva auto-login |
| `verify_email_code` | 402 | Verifica código email |
| `resend_verification_code` | 456 | Reenvía código verificación |
| `delete_cloud_account` | 516 | Elimina cuenta de nube |
| `change_cloud_email` | 569 | Cambia email de la cuenta |

#### Funciones internas
| Función | Línea | Descripción |
|---------|-------|-------------|
| `generate_verification_code()` | 26 | Código aleatorio 6 dígitos |

### `src-tauri/src/cloud/sync.rs` (1773 líneas)

#### Símbolos y constantes
| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `SyncResult` (pub) | 10 | Resultado sincronización |
| `TABLE_USER_CONFIG` | 27 | ID tabla config (1071740) |
| `TABLE_USER_PROFILES` | 32 | ID tabla perfiles (1071741) |
| `TABLE_USER_SESSIONS` | 37 | ID tabla sesiones (1071742) |
| `TABLE_USER_SESSION_QUESTIONS` | 42 | ID tabla preguntas (1071743) |
| `now_rfc3339()` | 45 | Timestamp UTC RFC3339 |
| `value_to_i64()` | 50 | Extrae i64 de JSON (Number o String) |
| `row_id()` | 55 | Extrae `id` de fila JSON Baserow |

#### Sync normal (upload local → remote, luego download remote → local)
| Función | Línea | Descripción |
|---------|-------|-------------|
| `read_local_app_settings()` | 73 | Lee settings locales |
| `write_remote_config()` | 93 | Escribe config remota en local |
| `upload_local_config()` | 121 | Sube config local a remoto |
| `sync_config_table()` | 172 | Sincroniza tabla config |
| `read_local_profiles()` | 221 | Lee perfiles locales |
| `write_remote_profiles()` | 259 | Escribe perfiles remotos en local |
| `upload_local_profiles()` | 337 | Sube perfiles locales a remoto |
| `sync_profiles_table()` | 407 | Sincroniza tabla perfiles |
| `read_local_sessions()` | 478 | Lee sesiones locales |
| `upload_local_sessions()` | 525 | Sube sesiones locales a remoto |
| `write_remote_sessions()` | 651 | Escribe sesiones remotas en local |
| `sync_sessions_table()` | 762 | Sincroniza tabla sesiones |
| `read_local_session_questions()` | 835 | Lee preguntas locales |
| `upload_local_session_questions()` | 889 | Sube preguntas locales a remoto |
| `write_remote_session_questions()` | 1028 | Escribe preguntas remotas en local |
| `sync_session_questions_table()` | 1152 | Sincroniza tabla preguntas |
| `sync_all()` (pub) | 1202 | Sincronización completa (punto entrada) |

#### Sync forzado "Desde la nube" (upload sólo locales nuevas, sobrescribe local con remoto)
| Función | Línea | Descripción |
|---------|-------|-------------|
| `upload_new_local_config()` | 1243 | Sube configs locales que no existen en remoto |
| `upload_new_local_profiles()` | 1274 | Sube perfiles locales que no existen en remoto |
| `upload_new_local_sessions()` | 1317 | Sube sesiones locales que no existen en remoto |
| `upload_new_local_session_questions()` | 1360 | Sube preguntas locales que no existen en remoto |
| `force_write_remote_config()` | 1408 | Sobrescribe config local con remota (incondicional) |
| `force_write_remote_profiles()` | 1433 | Sobrescribe perfiles locales con remotos (incondicional) |
| `force_write_remote_sessions()` | 1494 | Sobrescribe sesiones locales con remotas (incondicional) |
| `force_write_remote_session_questions()` | 1554 | Sobrescribe preguntas locales con remotas (incondicional) |
| `force_sync_config_table()` | 1621 | Sincroniza config forzada |
| `force_sync_profiles_table()` | 1652 | Sincroniza perfiles forzada |
| `force_sync_sessions_table()` | 1680 | Sincroniza sesiones forzada |
| `force_sync_session_questions_table()` | 1708 | Sincroniza preguntas forzada |
| `force_sync_all()` (pub) | 1739 | Sincronización forzada completa |

---

### `src-tauri/src/llm/mod.rs` (105 líneas)

| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `LLMQuestion` | 9 | Pregunta generada |
| `LLMExplanation` | 22 | Explicación generada |
| `LLMProviderEnum` | 34 | Enum dispatch: Ollama / Gemini / OpenAICompatible |
| `generate_question()` | 54 | Genera pregunta (dispatch) |
| `provide_explanation()` | 73 | Genera explicación (dispatch) |
| `reformulate_concept()` | 98 | Reformula concepto (dispatch) |

### `src-tauri/src/llm/common.rs` (262 líneas)

| Función | Línea | Descripción |
|---------|-------|-------------|
| `locale_to_language()` | 7 | ISO → nombre idioma (es, ca, eu, gl, en) |
| `log_llm_prompt()` | 25 | Loguea prompt en stderr |
| `log_llm_response()` | 44 | Loguea respuesta en stderr |
| `strip_reasoning_content()` | 57 | Elimina `reasoning_content` (DeepSeek) |
| `chat_request()` | 86 | Llamada HTTP genérica a API chat |
| `parse_json_response()` | 124 | Limpia y parsea JSON de respuesta LLM |
| `build_question_prompt()` | 152 | Construye prompt para generar pregunta |
| `parse_question_response()` | 178 | Parsea respuesta → LLMQuestion |
| `build_explanation_prompt()` | 199 | Construye prompt para explicación |
| `parse_explanation_response()` | 219 | Parsea respuesta → LLMExplanation |
| `build_reformulation_prompt()` | 237 | Construye prompt para reformulación |
| `build_manual_context()` | 256 | Contexto pedagógico adicional |

| Struct | Línea | Descripción |
|--------|-------|-------------|
| `ChatMessage` | 33 | Mensaje rol+contenido |

### `src-tauri/src/llm/commands.rs` (66 líneas)

| Función | Línea | Descripción |
|---------|-------|-------------|
| `build_provider()` | 19 | Construye LLMProviderEnum según config |
| `load_llm_config()` | 47 | Carga config desde DB con defaults |

### `src-tauri/src/llm/ollama.rs` (123 líneas)

| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `OllamaChatRequest` | 9 | Request body Ollama |
| `OllamaProvider` | 17 | Proveedor Ollama |
| `OllamaProvider::new()` | 30 | Constructor |
| `OllamaProvider::generate_text()` | 46 | Envía prompt a `/api/chat` |
| `OllamaProvider::generate_question()` | 87 | Genera pregunta |
| `OllamaProvider::provide_explanation()` | 104 | Genera explicación |
| `OllamaProvider::reformulate_concept()` | 119 | Reformula concepto |

### `src-tauri/src/llm/gemini.rs` (119 líneas)

| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `GeminiProvider` | 9 | Proveedor Google Gemini |
| `GeminiProvider::new()` | 22 | Constructor |
| `GeminiProvider::generate_text()` | 38 | Envía prompt a API Gemini |
| `GeminiProvider::generate_question()` | 83 | Genera pregunta |
| `GeminiProvider::provide_explanation()` | 100 | Genera explicación |
| `GeminiProvider::reformulate_concept()` | 115 | Reformula concepto |

### `src-tauri/src/llm/openai_compatible.rs` (133 líneas)

| Símbolo | Línea | Descripción |
|---------|-------|-------------|
| `OpenAIChatRequest` | 9 | Request body OpenAI-compatible |
| `OpenAICompatibleProvider` | 20 | Proveedor OpenAI-compatible |
| `OpenAICompatibleProvider::new()` | 35 | Constructor |
| `OpenAICompatibleProvider::generate_text()` | 52 | Envía prompt a `/v1/chat/completions` |
| `OpenAICompatibleProvider::generate_question()` | 97 | Genera pregunta |
| `OpenAICompatibleProvider::provide_explanation()` | 114 | Genera explicación |
| `OpenAICompatibleProvider::reformulate_concept()` | 129 | Reformula concepto |

---

## SCRIPTS

### `scripts/bump-version.mjs`
Script para incrementar versión (patch/minor) en `package.json` y `src-tauri/Cargo.toml`.

---

## ARCHIVOS DE CONFIGURACIÓN

| Archivo | Descripción |
|---------|-------------|
| `package.json` | Dependencias: Svelte 5, Tauri API v2, Vite 6 |
| `tsconfig.json` | Strict TS, ES2020, bundler resolution |
| `svelte.config.js` | Preprocessor vite |
| `vite.config.js` | Puerto 1420, HMR condicional, ignora src-tauri |
| `index.html` | Entrypoint HTML con `#app` |
| `.editorconfig` | Configuración editor |
| `.gitignore` | node_modules, dist, target, .env |
