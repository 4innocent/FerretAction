/// 管线编排模块 —— 将屏幕捕获、图像缩放、多尺度特征匹配、几何验证四个核心模块
/// 组合为完整的单帧检测管线。
///
/// 数据流：屏幕 BGRA 缓冲 → 缩放灰度图 → 图像金字塔 → 逐层 ORB 匹配 → RANSAC 验证 → 最佳检测结果。
use anyhow::Result;
use opencv::prelude::*;

use crate::{
    feature_matcher, geometric_verifier, image_scaler,
    types::{DetectionConfig, DetectionResult, OrbTemplate},
};

/// 根据屏幕尺寸和配置计算实际处理分辨率。
///
/// 当 `config.process_width/height` 为 0 时自动适配：
/// 以屏幕长边 1920 为上限等比缩放，在保持特征质量的同时控制计算开销。
pub fn resolve_process_size(config: &DetectionConfig, sw: u32, sh: u32) -> (u32, u32) {
    if config.process_width > 0 && config.process_height > 0 {
        return (config.process_width, config.process_height);
    }
    let max_dim = sw.max(sh);
    let cap = 1920u32;
    if max_dim <= cap {
        (sw, sh)
    } else {
        let ratio = cap as f64 / max_dim as f64;
        (
            (sw as f64 * ratio).round() as u32,
            (sh as f64 * ratio).round() as u32,
        )
    }
}

/// 对单帧画面执行完整的目标检测管线。
pub fn detect(
    screen_buffer: &[u8],
    screen_width: u32,
    screen_height: u32,
    template_gray: &opencv::prelude::Mat,
    orb_template: Option<&OrbTemplate>,
    orb: &mut opencv::core::Ptr<opencv::features2d::ORB>,
    config: &DetectionConfig,
) -> Result<Option<DetectionResult>> {
    // 阶段 1+2：计算处理分辨率并缩放到处理尺寸 + 转灰度。
    let (pw, ph) = resolve_process_size(config, screen_width, screen_height);
    let process_gray = image_scaler::scale_to_gray(
        screen_buffer,
        screen_width,
        screen_height,
        pw,
        ph,
    )?;
    let process_scale_x = pw as f64 / screen_width as f64;
    let process_scale_y = ph as f64 / screen_height as f64;

    // 阶段 2b：将模板灰度图等比缩放到处理分辨率（仅用于模板匹配）。
    let tpl_cols = orb_template.map(|t| t.cols).unwrap_or(template_gray.cols());
    let tpl_rows = orb_template.map(|t| t.rows).unwrap_or(template_gray.rows());
    let scaled_tw = (tpl_cols as f64 * process_scale_x).round() as i32;
    let scaled_th = (tpl_rows as f64 * process_scale_y).round() as i32;
    let scaled_template_gray = if scaled_tw > 0 && scaled_th > 0 {
        let mut scaled = opencv::prelude::Mat::default();
        opencv::imgproc::resize(
            template_gray,
            &mut scaled,
            opencv::core::Size::new(scaled_tw, scaled_th),
            0.0,
            0.0,
            opencv::imgproc::INTER_LANCZOS4,
        )
        .ok()
        .map(|_| scaled)
    } else {
        None
    };

    let mut best_result: Option<DetectionResult> = None;
    let mut best_confidence = 0.0;

    // 阶段 3：模板匹配（在处理分辨率灰度图上直接执行）。
    // 模板匹配对平面 UI 元素可靠性最高。
    if let Some(ref scaled_gray) = scaled_template_gray {
        if let Ok(Some(mut detection)) = geometric_verifier::detect_with_template_matching(
            scaled_gray,
            &process_gray,
            config.fallback_match_threshold,
            0usize,
            1.0,
        ) {
            detection.x = (detection.x as f64 / process_scale_x).round() as i32;
            detection.y = (detection.y as f64 / process_scale_y).round() as i32;
            detection.width = (detection.width as f64 / process_scale_x).round() as i32;
            detection.height = (detection.height as f64 / process_scale_y).round() as i32;
            detection.center_x /= process_scale_x;
            detection.center_y /= process_scale_y;

            if detection.confidence > best_confidence {
                best_confidence = detection.confidence;
                best_result = Some(detection);
            }
        }
    }

    // 阶段 4：ORB + 图像金字塔 + RANSAC（使用原始尺寸模板特征，ORB 自带尺度不变性）。
    if let Some(orb_tpl) = orb_template {
        let pyramid = feature_matcher::build_pyramid(&process_gray, config)?;

        for (level_idx, scale, level_mat) in &pyramid {
            if let Ok(Some(raw_matches)) =
                feature_matcher::match_at_level(orb, orb_tpl, level_mat, *level_idx, *scale)
            {
                if let Ok(Some(mut detection)) = geometric_verifier::verify(
                    &raw_matches,
                    orb_tpl.cols,
                    orb_tpl.rows,
                    *level_idx,
                    *scale,
                    "ORB",
                    config,
                ) {
                    let total_scale_x = process_scale_x * scale;
                    let total_scale_y = process_scale_y * scale;

                    detection.x = (detection.x as f64 / total_scale_x).round() as i32;
                    detection.y = (detection.y as f64 / total_scale_y).round() as i32;
                    detection.width = (detection.width as f64 / total_scale_x).round() as i32;
                    detection.height = (detection.height as f64 / total_scale_y).round() as i32;
                    detection.center_x /= total_scale_x;
                    detection.center_y /= total_scale_y;

                    if detection.confidence > best_confidence {
                        best_confidence = detection.confidence;
                        best_result = Some(detection);
                    }
                }
            }
        }
    }

    Ok(best_result)
}
