use anyhow::{anyhow, Context, Result};
use opencv::{
    core::{self, AlgorithmHint, Point},
    highgui, imgcodecs, imgproc,
    prelude::*,
};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use xcap::{Frame, Monitor};

const DETECT_DOWNSAMPLE_SCALE: f64 = 0.5;

/// 单次模板命中的结构化结果。
///
/// 这个结构体既可以用于日志打印，也方便后续直接透传到上层（例如 Tauri 命令返回给前端）。
#[derive(Debug, Clone)]
pub struct DetectionBox {
    /// 命中的模板文件名（例如 target_login.png）。
    pub target_name: String,
    /// 模板匹配置信度，取值范围通常在 [-1, 1]，越接近 1 代表越相似。
    pub confidence: f32,
    /// 命中框左上角 x（像素）。
    pub x: i32,
    /// 命中框左上角 y（像素）。
    pub y: i32,
    /// 命中框宽度（像素）。
    pub width: i32,
    /// 命中框高度（像素）。
    pub height: i32,
}

#[derive(Debug)]
struct TargetTemplate {
    /// 模板文件名（用于日志与窗口文字标注）。
    name: String,
    /// 预加载的原始灰度模板图（用于记录原始尺寸）。
    mat_gray: Mat,
    /// 预加载的降采样灰度模板图（用于实际匹配，降低计算量）。
    ///
    /// 模板匹配使用灰度可以降低计算量，并且对色彩变化不敏感。
    mat_gray_scaled: Mat,
}

/// 运行时性能统计（累加每帧耗时，结束后输出平均值）。
#[derive(Debug, Default)]
struct PerfStats {
    frames: usize,
    detect_runs: usize,
    detect_skips: usize,
    receive: Duration,
    convert: Duration,
    detect: Duration,
    process: Duration,
}

impl PerfStats {
    fn push_frame(
        &mut self,
        did_detect: bool,
        receive: Duration,
        convert: Duration,
        detect: Duration,
        process: Duration,
    ) {
        self.frames += 1;
        if did_detect {
            self.detect_runs += 1;
        } else {
            self.detect_skips += 1;
        }
        self.receive += receive;
        self.convert += convert;
        self.detect += detect;
        self.process += process;
    }

    fn print_summary(&self, interval_ms: u64) {
        if self.frames == 0 {
            println!("[perf summary] 没有可统计的帧");
            return;
        }

        let f = self.frames as f64;
        let avg_receive_ms = to_ms(self.receive) / f;
        let avg_convert_ms = to_ms(self.convert) / f;
        let avg_detect_ms = to_ms(self.detect) / f;
        let avg_process_ms = to_ms(self.process) / f;

        let process_fps = if avg_process_ms > 0.0 {
            1000.0 / avg_process_ms
        } else {
            f64::INFINITY
        };

        println!(
            "[perf summary] frames={} detect_runs={} detect_skips={} interval={}ms avg_receive={:.2}ms avg_convert={:.2}ms avg_detect={:.2}ms avg_process={:.2}ms process_fps={:.2}",
            self.frames,
            self.detect_runs,
            self.detect_skips,
            interval_ms,
            avg_receive_ms,
            avg_convert_ms,
            avg_detect_ms,
            avg_process_ms,
            process_fps,
        );
    }
}

/// 实时检测入口：
/// - 每 `interval_ms` 毫秒抓屏一次
/// - 在 static 中加载所有 target* 模板
/// - 在匹配到的位置绘制红框并实时展示
/// - 同时落盘最新标注图与原图，方便排查
pub fn run_realtime_detection(interval_ms: u64, max_frames: usize) -> Result<()> {
    run_realtime_detection_inner(interval_ms, max_frames, true)
}

/// 不弹窗版本，适合自动化测试或无界面环境。
pub fn run_realtime_detection_headless(interval_ms: u64, max_frames: usize) -> Result<()> {
    run_realtime_detection_inner(interval_ms, max_frames, false)
}

fn run_realtime_detection_inner(
    interval_ms: u64,
    max_frames: usize,
    show_window: bool,
) -> Result<()> {
    // 保护性检查：防止调用方误传 0 帧导致函数“看似执行但没有任何输出”。
    if max_frames == 0 {
        return Err(anyhow!("max_frames 必须大于 0"));
    }

    // 1) 定位 static 目录并加载模板。
    // 2) 选择主显示器作为录制来源。
    let static_dir = locate_static_dir().context("无法定位 static 目录")?;
    let targets = load_target_templates(&static_dir)?;
    let monitor = choose_primary_monitor()?;

    let (recorder, rx) = monitor.video_recorder().context("创建屏幕录制器失败")?;
    recorder.start().context("启动屏幕录制失败")?;

    let worker = thread::spawn(move || {
        run_detection_worker(rx, targets, interval_ms, max_frames, show_window)
    });

    let worker_result = worker.join().map_err(|_| anyhow!("检测线程 panic"))?;

    // 先尝试停止录制，再返回检测结果，避免采集会话泄漏。
    recorder.stop().context("停止屏幕录制失败")?;

    worker_result
}

