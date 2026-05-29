// 键盘与鼠标模拟模块 —— 基于 enigo 库实现跨平台输入操作。
///
/// 提供以下能力：
/// - 键盘按键（按下、抬起、点击、组合键）
/// - 鼠标移动（绝对坐标、相对偏移）
/// - 鼠标按钮（按下、抬起、点击）
use anyhow::{anyhow, Context, Result};
use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

/// 从字符串解析按键。
///
/// 单字符（如 "a", "1", ","）映射为 `Key::Unicode`，
/// 常用功能键名映射为对应的 `Key` 变体。
fn parse_key(s: &str) -> Result<Key> {
    match s {
        "enter" | "return" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "space" => Ok(Key::Space),
        "backspace" => Ok(Key::Backspace),
        "escape" | "esc" => Ok(Key::Escape),
        "left" | "arrowleft" => Ok(Key::LeftArrow),
        "right" | "arrowright" => Ok(Key::RightArrow),
        "up" | "arrowup" => Ok(Key::UpArrow),
        "down" | "arrowdown" => Ok(Key::DownArrow),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),
        "delete" | "del" => Ok(Key::Delete),
        #[cfg(target_os = "macos")]
        "insert" | "ins" => Ok(Key::Other(114)), // kVK_Help, macOS has no Insert key
        #[cfg(not(target_os = "macos"))]
        "insert" | "ins" => Ok(Key::Insert),
        "shift" => Ok(Key::Shift),
        "lshift" => Ok(Key::LShift),
        "rshift" => Ok(Key::RShift),
        "control" | "ctrl" => Ok(Key::Control),
        "lcontrol" | "lctrl" => Ok(Key::LControl),
        "rcontrol" | "rctrl" => Ok(Key::RControl),
        "alt" => Ok(Key::Alt),
        "meta" | "win" | "command" => Ok(Key::Meta),
        "capslock" => Ok(Key::CapsLock),
        #[cfg(target_os = "macos")]
        "numlock" => Ok(Key::Other(71)), // kVK_ANSI_KeypadClear
        #[cfg(not(target_os = "macos"))]
        "numlock" => Ok(Key::Numlock),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        _ => {
            let chars: Vec<char> = s.chars().collect();
            if chars.len() == 1 {
                Ok(Key::Unicode(chars[0]))
            } else {
                Err(anyhow!("无法识别的按键名称: {s}"))
            }
        }
    }
}

/// 从字符串解析鼠标按钮。
fn parse_button(s: &str) -> Result<Button> {
    match s {
        "left" => Ok(Button::Left),
        "right" => Ok(Button::Right),
        "middle" => Ok(Button::Middle),
        "back" | "x1" => Ok(Button::Back),
        "forward" | "x2" => Ok(Button::Forward),
        _ => Err(anyhow!(
            "无法识别的鼠标按钮: {s}，支持 left / right / middle / back / forward"
        )),
    }
}

// ─── 键盘操作 ────────────────────────────────────────────────

/// 输入文本（模拟键盘逐字输入，支持 Unicode 和大小写）。
pub fn type_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化键盘模拟器失败")?;
    enigo.text(text).context("输入文本失败")?;
    Ok(())
}

/// 点击按键（按下后立即抬起）。
pub fn key_press(key: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化键盘模拟器失败")?;
    let key = parse_key(key)?;
    enigo.key(key, Direction::Click).context("按键操作失败")?;
    Ok(())
}

/// 按下按键（保持按下状态，需配合 [`key_up`] 抬起）。
pub fn key_down(key: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化键盘模拟器失败")?;
    let key = parse_key(key)?;
    enigo.key(key, Direction::Press).context("按键按下失败")?;
    Ok(())
}

/// 抬起按键（释放之前通过 [`key_down`] 按下的按键）。
pub fn key_up(key: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化键盘模拟器失败")?;
    let key = parse_key(key)?;
    enigo.key(key, Direction::Release).context("按键抬起失败")?;
    Ok(())
}

