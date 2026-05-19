/// 屏幕捕获模块 —— 基于 `dxgi-capture-rs` 库实现，直接调用 Windows DXGI Desktop Duplication API。
///
/// 以极低开销获取当前桌面画面的完整像素缓冲，输出为 **BGRA 格式** 的连续内存块。
/// 捕获延迟通常在 1~3 毫秒，符合文档中 20ms 级低延迟方案的要求。
///
/// `ScreenCapture` 结构体持久化持有 DXGI 管理器，避免每次捕获重建导致的首帧黑屏问题。
use anyhow::{anyhow, Context, Result};
use dxgi_capture_rs::{FrameMetadata, DXGIManager};
use opencv::{core::AlgorithmHint, imgproc, prelude::*};

/// 持久化的屏幕捕获器。
///
/// 创建后复用同一个 DXGI Desktop Duplication 实例，保证每帧都能获取有效画面。
pub struct ScreenCapture {
    manager: DXGIManager,
    width: u32,
    height: u32,
}

impl ScreenCapture {
    /// 创建屏幕捕获器并预热（丢弃首帧避免黑屏）。
    pub fn new() -> Result<Self> {
        let mut manager = DXGIManager::new(1000).context("创建 DXGI 捕获管理器失败")?;
        let (w, h) = manager.geometry();

        // 预热：Desktop Duplication 首帧可能为黑屏，先取一帧丢弃。
        let _ = manager.capture_frame();

        Ok(Self {
            manager,
            width: w as u32,
            height: h as u32,
        })
    }

    /// 返回屏幕几何尺寸。
    pub fn geometry(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// 捕获一帧，返回 (宽度, 高度, BGRA 字节缓冲)。
    pub fn capture(&mut self) -> Result<(u32, u32, Vec<u8>)> {
        let (buf, dims) = self
            .manager
            .capture_frame_components()
            .context("捕获桌面画面失败")?;

        let (frame_w, frame_h) = dims;
        if frame_w as u32 != self.width || frame_h as u32 != self.height {
            return Err(anyhow!(
                "捕获画面尺寸与几何信息不一致: geometry={}x{}, frame={}x{}",
                self.width,
                self.height,
                frame_w,
                frame_h
            ));
        }

        Ok((frame_w as u32, frame_h as u32, buf))
    }

    /// 捕获一帧并附带帧元数据（脏矩形、移动矩形等）。
    ///
    /// 相比 [`capture`] 额外返回 [`FrameMetadata`]，
    /// 调用方可利用 `metadata.has_updates()` 跳过无变化的帧。
    pub fn capture_with_metadata(
        &mut self,
    ) -> Result<(u32, u32, Vec<u8>, FrameMetadata)> {
        let (buf, dims, metadata) = self
            .manager
            .capture_frame_components_with_metadata()
            .context("捕获桌面画面（含元数据）失败")?;

        let (frame_w, frame_h) = dims;
        if frame_w as u32 != self.width || frame_h as u32 != self.height {
            return Err(anyhow!(
                "捕获画面尺寸与几何信息不一致: geometry={}x{}, frame={}x{}",
                self.width,
                self.height,
                frame_w,
                frame_h
            ));
        }

        Ok((frame_w as u32, frame_h as u32, buf, metadata))
    }

    /// 捕获一帧并转为 OpenCV BGR Mat（用于绘制标注）。
    pub fn capture_to_bgr_mat(&mut self) -> Result<Mat> {
        let (_w, h, buf) = self.capture()?;

        let mat_1d = Mat::from_slice(&buf).context("将 BGRA 缓冲转为 Mat 失败")?;
        let bgra_mat = mat_1d
            .reshape(4, h as i32)
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
}

/// 便捷函数：创建捕获器并立即返回一帧。
///
/// 用于简单的一次性调用（内部仍会预热）。
pub fn capture_primary() -> Result<(u32, u32, Vec<u8>)> {
    let mut cap = ScreenCapture::new()?;
    cap.capture()
}

/// 便捷函数：创建捕获器并返回 BGR Mat。
pub fn capture_to_bgr_mat() -> Result<Mat> {
    let mut cap = ScreenCapture::new()?;
    cap.capture_to_bgr_mat()
}

