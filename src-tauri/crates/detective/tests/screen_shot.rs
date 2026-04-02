use detective::real_time_detective::{run_realtime_detection, run_realtime_detection_headless};
use detective::screen_shot::take_screenshot;

#[test]
fn take_screenshot_runs_without_panic() {
    take_screenshot();
}

#[test]
#[ignore = "手工验证实时展示时运行：cargo test --test screen_shot real_time_detective_runs_without_panic -- --ignored --nocapture"]
fn real_time_detective_runs_without_panic() {
    // 先按 100ms 间隔跑 100 帧，期间可按 ESC 提前退出。
    run_realtime_detection(100, 100).expect("实时检测执行失败");
}

#[test]
fn real_time_detective_headless_runs_without_panic() {
    // 无窗口版本，便于后端快速冒烟验证。
    run_realtime_detection_headless(100, 3).expect("无窗口实时检测执行失败");
}