// ─── 鼠标操作 ────────────────────────────────────────────────

/// 移动鼠标到屏幕绝对坐标 (x, y)，内部走 WindMouse 拟人化轨迹。
pub fn mouse_move_to(x: i32, y: i32) -> Result<()> {
    mouse_move_wind(x, y, 100)
}

/// 使用 WindMouse 算法生成拟人化鼠标移动轨迹，并沿轨迹逐点移动鼠标。
///
/// `max_duration_ms`: 最大执行时间（ms）。0 时无延迟直接跳到目标坐标。
pub fn mouse_move_wind(target_x: i32, target_y: i32, max_duration_ms: u64) -> Result<()> {
    // 0 = instant jump, no trajectory
    if max_duration_ms == 0 {
        let mut enigo = Enigo::new(&Settings::default()).context("初始化鼠标模拟器失败")?;
        enigo
            .move_mouse(target_x, target_y, Coordinate::Abs)
            .context("鼠标移动失败")?;
        return Ok(());
    }

    const MIN_STEP_INTERVAL_MS: u64 = 4;

    let (start_x, start_y) = mouse_location()?;
    let raw_points = windmouse_points(start_x as f32, start_y as f32, target_x as f32, target_y as f32);

    // Downsample if needed so each step has at least MIN_STEP_INTERVAL_MS
    let max_steps = (max_duration_ms / MIN_STEP_INTERVAL_MS).max(1);
    let step = if raw_points.len() <= max_steps as usize {
        1usize
    } else {
        (raw_points.len() + max_steps as usize - 1) / max_steps as usize
    };
    let points: Vec<&[i32; 3]> = raw_points
        .iter()
        .enumerate()
        .filter(|(i, _)| i % step == 0)
        .map(|(_, p)| p)
        .collect();

    // Always include the last point (target)
    let has_last = points.last().map_or(false, |p| p[0] == target_x && p[1] == target_y);
    let points = if !has_last {
        let mut pts = points;
        pts.push(raw_points.last().unwrap());
        pts
    } else {
        points
    };

    let mut enigo = Enigo::new(&Settings::default()).context("初始化鼠标模拟器失败")?;
    let t0 = std::time::Instant::now();
    let len = points.len();

    for (i, point) in points.iter().enumerate() {
        let [x, y, _wait_ms] = **point;

        enigo
            .move_mouse(x, y, Coordinate::Abs)
            .context("鼠标移动失败")?;

        if i + 1 < len {
            let target_elapsed = std::time::Duration::from_millis(
                max_duration_ms * (i as u64 + 1) / len as u64,
            );
            let elapsed = t0.elapsed();
            if elapsed < target_elapsed {
                std::thread::sleep(target_elapsed - elapsed);
            } else {
                // Behind schedule — jump to target and stop
                enigo
                    .move_mouse(target_x, target_y, Coordinate::Abs)
                    .context("鼠标移动失败")?;
                break;
            }
        }
    }

    Ok(())
}

