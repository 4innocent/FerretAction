/// 图像缩放模块 —— 将高分辨率屏幕画面压缩至检测尺寸，降低后续 ORB 特征提取的计算负担。
///
/// 使用 OpenCV 的 Lanczos4 重采样算法（等效于 `image` crate 的 Lanczos3），
/// 在保持边缘清晰度的同时实现高质量降采样。缩放后的图像直接转为灰度 Mat，
/// 供特征匹配模块使用，全程无额外内存搬移。
use anyhow::{Context, Result};
use opencv::{
    core::{self, AlgorithmHint},
    imgproc,
    prelude::*,
};

/// 将 BGRA 缓冲缩放到处理尺寸，并转为灰度 OpenCV Mat。
///
/// # 参数
/// - `buffer`: BGRA 像素缓冲（来自 DXGI 屏幕捕获）
/// - `src_width`, `src_height`: 原始画面宽高
/// - `dst_width`, `dst_height`: 目标处理尺寸（如 640×480）
///
/// # 返回
/// - 单通道灰度 Mat，尺寸为 `dst_width × dst_height`
pub fn scale_to_gray(
    buffer: &[u8],
    _src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> Result<Mat> {
    // BGRA 缓冲 → OpenCV Mat
    let mat_1d = Mat::from_slice(buffer).context("将 BGRA 缓冲转为 Mat 失败")?;
    let bgra_mat = mat_1d
        .reshape(4, src_height as i32)
        .context("重塑 BGRA Mat 形状失败")?;

    // 缩放：使用 INTER_LANCZOS4（等效于 image crate 的 Lanczos3），边缘保持好。
    let mut scaled_bgra = Mat::default();
    imgproc::resize(
        &bgra_mat,
        &mut scaled_bgra,
        core::Size::new(dst_width as i32, dst_height as i32),
        0.0,
        0.0,
        imgproc::INTER_LANCZOS4,
    )
    .context("缩放图像失败")?;

    // BGRA → 灰度
    let mut gray = Mat::default();
    imgproc::cvt_color(
        &scaled_bgra,
        &mut gray,
        imgproc::COLOR_BGRA2GRAY,
        0,
        AlgorithmHint::ALGO_HINT_DEFAULT,
    )
    .context("缩放图像转灰度失败")?;

    Ok(gray)
}

/// 将 OpenCV BGR Mat 缩放到处理尺寸并转为灰度。
///
/// 适用于已经通过 OpenCV 获取的图像（如实时检测中的帧）。
pub fn scale_bgr_to_gray(bgr: &Mat, dst_width: i32, dst_height: i32) -> Result<Mat> {
    let mut gray = Mat::default();
    imgproc::cvt_color(
        bgr,
        &mut gray,
        imgproc::COLOR_BGR2GRAY,
        0,
        AlgorithmHint::ALGO_HINT_DEFAULT,
    )?;

    let mut scaled = Mat::default();
    imgproc::resize(
        &gray,
        &mut scaled,
        core::Size::new(dst_width, dst_height),
        0.0,
        0.0,
        imgproc::INTER_AREA,
    )?;

    Ok(scaled)
}
