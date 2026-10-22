//! 进程级 Ctrl+C / SIGINT 信号处理。
//!
//! ## 为什么需要
//!
//! Windows 控制台默认 Ctrl+C handler 会直接 `TerminateProcess` 杀掉 browser，
//! 跳过所有 Rust 析构（`Drop`），导致 `shutdown_child_processes` 不执行，
//! `zero-renderer` 子进程成为孤儿并锁住自身 exe → 下次 `cargo build` 报 `os error 5`。
//!
//! 这里注册一个 handler 把信号转成 atomic flag，事件循环每轮检查 flag，
//! 命中后走和「窗口关闭按钮」一样的正常退出路径（`shutdown_child_processes` + `process::exit`）。
//!
//! Job Object 仍是兜底（覆盖断电 / 任务管理器强杀等场景），但本模块让最常见的
//! Ctrl+C 路径不再依赖 Job 是否绑定成功。
//!
//! ## 平台
//!
//! - Windows：`SetConsoleCtrlHandler` 捕获 `CTRL_C_EVENT` / `CTRL_BREAK_EVENT` 等，
//!   对 C/BREAK 返回 TRUE 阻止默认 TerminateProcess；CLOSE/LOGOFF/SHUTDOWN 返回 FALSE
//!   （系统会强制结束，但已先置 flag 给事件循环最后一次机会）。
//! - 非 Windows：`libc::signal(SIGINT, ...)` 把信号转成同一 atomic flag。此前为
//!   no-op，默认 SIGINT 直接终止进程会跳过 `persist_user_data`（会话 clean_exit
//!   丢失）并留下孤儿 renderer，与 Windows 侧动机相同。代价：事件循环卡死时
//!   Ctrl+C 不再强杀（Windows 侧已接受同一取舍）；SIGTERM 保持默认强杀语义。

use std::sync::atomic::{AtomicBool, Ordering};

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// 信号是否被触发。事件循环每轮调用。
pub fn is_set() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

/// 重置标志（仅供测试）。
#[cfg(test)]
pub fn reset() {
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
}

/// 注册信号 handler。在 `main` 早期调用一次，重复调用幂等（仅首次生效）。
///
/// 失败仅 `tracing::warn!`：信号处理失败时退化为「依赖 Job Object 兜底」，
/// 不致命。
pub fn install() {
    static INSTALLED: AtomicBool = AtomicBool::new(false);
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }

    #[cfg(windows)]
    install_windows_console_handler();

    #[cfg(not(windows))]
    {
        // Unix：SIGINT → atomic flag，事件循环检测后走 graceful 路径（见模块头）。
        // SAFETY: main 早期单线程安装；handler 只做 atomic store（async-signal-safe：
        // 无分配、无锁、不调用非重入函数）。
        unsafe {
            let handler = handle_sigint as extern "C" fn(libc::c_int) as libc::sighandler_t;
            libc::signal(libc::SIGINT, handler);
        }
    }
}

/// Unix SIGINT handler：仅置位 shutdown flag（async-signal-safe）。
#[cfg(not(windows))]
extern "C" fn handle_sigint(_sig: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

#[cfg(windows)]
fn install_windows_console_handler() {
    use windows_sys::Win32::Foundation::{BOOL, FALSE, TRUE};
    use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler};

    // SAFETY: handler 内只做 atomic store（lock-free，控制台信号上下文安全，
    // 不会分配内存、不会获取任何锁）。返回 TRUE 阻止默认 handler。
    unsafe extern "system" fn handler(ctrl_type: u32) -> BOOL {
        match ctrl_type {
            CTRL_C_EVENT | CTRL_BREAK_EVENT => {
                SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
                TRUE
            }
            // CLOSE/LOGOFF/SHUTDOWN：系统马上要结束进程，返回 FALSE 让默认流程继续，
            // 但仍先置 flag —— 若事件循环恰好有机会跑一轮就能清理，否则兜底靠 Job Object。
            _ => {
                SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
                FALSE
            }
        }
    }

    // SAFETY: 第二个参数 TRUE 表示 add（注册），不是 replace_last。
    let ok = unsafe { SetConsoleCtrlHandler(Some(handler), TRUE) };
    if ok == 0 {
        tracing::warn!("SetConsoleCtrlHandler failed; Ctrl+C will rely on Job Object fallback");
    } else {
        tracing::info!("Ctrl+C handler installed (graceful shutdown via event loop)");
    }
}
