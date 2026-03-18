use std::fs;
use xcap::Monitor;

pub fn take_screenshot() {
    let monitors = Monitor::all().expect("获取屏幕信息失败");

    for monitor in monitors {
        let image = monitor.capture_image().expect("截图失败");
    }
}
