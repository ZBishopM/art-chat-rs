// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

fn main() {
    #[cfg(target_os = "linux")]
    {
        // Si no existe la variable, la forzamos a 1
        if env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    art_chat_rs_lib::run()
}
