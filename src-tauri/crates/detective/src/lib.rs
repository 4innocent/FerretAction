// 核心检测管线模块（对应屏幕检测方案文档的 4 个阶段）
pub mod feature_matcher; // 阶段 3: ORB 特征提取 + 金字塔匹配
pub mod geometric_verifier; // 阶段 4: RANSAC 几何验证与定位
pub mod image_scaler; // 阶段 2: 图像缩放
pub mod pipeline;
pub mod screen_capture; // 阶段 1: 屏幕捕获
pub mod types; // 共享数据类型 // 管线编排

// 对外 API
pub mod real_time_detective; // 实时检测循环
pub mod screen_shot; // 单次截图识别
