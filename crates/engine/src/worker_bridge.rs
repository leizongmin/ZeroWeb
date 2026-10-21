//! M3-S1a 页面 worker 宿主命令桥槽（goal compositor-animation-continuity，RFC
//! `docs/goal/compositor-animation-continuity/m3-slice-worker-canvas.md` §2.1/§3）：
//! 真线程 worker 的宿主命令通道（spawn/post/terminate）。
//!
//! engine 不引 renderer 依赖——桥闭包由 renderer 装填（js_worker 脚本线程经
//! `__zw_worker_host_op` 回调进入本槽转发命令；spawn 同步等 worker id，post/
//! terminate fire-and-forget）。未装填（renderer 未起 / 测试）时回调不注册，shim
//! `typeof` 守卫回落 same-realm worker——flag 关零行为（workers-compat 钉住）。
//!
//! 进程级槽模式与动画命中修正提供者槽（`animation/hit_provider.rs`）同型：
//! renderer 线程写、js_worker 脚本线程读。

use std::sync::{Arc, LazyLock, RwLock};

/// 宿主命令桥：`args[0]` = op（spawn/post/terminate），其余为 op 参数；返回值
/// 直接作为 `__zw_worker_host_op` 的 JS 返回字符串（spawn → 十进制 id 或
/// `"err:…"`；post/terminate → `"ok"`/`"err:…"`）。
type WorkerHostOpFn = dyn Fn(&[String]) -> String + Send + Sync;

/// 槽内桥的句柄类型（回调注册时 Arc 克隆一次，逐调用只查表不解锁）。
pub(crate) type WorkerHostOp = Arc<WorkerHostOpFn>;

static WORKER_HOST_OP: LazyLock<RwLock<Option<WorkerHostOp>>> = LazyLock::new(|| RwLock::new(None));

/// renderer 侧装填/撤除 worker 宿主命令桥（`None` = 撤除）。进程级槽——重复装填
/// 后到者胜（单 renderer 进程一个页面运行时，正常仅构造期一次）。
pub fn set_worker_host_op(op: Option<WorkerHostOp>) {
    if let Ok(mut slot) = WORKER_HOST_OP.write() {
        *slot = op;
    }
}

/// `__zw_worker_host_op` 回调注册时读取当前桥（`None` = 未装填，回调不注册 →
/// shim 回落 same-realm worker）。
pub(crate) fn worker_host_op() -> Option<WorkerHostOp> {
    WORKER_HOST_OP.read().ok().and_then(|slot| slot.clone())
}
