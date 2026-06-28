//! Windows Job Object：把所有 `zero-renderer` 子进程绑定到进程级 Job，
//! browser 进程退出（含 Ctrl+C / `process::exit` / `abort` / 强杀）时
//! OS 自动 kill 整个 Job，避免孤儿 renderer 锁住自身 exe 导致下次构建 `os error 5`。
//!
//! Chromium / Chrome 也用同样的机制兜底子进程回收。
//!
//! ## 绑定方式
//!
//! 优先在 `CreateProcessW` 时通过 `STARTUPINFOEX` + `PROC_THREAD_ATTRIBUTE_JOB_LIST`
//! **预绑定**（suspended spawn 的子进程在恢复执行前就已挂入 Job，零竞态窗口）。
//! `assign_child`（spawn 之后再 `OpenProcess` 挂入）作为兜底。

use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::OpenProcess;

/// 失败日志的统一前缀，便于在 PowerShell → cargo → browser 的转发链路里被检索到。
const LOG_PREFIX: &str = "[zero-protocol/job]";

/// `HANDLE` 是 `*mut c_void`，不 `Send + Sync`，无法直接放进 `OnceLock`/`static`。
/// 这里用 newtype 包一层，手动声明 `Send + Sync` —— HANDLE 本质是内核对象 ID，
/// 在进程内任意线程使用都安全。
struct SyncHandle(HANDLE);

unsafe impl Send for SyncHandle {}
unsafe impl Sync for SyncHandle {}

/// 进程级 Job Object 句柄（首次访问时创建，生命周期 = browser 进程）。
///
/// 不显式 `CloseHandle`：browser 进程退出时 OS 自动回收句柄，触发
/// `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`，瞬间 kill 所有挂在该 Job 下的 renderer。
static JOB: OnceLock<SyncHandle> = OnceLock::new();

/// 创建 Job Object 并设置 `KILL_ON_JOB_CLOSE`：
/// 句柄关闭（含进程退出）时，OS 立刻 kill 所有挂在该 Job 下的进程。
fn create_job() -> Option<HANDLE> {
    unsafe {
        let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if handle.is_null() {
            eprintln!("{LOG_PREFIX} CreateJobObjectW returned null");
            return None;
        }

        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        let ok = SetInformationJobObject(
            handle,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        if ok == 0 {
            eprintln!("{LOG_PREFIX} SetInformationJobObject(KILL_ON_JOB_CLOSE) failed");
            CloseHandle(handle);
            return None;
        }
        Some(handle)
    }
}

/// 获取（首次时创建）进程级 Job Object 句柄。
///
/// 返回的句柄归 OS 内核对象所有，调用方**不要** `CloseHandle`。
/// 返回 `None` 表示 Job 创建失败（此时只能靠显式 `shutdown_child_processes` 路径兜底）。
fn job_handle() -> Option<HANDLE> {
    let sh = JOB.get_or_init(|| {
        let h = create_job().unwrap_or(std::ptr::null_mut());
        SyncHandle(h)
    });
    if sh.0.is_null() { None } else { Some(sh.0) }
}

/// 获取（首次时创建）进程级 Job Object 句柄，供 `STARTUPINFOEX` 预绑定使用。
///
/// 调用方把返回的 `HANDLE` 塞进 `PROC_THREAD_ATTRIBUTE_JOB_LIST`，
/// 让 `CreateProcessW` 直接把新进程挂到 Job 上 —— 这是无竞态的首选路径。
///
/// 失败返回 `None`：调用方应回退到 `assign_child` 兜底路径。
pub fn handle_for_pre_assign() -> Option<HANDLE> {
    job_handle()
}

/// 把已 spawn 的子进程（按 pid）加入进程级 Job Object。
///
/// **这是兜底路径**：首选在 `CreateProcessW` 时通过 `STARTUPINFOEX` 预绑定，
/// 见 [`handle_for_pre_assign`]。`assign_child` 仅在预绑定不可用时（如 spawn 实现不便于
/// 构造 STARTUPINFOEX、或预绑定失败）使用，存在 spawn 到 assign 之间的短竞态窗口。
///
/// 失败仅 `eprintln!` 记录：Job 仅作为最后兜底（Drop / 显式 shutdown 仍是主路径），
/// 失败不至于致命。日志前缀统一为 [`LOG_PREFIX`]，便于在 cargo/powershell 转发链路里检索。
/// `pid == 0` 直接返回（不该发生）。
pub fn assign_child(pid: u32) {
    if pid == 0 {
        return;
    }
    let Some(job) = job_handle() else {
        eprintln!("{LOG_PREFIX} assign_child({pid}): no job handle available");
        return;
    };

    unsafe {
        // PROCESS_SET_QUOTA (0x0200) | PROCESS_TERMINATE (0x0001) — Job Object 加入所需权限。
        // `inheritHandle = FALSE`。
        let process = OpenProcess(0x0200 | 0x0001, 0, pid);
        if process.is_null() {
            eprintln!("{LOG_PREFIX} assign_child({pid}): OpenProcess failed");
            return;
        }
        let ok = AssignProcessToJobObject(job, process);
        CloseHandle(process);
        if ok == 0 {
            // 常见原因：子进程已经被预绑定到 Job（这是好事，不算错误）；
            // 或权限不足。这里降级为 warn 风格的日志。
            eprintln!(
                "{LOG_PREFIX} assign_child({pid}): AssignProcessToJobObject failed (likely already assigned or permission denied)"
            );
        }
    }
}
