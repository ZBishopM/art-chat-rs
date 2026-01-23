<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { type UnlistenFn } from "@tauri-apps/api/event";

  // Componentes
  import Toolbar from "$lib/components/Toolbar.svelte";
  import ChatPanel from "$lib/components/ChatPanel.svelte";
  import UsersPanel from "$lib/components/UsersPanel.svelte";

  // Tipos
  import type { FadingStroke, ConnectionState, User } from "$lib/types";

  let unlisten: UnlistenFn;
  let unlistenConnection: UnlistenFn;
  let joinInterval: ReturnType<typeof setInterval>;

  // --- ESTADO ---
  let connectedUsers: User[] = [];
  let myStatus: 'online' | 'busy' = "online";
  let fadingStrokes: FadingStroke[] = [];
  let messages: string[] = [];

  // Canvas
  let canvasPermanent: HTMLCanvasElement;
  let canvasFade: HTMLCanvasElement;
  let ctxPerm: CanvasRenderingContext2D | null;
  let ctxFade: CanvasRenderingContext2D | null;

  // Buffer para preservar dibujos al redimensionar
  let bufferCanvas: HTMLCanvasElement;
  let bufferCtx: CanvasRenderingContext2D | null;
  let maxWidth = 0;
  let maxHeight = 0;

  // Función auxiliar para cargar o usar default
  const load = <T>(key: string, def: T): T => {
    const val = localStorage.getItem(key);
    return val ? JSON.parse(val) : def;
  };

  // Identidad
  let myNickname = load("artchat_nick", "Artista_" + Math.floor(Math.random() * 1000));
  const myId = load("artchat_id", Math.random().toString(36).substr(2, 9));
  localStorage.setItem("artchat_id", JSON.stringify(myId));

  // Preferencias
  let soundEnabled = load('artchat_sound', true);
  let selectedColor = load("artchat_color", "#ffffff");
  let brushSize = load("artchat_size", 5);
  let fadeEnabled = load("artchat_fade", false);
  let fadeSpeed = load("artchat_fadespeed", 50);
  let backgroundColor = load("artchat_bgcolor", "#1a1a1a");
  let onlineColor = load("artchat_online_color", "#00ff00");
  let busyColor = load("artchat_busy_color", "#ff0000");

  // Audio
  let notificationAudio: HTMLAudioElement;
  let connectedAudio: HTMLAudioElement;
  let buzzAudio: HTMLAudioElement;

  // Dibujo
  let isDrawing = false;
  let lastX = 0;
  let lastY = 0;

  // Conexión
  let connectionState: ConnectionState = 'connecting';

  // --- PERSISTENCIA AUTOMÁTICA ---
  $: localStorage.setItem('artchat_sound', JSON.stringify(soundEnabled));
  $: localStorage.setItem("artchat_nick", JSON.stringify(myNickname));
  $: localStorage.setItem("artchat_color", JSON.stringify(selectedColor));
  $: localStorage.setItem("artchat_size", JSON.stringify(brushSize));
  $: localStorage.setItem("artchat_fade", JSON.stringify(fadeEnabled));
  $: localStorage.setItem("artchat_fadespeed", JSON.stringify(fadeSpeed));
  $: localStorage.setItem("artchat_bgcolor", JSON.stringify(backgroundColor));
  $: localStorage.setItem("artchat_online_color", JSON.stringify(onlineColor));
  $: localStorage.setItem("artchat_busy_color", JSON.stringify(busyColor));

  // Fallback: si recibimos usuarios, estamos conectados
  $: if (connectedUsers.length > 0 && connectionState !== 'connected') {
    connectionState = 'connected';
  }

  // Aplicar color de fondo
  $: if (typeof document !== 'undefined') {
    document.body.style.backgroundColor = backgroundColor;
  }

  // --- BUCLE DE ANIMACIÓN ---
  function animationLoop() {
    if (!ctxFade) return;

    ctxFade.clearRect(0, 0, canvasFade.width, canvasFade.height);
    const now = Date.now();

    fadingStrokes = fadingStrokes.filter((stroke) => {
      const elapsed = now - stroke.startTime;
      if (elapsed >= stroke.duration) return false;

      const opacity = 1 - elapsed / stroke.duration;
      ctxFade!.globalAlpha = opacity;
      ctxFade!.lineWidth = stroke.size;
      ctxFade!.strokeStyle = stroke.color;
      ctxFade!.fillStyle = stroke.color;
      ctxFade!.lineCap = "round";

      ctxFade!.beginPath();
      if (stroke.x0 === stroke.x1 && stroke.y0 === stroke.y1) {
        ctxFade!.arc(stroke.x0, stroke.y0, stroke.size / 2, 0, Math.PI * 2);
        ctxFade!.fill();
      } else {
        ctxFade!.moveTo(stroke.x0, stroke.y0);
        ctxFade!.lineTo(stroke.x1, stroke.y1);
        ctxFade!.stroke();
      }
      return true;
    });

    ctxFade.globalAlpha = 1.0;
    requestAnimationFrame(animationLoop);
  }

  // --- PERSISTENCIA DE ESTADO ---
  function saveState() {
    try {
      // Guardar canvas como base64
      if (bufferCanvas) {
        const canvasData = bufferCanvas.toDataURL('image/png');
        localStorage.setItem('artchat_canvas', canvasData);
      }
      // Guardar mensajes (últimos 100)
      const recentMessages = messages.slice(-100);
      localStorage.setItem('artchat_messages', JSON.stringify(recentMessages));
    } catch (e) {
      console.warn('No se pudo guardar el estado:', e);
    }
  }

  function restoreState() {
    try {
      // Restaurar canvas
      const canvasData = localStorage.getItem('artchat_canvas');
      if (canvasData && ctxPerm && bufferCtx) {
        const img = new Image();
        img.onload = () => {
          bufferCtx!.drawImage(img, 0, 0);
          ctxPerm!.drawImage(img, 0, 0);
        };
        img.src = canvasData;
      }
      // Restaurar mensajes
      const savedMessages = localStorage.getItem('artchat_messages');
      if (savedMessages) {
        messages = JSON.parse(savedMessages);
      }
    } catch (e) {
      console.warn('No se pudo restaurar el estado:', e);
    }
  }

  // --- FUNCIONES DE RED ---
  async function sendJoin() {
    const payload = JSON.stringify({
      type: 'join',
      senderId: myId,
      nickname: myNickname,
      color: selectedColor
    });
    await invoke("send_message", { msg: payload }).catch(e => console.error("Fallo al enviar join:", e));
  }

  async function handleSendChat(event: CustomEvent<string>) {
    const content = event.detail;
    const payload = JSON.stringify({
      type: "chat",
      senderId: myId,
      nickname: myNickname,
      content,
    });
    messages = [...messages, `Yo: ${content}`];
    await invoke("send_message", { msg: payload });
  }

  async function handleSendBuzz() {
    const payload = JSON.stringify({
      type: "buzz",
      senderId: myId,
      nickname: myNickname,
    });
    messages = [...messages, `⚡ Enviaste un zumbido`];
    await invoke("send_message", { msg: payload });
  }

  function triggerBuzzEffect() {
    if (buzzAudio && soundEnabled) {
      buzzAudio.currentTime = 0;
      buzzAudio.play().catch(() => {});
    }
    // Efecto de shake
    document.body.classList.add('buzz-shake');
    setTimeout(() => {
      document.body.classList.remove('buzz-shake');
    }, 500);
  }

  async function handleClearCanvas() {
    if (ctxPerm && ctxFade) {
      ctxPerm.fillStyle = backgroundColor;
      ctxPerm.fillRect(0, 0, canvasPermanent.width, canvasPermanent.height);
      ctxFade.clearRect(0, 0, canvasFade.width, canvasFade.height);
    }
    // También limpiar el buffer
    if (bufferCtx) {
      bufferCtx.fillStyle = backgroundColor;
      bufferCtx.fillRect(0, 0, bufferCanvas.width, bufferCanvas.height);
    }
    const payload = JSON.stringify({ type: "clear", senderId: myId });
    await invoke("send_message", { msg: payload });
  }

  async function handleToggleStatus() {
    myStatus = myStatus === "online" ? "busy" : "online";
    const payload = JSON.stringify({
      type: "status",
      senderId: myId,
      status: myStatus,
    });
    await invoke("send_message", { msg: payload });
  }

  // --- DIBUJO ---
  function getMousePos(evt: MouseEvent) {
    const rect = canvasFade.getBoundingClientRect();
    return { x: evt.clientX - rect.left, y: evt.clientY - rect.top };
  }

  function startDrawing(e: MouseEvent) {
    isDrawing = true;
    const pos = getMousePos(e);
    lastX = pos.x;
    lastY = pos.y;

    if (fadeEnabled) {
      fadingStrokes.push({
        x0: lastX, y0: lastY, x1: lastX, y1: lastY,
        color: selectedColor, size: brushSize,
        startTime: Date.now(), duration: fadeSpeed * 10,
      });
    } else if (ctxPerm) {
      ctxPerm.lineWidth = brushSize;
      ctxPerm.strokeStyle = selectedColor;
      ctxPerm.fillStyle = selectedColor;
      ctxPerm.beginPath();
      ctxPerm.arc(lastX, lastY, ctxPerm.lineWidth / 2, 0, Math.PI * 2);
      ctxPerm.fill();

      // También dibujar en buffer
      if (bufferCtx) {
        bufferCtx.lineWidth = brushSize;
        bufferCtx.fillStyle = selectedColor;
        bufferCtx.beginPath();
        bufferCtx.arc(lastX, lastY, bufferCtx.lineWidth / 2, 0, Math.PI * 2);
        bufferCtx.fill();
      }
    }

    const payload = JSON.stringify({
      type: "draw", senderId: myId,
      x0: lastX, y0: lastY, x1: lastX, y1: lastY,
      color: selectedColor, size: brushSize,
      ephemeral: fadeEnabled, duration: fadeEnabled ? fadeSpeed * 10 : 0,
    });
    invoke("send_message", { msg: payload });
  }

  function stopDrawing() {
    isDrawing = false;
    ctxPerm?.beginPath();
    ctxFade?.beginPath();
  }

  async function draw(e: MouseEvent) {
    if (!isDrawing) return;
    const pos = getMousePos(e);
    const currentX = pos.x;
    const currentY = pos.y;

    if (fadeEnabled) {
      fadingStrokes.push({
        x0: lastX, y0: lastY, x1: currentX, y1: currentY,
        color: selectedColor, size: brushSize,
        startTime: Date.now(), duration: fadeSpeed * 10,
      });
    } else if (ctxPerm) {
      ctxPerm.lineWidth = brushSize;
      ctxPerm.strokeStyle = selectedColor;
      ctxPerm.fillStyle = selectedColor;
      ctxPerm.lineCap = "round";
      ctxPerm.beginPath();
      ctxPerm.moveTo(lastX, lastY);
      ctxPerm.lineTo(currentX, currentY);
      ctxPerm.stroke();

      // También dibujar en buffer
      if (bufferCtx) {
        bufferCtx.lineWidth = brushSize;
        bufferCtx.strokeStyle = selectedColor;
        bufferCtx.lineCap = "round";
        bufferCtx.beginPath();
        bufferCtx.moveTo(lastX, lastY);
        bufferCtx.lineTo(currentX, currentY);
        bufferCtx.stroke();
      }
    }

    const payload = JSON.stringify({
      type: "draw", senderId: myId,
      x0: lastX, y0: lastY, x1: currentX, y1: currentY,
      color: selectedColor, size: brushSize,
      ephemeral: fadeEnabled, duration: fadeEnabled ? fadeSpeed * 10 : 0,
    });

    lastX = currentX;
    lastY = currentY;
    await invoke("send_message", { msg: payload });
  }

  function drawRemote(data: any, isEphemeral: boolean) {
    if (isEphemeral) {
      fadingStrokes.push({
        x0: data.x0, y0: data.y0, x1: data.x1, y1: data.y1,
        color: data.color || "#00ff00", size: data.size || 5,
        startTime: Date.now(), duration: data.duration || 1000,
      });
    } else if (ctxPerm) {
      const size = data.size || 5;
      const color = data.color || "#00ff00";

      ctxPerm.lineWidth = size;
      ctxPerm.strokeStyle = color;
      ctxPerm.fillStyle = color;
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

      // También dibujar en buffer
      if (bufferCtx) {
        bufferCtx.lineWidth = size;
        bufferCtx.strokeStyle = color;
        bufferCtx.fillStyle = color;
        bufferCtx.beginPath();

        if (data.x0 === data.x1 && data.y0 === data.y1) {
          bufferCtx.arc(data.x0, data.y0, bufferCtx.lineWidth / 2, 0, Math.PI * 2);
          bufferCtx.fill();
        } else {
          bufferCtx.lineCap = "round";
          bufferCtx.moveTo(data.x0, data.y0);
          bufferCtx.lineTo(data.x1, data.y1);
          bufferCtx.stroke();
        }
      }
    }
  }

  // --- RESIZE ---
  function handleResize() {
    const newWidth = window.innerWidth;
    const newHeight = window.innerHeight;

    // Guardar contenido actual al buffer antes de redimensionar
    if (bufferCtx && ctxPerm) {
      bufferCtx.drawImage(canvasPermanent, 0, 0);
    }

    // Expandir buffer si la nueva ventana es más grande
    if (newWidth > maxWidth || newHeight > maxHeight) {
      const oldMaxWidth = maxWidth;
      const oldMaxHeight = maxHeight;
      maxWidth = Math.max(maxWidth, newWidth);
      maxHeight = Math.max(maxHeight, newHeight);

      // Crear nuevo buffer más grande preservando contenido
      const tempBuffer = document.createElement("canvas");
      tempBuffer.width = oldMaxWidth || newWidth;
      tempBuffer.height = oldMaxHeight || newHeight;
      tempBuffer.getContext("2d")?.drawImage(bufferCanvas, 0, 0);

      bufferCanvas.width = maxWidth;
      bufferCanvas.height = maxHeight;
      bufferCtx = bufferCanvas.getContext("2d");
      bufferCtx?.drawImage(tempBuffer, 0, 0);
    }

    // Redimensionar canvas visibles
    canvasPermanent.width = newWidth;
    canvasPermanent.height = newHeight;
    canvasFade.width = newWidth;
    canvasFade.height = newHeight;

    // Restaurar desde el buffer
    if (bufferCtx && ctxPerm) {
      ctxPerm.drawImage(bufferCanvas, 0, 0);
    }
  }

  // --- INICIALIZACIÓN ---
  onMount(async () => {
    ctxPerm = canvasPermanent.getContext("2d");
    canvasPermanent.width = window.innerWidth;
    canvasPermanent.height = window.innerHeight;

    ctxFade = canvasFade.getContext("2d");
    canvasFade.width = window.innerWidth;
    canvasFade.height = window.innerHeight;

    // Inicializar buffer para preservar dibujos
    maxWidth = window.innerWidth;
    maxHeight = window.innerHeight;
    bufferCanvas = document.createElement("canvas");
    bufferCanvas.width = maxWidth;
    bufferCanvas.height = maxHeight;
    bufferCtx = bufferCanvas.getContext("2d");

    // Restaurar estado guardado
    restoreState();

    // Guardar estado al cerrar
    window.addEventListener('beforeunload', saveState);

    notificationAudio = new Audio('/notify.mp3');
    notificationAudio.volume = 0.5;
    connectedAudio = new Audio('/connected.mp3');
    connectedAudio.volume = 0.5;
    buzzAudio = new Audio('/buzz.mp3');
    buzzAudio.volume = 0.7;

    window.addEventListener("resize", handleResize);
    requestAnimationFrame(animationLoop);

    // Escuchar mensajes
    unlisten = await listen<string>("chat-message", (event) => {
      try {
        const data = JSON.parse(event.payload);

        if (data.type === "users_update") {
          if (soundEnabled && connectedAudio && data.users.length > connectedUsers.length) {
            connectedAudio.play().catch(() => {});
          }
          connectedUsers = data.users;
          return;
        }

        if (data.senderId === myId) return;

        if (data.type === "chat") {
          messages = [...messages, `${data.nickname || "Anónimo"}: ${data.content}`];
          if (soundEnabled && notificationAudio) {
            notificationAudio.play().catch(() => {});
          }
        } else if (data.type === "draw") {
          drawRemote(data, data.ephemeral || false);
        } else if (data.type === "clear" && ctxPerm && ctxFade) {
          ctxPerm.clearRect(0, 0, canvasPermanent.width, canvasPermanent.height);
          ctxFade.clearRect(0, 0, canvasFade.width, canvasFade.height);
          if (bufferCtx) {
            bufferCtx.clearRect(0, 0, bufferCanvas.width, bufferCanvas.height);
          }
        } else if (data.type === "buzz") {
          messages = [...messages, `⚡ ${data.nickname || "Alguien"} te envió un zumbido!`];
          triggerBuzzEffect();
        } else if (data.type === "history") {
          // Aplicar historial de dibujos
          if (data.strokes && data.strokes.length > 0) {
            console.log(`📜 Cargando ${data.strokes.length} strokes del historial`);

            // Calcular el tamaño máximo necesario para el historial
            let historyMaxX = maxWidth;
            let historyMaxY = maxHeight;
            data.strokes.forEach((stroke: any) => {
              historyMaxX = Math.max(historyMaxX, stroke.x0, stroke.x1);
              historyMaxY = Math.max(historyMaxY, stroke.y0, stroke.y1);
            });

            // Expandir buffer si el historial requiere más espacio
            if (historyMaxX > maxWidth || historyMaxY > maxHeight) {
              const oldBuffer = document.createElement("canvas");
              oldBuffer.width = maxWidth;
              oldBuffer.height = maxHeight;
              oldBuffer.getContext("2d")?.drawImage(bufferCanvas, 0, 0);

              maxWidth = Math.max(maxWidth, historyMaxX + 100);
              maxHeight = Math.max(maxHeight, historyMaxY + 100);
              bufferCanvas.width = maxWidth;
              bufferCanvas.height = maxHeight;
              bufferCtx = bufferCanvas.getContext("2d");
              bufferCtx?.drawImage(oldBuffer, 0, 0);

              console.log(`📐 Buffer expandido a ${maxWidth}x${maxHeight} para historial`);
            }

            // Dibujar strokes del historial
            data.strokes.forEach((stroke: any) => {
              drawRemote(stroke, false);
            });
          }
          // Aplicar historial de mensajes
          if (data.messages && data.messages.length > 0) {
            console.log(`📜 Cargando ${data.messages.length} mensajes del historial`);
            const historyMessages = data.messages.map((m: any) =>
              `[hist] ${m.nickname}: ${m.content}`
            );
            messages = [...historyMessages, ...messages];
          }
        }
      } catch (e) {
        console.error("Error JSON:", e);
      }
    });

    // Escuchar conexión
    unlistenConnection = await listen<string>('connection-status', (event) => {
      const payload = event.payload;
      if (payload === 'connected') {
        connectionState = 'connected';
        sendJoin();
      } else if (payload.startsWith('connecting')) {
        connectionState = 'connecting';
      } else if (payload.startsWith('error') || payload === 'closed') {
        connectionState = 'error';
      }
    });

    // Red de seguridad
    joinInterval = setInterval(() => {
      if (!connectedUsers.some(u => u.id === myId)) {
        sendJoin();
      }
    }, 2000);
  });

  onDestroy(() => {
    saveState(); // Guardar al destruir componente
    window.removeEventListener('beforeunload', saveState);
    unlisten?.();
    unlistenConnection?.();
    if (joinInterval) clearInterval(joinInterval);
  });
