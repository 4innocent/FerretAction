/// 几何验证与定位模块 —— 通过 RANSAC 单应性估计剔除误匹配，并计算目标在原图中的精确位置。
///
/// 流程：
/// 1. 对匹配点对执行 RANSAC 单应矩阵估计，统计符合单应变换的内点数量。
/// 2. 当内点数量与内点率均超过阈值时，认为检测有效。
/// 3. 利用单应矩阵将模板的四个角点透视变换到场景坐标系，得到目标边界框。
/// 4. 计算目标中心坐标、宽高和置信度。
use anyhow::{Context, Result};
use log::info;
use opencv::{
    calib3d,
    core::{self, Point, Point2f, Scalar, Vector},
    imgproc,
    prelude::*,
};
use std::path::Path;

use crate::types::{DetectionConfig, DetectionResult, RawMatches};

/// 对单层匹配结果执行 RANSAC 验证并计算目标位置。
///
/// # 参数
/// - `raw`: 单层特征匹配的原始结果
/// - `template_cols/rows`: 模板图像尺寸，用于构造角点和坐标映射
/// - `config`: 检测配置（内点阈值等）
///
/// # 返回
/// - `Some(DetectionResult)` 当验证通过时
/// - `None` 当匹配质量不满足阈值时
pub fn verify(
    raw: &RawMatches,
    template_cols: i32,
    template_rows: i32,
    level_idx: usize,
    _scale: f64,
    method: &str,
    config: &DetectionConfig,
) -> Result<Option<DetectionResult>> {
    // RANSAC 单应矩阵估计。
    // 返回的 mask 标记了哪些点为内点（非零值）。
    let mut mask = Mat::default();
    let homography = calib3d::find_homography(
        &raw.obj_points,
        &raw.scene_points,
        &mut mask,
        calib3d::RANSAC,
        config.ransac_threshold,
    )
    .context("RANSAC 单应矩阵估计失败")?;

    if homography.empty() {
        return Ok(None);
    }

    // 统计 RANSAC 内点数量。
    let inlier_count = count_inliers(&mask).unwrap_or(0);

    // 双阈值验证：内点数量不足或内点率过低均视为无效。
    let inlier_ratio = inlier_count as f64 / raw.match_count.max(1) as f64;
    if inlier_count < config.min_inliers {
        return Ok(None);
    }
    if inlier_ratio < config.min_inlier_ratio {
        return Ok(None);
    }

    // 将模板四个角点透视投影到场景坐标系。
    let mut corners_obj = Vector::<Point2f>::new();
    corners_obj.push(Point2f::new(0.0, 0.0));
    corners_obj.push(Point2f::new(template_cols as f32, 0.0));
    corners_obj.push(Point2f::new(template_cols as f32, template_rows as f32));
    corners_obj.push(Point2f::new(0.0, template_rows as f32));

    let mut corners_scene = Vector::<Point2f>::new();
    core::perspective_transform(&corners_obj, &mut corners_scene, &homography)
        .context("模板角点透视变换失败")?;

    if corners_scene.len() < 4 {
        return Ok(None);
    }

    // 计算包围盒（AABB）。
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for i in 0..corners_scene.len() {
        let p = corners_scene.get(i)?;
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }

    let width = (max_x - min_x).round() as i32;
    let height = (max_y - min_y).round() as i32;

    // 几何合理性校验：过滤由局部特征聚集导致的退化单应矩阵。
    // - 检测框宽高不能过小。
    // - 检测框长宽比应与模板长宽比接近（平面内目标无透视畸变）。
    if width < 8 || height < 8 {
        return Ok(None);
    }
    let template_aspect = template_cols as f64 / template_rows.max(1) as f64;
    let detected_aspect = width as f64 / height.max(1) as f64;
    let aspect_ratio_diff = if template_aspect > detected_aspect {
        template_aspect / detected_aspect
    } else {
        detected_aspect / template_aspect
    };
    if aspect_ratio_diff > 4.0 {
        return Ok(None);
    }

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // 置信度 = 内点率（可在此处扩展更复杂的评分策略）。
    let confidence = inlier_ratio;

    Ok(Some(DetectionResult {
        center_x: center_x as f64,
        center_y: center_y as f64,
        x: min_x.round() as i32,
        y: min_y.round() as i32,
        width,
        height,
        confidence,
        inlier_count,
        best_level: level_idx,
        method: method.to_string(),
    }))
}

