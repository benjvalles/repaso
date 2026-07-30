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
  <label>Modelo <input bind:value={appState.llmForm.model} placeholder="ej: llama3, gemini-1.5-flash" /></label>
  <label>URL Base <input bind:value={appState.llmForm.base_url} placeholder="http://localhost:11434" /></label>
  {#if appState.llmForm.provider !== "ollama"}
    <label>API Key <input type="password" bind:value={appState.llmForm.api_key} placeholder="API key" /></label>
  {/if}
  <div class="row">
    <button type="submit">Guardar</button>
    <button class="secondary" type="button" onclick={() => appState.testLLM()}>Probar conexion</button>
  </div>
</form>
{#if appState.testResult}<p class="alert notice">{appState.testResult}</p>{/if}
