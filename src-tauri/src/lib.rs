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

// ─── 工作流命令 ──────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct StepNode {
    #[serde(rename = "type")]
    node_type: String,
    config: serde_json::Value,
}

#[tauri::command]
fn start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn generate_id() -> String {
    workflow::generate_id()
}

#[tauri::command]
fn execute_step(nodes: Vec<StepNode>) -> Result<String, String> {
    use std::thread::sleep;
    use std::time::Duration;

    for (i, node) in nodes.iter().enumerate() {
        log::info!("Step {}: executing {}", i + 1, node.node_type);

        match node.node_type.as_str() {
            "click" => {
                let button = node.config.get("button").and_then(|v| v.as_str()).unwrap_or("left");
                keyboard::mouse_click(button).map_err(|e| e.to_string())?;
            }
            "double-click" => {
                let button = node.config.get("button").and_then(|v| v.as_str()).unwrap_or("left");
                keyboard::mouse_click(button).map_err(|e| e.to_string())?;
                sleep(Duration::from_millis(80));
                keyboard::mouse_click(button).map_err(|e| e.to_string())?;
            }
            "move-mouse" => {
                let x = node.config.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let y = node.config.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                keyboard::mouse_move_to(x, y).map_err(|e| e.to_string())?;
            }
            "drag" => {
                let sx = node.config.get("startX").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let sy = node.config.get("startY").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let ex = node.config.get("endX").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let ey = node.config.get("endY").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                keyboard::mouse_move_to(sx, sy).map_err(|e| e.to_string())?;
                sleep(Duration::from_millis(50));
                keyboard::mouse_down("left").map_err(|e| e.to_string())?;
                sleep(Duration::from_millis(30));
                keyboard::mouse_move_to(ex, ey).map_err(|e| e.to_string())?;
                sleep(Duration::from_millis(30));
                keyboard::mouse_up("left").map_err(|e| e.to_string())?;
            }
            "type-text" => {
                let text = node.config.get("text").and_then(|v| v.as_str()).unwrap_or("");
                if !text.is_empty() {
                    keyboard::type_text(text).map_err(|e| e.to_string())?;
                }
            }
            "hotkey" => {
                let key = node.config.get("key").and_then(|v| v.as_str()).unwrap_or("");
                let modifiers: Vec<&str> = node.config
                    .get("modifiers")
                    .and_then(|v| v.as_str())
                    .map(|s| s.split('+').map(|m| m.trim()).collect())
                    .unwrap_or_default();
                keyboard::combo(&modifiers, key).map_err(|e| e.to_string())?;
            }
            "key-press" => {
                let key = node.config.get("key").and_then(|v| v.as_str()).unwrap_or("");
                keyboard::key_press(key).map_err(|e| e.to_string())?;
            }
            "wait" => {
                let ms = node.config.get("duration").and_then(|v| v.as_u64()).unwrap_or(1000);
                sleep(Duration::from_millis(ms));
            }
            "scroll" | "wait-condition" => {
                // Not directly actionable in step execution; skip silently
                log::info!("Step {}: skipping unsupported type '{}'", i + 1, node.node_type);
            }
            _ => {
                log::warn!("Step {}: unknown node type '{}'", i + 1, node.node_type);
            }
        }

        // Brief pause between actions
        sleep(Duration::from_millis(150));
    }

    Ok(format!("执行完成，共 {} 个步骤", nodes.len()))
}

// ─── 应用入口 ────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                use tauri::Manager;
                use tauri::TitleBarStyle;
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_title("");
                    let _ = window.set_title_bar_style(TitleBarStyle::Overlay);
                }
            }
            Ok(())
        })
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
            start_drag,
            generate_id,
            execute_step,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
