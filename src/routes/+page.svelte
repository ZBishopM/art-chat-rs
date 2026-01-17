<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { type UnlistenFn } from "@tauri-apps/api/event";


  let unlisten: UnlistenFn; // Para el chat
  let unlistenConnection: UnlistenFn; // Para el estado
  let joinInterval: ReturnType<typeof setInterval>; // Para el intervalo

  // Estructura de una línea que está muriendo
  interface FadingStroke {
    x0: number;
    y0: number;
    x1: number;
    y1: number;
    color: string;
    size: number;
    startTime: number; // Cuándo nació
    duration: number; // Cuánto debe vivir (en ms)
  }

  // status
  let connectedUsers: any[] = [];
  let myStatus = "online"; // 'online' | 'busy'

  // Almacén de trazos vivos
  let fadingStrokes: FadingStroke[] = [];
  // --- VARIABLES DE LIENZO (NUEVA ARQUITECTURA) ---
  let canvasPermanent: HTMLCanvasElement;
  let canvasFade: HTMLCanvasElement;

  let ctxPerm: CanvasRenderingContext2D | null;
  let ctxFade: CanvasRenderingContext2D | null;

  // --- ESTADO DE LA APP CON PERSISTENCIA ---

  // Función auxiliar para cargar o usar default
  const load = (key: string, def: any) => {
    const val = localStorage.getItem(key);
    return val ? JSON.parse(val) : def;
  };

  // Identidad
  // Si no tiene nombre guardado, generamos uno aleatorio, pero intentamos guardarlo
  let myNickname = load(
    "artchat_nick",
    "Artista_" + Math.floor(Math.random() * 1000),
  );
  // El ID técnico sigue siendo necesario para diferenciar máquinas,
  // pero ya no lo mostraremos al usuario.
  const myId = load("artchat_id", Math.random().toString(36).substr(2, 9));
  // Guardamos el ID para que siempre seas el mismo "nodo" técnico
  localStorage.setItem("artchat_id", JSON.stringify(myId));

  let soundEnabled = load('artchat_sound', true); // Por defecto activado
  
  // Guardar preferencia automáticamente
  $: localStorage.setItem('artchat_sound', JSON.stringify(soundEnabled));

  // Objeto de audio (fuera del onMount para que sea accesible)
  let notificationAudio: HTMLAudioElement;
  let connectedAudio: HTMLAudioElement;


  // Herramientas
  let isDrawing = false;
  let selectedColor = load("artchat_color", "#ffffff");
  let brushSize = load("artchat_size", 5);

  let fadeEnabled = load("artchat_fade", false);
  let fadeSpeed = load("artchat_fadespeed", 50);
  let fadeIntervalId: number | null = null;

  let chatInput = "";
  let messages: string[] = [];

  // --- BUCLE DE ANIMACIÓN (NUEVO) ---
  function animationLoop() {
    if (!ctxFade) return;

    // 1. Limpiar completamente el lienzo efímero (¡Adiós fantasmas!)
    ctxFade.clearRect(0, 0, canvasFade.width, canvasFade.height);

    // 2. Filtrar y Dibujar
    const now = Date.now();

    // Mantenemos solo los trazos que aún tienen tiempo de vida
    // (Esto es muy eficiente en JS moderno)
    fadingStrokes = fadingStrokes.filter((stroke) => {
      const elapsed = now - stroke.startTime;

      // Si ya pasó su tiempo, retorna false (se elimina del array y desaparece)
      if (elapsed >= stroke.duration) return false;

      // Calcular opacidad (de 1.0 a 0.0)
      const opacity = 1 - elapsed / stroke.duration;

      // Dibujar
      ctxFade!.globalAlpha = opacity; // Magia de transparencia
      ctxFade!.lineWidth = stroke.size;
      ctxFade!.strokeStyle = stroke.color;
      ctxFade!.fillStyle = stroke.color;
      ctxFade!.lineCap = "round";

      ctxFade!.beginPath();
      if (stroke.x0 === stroke.x1 && stroke.y0 === stroke.y1) {
        // Es un punto
        ctxFade!.arc(stroke.x0, stroke.y0, stroke.size / 2, 0, Math.PI * 2);
        ctxFade!.fill();
      } else {
        // Es una línea
        ctxFade!.moveTo(stroke.x0, stroke.y0);
        ctxFade!.lineTo(stroke.x1, stroke.y1);
        ctxFade!.stroke();
      }

      return true; // Sigue vivo
    });

    // Resetear opacidad para otras cosas
    ctxFade.globalAlpha = 1.0;

    // Pedir el siguiente frame
    requestAnimationFrame(animationLoop);
  }

  // 1. Extraemos la lógica de unirse a una función reutilizable
  async function sendJoin() {
    console.log("🚀 Enviando solicitud de JOIN...");
    const payload = JSON.stringify({
        type: 'join',
        senderId: myId,
        nickname: myNickname,
        color: selectedColor
    });
    // Usamos catch por si Rust aún no está listo (aunque no debería pasar con el evento)
    await invoke("send_message", { msg: payload }).catch(e => console.error("Fallo al enviar join:", e));
  }

  // --- INICIALIZACIÓN ---
  onMount(async () => {
    // 1. Configurar Capa Permanente (Fondo)
    ctxPerm = canvasPermanent.getContext("2d");
    canvasPermanent.width = window.innerWidth;
    canvasPermanent.height = window.innerHeight;

    // 2. Configurar Capa Efímera (Frente)
    ctxFade = canvasFade.getContext("2d");
    canvasFade.width = window.innerWidth;
    canvasFade.height = window.innerHeight;

    notificationAudio = new Audio('/notify.mp3'); // La ruta es relativa a 'public'
    notificationAudio.volume = 0.5; // 50% volumen para no asustar

    connectedAudio = new Audio('/connected.mp3'); // <--- CARGAMOS EL NUEVO
    connectedAudio.volume = 0.5;

    window.addEventListener("resize", handleResize);
    // ARRANCAR MOTOR
    requestAnimationFrame(animationLoop);

    // 3. Escuchar Red
    unlisten = await listen<string>("chat-message", (event) => {
      try {
        const data = JSON.parse(event.payload);
        if (data.type === "users_update") {
          if (soundEnabled && connectedAudio && data.users.length > connectedUsers.length) {
              connectedAudio.play().catch(e => console.error("Audio block:", e));
          }
          connectedUsers = data.users;
          return;
        }

        if (data.senderId === myId) return; // Ignorar eco propio

        if (data.type === "chat") {
          const name = data.nickname || "Anónimo";
          messages = [...messages, `${name}: ${data.content}`];
          // --- NUEVO: REPRODUCIR SONIDO ---
          if (soundEnabled && notificationAudio) {
              // El .catch evita errores si el usuario aún no ha interactuado con la página
              notificationAudio.play().catch(e => console.log("Audio bloqueado:", e));
          }
          // --------------------------------
        } else if (data.type === "draw") {
          // Leemos la bandera 'ephemeral' (antes fade)
          const isEphemeral = data.ephemeral || false;
          drawRemote(data, isEphemeral);
        } else if (data.type === "clear") {
          // Limpiar AMBOS lienzos si llega la orden
          if (ctxPerm && ctxFade) {
            ctxPerm.clearRect(0, 0, canvasPermanent.width, canvasPermanent.height);
            ctxFade.clearRect(0, 0, canvasFade.width, canvasFade.height);
          }
        }
      } catch (e) {
        console.error("Error JSON:", e);
      }
    });

    // --- NUEVO: ESCUCHAR ESTADO DE CONEXIÓN (La señal de Rust) ---
    unlistenConnection = await listen<string>('connection-status', (event) => {
        if (event.payload === 'connected') {
            console.log("⚡ Rust confirmó conexión. Enviando JOIN inmediato.");
            sendJoin();
        }
    });

    // --- RED DE SEGURIDAD (Reintentos inteligentes) ---
    // Si por alguna razón el evento de Rust llegó antes de que cargara Svelte,
    // o si el servidor se reinició, insistimos hasta vernos en la lista.
    joinInterval = setInterval(() => {
        // Si NO estoy en la lista de usuarios, intento unirme
        const amIConnected = connectedUsers.some(u => u.id === myId);
        if (!amIConnected) {
            sendJoin();
        }
    }, 2000); // Cada 2 segundos
  });

  // --- LIMPIEZA AL CERRAR ---
  onDestroy(() => {
    if (unlisten) unlisten();
    if (unlistenConnection) unlistenConnection(); // Aquí ya es una función, se llama directo
    if (joinInterval) clearInterval(joinInterval);
    if (fadeIntervalId) clearInterval(fadeIntervalId); // Limpiamos el otro intervalo si existe
  });

  // --- LÓGICA DE REDIMENSIONADO (ROBUSTA PARA 2 CAPAS) ---
  function handleResize() {
    // Guardamos el contenido de ambas capas
    const tempCanvasPerm = document.createElement("canvas");
    const tempCanvasFade = document.createElement("canvas");

    // Función helper para copiar
    const saveLayer = (
      sourceCanvas: HTMLCanvasElement,
      tempCanvas: HTMLCanvasElement,
    ) => {
      tempCanvas.width = sourceCanvas.width;
      tempCanvas.height = sourceCanvas.height;
      tempCanvas.getContext("2d")?.drawImage(sourceCanvas, 0, 0);
    };

    saveLayer(canvasPermanent, tempCanvasPerm);
    saveLayer(canvasFade, tempCanvasFade);

    // Redimensionamos
    [canvasPermanent, canvasFade].forEach((c) => {
      c.width = window.innerWidth;
      c.height = window.innerHeight;
    });

    // Restauramos
    ctxPerm?.drawImage(tempCanvasPerm, 0, 0);
    ctxFade?.drawImage(tempCanvasFade, 0, 0);
  }

  // --- LÓGICA DE DESVANECIMIENTO (CORREGIDA) ---
  // Este bloque reactivo gestiona el bucle de limpieza de la capa superior
  /*$: {
    // Si hay cambio de velocidad o se inicializa...
    if (fadeIntervalId) clearInterval(fadeIntervalId);
    
    // SIEMPRE corremos el intervalo en la capa Fade, pero solo borra si hay algo.
    // Usamos 'destination-out' para borrar hacia transparencia.
    fadeIntervalId = window.setInterval(() => {
        if (!ctxFade) return;
        
        ctxFade.globalCompositeOperation = 'destination-out';
        // Ajustamos la opacidad del borrado. 
        // Cuanto MENOR es fadeSpeed (más rápido), MÁS borramos (0.2).
        // Cuanto MAYOR es fadeSpeed (más lento), MENOS borramos (0.05).
        // (O simplemente usamos un valor fijo y dejamos que la velocidad del intervalo mande)
        ctxFade.fillStyle = `rgba(0, 0, 0, 0.1)`; 
        ctxFade.fillRect(0, 0, canvasFade.width, canvasFade.height);
        
        ctxFade.globalCompositeOperation = 'source-over';
    }, fadeSpeed);
  }*/
  // --- PERSISTENCIA AUTOMÁTICA ---
  $: localStorage.setItem("artchat_nick", JSON.stringify(myNickname));
  $: localStorage.setItem("artchat_color", JSON.stringify(selectedColor));
  $: localStorage.setItem("artchat_size", JSON.stringify(brushSize));
  $: localStorage.setItem("artchat_fade", JSON.stringify(fadeEnabled));
  $: localStorage.setItem("artchat_fadespeed", JSON.stringify(fadeSpeed));

  // --- UTILIDADES ---
  function getMousePos(evt: MouseEvent) {
    const rect = canvasFade.getBoundingClientRect();
    return {
      x: evt.clientX - rect.left,
      y: evt.clientY - rect.top,
    };
  }

  // Decide en qué capa pintar
  function getCtx(isEphemeral: boolean) {
    return isEphemeral ? ctxFade : ctxPerm;
  }

  // --- DIBUJO ---
  let lastX = 0;
  let lastY = 0;

  function startDrawing(e: MouseEvent) {
    isDrawing = true;
    const pos = getMousePos(e);
    lastX = pos.x;
    lastY = pos.y;

    if (fadeEnabled) {
      // MODO FADE: Solo agregamos al array
      fadingStrokes.push({
        x0: lastX,
        y0: lastY,
        x1: lastX,
        y1: lastY,
        color: selectedColor,
        size: brushSize,
        startTime: Date.now(),
        duration: fadeSpeed * 10, // Multiplico x10 para que el slider sea más usable (200ms a 5000ms)
      });
    } else {
      // MODO NORMAL: Pintamos directo en la capa permanente
      if (ctxPerm) {
        ctxPerm.lineWidth = brushSize;
        ctxPerm.strokeStyle = selectedColor;
        ctxPerm.fillStyle = selectedColor;
        ctxPerm.beginPath();
        ctxPerm.arc(lastX, lastY, ctxPerm.lineWidth / 2, 0, Math.PI * 2);
        ctxPerm.fill();
      }
    }

    // Red
    const payload = JSON.stringify({
      type: "draw",
      senderId: myId,
      x0: lastX,
      y0: lastY,
      x1: lastX,
      y1: lastY,
      color: selectedColor,
      size: brushSize,
      ephemeral: fadeEnabled,
      duration: fadeEnabled ? fadeSpeed * 10 : 0,
    });

    invoke("send_message", { msg: payload });
  }

  function stopDrawing() {
    isDrawing = false;
    // Reseteamos ambos caminos por seguridad
    ctxPerm?.beginPath();
    ctxFade?.beginPath();
  }

  async function draw(e: MouseEvent) {
    if (!isDrawing) return;
    const pos = getMousePos(e);
    const currentX = pos.x;
    const currentY = pos.y;

    if (fadeEnabled) {
      // MODO FADE: Agregar al array
      fadingStrokes.push({
        x0: lastX,
        y0: lastY,
        x1: currentX,
        y1: currentY,
        color: selectedColor,
        size: brushSize,
        startTime: Date.now(),
        duration: fadeSpeed * 10,
      });
    } else {
      // MODO NORMAL: Pintar directo
      if (ctxPerm) {
        ctxPerm.lineWidth = brushSize;
        ctxPerm.strokeStyle = selectedColor;
        ctxPerm.fillStyle = selectedColor;
        ctxPerm.lineCap = "round";
        ctxPerm.beginPath();
        ctxPerm.moveTo(lastX, lastY);
        ctxPerm.lineTo(currentX, currentY);
        ctxPerm.stroke();
      }
    }

    // Red
    const payload = JSON.stringify({
      type: "draw",
      senderId: myId,
      x0: lastX,
      y0: lastY,
      x1: currentX,
      y1: currentY,
      color: selectedColor,
      size: brushSize,
      ephemeral: fadeEnabled,
      duration: fadeEnabled ? fadeSpeed * 10 : 0,
    });

    lastX = currentX;
    lastY = currentY;

    await invoke("send_message", { msg: payload });
  }

  // --- DIBUJO REMOTO ---
  function drawRemote(data: any, isEphemeral: boolean) {
    if (isEphemeral) {
      // --- AQUÍ ESTÁ LA SOLUCIÓN AL BUG 2 ---
      // Usamos data.duration (la velocidad del REMITENTE), no la mía.
      fadingStrokes.push({
        x0: data.x0,
        y0: data.y0,
        x1: data.x1,
        y1: data.y1,
        color: data.color || "#00ff00",
        size: data.size || 5,
        startTime: Date.now(),
        duration: data.duration || 1000, // Fallback por si acaso
      });
    } else {
      // Dibujo permanente normal en ctxPerm
      if (!ctxPerm) return;
      ctxPerm.lineWidth = data.size || 5;
      ctxPerm.strokeStyle = data.color || "#00ff00";
      ctxPerm.fillStyle = data.color || "#00ff00";
      ctxPerm.beginPath();

      if (data.x0 === data.x1 && data.y0 === data.y1) {
        ctxPerm.arc(data.x0, data.y0, ctxPerm.lineWidth / 2, 0, Math.PI * 2);
        ctxPerm.fill();
      } else {
        ctxPerm.lineCap = "round";
        ctxPerm.moveTo(data.x0, data.y0);
        ctxPerm.lineTo(data.x1, data.y1);
        ctxPerm.stroke();
      }
    }
  }

  // --- CHAT Y LIMPIEZA ---
  async function sendMessage() {
    if (!chatInput.trim()) return;
    const payload = JSON.stringify({
      type: "chat",
      senderId: myId,
      nickname: myNickname,
      content: chatInput,
    });
    messages = [...messages, `Yo: ${chatInput}`];
    await invoke("send_message", { msg: payload });
    chatInput = "";
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") sendMessage();
  }

  async function clearCanvas() {
    // 1. Borrar localmente (AMBOS)
    if (ctxPerm && ctxFade) {
      // En la capa permanente usamos fillRect negro para tapar
      ctxPerm.fillStyle = "#1a1a1a";
      ctxPerm.fillRect(0, 0, canvasPermanent.width, canvasPermanent.height);

      // En la capa fade usamos clearRect para volver a transparente
      ctxFade.clearRect(0, 0, canvasFade.width, canvasFade.height);
    }

    // 2. Avisar a la red
    const payload = JSON.stringify({ type: "clear", senderId: myId });
    await invoke("send_message", { msg: payload });
  }

  async function toggleStatus() {
    myStatus = myStatus === "online" ? "busy" : "online";

    const payload = JSON.stringify({
      type: "status",
      senderId: myId,
      status: myStatus,
    });

    await invoke("send_message", { msg: payload });
  }
