<script lang="ts">
  import { createEventDispatcher, afterUpdate } from 'svelte';

  export let messages: string[] = [];

  const dispatch = createEventDispatcher<{ send: string; buzz: void }>();

  let chatInput = "";
  let messagesContainer: HTMLDivElement;
  let isCollapsed = localStorage.getItem('artchat_chat_collapsed') === 'true';
  let buzzCooldown = false;

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") sendMessage();
  }

  function sendMessage() {
    if (!chatInput.trim()) return;
    dispatch('send', chatInput);
    chatInput = "";
  }

  function sendBuzz() {
    if (buzzCooldown) return;
    dispatch('buzz');
    buzzCooldown = true;
    setTimeout(() => buzzCooldown = false, 3000); // Cooldown de 3 segundos
  }

  function toggleCollapse() {
    isCollapsed = !isCollapsed;
    localStorage.setItem('artchat_chat_collapsed', String(isCollapsed));
  }

  // Auto-scroll al último mensaje
  afterUpdate(() => {
    if (messagesContainer) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
  });
</script>

{#if isCollapsed}
  <button class="chat-toggle collapsed" on:click={toggleCollapse} title="Mostrar chat">
    💬
  </button>
{:else}
  <div class="chat-container">
    <div class="chat-header">
      <span class="chat-title">Chat</span>
      <button class="collapse-btn" on:click={toggleCollapse} title="Ocultar chat">−</button>
    </div>
    <div class="messages" bind:this={messagesContainer}>
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
      <button
        class="buzz-btn"
        class:cooldown={buzzCooldown}
        on:click={sendBuzz}
        title={buzzCooldown ? "Espera..." : "Enviar zumbido"}
        disabled={buzzCooldown}
      >
        📳
      </button>
    </div>
  </div>
{/if}

<style>
  .chat-container {
    position: absolute;
    bottom: 20px;
    left: 20px;
    width: 300px;
    height: 200px;
    background: #1a1a1a;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    padding: 10px;
    color: white;
    pointer-events: auto;
    border: 1px solid #444;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
  }

  .chat-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    padding-bottom: 5px;
    border-bottom: 1px solid #444;
  }

  .chat-title {
    font-size: 0.8rem;
    color: #888;
    text-transform: uppercase;
  }

  .collapse-btn {
    background: none;
    border: none;
    color: #888;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0 5px;
    line-height: 1;
  }

  .collapse-btn:hover {
    color: white;
  }

  .chat-toggle {
    position: absolute;
    bottom: 20px;
    left: 20px;
    width: 40px;
    height: 40px;
    background: #1a1a1a;
    border: 1px solid #444;
    border-radius: 8px;
    font-size: 1.2rem;
    cursor: pointer;
    pointer-events: auto;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
  }

  .chat-toggle:hover {
    background: #2a2a2a;
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

  .buzz-btn {
    background: #ff6600 !important;
    padding: 5px 8px !important;
  }

  .buzz-btn:hover:not(:disabled) {
    background: #ff8833 !important;
  }

  .buzz-btn.cooldown {
    background: #666 !important;
    cursor: not-allowed;
    opacity: 0.6;
  }

  /* Responsive */
  @media (max-width: 700px) {
    .chat-container {
      width: calc(100vw - 40px);
      max-width: 300px;
      height: 150px;
      bottom: 10px;
      left: 10px;
    }

    .chat-toggle {
      bottom: 10px;
      left: 10px;
    }

    .messages {
      font-size: 0.8rem;
    }
  }

  @media (max-width: 400px) {
    .chat-container {
      height: 120px;
      padding: 8px;
    }

    .input-area input {
      font-size: 0.8rem;
    }

    .input-area button {
      padding: 5px 8px;
      font-size: 0.8rem;
    }
  }
</style>
