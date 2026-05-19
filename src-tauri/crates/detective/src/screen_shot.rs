/// 截图识别对外 API —— 捕获当前屏幕，加载模板，执行检测管线，保存标注结果。
///
/// 调用方（如测试、上层 Tauri 命令）只需调用 `take_screenshot()`，
/// 无需关心内部管线细节。
use anyhow::{anyhow, Context, Result};
use log::{error, info, warn};
use opencv::prelude::*;
use std::path::PathBuf;

use crate::{
    feature_matcher, geometric_verifier, pipeline, screen_capture,
    types::DetectionConfig,
};

/// 对外入口：执行一次"截图 → 识别 → 标注保存"流程。
///
/// 设计上不向上抛出错误，避免因识别失败导致调用方 panic。
/// 识别失败时仅打印错误，便于在日志中排查原因。
pub fn take_screenshot() {
    if let Err(err) = detect_and_save() {
        error!("[detective] 截图识别失败: {err:#}");
    }
}

/// 诊断用：运行完整检测管线并返回原始结果与诊断信息。
///
/// 返回 `(Option<DetectionResult>, Vec<String>)`：
/// - 第一个字段为检测结果（与管线一致）
/// - 第二个字段为各阶段的诊断消息（模板尺寸、捕获尺寸、ORB 匹配情况等）
pub fn diagnose() -> (Option<crate::types::DetectionResult>, Vec<String>) {
    match detect_and_diagnose() {
        Ok(result) => result,
        Err(err) => (None, vec![format!("诊断执行失败: {err:#}")]),
    }
}

/// 核心流程：捕获屏幕 → 加载模板 → 运行管线 → 绘制并保存结果。
fn detect_and_save() -> Result<()> {
    // 1) 定位 static 目录
    let static_dir = locate_static_dir().context("无法定位 static 目录")?;
    let target_path = static_dir.join("target.png");
    let output_path = static_dir.join("screenshot_marked.png");

    if !target_path.exists() {
        return Err(anyhow!(
            "模板文件不存在: {}，请在 static/ 目录放置 target.png",
            target_path.display()
        ));
    }

    // 2) 离线阶段：加载模板并预提取 ORB 特征
    let config = DetectionConfig::default();
    let mut orb = feature_matcher::create_orb(&config)?;

    let template_gray = opencv::imgcodecs::imread(
        &target_path.to_string_lossy().into_owned(),
        opencv::imgcodecs::IMREAD_GRAYSCALE,
    )
    .context("读取模板图像失败")?;

    if template_gray.empty() {
        return Err(anyhow!("模板图像为空"));
    }

    let orb_template = feature_matcher::prepare_template(&target_path, &mut orb).ok();

    // 3) 创建持久化屏幕捕获器并捕获当前屏幕（复用避免 DXGI 首帧黑屏）。
    let mut cap = screen_capture::ScreenCapture::new()?;
    let (sw, sh, screen_buf) = cap.capture()?;

    // 4) 运行检测管线
    let result = pipeline::detect(
        &screen_buf,
        sw,
        sh,
        &template_gray,
        orb_template.as_ref(),
        &mut orb,
        &config,
    )?;

    match result {
        Some(detection) => {
            // 5) 在屏幕上绘制检测框并保存
            let mut screen_color = cap.capture_to_bgr_mat()?;
            geometric_verifier::draw_detection_and_save(
                &mut screen_color,
                &detection,
                &output_path,
            )?;
            info!(
                "[detective] 检测成功: 中心=({}, {}), 尺寸={}x{}, 置信度={:.3}",
                detection.center_x,
                detection.center_y,
                detection.width,
                detection.height,
                detection.confidence
            );
        }
        None => {
            warn!("[detective] 未在屏幕中找到目标");
        }
    }

    Ok(())
}

