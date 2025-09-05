// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod pgplib;
mod libutils;


use std::fs;
use std::path::Path;
use tauri::Manager;
use pgplib::{decrypt_message, encrypt_message, generate_keypair, get_email_ids_from_public_key};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::time::{SystemTime, Duration};

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

mod password_cache {
    use super::*;
    static CACHE: Lazy<Mutex<Option<(String, SystemTime)>>> = Lazy::new(|| Mutex::new(None));
    const CACHE_DURATION: Duration = Duration::from_secs(1 * 60); // Changed from 15 to 2 minutes

    pub fn set(password: String) {
        let mut cache = CACHE.lock().unwrap();
        *cache = Some((password, SystemTime::now()));
    }

    pub fn get() -> Option<String> {
        let mut cache = CACHE.lock().unwrap();
        if let Some((ref password, timestamp)) = *cache {
            if SystemTime::now().duration_since(timestamp).unwrap_or(Duration::ZERO) < CACHE_DURATION {
                return Some(password.clone());
            } else {
                *cache = None;
            }
        }
        None
    }

    pub fn clear() {
        let mut cache = CACHE.lock().unwrap();
        *cache = None;
    }
}

#[tauri::command]
fn cache_master_password(password: String) {
    password_cache::set(password);
}

#[tauri::command]
fn is_cache_valid() -> bool {
    password_cache::get().is_some()
}

#[tauri::command]
fn clear_master_password_cache() {
    password_cache::clear();
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
            decrypt_message,
            get_email_ids_from_public_key,
            cache_master_password,
            clear_master_password_cache,
            is_cache_valid,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

