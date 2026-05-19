/// 实时检测循环 —— 持续捕获屏幕并以预设间隔运行目标检测管线。
///
/// 提供两种模式：
/// - `run_realtime_detection`: 带 OpenCV 预览窗口，支持 ESC 提前退出。
/// - `run_realtime_detection_headless`: 无窗口版本，适合自动化测试或后端服务。
///
/// 内部使用 ORB + 图像金字塔 + RANSAC 管线，并预留模板匹配回退。
use anyhow::{anyhow, Context, Result};
use log::{error, info};
use opencv::{
    core::{AlgorithmHint, Point, Rect, Scalar},
    highgui, imgproc,
    prelude::*,
};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use crate::{
    feature_matcher, pipeline, screen_capture,
    types::{DetectionBox, DetectionConfig, OrbTemplate},
};

/// 实时检测入口（带 OpenCV 预览窗口）。
///
/// # 参数
/// - `interval_ms`: 两次检测之间的最小间隔（毫秒）
/// - `max_frames`: 最大处理帧数
pub fn run_realtime_detection(interval_ms: u64, max_frames: usize) -> Result<()> {
    run_inner(interval_ms, max_frames, true)
}

/// 实时检测入口（无窗口，适合自动化测试）。
pub fn run_realtime_detection_headless(interval_ms: u64, max_frames: usize) -> Result<()> {
    run_inner(interval_ms, max_frames, false)
}

