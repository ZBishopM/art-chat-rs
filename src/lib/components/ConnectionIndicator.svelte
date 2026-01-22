<script lang="ts">
  import type { ConnectionState } from '$lib/types';

  export let state: ConnectionState;
</script>

<div class="connection-indicator">
  {#if state === 'connected'}
    <span class="conn-dot green"></span>
    <span>Conectado</span>
  {:else if state === 'connecting'}
    <span class="conn-dot yellow"></span>
    <span>Conectando...</span>
  {:else if state === 'error'}
    <span class="conn-dot red"></span>
    <span>Error</span>
  {:else}
    <span class="conn-dot gray"></span>
    <span>Desconectado</span>
  {/if}
</div>

<style>
  .connection-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: #aaa;
  }

  .conn-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: inline-block;
  }

  .conn-dot.green {
    background-color: #00ff00;
    box-shadow: 0 0 6px #00ff00;
  }

  .conn-dot.yellow {
    background-color: #ffff00;
    box-shadow: 0 0 6px #ffff00;
    animation: pulse 1s infinite;
  }

  .conn-dot.red {
    background-color: #ff0000;
    box-shadow: 0 0 6px #ff0000;
  }

  .conn-dot.gray {
    background-color: #888;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
