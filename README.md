# artchat

Desktop app (Tauri 2 + SvelteKit) to chat and draw simultaneously with friends on a shared canvas — permanent strokes, optional fading ("ghost") strokes, an eraser, per-user background color, and live chat/buzz alongside the drawing.

Talks to [`art-chat-server`](../art-chat-server) (a small `ws` WebSocket relay) at `wss://artchat.danassistantassistant.website`, hardcoded in `src-tauri/src/lib.rs`. The server also keeps a bounded stroke/message history (`history.json`) so new joiners see recent drawing state.

## Run

```
npm install
npm run dev          # Vite dev server only (localhost:1420) — canvas drawing works,
                      # but invoke() calls to the Tauri backend (and so the WS
                      # broadcast) will throw outside the Tauri shell.
npm run tauri dev    # full desktop app — this is the real thing to test against
```

Needs [`art-chat-server`](../art-chat-server) reachable at the URL in `lib.rs` (run it locally on `:8080` and point `lib.rs` at `ws://localhost:8080` for local multiplayer testing).

## Structure

- `src/routes/+page.svelte` — canvas, drawing logic (permanent layer + fading-stroke layer), WS message handling
- `src/lib/components/Toolbar.svelte` — color, brush size, fade, eraser, background color
- `src-tauri/` — Rust/Tauri shell; owns the actual WebSocket connection to the server, exposes the `send_message` command to the frontend via `invoke()`

## Checks

```
npm run check   # svelte-kit sync + svelte-check
npm run build   # production build
```
