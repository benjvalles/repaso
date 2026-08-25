<script lang="ts">
  import type { Profile } from "./lib/types"

  /**
   * @property profiles - Lista de perfiles disponibles
   * @property selectedProfileId - ID del perfil seleccionado actualmente
   * @property onSelectProfile - Callback al pulsar un perfil (selecciona, no inicia sesion)
   * @property onStartChat - Callback para abrir el chat
   * @property onStartSession - Callback para iniciar entrenamiento
   * @property courseLabel - Funcion para formatear el curso
   */
  let {
    profiles = [] as Profile[],
    selectedProfileId = "",
    onSelectProfile = (_id: string) => {},
    onStartChat = () => {},
    onStartSession = (_id: string) => {},
    courseLabel = (_c: number) => "",
  }: {
    profiles: Profile[]
    selectedProfileId: string
    onSelectProfile: (id: string) => void
    onStartChat: () => void
    onStartSession: (id: string) => void
    courseLabel: (year: number) => string
  } = $props()

  let selectedProfile = $derived(profiles.find(p => p.id === selectedProfileId) || null)
</script>

<section class="panel child-zone">
  <p class="eyebrow">Zona infantil</p>
  <h2>Elige tu perfil</h2>
  {#if profiles.length === 0}
    <p class="empty">Todavia no hay perfiles.</p>
  {:else}
    <div class="profile-grid">
      {#each profiles as p (p.id)}
        <button
          class="profile-card"
          class:selected={p.id === selectedProfileId}
          type="button"
          onclick={() => onSelectProfile(p.id)}
        >
          {#if p.id === selectedProfileId}
            <span class="check-icon">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
            </span>
          {/if}
          <div class="profile-info">
            <strong>{p.display_name}</strong>
            <span>{courseLabel(p.school_year)} · nivel {p.current_level}</span>
          </div>
        </button>
      {/each}
    </div>

    {#if selectedProfile}
      <div class="action-buttons">
        <button class="primary" type="button" onclick={() => onStartSession(selectedProfileId)}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="6 3 20 12 6 21 6 3"/></svg>
          Entrenamiento
        </button>
        <button class="secondary" type="button" onclick={onStartChat}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z"/></svg>
          Chat
        </button>
      </div>
    {/if}
  {/if}
</section>

<style>
  .profile-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 0.75rem;
    margin-top: 1rem;
  }

  .profile-card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    padding: 1rem;
    border: 2px solid #e2e8f0;
    border-radius: 12px;
    background: white;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: center;
  }

  .profile-card:hover {
    border-color: #94a3b8;
    background: #f8fafc;
  }

  .profile-card.selected {
    border-color: #3b82f6;
    background: #eff6ff;
  }

  .check-icon {
    position: absolute;
    top: 8px;
    right: 8px;
    color: #3b82f6;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .profile-info {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .profile-info strong {
    font-size: 1rem;
    color: #1e293b;
  }

  .profile-info span {
    font-size: 0.85rem;
    color: #64748b;
  }

  .action-buttons {
    display: flex;
    gap: 0.75rem;
    margin-top: 1.25rem;
    justify-content: center;
  }

  .action-buttons button {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.6rem 1.2rem;
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: background 0.15s ease;
  }

  .action-buttons .primary {
    background: #3b82f6;
    color: white;
  }

  .action-buttons .primary:hover {
    background: #2563eb;
  }

  .action-buttons .secondary {
    background: #f1f5f9;
    color: #475569;
    border: 1px solid #e2e8f0;
  }

  .action-buttons .secondary:hover {
    background: #e2e8f0;
  }
</style>
