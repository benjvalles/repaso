<script lang="ts">
  import { appState } from "../lib/app-state.svelte"

  let f = $derived(appState.cloudForm)
  let cs = $derived(appState.cloudStatus)

  let verificationCode = $state("")
  let newEmail = $state("")
  let showChangeEmail = $state(false)

  function toggleMode() {
    appState.cloudForm = {
      mode: f.mode === "login" ? "register" : "login",
      name: "", email: "", password: "", confirmPassword: "", consent: false,
    }
  }
</script>

<section class="card cloud-tab">
  <h3>Nube / Sincronización</h3>

  {#if cs.connected}
    <div class="cloud-connected">
      <div class="cloud-badge">
        <span class="cloud-dot" class:connected={cs.email_verified} class:pending={!cs.email_verified}></span>
        {#if cs.email_verified}
          <span>Conectado como <strong>{cs.user_name}</strong> ({cs.email})</span>
        {:else}
          <span>Validación pendiente - {cs.email}</span>
        {/if}
      </div>

      {#if !cs.email_verified}
        <div class="verification-banner">
          <p>Email no verificado. Revisa tu bandeja de entrada e ingresa el codigo que te enviamos.</p>
          <div class="verification-row">
            <input
              type="text"
              maxlength={6}
              placeholder="Código de 6 dígitos"
              bind:value={verificationCode}
              onkeydown={(e) => { if (e.key === "Enter") appState.verifyEmailCode(verificationCode) }}
            />
            <button
              type="button"
              onclick={() => appState.verifyEmailCode(verificationCode)}
              disabled={verificationCode.length !== 6}
            >
              Verificar
            </button>
          </div>
          <button type="button" class="link" onclick={() => appState.resendVerificationCode()}>
            Reenviar código
          </button>
          <div class="verification-actions">
            <button type="button" class="secondary" onclick={() => showChangeEmail = !showChangeEmail}>
              Cambiar email
            </button>
            <button type="button" class="secondary danger" onclick={() => appState.deleteCloudAccount()}>
              Eliminar cuenta
            </button>
          </div>
          {#if showChangeEmail}
            <div class="change-email-row">
              <input
                type="email"
                placeholder="Nuevo email"
                bind:value={newEmail}
                onkeydown={(e) => { if (e.key === "Enter") { appState.changeCloudEmail(newEmail); newEmail = ""; showChangeEmail = false } }}
              />
              <button
                type="button"
                onclick={() => { appState.changeCloudEmail(newEmail); newEmail = ""; showChangeEmail = false }}
                disabled={!newEmail.includes('@')}
              >
                Guardar
              </button>
            </div>
          {/if}
        </div>
      {:else}
        {#if cs.last_sync}
          <p class="eyebrow">Ultima sincronización: {cs.last_sync}</p>
        {:else}
          <p class="eyebrow">Aun no se ha sincronizado</p>
        {/if}

        <label class="toggle-label">
          <input
            type="checkbox"
            checked={cs.auto_login}
            onchange={(e) => appState.setAutoLogin(e.currentTarget.checked)}
          />
          Auto-login al iniciar
        </label>

        <div class="form-actions" style="margin-top: 1.5rem">
          <button type="button" onclick={() => appState.syncNow()}>
            Sincronizar ahora
          </button>
          <button type="button" onclick={() => appState.forceSyncFromCloud()}>
            Forzar desde nube
          </button>
          <button type="button" class="secondary" onclick={() => appState.logoutCloudAccount()}>
            Cerrar sesión
          </button>
        </div>
      {/if}
    </div>

  {:else if f.mode === "login"}
    <form onsubmit={(e) => appState.loginCloudAccount(e)}>
      <label>
        Email
        <input type="email" bind:value={f.email} required placeholder="tucorreo@ejemplo.com" />
      </label>
      <label>
        Contraseña
        <input type="password" bind:value={f.password} required placeholder="Mínimo 8 caracteres" />
      </label>
      <div class="form-actions">
        <button type="submit">Iniciar sesión</button>
      </div>
      <p class="eyebrow" style="margin-top: 0.5rem">
        No tienes cuenta?
        <button type="button" class="link" onclick={toggleMode}>Crear cuenta</button>
      </p>
    </form>

  {:else}
    <form onsubmit={(e) => appState.registerCloudAccount(e)}>
      <label>
        Nombre
        <input type="text" bind:value={f.name} required placeholder="Tu nombre" minlength={2} />
      </label>
      <label>
        Email
        <input type="email" bind:value={f.email} required placeholder="tucorreo@ejemplo.com" />
      </label>
      <label>
        Contraseña
        <input type="password" bind:value={f.password} required placeholder="Mínimo 8 caracteres" minlength={8} />
      </label>
      <label>
        Confirmar contraseña
        <input type="password" bind:value={f.confirmPassword} required placeholder="Repite la contraseña" />
      </label>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={f.consent} />
        Acepto que mis datos se almacenen en la nube (Baserow.io)
      </label>
      <div class="form-actions">
        <button type="submit">Crear cuenta</button>
      </div>
      <p class="eyebrow" style="margin-top: 0.5rem">
        Ya tienes cuenta?
        <button type="button" class="link" onclick={toggleMode}>Iniciar sesión</button>
      </p>
    </form>
  {/if}
</section>

<style>
  .cloud-tab {
    max-width: 500px;
  }

  .cloud-connected {
    padding: 0.5rem 0;
  }

  .cloud-badge {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .cloud-dot {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #aaa;
  }

  .cloud-dot.connected {
    background: #2ecc71;
  }

  .cloud-dot.pending {
    background: #e74c3c;
  }

  .checkbox-label {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    font-size: 0.9rem;
    margin: 0.75rem 0;
  }

  .checkbox-label input[type="checkbox"] {
    margin-top: 0.15rem;
    width: auto;
  }

  .form-actions {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0.75rem 0;
    cursor: pointer;
  }

  .toggle-label input[type="checkbox"] {
    width: auto;
  }

  button.link {
    background: none;
    border: none;
    color: var(--primary, #3498db);
    text-decoration: underline;
    cursor: pointer;
    padding: 0;
    font: inherit;
  }

  .verification-banner {
    background: #fff3cd;
    border: 1px solid #ffc107;
    border-radius: 6px;
    padding: 1rem;
    margin: 0.75rem 0;
  }

  .verification-banner p {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
    color: #856404;
  }

  .verification-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .verification-row input {
    flex: 1;
    max-width: 180px;
    text-align: center;
    font-size: 1.1rem;
    letter-spacing: 0.25rem;
  }

  .verification-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
    flex-wrap: wrap;
  }

  .change-email-row {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .change-email-row input {
    flex: 1;
  }

  button.danger {
    border-color: #e74c3c;
    color: #e74c3c;
  }

  button.danger:hover {
    background: #e74c3c;
    color: #fff;
  }
</style>
