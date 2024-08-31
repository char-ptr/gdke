use std::{ffi::OsStr, path::Path};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn get_secret(program: &Path, sig: &str) -> Result<String, String> {
    spawn_and_inject_sig(program, sig)
        .map_err(|e| e.to_string())
        .map(|val| {
            let mut hex_str = "0x".to_string();
            for char in val {
                hex_str.push_str(&format!("{char:02X}"))
            }
            hex_str
        })
    // String::new()
}
fn spawn_and_inject_sig<T: AsRef<OsStr>>(proc: T, sig: &str) -> anyhow::Result<[u8; 32]> {
    Ok([62; 32])
    // Err(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![get_secret])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
