/// 特征检测与匹配模块 —— 整个检测管线的核心。
///
/// 完成三项工作：
/// 1. 离线阶段：对模板图像预提取 ORB 特征，供每帧复用。
/// 2. 构建图像金字塔：从处理尺寸的灰度图出发，逐层高斯下采样，模拟 mstpl 的尺度搜索能力。
/// 3. 逐层匹配：在金字塔每一层上提取 ORB 特征，使用汉明距离暴力匹配 + Lowe's ratio test 筛选可靠匹配对。
///
/// ORB 对旋转具备天然不变性，配合金字塔可补足其尺度不变性的不足。
use anyhow::{Context, Result};
use opencv::{
    core::{self, Vector},
    features2d, imgproc,
    prelude::*,
};
use std::path::Path;

use crate::types::{DetectionConfig, OrbTemplate, RawMatches};

/// 创建 ORB 特征检测器。
///
/// 参数含义：
/// - `nfeatures`: 期望提取的最大特征点数
/// - `scale_factor` (1.2): ORB 内部金字塔层间缩放因子
/// - `nlevels` (8): ORB 内部金字塔层数
/// - `patch_size` (31): 特征描述子的 patch 大小
/// - `HARRIS_SCORE`: 使用 Harris 角点评分策略，质量优于 FAST 评分
pub fn create_orb(config: &DetectionConfig) -> Result<opencv::core::Ptr<features2d::ORB>> {
    features2d::ORB::create(
        config.max_features,
        1.2,
        8,
        31,
        0,
        2,
        features2d::ORB_ScoreType::HARRIS_SCORE,
        31,
        20,
    )
    .map_err(|e| anyhow::anyhow!("创建 ORB 检测器失败: {e}"))
}

/// 离线阶段：从文件加载模板图像并预提取 ORB 特征。
///
/// 该函数仅在初始化时调用一次，提取的特征存储在 `OrbTemplate` 中供后续每帧复用。
pub fn prepare_template(
    path: &Path,
    orb: &mut opencv::core::Ptr<features2d::ORB>,
) -> Result<OrbTemplate> {
    let mat = opencv::imgcodecs::imread(
        &path.to_string_lossy().into_owned(),
        opencv::imgcodecs::IMREAD_GRAYSCALE,
    )
    .with_context(|| format!("读取模板图像失败: {}", path.display()))?;

    if mat.empty() {
        return Err(anyhow::anyhow!("模板图像为空: {}", path.display()));
    }

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    extract_orb_template(mat, name, orb)
}

/// 从灰度 Mat 提取 ORB 特征构造 OrbTemplate（用于缩放后的模板）。
fn extract_orb_template(
    mat: Mat,
    name: String,
    orb: &mut opencv::core::Ptr<features2d::ORB>,
) -> Result<OrbTemplate> {
    let mut keypoints = Vector::<core::KeyPoint>::new();
    let mut descriptors = Mat::default();

    orb.detect_and_compute(&mat, &Mat::default(), &mut keypoints, &mut descriptors, false)
        .with_context(|| format!("提取模板 ORB 特征失败: {name}"))?;

    if descriptors.empty() {
        return Err(anyhow::anyhow!("模板未提取到可用特征点: {name}"));
    }

    Ok(OrbTemplate {
        name,
        keypoints,
        descriptors,
        cols: mat.cols(),
        rows: mat.rows(),
    })
}

/// 将模板灰度图和 ORB 特征等比缩放到目标分辨率，使模板与处理后的场景图像尺度一致。
///
/// 返回 (缩放后的灰度图, 缩放后的 OrbTemplate)。
pub fn scale_template(
    template_gray: &Mat,
    orb_template: &OrbTemplate,
    dst_width: u32,
    dst_height: u32,
    orb: &mut opencv::core::Ptr<features2d::ORB>,
) -> Result<(Mat, OrbTemplate)> {
    let mut scaled_gray = Mat::default();
    imgproc::resize(
        template_gray,
        &mut scaled_gray,
        core::Size::new(dst_width as i32, dst_height as i32),
        0.0,
        0.0,
        imgproc::INTER_LANCZOS4,
    )
    .context("缩放模板图像失败")?;

    let scaled_orb = extract_orb_template(
        scaled_gray.clone(),
        format!("{}_scaled", orb_template.name),
        orb,
    )?;

    Ok((scaled_gray, scaled_orb))
}

