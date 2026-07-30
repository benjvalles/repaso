<script lang="ts">
  let pin = $state("")
  let input: HTMLInputElement | undefined = $state(undefined)

  /**
   * @property onSubmit - Callback con el PIN al desbloquear
   * @property onBack - Callback para volver atrás
   */
  let {
    onSubmit = (_pin: string) => {},
    onBack = () => {},
  } = $props()

  function handleSubmit(e: Event) {
    e.preventDefault()
    onSubmit(pin)
    pin = ""
  }

  $effect(() => { input?.focus() })
</script>

<section class="grid two-columns">
  <div class="panel intro-panel">
    <p class="eyebrow">Zona adulta</p>
    <h2>Introduce el PIN</h2>
    <p>Introduce el PIN de 4 a 6 digitos para acceder a la configuracion.</p>
  </div>
  <form class="card" onsubmit={handleSubmit}>
    <label for="unlock-pin">PIN</label>
    <input id="unlock-pin" type="password" inputmode="numeric" bind:value={pin} bind:this={input} />
    <button type="submit" disabled={!pin}>Desbloquear</button>
    <button class="secondary" type="button" onclick={() => onBack()}>Volver</button>
  </form>
</section>
