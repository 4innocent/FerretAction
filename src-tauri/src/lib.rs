// ─── 键盘命令 ────────────────────────────────────────────────

#[tauri::command]
fn key_press(key: &str) -> Result<(), String> {
    keyboard::key_press(key).map_err(|e| e.to_string())
}

#[tauri::command]
fn key_down(key: &str) -> Result<(), String> {
    keyboard::key_down(key).map_err(|e| e.to_string())
}

#[tauri::command]
fn key_up(key: &str) -> Result<(), String> {
    keyboard::key_up(key).map_err(|e| e.to_string())
}

#[tauri::command]
fn key_combo(modifiers: Vec<String>, key: String) -> Result<(), String> {
    let mods: Vec<&str> = modifiers.iter().map(String::as_str).collect();
    keyboard::combo(&mods, &key).map_err(|e| e.to_string())
}

// ─── 鼠标命令 ────────────────────────────────────────────────

#[tauri::command]
fn mouse_move_to(x: i32, y: i32) -> Result<(), String> {
    keyboard::mouse_move_to(x, y).map_err(|e| e.to_string())
}

#[tauri::command]
fn mouse_move_relative(dx: i32, dy: i32) -> Result<(), String> {
    keyboard::mouse_move_relative(dx, dy).map_err(|e| e.to_string())
}

#[tauri::command]
fn mouse_click(button: &str) -> Result<(), String> {
    keyboard::mouse_click(button).map_err(|e| e.to_string())
}

#[tauri::command]
fn mouse_down(button: &str) -> Result<(), String> {
    keyboard::mouse_down(button).map_err(|e| e.to_string())
}

#[tauri::command]
fn mouse_up(button: &str) -> Result<(), String> {
    keyboard::mouse_up(button).map_err(|e| e.to_string())
}

// ─── 应用入口 ────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            key_press,
            key_down,
            key_up,
            key_combo,
            mouse_move_to,
            mouse_move_relative,
            mouse_click,
            mouse_down,
            mouse_up,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
