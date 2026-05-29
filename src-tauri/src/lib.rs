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

#[tauri::command]
fn get_mouse_location() -> Result<(i32, i32), String> {
    keyboard::mouse_location().map_err(|e| e.to_string())
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
fn execute_step(
    nodes: Vec<StepNode>,
    mouse_duration: u64,
    step_delay: u64,
    stop_strategy: String,
) -> Result<String, String> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread::{sleep, spawn};
    use std::time::Duration;

    let stopped = Arc::new(AtomicBool::new(false));
    let expected_pos: Arc<Mutex<Option<(i32, i32)>>> = Arc::new(Mutex::new(None));

    // ── Background monitor thread ───────────────────────────────
    if stop_strategy != "none" {
        let stopped_flag = Arc::clone(&stopped);
        let expected = Arc::clone(&expected_pos);
        let strategy = stop_strategy.clone();

        spawn(move || {
            let mut prev_esc = false;     // edge detection for ESC
            let mut baseline = keyboard::mouse_location().ok(); // known "ours" position

            loop {
                if stopped_flag.load(Ordering::Relaxed) {
                    break;
                }

                let should_stop = match strategy.as_str() {
                    "esc" => {
                        let cur = keyboard::is_escape_pressed();
                        let edge = cur && !prev_esc; // rising edge only
                        prev_esc = cur;
                        edge
                    }
                    "mouse" => {
                        let cur = keyboard::mouse_location().ok();
                        // Update baseline to match what main thread says is our target
                        let exp = *expected.lock().unwrap();
                        if exp.is_some() {
                            baseline = exp;
                        }
                        // Compare current position against baseline
                        match (baseline, cur) {
                            (Some((bx, by)), Some((cx, cy))) => {
                                let d2 = (cx - bx).pow(2) + (cy - by).pow(2);
                                d2 > 225 // >15px deviation from expected
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                };

                if should_stop {
                    stopped_flag.store(true, Ordering::Relaxed);
                }

                sleep(Duration::from_millis(1));
            }
        });
    }

    for (i, node) in nodes.iter().enumerate() {
        // Check stop flag set by monitor thread
        if stopped.load(Ordering::Relaxed) {
            log::info!("Step {}: stopped by strategy '{}'", i + 1, stop_strategy);
            return Ok(format!("已停止 (策略: {})", stop_strategy));
        }

        log::info!("Step {}: executing {}", i + 1, node.node_type);

        match node.node_type.as_str() {
            "click" => {
                let button = node
                    .config
                    .get("button")
                    .and_then(|v| v.as_str())
                    .unwrap_or("left");
                keyboard::mouse_click(button).map_err(|e| e.to_string())?;
            }
            "double-click" => {
                let button = node
                    .config
                    .get("button")
                    .and_then(|v| v.as_str())
                    .unwrap_or("left");
                keyboard::mouse_click(button).map_err(|e| e.to_string())?;
                sleep(Duration::from_millis(80));
                keyboard::mouse_click(button).map_err(|e| e.to_string())?;
            }
            "move-mouse" => {
                let x = node.config.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let y = node.config.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let button = node.config.get("button").and_then(|v| v.as_str()).unwrap_or("left");
                let pre_action = node.config.get("preAction").and_then(|v| v.as_str()).unwrap_or("none");
                let post_action = node.config.get("postAction").and_then(|v| v.as_str()).unwrap_or("none");

                let exec_action = |action: &str| -> Result<(), String> {
                    match action {
                        "click" => keyboard::mouse_click(button).map_err(|e| e.to_string()),
                        "double-click" => {
                            keyboard::mouse_click(button).map_err(|e| e.to_string())?;
                            sleep(Duration::from_millis(80));
                            keyboard::mouse_click(button).map_err(|e| e.to_string())
                        }
                        "mouse-down" => keyboard::mouse_down(button).map_err(|e| e.to_string()),
                        "mouse-up" => keyboard::mouse_up(button).map_err(|e| e.to_string()),
                        _ => Ok(()),
                    }
                };

                exec_action(pre_action)?;
                keyboard::mouse_move_wind(x, y, mouse_duration).map_err(|e| e.to_string())?;
                *expected_pos.lock().unwrap() = Some((x, y));
                exec_action(post_action)?;
            }
            "type-text" => {
                let text = node
                    .config
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !text.is_empty() {
                    keyboard::type_text(text).map_err(|e| e.to_string())?;
                }
            }
            "hotkey" => {
                let key = node
                    .config
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let modifiers: Vec<&str> = node
                    .config
                    .get("modifiers")
                    .and_then(|v| v.as_str())
                    .map(|s| s.split('+').map(|m| m.trim()).collect())
                    .unwrap_or_default();
                keyboard::combo(&modifiers, key).map_err(|e| e.to_string())?;
            }
            "key-press" => {
                let key = node
                    .config
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                keyboard::key_press(key).map_err(|e| e.to_string())?;
            }
            "wait" => {
                let ms = node
                    .config
                    .get("duration")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000);
                sleep(Duration::from_millis(ms));
            }
            "scroll" | "wait-condition" => {
                // Not directly actionable in step execution; skip silently
                log::info!(
                    "Step {}: skipping unsupported type '{}'",
                    i + 1,
                    node.node_type
                );
            }
            _ => {
                log::warn!("Step {}: unknown node type '{}'", i + 1, node.node_type);
            }
        }

        // Brief pause between actions
        sleep(Duration::from_millis(step_delay));
    }

    Ok(format!("执行完成，共 {} 个步骤", nodes.len()))
}

// ─── 应用入口 ────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
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
            get_mouse_location,
            start_drag,
            generate_id,
            execute_step,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
