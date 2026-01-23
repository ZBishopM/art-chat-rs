<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { User } from '$lib/types';

  export let users: User[] = [];
  export let myId: string;
  export let myNickname: string;
  export let myStatus: 'online' | 'busy' = 'online';

  const dispatch = createEventDispatcher<{ toggleStatus: void }>();
</script>

<div class="users-panel">
  <div class="panel-title">Usuarios ({users.length})</div>

  <button
    class="user-row myself"
    on:click={() => dispatch('toggleStatus')}
    title="Click para cambiar estado"
  >
    <div class="status-dot {myStatus}"></div>
    <span class="user-name">{myNickname} (Tú)</span>
  </button>

  {#each users as user}
    {#if user.id !== myId}
      <div class="user-row">
        <div class="status-dot {user.status}"></div>
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

  .panel-title {
    font-size: 0.8rem;
    color: #888;
    border-bottom: 1px solid #444;
    padding-bottom: 5px;
    margin-bottom: 5px;
    text-transform: uppercase;
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
    background-color: gray;
    box-shadow: 0 0 5px rgba(0, 0, 0, 0.5);
    flex-shrink: 0;
  }

  .status-dot.online {
    background-color: #00ff00;
    box-shadow: 0 0 8px #00ff00;
  }

  .status-dot.busy {
    background-color: #ff0000;
    box-shadow: 0 0 8px #ff0000;
  }
</style>
