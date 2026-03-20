use std::fs;
use std::path::PathBuf;
use xcap::Monitor;

pub fn take_screenshot() {
    let monitors = Monitor::all().expect("获取屏幕信息失败");
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
    fs::create_dir_all(&output_dir).expect("创建截图目录失败");

    for monitor in monitors {
        let image = monitor.capture_image().expect("截图失败");
        let output_path = output_dir.join("screenshot.png");
        image.save(&output_path).expect("保存截图失败");
    }
}
