<script lang="ts">
  import { onMount } from "svelte"
  import { invoke } from "@tauri-apps/api/core"
  import { appState } from "./lib/app-state.svelte"
  import type { SyncResult } from "./lib/types"

  import ChildSelectView from "./ChildSelectView.svelte"
  import ChildSessionView from "./ChildSessionView.svelte"
  import ChildSummaryView from "./ChildSummaryView.svelte"
  import ChildChatView from "./ChildChatView.svelte"
  import SetupPinView from "./SetupPinView.svelte"
  import AdultUnlockView from "./AdultUnlockView.svelte"
  import AdultPanelView from "./panels/AdultPanelView.svelte"

  let {
    courseLabel,
  } = appState

  let cs = $derived(appState.cloudStatus)

  onMount(async () => {
    const locale = navigator.language || "es-ES"
    await invoke("set_locale", { locale })
    await appState.refreshStatus()
    if (appState.cloudStatus.connected && appState.cloudStatus.auto_login) {
      if (appState.cloudStatus.email_verified) {
        await invoke<SyncResult>("sync_all_data")
        await appState.refreshStatus()
        appState.notice = `Sesión de nube restaurada`
      } else {
        appState.notice = `Sesión de nube restaurada. Verifica tu email en la pestana Nube`
      }
    }
    await appState.purgeOldSessions()
  })
</script>

<main class="shell">
  <section class="app-frame">
    <header class="topbar">
      <div><p class="eyebrow">Mates</p><h1>Repaso de matemáticas</h1></div>
      <div class="topbar-actions">
        <span class="cloud-indicator" title={cs.connected ? (cs.email_verified ? `Nube: Conectado (${cs.email})` : `Nube: Validación pendiente (${cs.email})`) : "Nube: Desconectado"}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"/></svg>
          <span class="dot {cs.connected ? (cs.email_verified ? 'on' : 'pending') : 'off'}"></span>
        </span>
        {#if appState.view !== "child_session" && appState.view !== "child_summary"}
          {#if appState.status?.guardian_pin_set}
            <button class="secondary" type="button" onclick={() => appState.view = "adult_unlock"}>Zona adulta</button>
          {/if}
        {/if}
      </div>
    </header>

    {#if appState.error}<p class="alert error">{appState.error}</p>{/if}
    {#if appState.notice}<p class="alert notice">{appState.notice}</p>{/if}

    {#if appState.isWaitingForServer || appState.pendingServerRequests > 0}
      <div class="spinner-overlay" role="status" aria-live="polite">
        <div class="spinner"></div>
        <span>Esperando respuesta del servidor...</span>
      </div>
    {/if}

    {#if appState.view === "loading"}
      <section class="card"><p>Cargando...</p></section>

    {:else if appState.view === "setup_pin"}
      <SetupPinView onSubmit={(pin: string) => appState.setupPin(pin)} />

    {:else if appState.view === "child_select"}
      <ChildSelectView
        profiles={appState.status?.profiles ?? []}
        selectedProfileId={appState.selectedProfileId}
        {courseLabel}
        onSelectProfile={(id: string) => appState.selectProfile(id)}
        onStartChat={() => { appState.startChat(); appState.view = "child_chat" }}
        onStartSession={(id: string) => appState.startSession(id)}
      />

    {:else if appState.view === "child_session" && appState.currentQuestion}
      <ChildSessionView
        currentQuestion={appState.currentQuestion}
        answerFeedback={appState.answerFeedback}
        showExplanation={appState.showExplanation}
        explanationData={appState.explanationData}
        isSubmitting={appState.isSubmitting}
        onSubmit={(answer: string) => {
          appState.studentAnswer = answer
          appState.submitAnswer()
        }}
        onLoadExplanation={() => appState.loadExplanation()}
        onNextQuestion={() => appState.nextQuestion()}
        onGoHome={() => appState.goHome()}
      />

    {:else if appState.view === "child_session"}
      <section class="card session-loading">
        <div class="spinner" aria-label="Generando pregunta..."></div>
        <p>Preparando tu pregunta...</p>
      </section>

    {:else if appState.view === "child_summary" && appState.sessionSummary}
      <ChildSummaryView
        sessionSummary={appState.sessionSummary}
        childName={appState.status?.profiles.find(p => p.id === appState.selectedProfileId)?.display_name || "amigo"}
        onGoHome={() => appState.goHome()}
      />

    {:else if appState.view === "child_chat"}
      <ChildChatView
        messages={appState.chatMessages}
        isTyping={appState.isChatLoading}
        typewriterText={appState.chatTypewriterText}
        onSend={(msg: string) => appState.sendChatMessage(msg)}
        onBack={() => appState.view = "child_select"}
      />

    {:else if appState.view === "adult_unlock"}
      <AdultUnlockView
        onSubmit={(pin: string) => appState.unlockAdult(pin)}
        onBack={() => appState.view = "child_select"}
      />

    {:else if appState.view === "adult_panel"}
      <AdultPanelView />
    {/if}
  </section>
</main>

<style>
  .topbar-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .cloud-indicator {
    position: relative;
    display: inline-flex;
    align-items: center;
    font-size: 1.2rem;
    cursor: default;
  }

  .cloud-indicator .dot {
    position: absolute;
    top: 0;
    right: -4px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .cloud-indicator .dot.on { background: #2ecc71; }
  .cloud-indicator .dot.off { background: #aaa; }
  .cloud-indicator .dot.pending { background: #e74c3c; }

  .session-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    min-height: 200px;
    text-align: center;
    color: #607086;
  }
</style>
