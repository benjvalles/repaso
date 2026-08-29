<script lang="ts">
  import { tick } from "svelte"

  /**
   * @property messages - Lista de mensajes del chat
   * @property isTyping - Indica si el asistente esta escribiendo
   * @property typewriterText - Texto parcial del efecto typewriter
   * @property onSend - Callback para enviar un mensaje
   * @property onBack - Callback para volver a la pantalla anterior
   */
  let {
    messages = [] as Array<{ role: "user" | "assistant", content: string }>,
    isTyping = false,
    typewriterText = "",
    onSend = (_msg: string) => {},
    onBack = () => {},
  }: {
    messages: Array<{ role: "user" | "assistant", content: string }>
    isTyping: boolean
    typewriterText: string
    onSend: (msg: string) => void
    onBack: () => void
  } = $props()

  let inputValue = $state("")
  let messagesEl: HTMLDivElement

  /** Envia el mensaje si hay texto */
  const handleSend = () => {
    const text = inputValue.trim()
    if (!text || isTyping) return
    inputValue = ""
    onSend(text)
  }

  /** Maneja Enter para enviar */
  const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  /** Auto-scroll al fondo cuando hay nuevos mensajes */
  $effect(() => {
    if (messages || typewriterText) {
      tick().then(() => {
        if (messagesEl) {
          messagesEl.scrollTop = messagesEl.scrollHeight
        }
      })
    }
  })
</script>

<section class="panel chat-view">
  <header class="chat-header">
    <button class="icon-btn" type="button" onclick={onBack} title="Volver">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
    </button>
    <h2>Chat</h2>
  </header>

  <div class="chat-messages" bind:this={messagesEl}>
    {#each messages as msg, i (i)}
      <div class="chat-bubble {msg.role}">
        <p>{msg.content}</p>
      </div>
    {/each}

    {#if typewriterText}
      <div class="chat-bubble assistant">
        <p>{typewriterText}<span class="cursor">|</span></p>
      </div>
    {/if}

    {#if isTyping && !typewriterText}
      <div class="chat-bubble assistant">
        <p class="typing-dots"><span>.</span><span>.</span><span>.</span></p>
      </div>
    {/if}
  </div>

  <div class="chat-input-row">
    <input
      type="text"
      bind:value={inputValue}
      onkeydown={handleKeydown}
      placeholder="Escribe tu mensaje..."
      disabled={isTyping}
    />
    <button class="primary" type="button" onclick={handleSend} disabled={isTyping || !inputValue.trim()} aria-label="Enviar mensaje">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m22 2-7 20-4-9-9-4Z"/><path d="M22 2 11 13"/></svg>
    </button>
  </div>
</section>

<style>
  .chat-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-height: calc(100vh - 80px);
    padding: 0;
    overflow: hidden;
  }

  .chat-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #e2e8f0;
    flex-shrink: 0;
  }

  .chat-header h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  .icon-btn {
    background: none;
    border: none;
    padding: 0.25rem;
    cursor: pointer;
    color: inherit;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-btn:hover {
    background: #f1f5f9;
  }

  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .chat-bubble {
    max-width: 80%;
    padding: 0.6rem 0.9rem;
    border-radius: 12px;
    line-height: 1.4;
    font-size: 0.95rem;
  }

  .chat-bubble p {
    margin: 0;
    white-space: pre-wrap;
  }

  .chat-bubble.user {
    align-self: flex-end;
    background: #3b82f6;
    color: white;
    border-bottom-right-radius: 4px;
  }

  .chat-bubble.assistant {
    align-self: flex-start;
    background: #f1f5f9;
    color: #1e293b;
    border-bottom-left-radius: 4px;
  }

  .cursor {
    animation: blink 0.7s step-end infinite;
    font-weight: bold;
  }

  @keyframes blink {
    50% { opacity: 0; }
  }

  .typing-dots span {
    animation: dot-pulse 1.4s infinite ease-in-out;
    font-size: 1.2rem;
    line-height: 1;
  }

  .typing-dots span:nth-child(1) { animation-delay: 0s; }
  .typing-dots span:nth-child(2) { animation-delay: 0.2s; }
  .typing-dots span:nth-child(3) { animation-delay: 0.4s; }

  @keyframes dot-pulse {
    0%, 80%, 100% { opacity: 0.3; }
    40% { opacity: 1; }
  }

  .chat-input-row {
    display: flex;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-top: 1px solid #e2e8f0;
    flex-shrink: 0;
  }

  .chat-input-row input {
    flex: 1;
    padding: 0.5rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    font-size: 0.95rem;
    outline: none;
  }

  .chat-input-row input:focus {
    border-color: #3b82f6;
  }

  .chat-input-row input:disabled {
    background: #f9fafb;
    opacity: 0.7;
  }

  .chat-input-row button {
    padding: 0.5rem 0.75rem;
    border-radius: 8px;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .chat-input-row button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
