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

/// 移动鼠标到屏幕绝对坐标 (x, y)。
pub fn mouse_move_to(x: i32, y: i32) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化鼠标模拟器失败")?;
    enigo
        .move_mouse(x, y, Coordinate::Abs)
        .context("鼠标移动失败")?;
    Ok(())
}

/// 以当前鼠标位置为原点，相对移动 (dx, dy) 像素。
pub fn mouse_move_relative(dx: i32, dy: i32) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).context("初始化鼠标模拟器失败")?;
    enigo
        .move_mouse(dx, dy, Coordinate::Rel)
        .context("鼠标相对移动失败")?;
    Ok(())
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
