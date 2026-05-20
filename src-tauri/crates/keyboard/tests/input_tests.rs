use keyboard;

// ─── 键名解析错误路径 ──────────────────────────────────────

#[test]
fn key_press_rejects_invalid_name() {
    assert!(keyboard::key_press("bogus_key_name").is_err());
}

#[test]
fn key_down_rejects_invalid_name() {
    assert!(keyboard::key_down("invalid").is_err());
}

#[test]
fn key_up_rejects_invalid_name() {
    assert!(keyboard::key_up("nope").is_err());
}

#[test]
fn combo_rejects_invalid_modifier() {
    assert!(keyboard::combo(&["bogus"], "a").is_err());
}

#[test]
fn combo_rejects_invalid_key() {
    assert!(keyboard::combo(&["ctrl"], "invalid_key").is_err());
}

// ─── 鼠标按钮名解析错误路径 ────────────────────────────────

#[test]
fn mouse_click_rejects_invalid_button() {
    assert!(keyboard::mouse_click("wheel").is_err());
}

#[test]
fn mouse_down_rejects_invalid_button() {
    assert!(keyboard::mouse_down("scroll").is_err());
}

#[test]
fn mouse_up_rejects_invalid_button() {
    assert!(keyboard::mouse_up("none").is_err());
}

// ─── 真实输入冒烟测试（手工验证） ───────────────────────────

#[test]
#[ignore = "会实际操作键盘鼠标，按需手工运行：cargo test --test input_tests real_input_smoke -- --ignored --nocapture"]
fn real_input_smoke() {
    // 相对移动（轻微调整，不会跑出屏幕）。
    keyboard::mouse_move_relative(0, 0).expect("相对移动失败");

    // 点击按键 "a"（在当前焦点窗口输入字符 a）。
    keyboard::key_press("a").expect("按键失败");

    // 组合键 Ctrl+C（复制）。
    keyboard::combo(&["ctrl"], "c").expect("组合键失败");
}