</script>

<main>
  <!-- Canvas -->
  <div class="canvas-wrapper" style="background-color: {backgroundColor}">
    <canvas bind:this={canvasPermanent}></canvas>
    <canvas
      bind:this={canvasFade}
      on:mousedown={startDrawing}
      on:mouseup={stopDrawing}
      on:mousemove={draw}
      on:mouseleave={stopDrawing}
    ></canvas>
  </div>

  <!-- UI Components -->
  <Toolbar
    bind:nickname={myNickname}
    bind:soundEnabled
    bind:selectedColor
    bind:brushSize
    bind:fadeEnabled
    bind:fadeSpeed
    bind:backgroundColor
    {connectionState}
    on:clear={handleClearCanvas}
    on:toggleSound={() => soundEnabled = !soundEnabled}
  />

  <ChatPanel {messages} on:send={handleSendChat} on:buzz={handleSendBuzz} />

  <UsersPanel
    users={connectedUsers}
    {myId}
    myNickname={myNickname}
    {myStatus}
    bind:onlineColor
    bind:busyColor
    on:toggleStatus={handleToggleStatus}
  />
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  :global(body.buzz-shake) {
    animation: shake 0.5s ease-in-out;
  }

  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    10%, 30%, 50%, 70%, 90% { transform: translateX(-10px); }
    20%, 40%, 60%, 80% { transform: translateX(10px); }
  }

  .canvas-wrapper {
    position: relative;
    width: 100vw;
    height: 100vh;
  }

  canvas {
    position: absolute;
    top: 0;
    left: 0;
    display: block;
    background: transparent;
    cursor: crosshair;
  }
</style>
