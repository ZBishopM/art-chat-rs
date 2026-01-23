<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import ConnectionIndicator from './ConnectionIndicator.svelte';
  import type { ConnectionState } from '$lib/types';

  export let nickname: string;
  export let soundEnabled: boolean;
  export let selectedColor: string;
  export let brushSize: number;
  export let fadeEnabled: boolean;
  export let fadeSpeed: number;
  export let connectionState: ConnectionState;
  export let backgroundColor: string;

  const dispatch = createEventDispatcher<{
    clear: void;
    toggleSound: void;
  }>();
</script>

<div class="toolbar">
  <input
    type="text"
    bind:value={nickname}
    placeholder="Tu Nick"
    class="nick-input"
  />

  <button
    on:click={() => dispatch('toggleSound')}
    title={soundEnabled ? "Silenciar" : "Activar Sonido"}
    class="sound-btn"
  >
    {soundEnabled ? '🔊' : '🔇'}
  </button>

  <div class="separator"></div>

  <input type="color" bind:value={selectedColor} title="Color de pincel" class="color-picker" />
  <input type="color" bind:value={backgroundColor} title="Color de fondo" class="color-picker bg-picker" />

  <div class="separator"></div>

  <input
    type="range"
    min="1"
    max="50"
    bind:value={brushSize}
    title="Grosor"
  />
  <span class="brush-size">{brushSize}</span>

  <div class="separator"></div>

  <button
    on:click={() => dispatch('clear')}
    title="Borrar Todo"
    class="clear-btn"
  >
    🗑️
  </button>

  <div class="separator"></div>

  <label class="fade-label">
    <input type="checkbox" bind:checked={fadeEnabled} />
    👻 Fade
  </label>

  {#if fadeEnabled}
    <input
      type="range"
      min="20"
      max="500"
      step="10"
      bind:value={fadeSpeed}
      title="Velocidad de desvanecimiento"
      class="fade-slider"
    />
  {/if}

  <div class="separator"></div>

  <ConnectionIndicator state={connectionState} />
</div>

<style>
  .toolbar {
    position: absolute;
    top: 10px;
    left: 10px;
    right: 200px;
    background: #1a1a1a;
    padding: 10px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    pointer-events: auto;
    border: 1px solid #444;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
  }

  .separator {
    width: 1px;
    height: 20px;
    background: #444;
  }

  .nick-input {
    width: 80px;
    background: #2a2a2a;
    color: #00ffff;
    border: 1px solid #444;
    padding: 5px;
    border-radius: 4px;
    text-align: center;
  }

  .sound-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1.2rem;
  }

  .color-picker {
    border: none;
    width: 30px;
    height: 30px;
    cursor: pointer;
    background: none;
  }

  .bg-picker {
    border: 2px dashed #888;
    border-radius: 4px;
  }

  .brush-size {
    color: white;
    font-size: 0.8rem;
    min-width: 20px;
    text-align: center;
  }

  .clear-btn {
    background: #c00;
    color: white;
    border: none;
    padding: 5px 10px;
    border-radius: 4px;
    cursor: pointer;
  }

  .clear-btn:hover {
    background: #e00;
  }

  .fade-label {
    color: white;
    font-size: 0.8rem;
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }

  .fade-slider {
    width: 80px;
  }

  input[type="range"] {
    cursor: pointer;
  }

  /* Responsive */
  @media (max-width: 700px) {
    .toolbar {
      right: 10px;
      top: 10px;
    }

    .separator {
      display: none;
    }

    .nick-input {
      width: 60px;
      font-size: 0.8rem;
    }

    .fade-slider {
      width: 60px;
    }

    input[type="range"] {
      width: 60px;
    }
  }

  @media (max-width: 400px) {
    .toolbar {
      padding: 6px;
      gap: 5px;
    }

    .nick-input {
      width: 50px;
    }

    .color-picker {
      width: 25px;
      height: 25px;
    }

    .clear-btn {
      padding: 4px 8px;
      font-size: 0.9rem;
    }

    .fade-label {
      font-size: 0.7rem;
    }
  }
</style>
