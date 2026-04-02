use anyhow::{anyhow, Context, Result};
use opencv::{
    calib3d,
    core::{self, Point, Point2f, Scalar, Vector},
    features2d, imgcodecs, imgproc,
    prelude::*,
};
use std::path::{Path, PathBuf};

/// 对外入口：执行目标识别流程。
///
/// 设计上不向上抛出错误，避免测试或调用方因为识别失败而 panic。
/// 识别失败时仅打印错误，便于在日志中排查原因。
pub fn take_screenshot() {
    if let Err(err) = detect_target_with_orb() {
        eprintln!("[detective] ORB 识别失败: {err:#}");
    }
}

/// 使用 ORB 进行目标定位；当 ORB 结果不稳定时自动回退到模板匹配。
///
/// 输入：
/// - static/target.png：待识别的小图。
/// - static/screenshot.png：大图（截图）。
///
/// 输出：
/// - static/screenshot_marked.png：在截图上用红框圈出目标区域。
/// - stdout：打印绝对坐标和相对坐标（归一化到 0~1）。
fn detect_target_with_orb() -> Result<()> {
    // 兼容不同执行目录（直接跑 crate、在 workspace 根目录跑等）下的 static 路径。
    let static_dir = locate_static_dir().context("无法定位 static 目录")?;
    let target_path = static_dir.join("target.png");
    let screenshot_path = static_dir.join("screenshot.png");
    let output_path = static_dir.join("screenshot_marked.png");

    // ORB 特征提取与匹配使用灰度图；绘制红框需要彩色图。
    let target = imgcodecs::imread(&path_to_string(&target_path), imgcodecs::IMREAD_GRAYSCALE)
        .context("读取 target.png 失败")?;
    let screenshot_gray = imgcodecs::imread(
        &path_to_string(&screenshot_path),
        imgcodecs::IMREAD_GRAYSCALE,
    )
    .context("读取 screenshot.png 失败")?;
    let mut screenshot_color =
        imgcodecs::imread(&path_to_string(&screenshot_path), imgcodecs::IMREAD_COLOR)
            .context("读取 screenshot 彩色图失败")?;

    if target.empty() || screenshot_gray.empty() || screenshot_color.empty() {
        return Err(anyhow!("输入图片为空"));
    }

    // ORB 参数说明（常用）：
    // - 1500: 期望提取特征点上限；数值越高越可能找到更多匹配，但也更耗时。
    // - 1.2, 8: 金字塔缩放与层数。
    // - 31: patch 大小。
    // - HARRIS_SCORE: 角点评分策略。
    let mut orb = features2d::ORB::create(
        1500,
        1.2,
        8,
        31,
        0,
        2,
        features2d::ORB_ScoreType::HARRIS_SCORE,
        31,
        20,
    )?;

    let mut keypoints_target = Vector::<core::KeyPoint>::new();
    let mut keypoints_scene = Vector::<core::KeyPoint>::new();
    let mut descriptors_target = Mat::default();
    let mut descriptors_scene = Mat::default();

    // 对目标图与场景图分别提取关键点和描述子。
    orb.detect_and_compute(
        &target,
        &Mat::default(),
        &mut keypoints_target,
        &mut descriptors_target,
        false,
    )?;
    orb.detect_and_compute(
        &screenshot_gray,
        &Mat::default(),
        &mut keypoints_scene,
        &mut descriptors_scene,
        false,
    )?;

    if descriptors_target.empty() || descriptors_scene.empty() {
        return Err(anyhow!("未提取到可用特征点"));
    }

    // 使用汉明距离进行暴力匹配（ORB 描述子为二进制，适合 NORM_HAMMING）。
    // 每个 query 点保留 2 个最近邻，后续做 ratio test 去除误匹配。
    let matcher = features2d::BFMatcher::new(core::NORM_HAMMING, false)?;
    let mut knn_matches = Vector::<Vector<core::DMatch>>::new();
    matcher.knn_train_match_def(&descriptors_target, &descriptors_scene, &mut knn_matches, 2)?;

    let mut good_matches = Vector::<core::DMatch>::new();
    for i in 0..knn_matches.len() {
        let mset = knn_matches.get(i)?;
        if mset.len() < 2 {
            continue;
        }
        let m1 = mset.get(0)?;
        let m2 = mset.get(1)?;
        // Lowe ratio test：第一近邻明显优于第二近邻，匹配才更可靠。
        if m1.distance < 0.75 * m2.distance {
            good_matches.push(m1);
        }
    }

    // ORB 匹配过少时，单应矩阵容易退化，因此切换到模板匹配兜底。
    if good_matches.len() < 8 {
        return detect_with_template_matching(
            &target,
            &screenshot_gray,
            &mut screenshot_color,
            &output_path,
        )
        .with_context(|| format!("ORB 有效匹配点不足(good_matches={})", good_matches.len()));
    }

    let mut obj_points = Vector::<Point2f>::new();
    let mut scene_points = Vector::<Point2f>::new();

    // 将 good matches 里的索引转换成实际的二维点坐标对。
    for i in 0..good_matches.len() {
        let m = good_matches.get(i)?;
        let kp_obj = keypoints_target.get(m.query_idx as usize)?;
        let kp_scene = keypoints_scene.get(m.train_idx as usize)?;
        obj_points.push(kp_obj.pt());
        scene_points.push(kp_scene.pt());
    }

    // 通过 RANSAC 估计单应矩阵，过滤离群点。
    // 该矩阵将 target 坐标系映射到 screenshot 坐标系。
    let homography = calib3d::find_homography(
        &obj_points,
        &scene_points,
        &mut Mat::default(),
        calib3d::RANSAC,
        3.0,
    )?;

    if homography.empty() {
        return Err(anyhow!("单应矩阵求解失败"));
    }

    // 构造 target 四个角点，并透视变换到 screenshot 上得到检测框多边形。
    let tw = target.cols();
    let th = target.rows();
    let mut corners_obj = Vector::<Point2f>::new();
    corners_obj.push(Point2f::new(0.0, 0.0));
    corners_obj.push(Point2f::new(tw as f32, 0.0));
    corners_obj.push(Point2f::new(tw as f32, th as f32));
    corners_obj.push(Point2f::new(0.0, th as f32));

    let mut corners_scene = Vector::<Point2f>::new();
    core::perspective_transform(&corners_obj, &mut corners_scene, &homography)?;

    if corners_scene.len() != 4 {
        return Err(anyhow!("投影角点数量异常"));
    }

    // 同时计算包围盒坐标（便于打印左上/右下）和多边形顶点（用于绘制）。
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    let mut poly = Vector::<Point>::new();
    for i in 0..corners_scene.len() {
        let p = corners_scene.get(i)?;
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
        poly.push(Point::new(p.x.round() as i32, p.y.round() as i32));
    }

    // 绘制红框并输出坐标日志。
    draw_polygon_and_log(
        &mut screenshot_color,
        poly,
        min_x,
        min_y,
        max_x,
        max_y,
        &output_path,
        "ORB",
    )?;

    Ok(())
}

