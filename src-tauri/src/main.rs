#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod terminal;

use terminal::TerminalManager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(TerminalManager::new())
        .invoke_handler(tauri::generate_handler![
            terminal::spawn_terminal,
            terminal::spawn_terminal_with_command,
            terminal::write_to_terminal,
            terminal::resize_terminal,
            terminal::close_terminal,
            terminal::list_terminals,
            get_home_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
}