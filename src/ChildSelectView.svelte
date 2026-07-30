<script lang="ts">
  import type { Profile } from "./lib/types"

  /**
   * @property profiles - Lista de perfiles disponibles
   * @property onStartSession - Callback al pulsar un perfil, recibe el ID
   * @property courseLabel - Función para formatear el curso (ej: "3o Primaria")
   */
  let {
    profiles = [] as Profile[],
    onStartSession = (_profileId: string) => {},
    courseLabel = (_c: number) => "",
  } = $props()
</script>

<section class="panel child-zone">
  <p class="eyebrow">Zona infantil</p>
  <h2>Elige tu perfil</h2>
  {#if profiles.length === 0}
    <p class="empty">Todavía no hay perfiles.</p>
  {:else}
    <div class="profile-grid">
      {#each profiles as p (p.id)}
        <button class="profile-card" type="button" onclick={() => onStartSession(p.id)}>
          <strong>{p.display_name}</strong>
          <span>{courseLabel(p.school_year)} · nivel {p.current_level}</span>
        </button>
      {/each}
    </div>
  {/if}
</section>
