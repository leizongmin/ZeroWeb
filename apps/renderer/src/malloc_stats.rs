//! T79 残留归因：glibc 分配器滞留面劈分（env 门测量读数，非行为路径）。
//!
//! [`mallinfo2_snapshot`] 返回进程级四元读数：arena/hblkhd（分配器持有的保留
//! 面）与 uordblks/fordblks（arena 内在用/空闲）。用途 = 把 PSS 残留（PSS − 域
//! 读数）劈分为「分配器空闲滞留页」（trim 可让，部分占用页除外）与「真实未计入
//! 活对象」两个互斥归因桶；`uordblks + hblkhd` 与域读数共面（域读数的底层分配
//! 就在其中），聚合侧不得与域读数相加解读。mallinfo2 仅 linux-gnu glibc 提供
//! （32 位 mallinfo 计数器在数 GB 进程上溢出，不可用）；其余目标恒 `None`
//! （分析侧如实记零）。

/// 一次分配器快照（字节；`hblkhd` 为 mmap 大块持有面，不含在 arena）。
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MallocSnapshot {
    pub(crate) arena_bytes: u64,
    pub(crate) hblkhd_bytes: u64,
    pub(crate) in_use_bytes: u64,
    pub(crate) free_bytes: u64,
}

/// glibc `mallinfo2(3)` 快照；非 linux-gnu 目标恒 `None`。
pub(crate) fn mallinfo2_snapshot() -> Option<MallocSnapshot> {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        // SAFETY: `mallinfo2` 无参数、读全局分配器簿记并持内部锁；多线程下
        // glibc 自串行化，返回值纯数据无指针语义。
        let info = unsafe { libc::mallinfo2() };
        Some(MallocSnapshot {
            arena_bytes: info.arena as u64,
            hblkhd_bytes: info.hblkhd as u64,
            in_use_bytes: info.uordblks as u64,
            free_bytes: info.fordblks as u64,
        })
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// glibc 下快照可用且在用面非零（进程本身就有活分配）；非 glibc 恒 None。
    #[test]
    fn snapshot_reports_nonzero_in_use_on_glibc() {
        let snap = mallinfo2_snapshot();
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        {
            let snap = snap.expect("mallinfo2 available on linux-gnu");
            assert!(snap.in_use_bytes > 0, "live process always has in-use bytes");
            assert!(snap.arena_bytes > 0, "arena space nonzero once allocation happened");
            // 在用面不得超过 arena + mmap 持有面（同源簿记的包含关系）。
            assert!(
                snap.in_use_bytes <= snap.arena_bytes + snap.hblkhd_bytes,
                "uordblks {} exceeds arena {} + hblkhd {}",
                snap.in_use_bytes,
                snap.arena_bytes,
                snap.hblkhd_bytes
            );
        }
        #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
        {
            assert!(snap.is_none());
        }
    }
}
