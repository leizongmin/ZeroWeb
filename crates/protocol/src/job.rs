//! Windows Job Object：把所有 `zero-renderer` 子进程绑定到进程级 Job，
//! browser 进程退出（含 Ctrl+C / `process::exit` / `abort` / 强杀）时
//! OS 自动 kill 整个 Job，避免孤儿 renderer 锁住自身 exe 导致下次构建 `os error 5`。
//!
//! Chromium / Chrome 也用同样的机制兜底子进程回收。

use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::OpenProcess;

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
            CloseHandle(handle);
            return None;
        }
        Some(handle)
    }
}

/// 获取（首次时创建）进程级 Job Object 句柄。
fn job_handle() -> Option<HANDLE> {
    let sh = JOB.get_or_init(|| {
        let h = create_job().unwrap_or(std::ptr::null_mut());
        SyncHandle(h)
    });
    if sh.0.is_null() { None } else { Some(sh.0) }
}

/// 把已 spawn 的子进程（按 pid）加入进程级 Job Object。
///
/// 失败仅 `eprintln!` 记录：Job 仅作为最后兜底（Drop / 显式 shutdown 仍是主路径），
/// 失败不至于致命；不引 `tracing` 依赖以避免 `zero-protocol` 多加 crate。
/// `pid == 0` 直接返回（不该发生）。
pub fn assign_child(pid: u32) {
    if pid == 0 {
        return;
    }
    let Some(job) = job_handle() else {
        return;
    };

    unsafe {
        // PROCESS_SET_QUOTA (0x0200) | PROCESS_TERMINATE (0x0001) — Job Object 加入所需权限。
        // `inheritHandle = FALSE`。
        let process = OpenProcess(0x0200 | 0x0001, 0, pid);
        if process.is_null() {
            eprintln!("[zero-protocol] OpenProcess({pid}) failed for Job assign");
            return;
        }
        let ok = AssignProcessToJobObject(job, process);
        CloseHandle(process);
        if ok == 0 {
            eprintln!("[zero-protocol] AssignProcessToJobObject(pid={pid}) failed");
        }
    }
}