fn run_detection_worker(
    rx: std::sync::mpsc::Receiver<Frame>,
    targets: Vec<TargetTemplate>,
    interval_ms: u64,
    max_frames: usize,
    show_window: bool,
) -> Result<()> {
    let mut perf = PerfStats::default();
    let detect_interval = Duration::from_millis(interval_ms);
    let mut last_detect_at: Option<Instant> = None;

    if show_window {
        highgui::named_window("FerretAction Realtime Detect", highgui::WINDOW_NORMAL)
            .context("创建实时预览窗口失败")?;
    }

    for frame_idx in 0..max_frames {
        let process_start = Instant::now();

        let receive_start = Instant::now();
        let frame = rx
            .recv_timeout(Duration::from_secs(3))
            .context("等待录制帧超时")?;
        let receive_elapsed = receive_start.elapsed();

        let convert_start = Instant::now();
        let frame_bgr = frame_to_bgr_mat(frame)?;
        let convert_elapsed = convert_start.elapsed();

        let detect_start = Instant::now();
        let now = Instant::now();
        let should_detect = match last_detect_at {
            None => true,
            Some(last) => now.duration_since(last) >= detect_interval,
        };

        let detections = if should_detect {
            let detected =
                detect_targets_scaled(&targets, &frame_bgr, 0.80, DETECT_DOWNSAMPLE_SCALE)?;
            last_detect_at = Some(now);
            detected
        } else {
            Vec::new()
        };
        let detect_elapsed = detect_start.elapsed();

        println!(
            "[frame={}] detect_run={} detections={}",
            frame_idx + 1,
            should_detect,
            detections.len()
        );
        for d in &detections {
            println!(
                "  - {} conf={:.3} rect=({}, {}, {}, {})",
                d.target_name, d.confidence, d.x, d.y, d.width, d.height
            );
        }

        if show_window {
            highgui::imshow("FerretAction Realtime Detect", &frame_bgr)
                .context("刷新实时预览窗口失败")?;
            let key = highgui::wait_key(1).context("轮询键盘事件失败")?;
            if key == 27 {
                println!("收到 ESC，提前结束实时检测");
                break;
            }
        }

        let process_elapsed = process_start.elapsed();
        println!(
            "[perf frame={}] receive={:.2}ms convert={:.2}ms detect={:.2}ms process={:.2}ms",
            frame_idx + 1,
            to_ms(receive_elapsed),
            to_ms(convert_elapsed),
            to_ms(detect_elapsed),
            to_ms(process_elapsed),
        );

        perf.push_frame(
            should_detect,
            receive_elapsed,
            convert_elapsed,
            detect_elapsed,
            process_elapsed,
        );
    }

    perf.print_summary(interval_ms);
    if show_window {
        highgui::destroy_window("FerretAction Realtime Detect").context("关闭实时预览窗口失败")?;
    }

    Ok(())
}

fn detect_targets_scaled(
    targets: &[TargetTemplate],
    frame_bgr: &Mat,
    threshold: f32,
    scale: f64,
) -> Result<Vec<DetectionBox>> {
    if !(0.0 < scale && scale <= 1.0) {
        return Err(anyhow!("scale 必须在 (0, 1] 范围内"));
    }

    let mut frame_gray = Mat::default();
    imgproc::cvt_color(
        frame_bgr,
        &mut frame_gray,
        imgproc::COLOR_BGR2GRAY,
        0,
        AlgorithmHint::ALGO_HINT_DEFAULT,
    )?;

    // 将截图降采样后进行模板匹配，再把坐标映射回原图，确保绘制位置准确。
    let mut frame_gray_scaled = Mat::default();
    imgproc::resize(
        &frame_gray,
        &mut frame_gray_scaled,
        core::Size::new(0, 0),
        scale,
        scale,
        imgproc::INTER_AREA,
    )?;

    let mut detections = Vec::new();

    for target in targets {
        let t_scaled = &target.mat_gray_scaled;
        let t_cols = t_scaled.cols();
        let t_rows = t_scaled.rows();
        if t_cols <= 0 || t_rows <= 0 {
            continue;
        }

        let result_cols = frame_gray_scaled.cols() - t_cols + 1;
        let result_rows = frame_gray_scaled.rows() - t_rows + 1;
        if result_cols <= 0 || result_rows <= 0 {
            continue;
        }

        let mut result = Mat::zeros(result_rows, result_cols, core::CV_32FC1)?.to_mat()?;
        imgproc::match_template(
            &frame_gray_scaled,
            t_scaled,
            &mut result,
            imgproc::TM_CCOEFF_NORMED,
            &Mat::default(),
        )?;

        let mut min_val = 0.0;
        let mut max_val = 0.0;
        let mut min_loc = Point::new(0, 0);
        let mut max_loc = Point::new(0, 0);
        core::min_max_loc(
            &result,
            Some(&mut min_val),
            Some(&mut max_val),
            Some(&mut min_loc),
            Some(&mut max_loc),
            &Mat::default(),
        )?;

        if max_val < threshold as f64 {
            continue;
        }

        // 将降采样坐标映射回原图坐标，宽高使用“原始模板尺寸”，避免框尺寸漂移。
        let x = ((max_loc.x as f64) / scale).round() as i32;
        let y = ((max_loc.y as f64) / scale).round() as i32;
        let width = target.mat_gray.cols();
        let height = target.mat_gray.rows();

        detections.push(DetectionBox {
            target_name: target.name.clone(),
            confidence: max_val as f32,
            x: x.max(0),
            y: y.max(0),
            width: width.max(1),
            height: height.max(1),
        });
    }

    Ok(detections)
}