/// 统计 RANSAC 输出的内点掩码中非零元素的数量。
///
/// `find_homography` 返回的 mask 为 N×1 的 CV_8U 矩阵，
/// 每行对应一个输入点对，1 表示内点，0 表示外点。
fn count_inliers(mask: &Mat) -> Result<usize> {
    if mask.empty() {
        return Ok(0);
    }

    let mut count = 0usize;
    // mask 可能是 N×1 或 1×N 格式，统一按行优先遍历。
    let rows = mask.rows();
    let cols = mask.cols();
    let total = (rows * cols) as usize;

    for i in 0..total {
        let row = i as i32 / cols;
        let col = i as i32 % cols;
        // 使用 at_2d 安全访问。
        if mask.at_2d::<u8>(row, col).map(|&v| v).unwrap_or(0) != 0 {
            count += 1;
        }
    }

    Ok(count)
}

/// 在彩色屏幕上绘制检测框并保存结果图像。
///
/// - 红色多边形：透视变换后的检测框（非轴对齐）。
/// - 绿色矩形：轴对齐包围盒（AABB），便于前端展示。
pub fn draw_detection_and_save(
    image: &mut Mat,
    result: &DetectionResult,
    output_path: &Path,
) -> Result<()> {
    let poly = build_detection_poly(image, result)?;

    // 红色多边形（精确边框）
    let mut polys = Vector::<Vector<Point>>::new();
    polys.push(poly);
    imgproc::polylines(
        image,
        &polys,
        true,
        Scalar::new(0.0, 0.0, 255.0, 0.0), // BGR 红色
        3,
        imgproc::LINE_AA,
        0,
    )?;

    // 绿色 AABB 矩形框
    imgproc::rectangle(
        image,
        core::Rect::new(result.x, result.y, result.width, result.height),
        Scalar::new(0.0, 255.0, 0.0, 0.0), // BGR 绿色
        2,
        imgproc::LINE_AA,
        0,
    )?;

    opencv::imgcodecs::imwrite(
        &output_path.to_string_lossy().into_owned(),
        image,
        &Vector::<i32>::new(),
    )
    .with_context(|| format!("保存检测结果图像失败: {}", output_path.display()))?;

    let iw = image.cols().max(1) as f32;
    let ih = image.rows().max(1) as f32;

    info!(
        "目标匹配成功({}): 左上=({}, {}), 右下=({}, {}), 尺寸={}x{}, 相对坐标=({:.4}, {:.4}) -> ({:.4}, {:.4}), 置信度={:.3}, 输出={}",
        result.method,
        result.x,
        result.y,
        result.x + result.width,
        result.y + result.height,
        result.width,
        result.height,
        result.x as f32 / iw,
        result.y as f32 / ih,
        (result.x + result.width) as f32 / iw,
        (result.y + result.height) as f32 / ih,
        result.confidence,
        output_path.display()
    );

    Ok(())
}

/// 构建检测结果对应的四边形顶点（用于绘制）。
fn build_detection_poly(image: &Mat, result: &DetectionResult) -> Result<Vector<Point>> {
    let mut poly = Vector::<Point>::new();
    // 以 AABB 的四个角点近似（若需要精确透视框，可在 verify 阶段保存 corners_scene）。
    let x2 = (result.x + result.width).min(image.cols() - 1);
    let y2 = (result.y + result.height).min(image.rows() - 1);
    poly.push(Point::new(result.x, result.y));
    poly.push(Point::new(x2, result.y));
    poly.push(Point::new(x2, y2));
    poly.push(Point::new(result.x, y2));
    Ok(poly)
}

/// 模板匹配回退：当 ORB 匹配失败时，使用归一化相关系数模板匹配作为兜底方案。
///
/// 模板匹配对尺度和旋转敏感，但计算简单、对平面 UI 元素可靠性高。
pub fn detect_with_template_matching(
    template_gray: &Mat,
    scene_gray: &Mat,
    threshold: f64,
    level_idx: usize,
    _scale: f64,
) -> Result<Option<DetectionResult>> {
    let result_cols = scene_gray.cols() - template_gray.cols() + 1;
    let result_rows = scene_gray.rows() - template_gray.rows() + 1;

    if result_cols <= 0 || result_rows <= 0 {
        return Ok(None);
    }

    let mut result = Mat::zeros(result_rows, result_cols, core::CV_32FC1)?.to_mat()?;
    imgproc::match_template(
        scene_gray,
        template_gray,
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

    if max_val < threshold {
        return Ok(None);
    }

    let x = max_loc.x;
    let y = max_loc.y;
    let width = template_gray.cols();
    let height = template_gray.rows();

    Ok(Some(DetectionResult {
        center_x: (x + width / 2) as f64,
        center_y: (y + height / 2) as f64,
        x,
        y,
        width,
        height,
        confidence: max_val,
        inlier_count: 0,
        best_level: level_idx,
        method: "TemplateFallback".to_string(),
    }))
}
