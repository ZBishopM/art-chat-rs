use tauri::{Emitter, Manager, State}; // AppHandle quitado para limpiar el warning
use url::Url;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::Mutex;

// Definimos el tipo del Escritor
type WsWriter = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Message
>;

struct ChatConnection {
    sender: Mutex<Option<WsWriter>>,
}

#[tauri::command]
async fn send_message(state: State<'_, ChatConnection>, msg: String) -> Result<(), String> {
    let mut sender_guard = state.sender.lock().await;
    
    if let Some(sender) = sender_guard.as_mut() {
        // FIX 1: Convertimos el String a "Utf8Bytes" usando .into()
        let message = tokio_tungstenite::tungstenite::Message::Text(msg.clone().into());
        
        match sender.send(message).await {
            Ok(_) => {
                println!("📤 Enviado: {}", msg);
                Ok(())
            },
            Err(e) => {
                eprintln!("Error enviando: {}", e);
                Err(e.to_string())
            }
        }
    } else {
        Err("No hay conexión con el servidor".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ChatConnection { sender: Mutex::new(None) }) 
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            tauri::async_runtime::spawn(async move {
                println!("⏳ Conectando al servidor...");
                
                // --- TU IP AQUÍ ---
                // Asegúrate de que esta IP es la correcta de tu laptop
                let url = Url::parse("ws://100.48.213.255:8080").expect("URL inválida"); 
                // ------------------

                // FIX 2: Pasamos 'url.as_str()' en vez de 'url' directo
                match tokio_tungstenite::connect_async(url.as_str()).await {
                    Ok((ws_stream, _)) => {
                        println!("✅ ¡Conectado a la Laptop!");
                        // --- NUEVO: AVISAR AL FRONTEND ---
                        app_handle.emit("connection-status", "connected").unwrap();
                        // ---------------------------------
                        let (write, mut read) = ws_stream.split();
                        
                        let state = app_handle.state::<ChatConnection>();
                        *state.sender.lock().await = Some(write);
                        
                        while let Some(msg) = read.next().await {
                            // FIX 3: Capturamos 'text' que ahora es Utf8Bytes
                            if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = msg {
                                // Lo convertimos a String normal con .to_string() para que Tauri lo entienda
                                let final_msg = text.to_string();
                                println!("📩 Recibido del server: {}", final_msg);
                                app_handle.emit("chat-message", final_msg).unwrap();
                            }
                        }
                    },
                    Err(e) => {
                        println!("❌ Error conectando: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}