/// 构建图像金字塔。
///
/// 以处理尺寸的灰度图为基底，逐层按 `config.pyramid_scale` 缩放，生成 `config.pyramid_levels` 层。
/// - 第 0 层：原始处理尺寸（scale = 1.0）
/// - 第 1 层：缩小为 0.8×
/// - 第 2 层：缩小为 0.64×
/// - 以此类推
///
/// 使用 `INTER_AREA` 插值进行降采样，对缩小图像质量最佳。
pub fn build_pyramid(base_gray: &Mat, config: &DetectionConfig) -> Result<Vec<(usize, f64, Mat)>> {
    let base_w = base_gray.cols();
    let base_h = base_gray.rows();
    let mut pyramid = Vec::with_capacity(config.pyramid_levels);

    for level in 0..config.pyramid_levels {
        let scale = config.pyramid_scale.powi(level as i32);
        if level == 0 {
            // 第 0 层直接使用原图，避免不必要的拷贝。
            pyramid.push((0, 1.0, base_gray.clone()));
        } else {
            let new_w = ((base_w as f64) * scale).round() as i32;
            let new_h = ((base_h as f64) * scale).round() as i32;

            // 图像过小（任一维度 < 10 像素）时终止金字塔构建。
            if new_w < 10 || new_h < 10 {
                break;
            }

            let mut level_mat = Mat::default();
            imgproc::resize(
                base_gray,
                &mut level_mat,
                core::Size::new(new_w, new_h),
                0.0,
                0.0,
                imgproc::INTER_AREA,
            )
            .context("构建金字塔层失败")?;

            pyramid.push((level, scale, level_mat));
        }
    }

    Ok(pyramid)
}

/// 在单个金字塔层级上执行 ORB 特征匹配。
///
/// 流程：
/// 1. 对该层图像提取 ORB 关键点与描述子。
/// 2. 使用 BFMatcher（汉明距离）做 KNN 匹配（k=2）。
/// 3. 应用 Lowe's ratio test（阈值 0.75）过滤误匹配。
/// 4. 若有效匹配数 >= 4（单应矩阵最少需求），返回匹配点对；否则返回 None。
pub fn match_at_level(
    orb: &mut opencv::core::Ptr<features2d::ORB>,
    template: &OrbTemplate,
    level_image: &Mat,
    level_idx: usize,
    scale: f64,
) -> Result<Option<RawMatches>> {
    // 提取场景层 ORB 特征。
    let mut scene_keypoints = Vector::<core::KeyPoint>::new();
    let mut scene_descriptors = Mat::default();

    orb.detect_and_compute(
        level_image,
        &Mat::default(),
        &mut scene_keypoints,
        &mut scene_descriptors,
        false,
    )
    .context("提取场景层 ORB 特征失败")?;

    let scene_kp_count = scene_keypoints.len();
    if scene_descriptors.empty() {
        return Ok(None);
    }

    // 暴力匹配：汉明距离 + KNN (k=2)。
    let matcher = features2d::BFMatcher::new(core::NORM_HAMMING, false)?;
    let mut knn_matches = Vector::<Vector<core::DMatch>>::new();
    matcher.knn_train_match_def(
        &template.descriptors,
        &scene_descriptors,
        &mut knn_matches,
        2,
    )?;

    // Lowe's ratio test：第一近邻必须显著优于第二近邻。
    let mut good_matches = Vector::<core::DMatch>::new();
    for i in 0..knn_matches.len() {
        let mset = knn_matches.get(i)?;
        if mset.len() < 2 {
            continue;
        }
        let m1 = mset.get(0)?;
        let m2 = mset.get(1)?;
        if m1.distance < 0.80 * m2.distance {
            good_matches.push(m1);
        }
    }

    // 单应矩阵估计至少需要 4 个点对。
    if good_matches.len() < 4 {
        return Ok(None);
    }

    // 将匹配索引转换为实际坐标点对。
    let mut obj_points = Vector::<core::Point2f>::new();
    let mut scene_points = Vector::<core::Point2f>::new();

    for i in 0..good_matches.len() {
        let m = good_matches.get(i)?;
        let kp_obj = template.keypoints.get(m.query_idx as usize)?;
        let kp_scene = scene_keypoints.get(m.train_idx as usize)?;
        obj_points.push(kp_obj.pt());
        scene_points.push(kp_scene.pt());
    }

    let match_count = good_matches.len();

    Ok(Some(RawMatches {
        level: level_idx,
        scale,
        obj_points,
        scene_points,
        scene_kp_count,
        match_count,
    }))
}