/// 在常见工作目录下定位 static 目录。
///
/// 兼容场景：
/// - 在 detective crate 目录运行测试
/// - 在 src-tauri 根目录运行命令
/// - 在 workspace 根目录运行命令
fn locate_static_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("无法读取当前工作目录")?;

    let candidates = [
        cwd.join("static"),
        cwd.join("crates/detective/static"),
        cwd.join("src-tauri/crates/detective/static"),
    ];

    for path in candidates {
        if path.join("target.png").exists() {
            return Ok(path);
        }
    }

    Err(anyhow!("未找到包含 target.png 的 static 目录"))
}

/// 带诊断信息的检测流程：逐步执行并收集每阶段的关键数据。
fn detect_and_diagnose(
) -> Result<(Option<crate::types::DetectionResult>, Vec<String>)> {
    let mut diag = Vec::new();

    // 1) 定位 static 目录
    let static_dir = locate_static_dir()
        .map_err(|e| anyhow!("无法定位 static 目录: {e}"))?;
    let target_path = static_dir.join("target.png");
    let _output_path = static_dir.join("screenshot_marked.png");

    if !target_path.exists() {
        return Err(anyhow!("模板文件不存在: {}", target_path.display()));
    }

    // 2) 加载模板
    let config = DetectionConfig::default();
    let mut orb = feature_matcher::create_orb(&config)?;

    let template_gray = opencv::imgcodecs::imread(
        &target_path.to_string_lossy().into_owned(),
        opencv::imgcodecs::IMREAD_GRAYSCALE,
    )
    .context("读取模板图像失败")?;

    if template_gray.empty() {
        return Err(anyhow!("模板图像为空"));
    }

    diag.push(format!(
        "模板: {}x{} ({}x{} 像素)",
        template_gray.cols(),
        template_gray.rows(),
        template_gray.cols(),
        template_gray.rows()
    ));
    diag.push(format!(
        "管线配置: process={}x{} (auto), pyramid_levels={}, pyramid_scale={}, max_features={}",
        config.process_width,
        config.process_height,
        config.pyramid_levels,
        config.pyramid_scale,
        config.max_features
    ));

    let orb_template = feature_matcher::prepare_template(&target_path, &mut orb).ok();
    match &orb_template {
        Some(tpl) => diag.push(format!("模板 ORB 特征点数: {}", tpl.keypoints.len())),
        None => diag.push("模板 ORB 特征: 提取失败，仅使用模板匹配".to_string()),
    }

    // 3) 捕获屏幕
    let (sw, sh, screen_buf) = screen_capture::capture_primary()?;
    diag.push(format!("屏幕捕获: {}x{} ({} 字节 BGRA)", sw, sh, screen_buf.len()));
    let non_zero_pixels = screen_buf.chunks(4).filter(|c| c != &[0, 0, 0, 0]).count();
    let total_pixels = screen_buf.len() / 4;
    diag.push(format!(
        "缓冲非零像素: {}/{} ({:.1}%)",
        non_zero_pixels,
        total_pixels,
        non_zero_pixels as f64 / total_pixels.max(1) as f64 * 100.0
    ));

    let (pw, ph) = pipeline::resolve_process_size(&config, sw, sh);
    diag.push(format!("实际处理分辨率: {}x{}", pw, ph));
    let process_scale_x = pw as f64 / sw as f64;
    let process_scale_y = ph as f64 / sh as f64;
    diag.push(format!(
        "缩放因子: process/screen = {:.4}x{:.4}",
        process_scale_x, process_scale_y
    ));

    // 4) 直接调用管线（内部已完成缩放 + 金字塔 + 匹配 + 验证）。
    let result = pipeline::detect(
        &screen_buf, sw, sh,
        &template_gray, orb_template.as_ref(), &mut orb, &config,
    )?;

    match &result {
        Some(det) => {
            diag.push(format!(
                "最终结果: 中心=({}, {}), 左上=({}, {}), 尺寸={}x{}, conf={:.3}, method={} inliers={}",
                det.center_x, det.center_y,
                det.x, det.y,
                det.width, det.height,
                det.confidence,
                det.method,
                det.inlier_count
            ));
        }
        None => {
            diag.push("最终结果: 未检测到目标".to_string());
        }
    }

    Ok((result, diag))
}