fn run_inner(interval_ms: u64, max_frames: usize, show_window: bool) -> Result<()> {
    if max_frames == 0 {
        return Err(anyhow!("max_frames 必须大于 0"));
    }

    // 1) 离线阶段：定位 static 目录，加载全部模板并预提取 ORB 特征。
    let static_dir = locate_static_dir().context("无法定位 static 目录")?;
    let loaded = load_templates(&static_dir).context("加载模板失败")?;

    if show_window {
        highgui::named_window("FerretAction Realtime Detect", highgui::WINDOW_NORMAL)
            .context("创建实时预览窗口失败")?;
    }

    let config = DetectionConfig::default();
    let detect_interval = Duration::from_millis(interval_ms);
    let mut last_detect_at: Option<Instant> = None;
    let mut perf = PerfStats::default();

    // 持久化捕获器，避免每帧重建 DXGI 管理器。
    let mut cap = match screen_capture::ScreenCapture::new() {
        Ok(c) => c,
        Err(e) => return Err(anyhow!("创建屏幕捕获器失败: {e:#}")),
    };

    // 跨帧持久的检测结果 —— 当屏幕无变化时复用上一帧的结果。
    let mut detections: Vec<DetectionBox> = Vec::new();

    for frame_idx in 0..max_frames {
        let frame_start = Instant::now();

        // 2) 捕获当前屏幕画面及帧元数据（脏矩形）。
        let capture_start = Instant::now();
        let (sw, sh, screen_buf, frame_meta) = match cap.capture_with_metadata() {
            Ok(cap) => cap,
            Err(e) => {
                error!("[frame={}] 屏幕捕获失败: {e:#}", frame_idx + 1);
                continue;
            }
        };
        let capture_elapsed = capture_start.elapsed();
        let has_screen_updates = frame_meta.has_updates();

        // 3) 将 BGRA 缓冲转为 BGR Mat（用于显示）。
        let convert_start = Instant::now();
        let mut display_mat = match bgra_to_bgr_mat(&screen_buf, sw, sh) {
            Ok(mat) => mat,
            Err(e) => {
                error!("[frame={}] BGRA 转 BGR 失败: {e:#}", frame_idx + 1);
                continue;
            }
        };
        let convert_elapsed = convert_start.elapsed();

        // 4) 按间隔且仅在屏幕有更新时执行检测。
        let detect_start = Instant::now();
        let now = Instant::now();
        let interval_ok = match last_detect_at {
            None => true,
            Some(last) => now.duration_since(last) >= detect_interval,
        };
        let should_detect = interval_ok && has_screen_updates;

        if should_detect {
            let detect_start_inner = Instant::now();
            let mut new_detections = Vec::new();
            for (template_gray, orb_template) in &loaded.templates {
                let mut orb = match feature_matcher::create_orb(&config) {
                    Ok(o) => o,
                    Err(_) => continue,
                };

                match pipeline::detect(
                    &screen_buf,
                    sw,
                    sh,
                    template_gray,
                    orb_template.as_ref(),
                    &mut orb,
                    &config,
                ) {
                    Ok(Some(result)) => {
                        let name = orb_template
                            .as_ref()
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| "unknown".to_string());
                        new_detections.push(DetectionBox {
                            target_name: name,
                            confidence: result.confidence as f32,
                            x: result.x,
                            y: result.y,
                            width: result.width,
                            height: result.height,
                        });
                    }
                    Ok(None) => {} // 未找到，正常情况。
                    Err(e) => {
                        let name = orb_template
                            .as_ref()
                            .map(|t| t.name.as_str())
                            .unwrap_or("unknown");
                        error!(
                            "[frame={}] 模板 {} 检测失败: {e:#}",
                            frame_idx + 1,
                            name
                        );
                    }
                }
            }
            detections = new_detections;
            last_detect_at = Some(now);
            let _detect_inner_elapsed = detect_start_inner.elapsed();
        }
        let detect_elapsed = detect_start.elapsed();

        // 5) 日志输出。
        info!(
            "[frame={}] detect_run={} screen_updated={} detections={}",
            frame_idx + 1,
            should_detect,
            has_screen_updates,
            detections.len()
        );
        for d in &detections {
            info!(
                "  - {} conf={:.3} rect=({}, {}, {}, {})",
                d.target_name, d.confidence, d.x, d.y, d.width, d.height
            );
        }

        // 6) 在显示画面绘制检测框。
        for d in &detections {
            imgproc::rectangle(
                &mut display_mat,
                Rect::new(d.x, d.y, d.width, d.height),
                Scalar::new(0.0, 0.0, 255.0, 0.0),
                2,
                imgproc::LINE_AA,
                0,
            )?;
            // 绘制模板名称标签。
            imgproc::put_text(
                &mut display_mat,
                &format!("{}: {:.2}", d.target_name, d.confidence),
                Point::new(d.x, (d.y - 5).max(5)),
                imgproc::FONT_HERSHEY_SIMPLEX,
                0.6,
                Scalar::new(0.0, 255.0, 0.0, 0.0),
                2,
                imgproc::LINE_AA,
                false,
            )?;
        }

        // 7) 显示预览窗口。
        if show_window {
            highgui::imshow("FerretAction Realtime Detect", &display_mat)
                .context("刷新实时预览窗口失败")?;
            let key = highgui::wait_key(1).context("轮询键盘事件失败")?;
            if key == 27 {
                // ESC
                info!("收到 ESC，提前结束实时检测");
                break;
            }
        }

        let frame_elapsed = frame_start.elapsed();
        info!(
            "[perf frame={}] capture={:.2}ms convert={:.2}ms detect={:.2}ms total={:.2}ms",
            frame_idx + 1,
            to_ms(capture_elapsed),
            to_ms(convert_elapsed),
            to_ms(detect_elapsed),
            to_ms(frame_elapsed),
        );

        perf.push_frame(
            should_detect,
            capture_elapsed,
            convert_elapsed,
            detect_elapsed,
            frame_elapsed,
        );

        // 帧率控制（如果处理过快，休眠至满足间隔）。
        if let Some(elapsed) = Instant::now().checked_duration_since(frame_start) {
            if elapsed < detect_interval {
                thread::sleep(detect_interval - elapsed);
            }
        }
    }

    perf.print_summary(interval_ms);

    if show_window {
        highgui::destroy_window("FerretAction Realtime Detect").context("关闭实时预览窗口失败")?;
    }

    Ok(())
}

// ─── 内部辅助 ────────────────────────────────────────────────

/// 一次性加载的模板集合（离线阶段产物）。
struct LoadedTemplates {
    /// (模板灰度图, 预提取的 ORB 特征 —— 小模板可能为 None)
    templates: Vec<(Mat, Option<OrbTemplate>)>,
}

