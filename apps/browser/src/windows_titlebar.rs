//! Windows 11 Snap Layout 的最大化按钮命中测试。
//!
//! Chrome 绘制自己的按钮，但最大化按钮在 `WM_NCHITTEST` 中返回 `HTMAXBUTTON`。
//! Windows 因而识别其为系统最大化控件，并在悬停时显示 Snap Layout。

use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetClientRect, HTMAXBUTTON, IsZoomed, SW_MAXIMIZE, SW_RESTORE, ShowWindow, WM_NCHITTEST, WM_NCLBUTTONDOWN,
    WM_NCMOUSELEAVE,
};

const SUBCLASS_ID: usize = 0x5A57_5442;
const WINDOW_CONTROL_BUTTON_WIDTH: f32 = 46.0;
const TAB_STRIP_HEIGHT: f32 = 40.0;
static MAXIMIZE_HOVERED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 当前鼠标是否位于由系统命中的自绘最大化/还原按钮上。
pub fn maximize_hovered() -> bool {
    MAXIMIZE_HOVERED.load(std::sync::atomic::Ordering::Relaxed)
}

/// 在无装饰 winit 窗口上安装最大化按钮命中测试。
pub fn install(window: &winit::window::Window) -> Result<(), String> {
    let handle = window
        .window_handle()
        .map_err(|error| format!("读取窗口句柄失败: {error}"))?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return Err("当前窗口不是 Win32 HWND".to_string());
    };
    let hwnd = handle.hwnd.get() as HWND;
    unsafe {
        if SetWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID, 0) == 0 {
            return Err("SetWindowSubclass 失败".to_string());
        }
    }
    Ok(())
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _reference_data: usize,
) -> LRESULT {
    match message {
        WM_NCHITTEST => {
            let mut point = POINT {
                x: lparam as i16 as i32,
                y: (lparam >> 16) as i16 as i32,
            };
            let on_maximize =
                unsafe { ScreenToClient(hwnd, &mut point) } != 0 && unsafe { is_maximize_button(hwnd, point) };
            MAXIMIZE_HOVERED.store(on_maximize, std::sync::atomic::Ordering::Relaxed);
            if on_maximize {
                return HTMAXBUTTON as isize;
            }
        }
        WM_NCLBUTTONDOWN if wparam == HTMAXBUTTON as usize => {
            let command = if unsafe { IsZoomed(hwnd) } != 0 {
                SW_RESTORE
            } else {
                SW_MAXIMIZE
            };
            let _ = unsafe { ShowWindow(hwnd, command) };
            return 0;
        }
        WM_NCMOUSELEAVE => {
            MAXIMIZE_HOVERED.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

unsafe fn is_maximize_button(hwnd: HWND, point: POINT) -> bool {
    let dpi_scale = unsafe { GetDpiForWindow(hwnd) }.max(96) as f32 / 96.0;
    let button_width = WINDOW_CONTROL_BUTTON_WIDTH * dpi_scale;
    let client_width = unsafe { client_width(hwnd) };
    let maximize_left = client_width - button_width * 2.0;
    let maximize_right = client_width - button_width;
    point.y >= 0
        && (point.y as f32) < TAB_STRIP_HEIGHT * dpi_scale
        && (point.x as f32) >= maximize_left
        && (point.x as f32) < maximize_right
}

unsafe fn client_width(hwnd: HWND) -> f32 {
    let mut rect = unsafe { std::mem::zeroed() };
    let _ = unsafe { GetClientRect(hwnd, &mut rect) };
    rect.right.max(0) as f32
}
