<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { User } from '$lib/types';

  export let users: User[] = [];
  export let myId: string;
  export let myNickname: string;
  export let myStatus: 'online' | 'busy' = 'online';
  export let onlineColor: string = '#00ff00';
  export let busyColor: string = '#ff0000';

  const dispatch = createEventDispatcher<{ toggleStatus: void }>();

  function getStatusColor(status: string): string {
    return status === 'online' ? onlineColor : busyColor;
  }
</script>

<div class="users-panel">
  <div class="panel-header">
    <span class="panel-title">Usuarios ({users.length})</span>
    <div class="status-colors">
      <input type="color" bind:value={onlineColor} title="Color online" class="status-color-picker" />
      <input type="color" bind:value={busyColor} title="Color busy" class="status-color-picker" />
    </div>
  </div>

  <button
    class="user-row myself"
    on:click={() => dispatch('toggleStatus')}
    title="Click para cambiar estado"
  >
    <div
      class="status-dot"
      style="background-color: {getStatusColor(myStatus)}; box-shadow: 0 0 8px {getStatusColor(myStatus)};"
    ></div>
    <span class="user-name">{myNickname} (Tú)</span>
  </button>

  {#each users as user}
    {#if user.id !== myId}
      <div class="user-row">
        <div
          class="status-dot"
          style="background-color: {getStatusColor(user.status)}; box-shadow: 0 0 8px {getStatusColor(user.status)};"
        ></div>
        <span class="user-name" style="color: {user.color}">{user.nickname}</span>
      </div>
    {/if}
  {/each}
</div>

<style>
  .users-panel {
    position: absolute;
    top: 20px;
    right: 20px;
    width: 180px;
    background: #1a1a1a;
    border-radius: 8px;
    padding: 10px;
    color: white;
    pointer-events: auto;
    border: 1px solid #444;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #444;
    padding-bottom: 5px;
    margin-bottom: 5px;
  }

  .panel-title {
    font-size: 0.8rem;
    color: #888;
    text-transform: uppercase;
  }

  .status-colors {
    display: flex;
    gap: 4px;
  }

  .status-color-picker {
    width: 18px;
    height: 18px;
    border: none;
    cursor: pointer;
    background: none;
    padding: 0;
  }

  .user-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
    font-size: 0.9rem;
  }

  .myself {
    cursor: pointer;
    background: rgba(255, 255, 255, 0.1);
    padding: 5px;
    border-radius: 4px;
    margin-bottom: 5px;
    border: none;
    width: 100%;
    text-align: left;
    color: white;
  }

  .myself:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  /* Responsive */
  @media (max-width: 700px) {
    .users-panel {
      top: auto;
      bottom: 10px;
      right: 10px;
      width: 140px;
      max-height: 150px;
      overflow-y: auto;
      padding: 8px;
    }

    .panel-title {
      font-size: 0.7rem;
    }

    .user-row {
      font-size: 0.8rem;
      padding: 3px 0;
    }

    .status-dot {
      width: 8px;
      height: 8px;
    }
  }

  @media (max-width: 400px) {
    .users-panel {
      width: 100px;
      max-height: 100px;
    }

    .user-name {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      max-width: 70px;
    }
  }
</style>