/// 从 static 目录加载所有 target* 模板并预提取 ORB 特征。
fn load_templates(static_dir: &Path) -> Result<LoadedTemplates> {
    let config = DetectionConfig::default();
    let mut orb = feature_matcher::create_orb(&config)?;
    let mut templates = Vec::new();

    for entry in fs::read_dir(static_dir)
        .with_context(|| format!("读取 static 目录失败: {}", static_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };

        // 只处理 target* 命名约定的图片。
        if !file_name.starts_with("target") {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "webp") {
            continue;
        }

        // 加载灰度模板。
        let template_gray = opencv::imgcodecs::imread(
            &path.to_string_lossy().into_owned(),
            opencv::imgcodecs::IMREAD_GRAYSCALE,
        )
        .with_context(|| format!("读取目标模板失败: {}", path.display()))?;

        if template_gray.empty() {
            continue;
        }

        // 预提取 ORB 特征（小模板可能失败，此时降级为纯模板匹配）。
        let orb_template = feature_matcher::prepare_template(&path, &mut orb).ok();

        templates.push((template_gray, orb_template));
    }

    if templates.is_empty() {
        return Err(anyhow!(
            "static 目录中未找到 target* 模板图片（支持 png/jpg/jpeg/bmp/webp）"
        ));
    }

    Ok(LoadedTemplates { templates })
}

/// 将 BGRA 字节缓冲转为 BGR Mat（用于 OpenCV 显示）。
fn bgra_to_bgr_mat(buffer: &[u8], _width: u32, height: u32) -> Result<Mat> {
    let mat_1d = Mat::from_slice(buffer).context("将 BGRA 缓冲转为 Mat 失败")?;
    let bgra_mat = mat_1d
        .reshape(4, height as i32)
        .context("重塑 BGRA Mat 形状失败")?;
    let mut bgr = Mat::default();
    imgproc::cvt_color(
        &bgra_mat,
        &mut bgr,
        imgproc::COLOR_BGRA2BGR,
        0,
        AlgorithmHint::ALGO_HINT_DEFAULT,
    )
    .context("BGRA 转 BGR 失败")?;
    Ok(bgr)
}

/// 在常见执行目录下定位 static 目录。
fn locate_static_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("无法读取当前工作目录")?;
    let candidates = [
        cwd.join("static"),
        cwd.join("crates/detective/static"),
        cwd.join("src-tauri/crates/detective/static"),
    ];

    for path in candidates {
        if !path.exists() {
            continue;
        }
        let mut has_target = false;
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("target") {
                        has_target = true;
                        break;
                    }
                }
            }
        }
        if has_target {
            return Ok(path);
        }
    }

    Err(anyhow!("未找到包含 target* 模板图片的 static 目录"))
}

fn to_ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

// ─── 性能统计 ────────────────────────────────────────────────

#[derive(Debug, Default)]
struct PerfStats {
    frames: usize,
    detect_runs: usize,
    detect_skips: usize,
    capture: Duration,
    convert: Duration,
    detect: Duration,
    total: Duration,
}

impl PerfStats {
    fn push_frame(
        &mut self,
        did_detect: bool,
        capture: Duration,
        convert: Duration,
        detect: Duration,
        total: Duration,
    ) {
        self.frames += 1;
        if did_detect {
            self.detect_runs += 1;
        } else {
            self.detect_skips += 1;
        }
        self.capture += capture;
        self.convert += convert;
        self.detect += detect;
        self.total += total;
    }

    fn print_summary(&self, interval_ms: u64) {
        if self.frames == 0 {
            info!("[perf summary] 没有可统计的帧");
            return;
        }

        let f = self.frames as f64;
        let avg_capture = to_ms(self.capture) / f;
        let avg_convert = to_ms(self.convert) / f;
        let avg_detect = to_ms(self.detect) / f;
        let avg_total = to_ms(self.total) / f;

        let fps = if avg_total > 0.0 {
            1000.0 / avg_total
        } else {
            f64::INFINITY
        };

        info!(
            "[perf summary] frames={} detect_runs={} detect_skips={} interval={}ms \
             avg_capture={:.2}ms avg_convert={:.2}ms avg_detect={:.2}ms avg_total={:.2}ms fps={:.2}",
            self.frames,
            self.detect_runs,
            self.detect_skips,
            interval_ms,
            avg_capture,
            avg_convert,
            avg_detect,
            avg_total,
            fps,
        );
    }
}
