<script lang="ts">
  import { appState } from "../lib/app-state.svelte"
</script>

<form class="card stack" onsubmit={(e) => appState.saveLLMConfig(e)}>
  <h3>Configuracion LLM</h3>
  <label>Proveedor
    <select bind:value={appState.llmForm.provider}>
      <option value="ollama">Ollama (local)</option>
      <option value="gemini">Google Gemini</option>
      <option value="openai">OpenAI Compatible</option>
    </select>
  </label>
  <label>Modelo <input bind:value={appState.llmForm.model} placeholder="ej: qwen2.5-math, llama3, gemini-1.5-flash" /></label>
  {#if appState.llmForm.provider === "ollama"}
    <p class="model-hint">Recomendado: <strong>qwen2.5-math</strong> (matematicas). Descarga con <code>ollama pull qwen2.5-math</code></p>
  {/if}
  <label>URL Base <input bind:value={appState.llmForm.base_url} placeholder="http://localhost:11434 o https://opencode.ai/inference/go/openai" /></label>
  {#if appState.llmForm.provider !== "ollama"}
    <label>API Key <input type="password" bind:value={appState.llmForm.api_key} placeholder="oc_sk_... (OpenAI, Gemini, OpenCode)" /></label>
  {/if}
  <div class="row">
    <button type="submit">Guardar</button>
    <button class="secondary" type="button" onclick={() => appState.testLLM()}>Probar conexion</button>
  </div>
</form>
{#if appState.testResult}<p class="alert notice">{appState.testResult}</p>{/if}

<style>
  .model-hint { margin: -6px 0 0; font-size: 0.85rem; color: #607086; }

  .model-hint code { background: #f0e9dc; padding: 1px 6px; border-radius: 6px; }
</style>