/// 从 static 目录加载所有 target* 模板到内存。
///
/// 约定：
/// - 文件名必须以 target 开头。
/// - 扩展名支持 png/jpg/jpeg/bmp/webp。
///
/// 预加载的意义：避免在实时循环中反复从磁盘读取模板，减少抖动。
fn load_target_templates(static_dir: &Path) -> Result<Vec<TargetTemplate>> {
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

        let mat_gray = imgcodecs::imread(&path_to_string(&path), imgcodecs::IMREAD_GRAYSCALE)
            .with_context(|| format!("读取目标模板失败: {}", path.display()))?;
        // 防御性判断：损坏图片可能导致空 Mat。
        if mat_gray.empty() {
            continue;
        }

        let scaled_w = ((mat_gray.cols() as f64) * DETECT_DOWNSAMPLE_SCALE)
            .round()
            .max(1.0) as i32;
        let scaled_h = ((mat_gray.rows() as f64) * DETECT_DOWNSAMPLE_SCALE)
            .round()
            .max(1.0) as i32;
        let mut mat_gray_scaled = Mat::default();
        imgproc::resize(
            &mat_gray,
            &mut mat_gray_scaled,
            core::Size::new(scaled_w, scaled_h),
            0.0,
            0.0,
            imgproc::INTER_AREA,
        )?;

        templates.push(TargetTemplate {
            name: file_name.to_string(),
            mat_gray,
            mat_gray_scaled,
        });
    }

    if templates.is_empty() {
        return Err(anyhow!(
            "static 目录中未找到 target* 模板图片（支持 png/jpg/jpeg/bmp/webp）"
        ));
    }

    Ok(templates)
}

/// 选择主显示器；若未标记主屏，则回退到第一个显示器。
fn choose_primary_monitor() -> Result<Monitor> {
    let monitors = Monitor::all().context("读取显示器列表失败")?;
    if monitors.is_empty() {
        return Err(anyhow!("未检测到可用显示器"));
    }

    for monitor in &monitors {
        if monitor.is_primary().unwrap_or(false) {
            return Ok(monitor.clone());
        }
    }

    Ok(monitors[0].clone())
}

/// 在常见执行目录下定位 static 目录。
///
/// 兼容场景：
/// - 在 detective crate 目录运行
/// - 在 src-tauri 目录运行
/// - 在 workspace 根目录运行
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
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("target") {
                    has_target = true;
                    break;
                }
            }
        }

        if has_target {
            return Ok(path);
        }
    }

    Err(anyhow!("未找到包含 target* 模板图片的 static 目录"))
}

/// 将路径转换为 OpenCV 所需字符串。
///
/// 使用 lossy 可避免极端情况下非 UTF-8 路径直接失败。
fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn to_ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn frame_to_bgr_mat(frame: Frame) -> Result<Mat> {
    let width = frame.width as i32;
    let height = frame.height as i32;

    let bgra_1d = Mat::from_slice(&frame.raw).context("将录制帧缓冲区转为 Mat 失败")?;
    let bgra_mat = bgra_1d
        .reshape(4, height)
        .context("重塑 BGRA Mat 形状失败")?;

    let mut frame_bgr = Mat::default();
    imgproc::cvt_color(
        &bgra_mat,
        &mut frame_bgr,
        imgproc::COLOR_BGRA2BGR,
        0,
        AlgorithmHint::ALGO_HINT_DEFAULT,
    )
    .context("BGRA 转 BGR 失败")?;

    if frame_bgr.cols() != width || frame_bgr.rows() != height {
        return Err(anyhow!(
            "录制帧尺寸异常: expected={}x{}, actual={}x{}",
            width,
            height,
            frame_bgr.cols(),
            frame_bgr.rows()
        ));
    }

    if frame_bgr.empty() {
        return Err(anyhow!("录制帧为空"));
    }

    Ok(frame_bgr)
}
