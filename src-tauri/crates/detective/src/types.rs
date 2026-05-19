use opencv::{core::Vector, prelude::*};
use serde::Serialize;

/// 检测管线配置。
///
/// 各项参数直接影响延迟与鲁棒性：
/// - 降低 `process_width/height`、减少 `pyramid_levels`、减小 `max_features` 可降低延迟。
/// - 增大 `max_features`、放宽 `ransac_threshold` 可提高遮挡/旋转场景下的鲁棒性。
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// 处理图像宽度（全屏截图缩放后的目标宽）
    pub process_width: u32,
    /// 处理图像高度（全屏截图缩放后的目标高）
    pub process_height: u32,
    /// 图像金字塔层数（含原始尺寸层）
    pub pyramid_levels: usize,
    /// 金字塔相邻层缩放因子（< 1.0，如 0.8 表示每层缩小为上一层的 80%）
    pub pyramid_scale: f64,
    /// ORB 最大特征点数量
    pub max_features: i32,
    /// RANSAC 重投影误差阈值（像素）
    pub ransac_threshold: f64,
    /// 最小内点数量（低于此值视为匹配无效）
    pub min_inliers: usize,
    /// 最小内点比率（内点数 / 总匹配数，低于此值视为匹配无效）
    pub min_inlier_ratio: f64,
    /// 模板匹配回退阈值（当 ORB + RANSAC 失败时，回退到模板匹配的最小置信度）
    pub fallback_match_threshold: f64,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            process_width: 0,
            process_height: 0,
            pyramid_levels: 3,
            pyramid_scale: 0.8,
            max_features: 2000,
            ransac_threshold: 5.0,
            min_inliers: 10,
            min_inlier_ratio: 0.15,
            fallback_match_threshold: 0.60,
        }
    }
}

/// 单帧检测结果，可直接序列化传递给前端。
#[derive(Debug, Clone, Serialize)]
pub struct DetectionResult {
    /// 目标中心点 x（原始屏幕坐标系）
    pub center_x: f64,
    /// 目标中心点 y（原始屏幕坐标系）
    pub center_y: f64,
    /// 边界框左上角 x
    pub x: i32,
    /// 边界框左上角 y
    pub y: i32,
    /// 边界框宽度
    pub width: i32,
    /// 边界框高度
    pub height: i32,
    /// 匹配置信度（0.0 ~ 1.0）
    pub confidence: f64,
    /// RANSAC 内点数量
    pub inlier_count: usize,
    /// 最佳匹配所在金字塔层级
    pub best_level: usize,
    /// 检测方法标识（"ORB" 或 "TemplateFallback"）
    pub method: String,
}

/// 实时检测中单次命中的结构化结果。
#[derive(Debug, Clone, Serialize)]
pub struct DetectionBox {
    /// 命中的模板文件名
    pub target_name: String,
    /// 匹配置信度
    pub confidence: f32,
    /// 命中框左上角 x（像素）
    pub x: i32,
    /// 命中框左上角 y（像素）
    pub y: i32,
    /// 命中框宽度（像素）
    pub width: i32,
    /// 命中框高度（像素）
    pub height: i32,
}

/// 预提取的 ORB 模板特征（离线阶段产物）。
///
/// 在实时循环中直接复用，避免每帧重复提取模板特征。
pub struct OrbTemplate {
    /// 模板名称（通常是文件名）
    pub name: String,
    /// 模板关键点
    pub keypoints: Vector<opencv::core::KeyPoint>,
    /// 模板 ORB 描述子
    pub descriptors: Mat,
    /// 模板原始宽度
    pub cols: i32,
    /// 模板原始高度
    pub rows: i32,
}

/// 单层特征匹配的原始结果，供几何验证模块使用。
pub struct RawMatches {
    /// 金字塔层级索引
    pub level: usize,
    /// 该层相对于处理尺寸的缩放因子
    pub scale: f64,
    /// 模板侧匹配点坐标
    pub obj_points: Vector<opencv::core::Point2f>,
    /// 场景侧匹配点坐标（在金字塔层坐标系内）
    pub scene_points: Vector<opencv::core::Point2f>,
    /// 场景侧在该层提取到的 ORB 关键点数量
    pub scene_kp_count: usize,
    /// 通过 ratio test 的有效匹配数
    pub match_count: usize,
}