</script>

<main>
  <div class="canvas-wrapper">
    <canvas bind:this={canvasPermanent}></canvas>

    <canvas
      bind:this={canvasFade}
      on:mousedown={startDrawing}
      on:mouseup={stopDrawing}
      on:mousemove={draw}
      on:mouseleave={stopDrawing}
    ></canvas>
  </div>
  <div class="toolbar">
    <input
      type="text"
      bind:value={myNickname}
      placeholder="Tu Nick"
      style="width: 80px; background: #333; color: cyan; border: 1px solid #555; padding: 5px; border-radius: 4px; text-align: center;"
    />
    <button 
        on:click={() => soundEnabled = !soundEnabled} 
        title={soundEnabled ? "Silenciar" : "Activar Sonido"}
        style="background: none; border: none; cursor: pointer; font-size: 1.2rem;"
    >
        {soundEnabled ? '🔊' : '🔇'}
    </button>
    <div style="width: 1px; height: 20px; background: #555;"></div>
    <input type="color" bind:value={selectedColor} title="Color" />

    <div style="width: 1px; height: 20px; background: #555;"></div>

    <input
      type="range"
      min="1"
      max="50"
      bind:value={brushSize}
      title="Grosor"
    />
    <span
      style="color: white; font-size: 0.8rem; min-width: 20px; text-align: center;"
      >{brushSize}</span
    >

    <div style="width: 1px; height: 20px; background: #555;"></div>

    <button
      on:click={clearCanvas}
      title="Borrar Todo"
      style="background: #c00; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer;"
    >
      🗑️
    </button>

    <div style="width: 1px; height: 20px; background: #555;"></div>

    <label
      style="color: white; font-size: 0.8rem; display: flex; align-items: center; gap: 5px; cursor: pointer;"
    >
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
        title="Velocidad de desvanecimiento (Menos es más rápido)"
        style="width: 80px;"
      />
    {/if}
  </div>
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
  <div class="users-panel">
    <div class="panel-title">Usuarios ({connectedUsers.length})</div>

    <div
      class="user-row myself"
      on:click={toggleStatus}
      title="Click para cambiar estado"
    >
      <div class="status-dot {myStatus}"></div>
      <span class="user-name">{myNickname} (Tú)</span>
    </div>

    {#each connectedUsers as user}
      {#if user.id !== myId}
        <div class="user-row">
          <div class="status-dot {user.status}"></div>
          <span class="user-name" style="color: {user.color}"
            >{user.nickname}</span
          >
        </div>
      {/if}
    {/each}
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden; /* Evitar barras de scroll */
    background-color: #1a1a1a;
  }

  canvas {
    display: block;
    cursor: crosshair;
  }
  /* Agrega esto a tus estilos existentes */
  .chat-container {
    position: absolute;
    bottom: 20px;
    left: 20px;
    width: 300px;
    height: 200px;
    background: rgba(0, 0, 0, 0.8); /* Semitransparente */
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    padding: 10px;
    color: white;
    pointer-events: auto; /* Para poder clickear aunque esté sobre el canvas */
  }

  .messages {
    flex-grow: 1;
    overflow-y: auto;
    font-size: 0.9rem;
    margin-bottom: 8px;
  }

  .input-area {
    display: flex;
    gap: 5px;
  }

  input {
    flex-grow: 1;
    background: #333;
    border: none;
    color: white;
    padding: 5px;
    border-radius: 4px;
  }
  .system {
    color: bisque;
    padding-bottom: 1rem;
  }
  .toolbar {
    position: absolute;
    top: 20px;
    left: 20px;
    background: rgba(0, 0, 0, 0.8);
    padding: 10px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 10px;
    pointer-events: auto;
  }

  input[type="color"] {
    border: none;
    width: 30px;
    height: 30px;
    cursor: pointer;
    background: none;
  }
  /* Nuevo estilo para el wrapper */
  .canvas-wrapper {
    position: relative;
    width: 100vw;
    height: 100vh;
    background-color: #1a1a1a; /* El color de fondo va en el contenedor */
  }

  canvas {
    position: absolute; /* Para que se monten uno sobre otro */
    top: 0;
    left: 0;
    display: block;
    background: transparent; /* IMPORTANTE: Fondos transparentes */
  }
  /* Panel de usuarios */
  .users-panel {
    position: absolute;
    top: 20px;
    right: 20px;
    width: 180px;
    background: rgba(0, 0, 0, 0.8);
    border-radius: 8px;
    padding: 10px;
    color: white;
    pointer-events: auto;
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
  }
  .myself:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  /* Semáforo */
  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background-color: gray;
    box-shadow: 0 0 5px rgba(0, 0, 0, 0.5);
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
