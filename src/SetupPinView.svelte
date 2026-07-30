<script lang="ts">
  let pin = $state("")

  /**
   * @property onSubmit - Callback con el PIN ingresado al enviar el formulario
   */
  let {
    onSubmit = (_pin: string) => {},
  } = $props()

  function handleSubmit(e: Event) {
    e.preventDefault()
    onSubmit(pin)
    pin = ""
  }
</script>

<section class="grid two-columns">
  <div class="panel intro-panel">
    <p class="eyebrow">Primer arranque</p>
    <h2>Configura el PIN adulto</h2>
    <p>Este PIN protege perfiles y configuracion. Se guarda con hash, nunca en claro.</p>
  </div>
  <form class="card" onsubmit={handleSubmit}>
    <label for="setup-pin">PIN de 4 a 6 digitos</label>
    <input id="setup-pin" type="password" inputmode="numeric" bind:value={pin} />
    <button type="submit">Guardar PIN</button>
  </form>
</section>