/// 检测 ESC 键是否处于按下状态。
#[allow(unused)]
pub fn is_escape_pressed() -> bool {
    #[cfg(target_os = "macos")]
    {
        // CGEventSourceKeyState(kCGEventSourceStateCombinedSessionState, kVK_Escape)
        extern "C" {
            fn CGEventSourceKeyState(state_id: i32, keycode: u16) -> bool;
        }
        unsafe { CGEventSourceKeyState(0, 0x35) }
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

// ─── WindMouse 算法（内置实现，无外部依赖） ─────────────────────

/// 简单的 LCG 伪随机数生成器，用于 WindMouse 算法。
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(12345);
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        ((self.state >> 16) & 0x7FFF) as f32 / 32768.0
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

/// 使用 WindMouse 算法生成鼠标移动轨迹点。
///
/// 返回 `Vec<[i32; 3]>`，每项为 `[x, y, wait_ms]`。
fn windmouse_points(start_x: f32, start_y: f32, end_x: f32, end_y: f32) -> Vec<[i32; 3]> {
    const GRAVITY: f32 = 9.0;
    const WIND: f32 = 3.0;
    const MIN_WAIT: f32 = 2.0;
    const MAX_WAIT: f32 = 8.0;
    const MAX_STEP: f32 = 10.0;
    const TARGET_AREA: f32 = 100.0;

    let mut rng = SimpleRng::new();
    let mut points: Vec<[i32; 3]> = Vec::new();

    let mut cx = start_x;
    let mut cy = start_y;
    let mut vx = 0.0f32;
    let mut vy = 0.0f32;
    let mut wind_x = rng.range(-WIND, WIND);
    let mut wind_y = rng.range(-WIND, WIND);

    loop {
        let dx = end_x - cx;
        let dy = end_y - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1.0 { break; }

        let w = if dist >= TARGET_AREA { dist.min(WIND) } else { WIND * (dist / TARGET_AREA) };

        if dist >= TARGET_AREA {
            vx += wind_x / dist * MAX_STEP;
            vy += wind_y / dist * MAX_STEP;
        }

        // Gravity pulls toward target
        vx += dx / dist * GRAVITY;
        vy += dy / dist * GRAVITY;

        // Clamp speed with randomness
        let speed = (vx * vx + vy * vy).sqrt();
        if speed > MAX_STEP {
            let rd = rng.range(0.0, w.max(0.0));
            vx = (vx / speed) * (MAX_STEP + rd);
            vy = (vy / speed) * (MAX_STEP + rd);
        }

        cx += vx;
        cy += vy;

        let step = (vx * vx + vy * vy).sqrt();
        let wait = (MAX_WAIT - MIN_WAIT) * (step / MAX_STEP) + MIN_WAIT;

        let px = cx.round() as i32;
        let py = cy.round() as i32;

        // Deduplicate consecutive points
        if points.last().map_or(true, |last: &[i32; 3]| last[0] != px || last[1] != py) {
            points.push([px, py, wait as i32]);
        }

        // Occasionally refresh wind direction
        if rng.next_f32() < 0.1 {
            wind_x = rng.range(-WIND, WIND);
            wind_y = rng.range(-WIND, WIND);
        }
    }

    // Ensure exact endpoint
    let ex = end_x.round() as i32;
    let ey = end_y.round() as i32;
    if points.last().map_or(true, |last: &[i32; 3]| last[0] != ex || last[1] != ey) {
        points.push([ex, ey, 0]);
    }

    points
}

/// 以当前鼠标位置为原点，相对移动 (dx, dy) 像素，内部走 WindMouse 拟人化轨迹。
pub fn mouse_move_relative(dx: i32, dy: i32) -> Result<()> {
    let (cx, cy) = mouse_location()?;
    mouse_move_wind(cx + dx, cy + dy, 100)
}

/// 点击鼠标按钮（按下后立即抬起）。
///
/// `button` 取值: `"left"`, `"right"`, `"middle"`, `"back"`, `"forward"`.
pub fn mouse_click(button: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化鼠标模拟器失败")?;
    let btn = parse_button(button)?;
    enigo
        .button(btn, Direction::Click)
        .context("鼠标点击失败")?;
    Ok(())
}

/// 按下鼠标按钮（保持按下状态，需配合 [`mouse_up`] 抬起）。
///
/// `button` 取值: `"left"`, `"right"`, `"middle"`, `"back"`, `"forward"`.
pub fn mouse_down(button: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化鼠标模拟器失败")?;
    let btn = parse_button(button)?;
    enigo
        .button(btn, Direction::Press)
        .context("鼠标按下失败")?;
    Ok(())
}

/// 抬起鼠标按钮（释放之前通过 [`mouse_down`] 按下的按钮）。
///
/// `button` 取值: `"left"`, `"right"`, `"middle"`, `"back"`, `"forward"`.
pub fn mouse_up(button: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化鼠标模拟器失败")?;
    let btn = parse_button(button)?;
    enigo
        .button(btn, Direction::Release)
        .context("鼠标抬起失败")?;
    Ok(())
}

/// 获取当前鼠标坐标。返回 `(x, y)` 屏幕绝对坐标。
pub fn mouse_location() -> Result<(i32, i32)> {
    #[cfg(target_os = "macos")]
    {
        use core_graphics::event::CGEvent;
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
        let src = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .map_err(|()| anyhow::anyhow!("创建事件源失败"))?;
        let event = CGEvent::new(src)
            .map_err(|()| anyhow::anyhow!("创建事件失败"))?;
        let loc = CGEvent::location(&event);
        Ok((loc.x.round() as i32, loc.y.round() as i32))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let enigo = Enigo::new(&Settings::default()).context("初始化输入模拟器失败")?;
        let (x, y) = enigo.mouse_location().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok((x as i32, y as i32))
    }
}

// ─── 组合键操作 ──────────────────────────────────────────────

/// 按下组合键（依次按下所有修饰键和主键，然后反向抬起）。
///
/// 例如: `combo(&["ctrl"], "c")` 模拟 Ctrl+C。
/// 例如: `combo(&["ctrl", "shift"], "escape")` 模拟 Ctrl+Shift+Esc。
pub fn combo(modifiers: &[&str], key: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化输入模拟器失败")?;

    // 按下所有修饰键。
    for m in modifiers {
        let k = parse_key(m)?;
        enigo
            .key(k, Direction::Press)
            .context("组合键修饰键按下失败")?;
    }

    // 点击主键。
    let main_key = parse_key(key)?;
    enigo
        .key(main_key, Direction::Click)
        .context("组合键主键操作失败")?;

    // 反向抬起修饰键。
    for m in modifiers.iter().rev() {
        let k = parse_key(m)?;
        enigo
            .key(k, Direction::Release)
            .context("组合键修饰键抬起失败")?;
    }

    Ok(())
}

// ─── 测试 ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── parse_key ──────────────────────────────────────────

    #[test]
    fn parse_key_named_special() {
        assert_eq!(parse_key("enter").unwrap(), Key::Return);
        assert_eq!(parse_key("return").unwrap(), Key::Return);
        assert_eq!(parse_key("tab").unwrap(), Key::Tab);
        assert_eq!(parse_key("space").unwrap(), Key::Space);
        assert_eq!(parse_key("backspace").unwrap(), Key::Backspace);
        assert_eq!(parse_key("escape").unwrap(), Key::Escape);
        assert_eq!(parse_key("esc").unwrap(), Key::Escape);
    }

    #[test]
    fn parse_key_named_arrows() {
        assert_eq!(parse_key("left").unwrap(), Key::LeftArrow);
        assert_eq!(parse_key("arrowleft").unwrap(), Key::LeftArrow);
        assert_eq!(parse_key("right").unwrap(), Key::RightArrow);
        assert_eq!(parse_key("arrowright").unwrap(), Key::RightArrow);
        assert_eq!(parse_key("up").unwrap(), Key::UpArrow);
        assert_eq!(parse_key("arrowup").unwrap(), Key::UpArrow);
        assert_eq!(parse_key("down").unwrap(), Key::DownArrow);
        assert_eq!(parse_key("arrowdown").unwrap(), Key::DownArrow);
    }

    #[test]
    fn parse_key_named_navigation() {
        assert_eq!(parse_key("home").unwrap(), Key::Home);
        assert_eq!(parse_key("end").unwrap(), Key::End);
        assert_eq!(parse_key("pageup").unwrap(), Key::PageUp);
        assert_eq!(parse_key("pagedown").unwrap(), Key::PageDown);
        assert_eq!(parse_key("delete").unwrap(), Key::Delete);
        assert_eq!(parse_key("del").unwrap(), Key::Delete);
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(parse_key("insert").unwrap(), Key::Insert);
            assert_eq!(parse_key("ins").unwrap(), Key::Insert);
        }
        #[cfg(target_os = "macos")]
        {
            assert_eq!(parse_key("insert").unwrap(), Key::Other(114)); // kVK_Help
            assert_eq!(parse_key("ins").unwrap(), Key::Other(114));
        }
    }

    #[test]
    fn parse_key_named_modifiers() {
        assert_eq!(parse_key("shift").unwrap(), Key::Shift);
        assert_eq!(parse_key("lshift").unwrap(), Key::LShift);
        assert_eq!(parse_key("rshift").unwrap(), Key::RShift);
        assert_eq!(parse_key("ctrl").unwrap(), Key::Control);
        assert_eq!(parse_key("control").unwrap(), Key::Control);
        assert_eq!(parse_key("lctrl").unwrap(), Key::LControl);
        assert_eq!(parse_key("rctrl").unwrap(), Key::RControl);
        assert_eq!(parse_key("alt").unwrap(), Key::Alt);
        assert_eq!(parse_key("win").unwrap(), Key::Meta);
        assert_eq!(parse_key("meta").unwrap(), Key::Meta);
        assert_eq!(parse_key("command").unwrap(), Key::Meta);
    }

    #[test]
    fn parse_key_named_locks() {
        assert_eq!(parse_key("capslock").unwrap(), Key::CapsLock);
        #[cfg(not(target_os = "macos"))]
        assert_eq!(parse_key("numlock").unwrap(), Key::Numlock);
        #[cfg(target_os = "macos")]
        assert_eq!(parse_key("numlock").unwrap(), Key::Other(71)); // kVK_ANSI_KeypadClear
    }

    #[test]
    fn parse_key_named_function_keys() {
        for (name, expected) in [
            ("f1", Key::F1),
            ("f2", Key::F2),
            ("f3", Key::F3),
            ("f4", Key::F4),
            ("f5", Key::F5),
            ("f6", Key::F6),
            ("f7", Key::F7),
            ("f8", Key::F8),
            ("f9", Key::F9),
            ("f10", Key::F10),
            ("f11", Key::F11),
            ("f12", Key::F12),
        ] {
            assert_eq!(parse_key(name).unwrap(), expected);
        }
    }

    #[test]
    fn parse_key_single_chars() {
        assert_eq!(parse_key("a").unwrap(), Key::Unicode('a'));
        assert_eq!(parse_key("Z").unwrap(), Key::Unicode('Z'));
        assert_eq!(parse_key("1").unwrap(), Key::Unicode('1'));
        assert_eq!(parse_key(",").unwrap(), Key::Unicode(','));
        assert_eq!(parse_key("!").unwrap(), Key::Unicode('!'));
        assert_eq!(parse_key("测").unwrap(), Key::Unicode('测'));
    }

    #[test]
    fn parse_key_invalid() {
        assert!(parse_key("invalid_key").is_err());
        assert!(parse_key("").is_err());
        assert!(parse_key("ab").is_err());
    }

    // ─── parse_button ───────────────────────────────────────

    #[test]
    fn parse_button_valid() {
        assert_eq!(parse_button("left").unwrap(), Button::Left);
        assert_eq!(parse_button("right").unwrap(), Button::Right);
        assert_eq!(parse_button("middle").unwrap(), Button::Middle);
        assert_eq!(parse_button("back").unwrap(), Button::Back);
        assert_eq!(parse_button("x1").unwrap(), Button::Back);
        assert_eq!(parse_button("forward").unwrap(), Button::Forward);
        assert_eq!(parse_button("x2").unwrap(), Button::Forward);
    }

    #[test]
    fn parse_button_invalid() {
        assert!(parse_button("invalid").is_err());
        assert!(parse_button("").is_err());
        assert!(parse_button("middle_click").is_err());
    }
}
