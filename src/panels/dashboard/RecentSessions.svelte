<script lang="ts">
  import { appState } from "../../lib/app-state.svelte"
</script>

{#if appState.recentSessions.length > 0}
  <div class="card">
    <h3>Sesiones recientes</h3>
    <div class="recent-sessions-list">
      {#each appState.recentSessions as s}
        <div class="recent-session-row">
          <div class="recent-session-date">{new Date(s.started_at).toLocaleDateString("es-ES")}</div>
          <div class="recent-session-accuracy">
            <div class="recent-session-bar">
              <div class="recent-session-fill" style="width: {s.total_questions > 0 ? (s.correct_count / s.total_questions * 100) : 0}%"></div>
            </div>
            <span>{s.correct_count}/{s.total_questions} · {s.total_questions > 0 ? Math.round(s.correct_count / s.total_questions * 100) : 0}%</span>
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}
