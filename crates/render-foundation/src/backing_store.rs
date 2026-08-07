//! Backing store 管理（#3 渲染线程化 RFC 的 C1：合成执行层显式化）。
//!
//! 双缓冲帧缓冲：渲染线程写 back buffer → swap → 显示线程读 front buffer。
//! 对应 Ladybird 的 BackingStoreManager（调研报告 §3.4）。

use crate::surface::FrameBuffer;

/// 双缓冲帧缓冲管理器。
#[derive(Debug)]
pub struct BackingStoreManager {
    front: FrameBuffer,
    back: FrameBuffer,
}

impl BackingStoreManager {
    /// 创建指定尺寸的双缓冲。
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            front: FrameBuffer::new(width, height),
            back: FrameBuffer::new(width, height),
        }
    }

    /// 视图尺寸变更时重建（渲染线程持有 back 期间调用方须同步）。
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.front.width != width || self.front.height != height {
            self.front = FrameBuffer::new(width, height);
            self.back = FrameBuffer::new(width, height);
        }
    }

    /// 当前可写的 back buffer（渲染线程独占写入）。
    pub fn back_mut(&mut self) -> &mut FrameBuffer {
        &mut self.back
    }

    /// 当前只读的 front buffer（显示线程读取）。
    pub fn front(&self) -> &FrameBuffer {
        &self.front
    }

    /// 交换前后缓冲（渲染完成提交时调用）。
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }

    /// 尺寸。
    pub fn size(&self) -> (u32, u32) {
        (self.front.width, self.front.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_roundtrip_preserves_frames() {
        let mut bs = BackingStoreManager::new(2, 2);
        // 写 back → swap → front 可见
        bs.back_mut().data.fill(0xAA);
        bs.swap();
        assert_eq!(bs.front().data[0], 0xAA);
        // swap 后再写新 back（原 front）不影响已提交的 front
        bs.back_mut().data.fill(0xBB);
        assert_eq!(bs.front().data[0], 0xAA, "swap 前 front 不应被 back 写入污染");
        bs.swap();
        assert_eq!(bs.front().data[0], 0xBB);
    }

    #[test]
    fn resize_rebuilds_buffers() {
        let mut bs = BackingStoreManager::new(2, 2);
        bs.resize(4, 4);
        assert_eq!(bs.size(), (4, 4));
        assert_eq!(bs.front().data.len(), 4 * 4 * 4);
        bs.resize(4, 4); // 同尺寸不重建
        assert_eq!(bs.size(), (4, 4));
    }
}
