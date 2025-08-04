// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod pgplib;
mod libutils;


use std::fs;
use std::path::Path;
use tauri::Manager;
use pgplib::{decrypt_message, encrypt_message, generate_keypair};

#[tauri::command]
fn get_app_data_dir(app_handle: tauri::AppHandle) -> Result<String, String> {
    match app_handle.path().app_data_dir() {
        Ok(path) => {
            let path_str = path.display().to_string();
            println!("App data directory: {}", path_str);
            Ok(path_str)
        },
        Err(e) => Err(format!("Failed to get app data directory: {}", e)),
    }
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    match fs::read_to_string(&path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

#[tauri::command]
fn write_text_file(path: String, content: String) -> Result<(), String> {
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(&path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory: {}", e));
        }
    }

    match fs::write(&path, content) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write file: {}", e)),
    }
}

#[tauri::command]
fn file_exists(path: String) -> bool {
    Path::new(&path).exists()
}

#[tauri::command]
fn join_path(base: String, segment: String) -> String {
    Path::new(&base).join(segment).to_string_lossy().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_app_data_dir,
            read_text_file,
            write_text_file,
            file_exists,
            join_path,
            generate_keypair,
            encrypt_message,
            decrypt_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
    //         write_text_file,
    //         file_exists,
    //         join_path
    //     ])
    //     .run(tauri::generate_context!())
    //     .expect("error while running tauri application");

