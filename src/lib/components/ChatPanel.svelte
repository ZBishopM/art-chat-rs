<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let messages: string[] = [];

  const dispatch = createEventDispatcher<{ send: string }>();

  let chatInput = "";

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") sendMessage();
  }

  function sendMessage() {
    if (!chatInput.trim()) return;
    dispatch('send', chatInput);
    chatInput = "";
  }
</script>

<div class="chat-container">
  <div class="messages">
    <div class="msg system">Bienvenido al Art-Chat</div>
    {#each messages as msg}
      <div class="msg">{msg}</div>
    {/each}
  </div>
  <div class="input-area">
    <input
      type="text"
      bind:value={chatInput}
      on:keydown={handleKeydown}
      placeholder="Escribe algo..."
    />
    <button on:click={sendMessage}>Enviar</button>
  </div>
</div>

<style>
  .chat-container {
    position: absolute;
    bottom: 20px;
    left: 20px;
    width: 300px;
    height: 200px;
    background: rgba(0, 0, 0, 0.8);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    padding: 10px;
    color: white;
    pointer-events: auto;
  }

  .messages {
    flex-grow: 1;
    overflow-y: auto;
    font-size: 0.9rem;
    margin-bottom: 8px;
  }

  .msg {
    margin-bottom: 4px;
  }

  .system {
    color: bisque;
    padding-bottom: 1rem;
  }

  .input-area {
    display: flex;
    gap: 5px;
  }

  .input-area input {
    flex-grow: 1;
    background: #333;
    border: none;
    color: white;
    padding: 5px;
    border-radius: 4px;
  }

  .input-area button {
    background: #555;
    border: none;
    color: white;
    padding: 5px 10px;
    border-radius: 4px;
    cursor: pointer;
  }

  .input-area button:hover {
    background: #666;
  }
</style>