fn detect_with_template_matching(
    target: &Mat,
    screenshot_gray: &Mat,
    screenshot_color: &mut Mat,
    output_path: &Path,
) -> Result<()> {
    // OpenCV 模板匹配输出图尺寸：
    // (W - w + 1, H - h + 1)，其中 W/H 为大图尺寸，w/h 为模板尺寸。
    let result_cols = screenshot_gray.cols() - target.cols() + 1;
    let result_rows = screenshot_gray.rows() - target.rows() + 1;
    if result_cols <= 0 || result_rows <= 0 {
        return Err(anyhow!("模板尺寸大于截图尺寸"));
    }

    // TM_CCOEFF_NORMED 输出范围约 [-1, 1]，越接近 1 代表越匹配。
    let mut result = Mat::zeros(result_rows, result_cols, core::CV_32FC1)?.to_mat()?;
    imgproc::match_template(
        screenshot_gray,
        target,
        &mut result,
        imgproc::TM_CCOEFF_NORMED,
        &Mat::default(),
    )?;

    // 提取匹配响应图中的最优位置。
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

    // 经验阈值：置信度过低时认为结果不可信，返回错误。
    if max_val < 0.60 {
        return Err(anyhow!("模板匹配置信度过低: {:.3}", max_val));
    }

    // 模板匹配返回的是最佳左上角坐标；右下角由模板宽高推导。
    let tl = max_loc;
    let br = Point::new(max_loc.x + target.cols(), max_loc.y + target.rows());

    let mut poly = Vector::<Point>::new();
    poly.push(Point::new(tl.x, tl.y));
    poly.push(Point::new(br.x, tl.y));
    poly.push(Point::new(br.x, br.y));
    poly.push(Point::new(tl.x, br.y));

    draw_polygon_and_log(
        screenshot_color,
        poly,
        tl.x as f32,
        tl.y as f32,
        br.x as f32,
        br.y as f32,
        output_path,
        "TemplateFallback",
    )
}

fn draw_polygon_and_log(
    screenshot_color: &mut Mat,
    poly: Vector<Point>,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    output_path: &Path,
    method: &str,
) -> Result<()> {
    // 统一绘制红色多边形边框（BGR: 0,0,255）。
    let mut polys = Vector::<Vector<Point>>::new();
    polys.push(poly);
    imgproc::polylines(
        screenshot_color,
        &polys,
        true,
        Scalar::new(0.0, 0.0, 255.0, 0.0),
        3,
        imgproc::LINE_AA,
        0,
    )?;

    // 输出标注图到磁盘，便于人工核对定位结果。
    imgcodecs::imwrite(
        &path_to_string(output_path),
        screenshot_color,
        &Vector::<i32>::new(),
    )?;

    // 相对坐标 = 像素坐标 / 图像宽高，用于分辨率无关的前端展示。
    let sw = screenshot_color.cols().max(1) as f32;
    let sh = screenshot_color.rows().max(1) as f32;
    println!(
        "target 匹配成功({}): 左上=({:.1}, {:.1}), 右下=({:.1}, {:.1}), 相对坐标=({:.4}, {:.4}) -> ({:.4}, {:.4}), 输出={}",
        method,
        min_x,
        min_y,
        max_x,
        max_y,
        min_x / sw,
        min_y / sh,
        max_x / sw,
        max_y / sh,
        output_path.display()
    );

    Ok(())
}

/// 在常见工作目录下定位 static 目录。
///
/// 之所以做多候选路径，是为了兼容：
/// - 在 crate 目录运行测试
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
        if path.join("target.png").exists() && path.join("screenshot.png").exists() {
            return Ok(path);
        }
    }

    Err(anyhow!(
        "未找到包含 target.png 与 screenshot.png 的 static 目录"
    ))
}

/// 将路径转为 OpenCV 接口需要的字符串。
/// 使用 lossy 转换可避免非 UTF-8 路径导致的直接失败。
fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
