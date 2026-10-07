# ArtChat (egui)

Cliente nativo de [art-chat-rs](https://github.com/ZBishopM/art-chat-rs) para el relé `art-chat-server`:
dibujo en vivo, chat, usuarios con estado y zumbido. Un solo `.exe` portable (~6,5 MB), sin instalador
y sin WebView. Habla el mismo protocolo que el cliente Tauri/Svelte, así que se mezcla con él en la misma sala.

    cargo build --release        # target\release\artchat.exe
    cargo test                   # unidades (protocolo, lienzo, paleta)

Para probar contra un servidor de verdad (`node chat-server.js` con `PORT=18480` en local):

    $env:ARTCHAT_TEST_URL = 'ws://127.0.0.1:18480'
    cargo test -- --include-ignored --test-threads=1

`ARTCHAT_URL` cambia el servidor al que se conecta el programa (por defecto el de producción).

## Moverse por el lienzo

El lienzo mide hasta 4096 × 4096 puntos, más que la ventana:

- **Arrastrar** con el botón del medio, o con **Espacio + clic**.
- **Rueda** o **touchpad**.
- **Minimapa** (abajo a la derecha): enseña el lienzo entero, también lo que otros dibujan fuera de la vista; pulsar o arrastrar en él lleva la vista ahí.
- **Inicio** vuelve a la esquina.

Los trazos viajan en coordenadas del lienzo, no de la pantalla. El cliente Tauri no se desplaza: ve solo lo que cabe en su ventana desde la esquina.

## Qué hay en cada archivo

| archivo | qué |
|---|---|
| `types.rs` | el protocolo, campo por campo como en `chat-server.js` |
| `network.rs` | WebSocket en su hilo; reconecta con espera creciente y manda un ping cada 25 s (nginx cierra a los 60 s) |
| `canvas.rs` | el lienzo es una capa de píxeles: cada segmento se rasteriza una vez y la GPU solo recibe la caja que cambió; borrar es borrar de verdad |
| `theme.rs` | identidad de windots: paleta cálida, JetBrains Mono Nerd Font (incrustada), radios 14/8 |
| `audio.rs` | los tres MP3 con MCI de Windows, sin motor de audio |
| `settings.rs` | `%APPDATA%\ArtChat\settings.json` y `lienzo-x1.png` (el dibujo sobrevive al cierre) |
| `app.rs` | la ventana: lienzo a sangre (con su vista desplazable) y cuatro islas flotantes, la cuarta el minimapa |

## Decisiones que conviene no deshacer

- **CRT estático** (`.cargo/config.toml`): el `.exe` no pide el runtime de Visual C++ en la otra máquina.
- **El historial se aplica una vez por conexión.** El servidor lo reenvía con cada `join`; repetirlo duplicaría mensajes.
  Los trazos del historial se pintan encima del lienzo local, nunca lo reemplazan: el servidor solo guarda 500 segmentos.
- **El zumbido sacude el contenido, no la ventana**: con un gestor en mosaico (GlazeWM) la ventana no se deja mover.
- **Los trazos efímeros se pintan como formas** (como el canvas web), así que con transparencia parcial los extremos
  redondeados se superponen y dejan «cuentas». No es un fallo nuevo: el cliente Tauri hace lo mismo.
