//! Android JNI entry points for the ZeroWeb browser host.

mod facade;

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jstring};
#[cfg(target_os = "android")]
use jni::sys::{jbyteArray, jfloat};

#[cfg(target_os = "android")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(target_os = "android")]
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "android")]
use zero_protocol::CompositorUiSurfaceInfo;
#[cfg(target_os = "android")]
use zero_protocol::IpcChannel;
#[cfg(target_os = "android")]
use zero_protocol::message::{
    FetchParams, FetchResponseParams, FramePublishMode, ImageDecodeParams, IpcMessage, IpcMessageKind, LoadHtmlParams,
    NavigateParams, ScrollEventParams, SetViewportParams,
};

const NATIVE_VERSION: &str = "ZeroWeb Android M2";

#[cfg(target_os = "android")]
const ANDROID_COMPOSITOR_SURFACE_ID: u64 = 1;
#[cfg(any(target_os = "android", test))]
const MAX_COMPOSITOR_SURFACE_DIMENSION: u32 = 4_096;
#[cfg(target_os = "android")]
const ANDROID_PAGE_VIEWPORT_WIDTH: u32 = 320;
#[cfg(target_os = "android")]
const ANDROID_PAGE_VIEWPORT_HEIGHT: u32 = 180;
#[cfg(target_os = "android")]
type AndroidCompositorTransport =
    zero_protocol::PipeTransport<std::os::unix::net::UnixStream, std::os::unix::net::UnixStream>;
#[cfg(target_os = "android")]
type AndroidRendererTransport =
    zero_protocol::PipeTransport<std::os::unix::net::UnixStream, std::os::unix::net::UnixStream>;
#[cfg(target_os = "android")]
static ANDROID_COMPOSITOR: OnceLock<Mutex<Option<AndroidCompositorTransport>>> = OnceLock::new();
#[cfg(target_os = "android")]
static ANDROID_RENDERER: OnceLock<Vec<Mutex<Option<AndroidRendererTransport>>>> = OnceLock::new();
#[cfg(target_os = "android")]
static ANDROID_PAGE_FRAME: OnceLock<Vec<Mutex<Option<Vec<u8>>>>> = OnceLock::new();
#[cfg(target_os = "android")]
type AndroidPageMeta = (u64, u64, u64, f32);
#[cfg(target_os = "android")]
static ANDROID_PAGE_META: OnceLock<Vec<Mutex<Option<AndroidPageMeta>>>> = OnceLock::new();
#[cfg(target_os = "android")]
static ANDROID_SECURITY: OnceLock<Mutex<zero_security::SecurityContext>> = OnceLock::new();
#[cfg(target_os = "android")]
static ANDROID_NAVIGATION_EPOCH: AtomicU64 = AtomicU64::new(1);

#[cfg(target_os = "android")]
fn android_compositor() -> &'static Mutex<Option<AndroidCompositorTransport>> {
    ANDROID_COMPOSITOR.get_or_init(|| Mutex::new(None))
}

/// RFC §6.3 槽位注册表：每个 renderer isolated Service slot 一个 transport 槽。
#[cfg(target_os = "android")]
fn android_renderer() -> &'static Vec<Mutex<Option<AndroidRendererTransport>>> {
    ANDROID_RENDERER.get_or_init(|| (0..facade::RENDERER_SLOT_COUNT).map(|_| Mutex::new(None)).collect())
}

/// 槽号越界视为未附着（防御 Kotlin 侧错传）。
#[cfg(target_os = "android")]
fn renderer_slot(slot: usize) -> Option<&'static Mutex<Option<AndroidRendererTransport>>> {
    android_renderer().get(slot)
}

#[cfg(target_os = "android")]
fn android_page_frame(slot: usize) -> Option<&'static Mutex<Option<Vec<u8>>>> {
    let frames =
        ANDROID_PAGE_FRAME.get_or_init(|| (0..facade::RENDERER_SLOT_COUNT).map(|_| Mutex::new(None)).collect());
    frames.get(slot)
}

#[cfg(target_os = "android")]
fn android_page_meta(slot: usize) -> Option<&'static Mutex<Option<AndroidPageMeta>>> {
    let metas = ANDROID_PAGE_META.get_or_init(|| (0..facade::RENDERER_SLOT_COUNT).map(|_| Mutex::new(None)).collect());
    metas.get(slot)
}

#[cfg(target_os = "android")]
fn android_security() -> &'static Mutex<zero_security::SecurityContext> {
    ANDROID_SECURITY.get_or_init(|| Mutex::new(zero_security::SecurityContext::default()))
}

/// Satisfies winit's Android native-activity link contract.
///
/// ZeroWeb uses a Kotlin `Activity` rather than a manifest `NativeActivity`, so
/// Android never invokes this symbol. Keeping the anchor in the JNI cdylib lets
/// shared rendering dependencies compile without changing the host lifecycle.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn android_main() {}

fn is_known_role(role: &str) -> bool {
    matches!(role, "renderer" | "compositor" | "image-decoder")
}

// RFC §6.3：8 个预声明 isolated Service slots。slot 号由 Kotlin Service 类名
// （RendererService0-7）透传为 renderer_id，兼作 compositor 帧的 surface_id。
// attach 协议在 renderer-less 构建同样存在（需要拒绝越界槽并关 FD），故仅对宿主
// 非 android 构建豁免。
#[cfg(any(target_os = "android", test))]
fn renderer_slot_id(slot: jni::sys::jint) -> Result<u64, String> {
    u64::try_from(slot)
        .ok()
        .filter(|id| *id < 8)
        .ok_or_else(|| "renderer slot must be within 0-7".to_string())
}

/// Returns the native host version shown by the Android bootstrap screen.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeVersion(env: JNIEnv, _class: JClass) -> jstring {
    env.new_string(NATIVE_VERSION)
        .map_or(std::ptr::null_mut(), |value| value.into_raw())
}

/// Reports whether this Android host binary includes the real renderer role.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeRendererLinked(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    #[cfg(feature = "android-renderer")]
    {
        JNI_TRUE
    }
    #[cfg(not(feature = "android-renderer"))]
    {
        JNI_FALSE
    }
}

/// Loads the Android profile into the Rust-owned browser shell and returns its chrome snapshot.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeLoadProfile(
    mut env: JNIEnv,
    _class: JClass,
    root: JString,
) -> jstring {
    let result = env
        .get_string(&root)
        .map_err(|error| format!("read Android profile path failed: {error}"))
        .and_then(|root| facade::load_profile(root.to_str().map_err(|error| error.to_string())?));
    jni_string(&mut env, result)
}

/// Returns the current Rust-owned browser chrome snapshot.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeBrowserSnapshot(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    jni_string(&mut env, facade::snapshot())
}

/// Navigates the active tab and persists the browser profile.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeNavigate(
    mut env: JNIEnv,
    _class: JClass,
    url: JString,
) -> jboolean {
    env.get_string(&url)
        .map_err(|error| error.to_string())
        .and_then(|url| {
            let url = url.to_str().map_err(|error| error.to_string())?;
            facade::navigate(url)?;
            #[cfg(target_os = "android")]
            navigate_renderer(url)?;
            Ok(())
        })
        .map_or(JNI_FALSE, |_| JNI_TRUE)
}

/// Creates a new tab and persists the browser profile.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeNewTab(_env: JNIEnv, _class: JClass) -> jboolean {
    facade::new_tab().map_or(JNI_FALSE, |_| JNI_TRUE)
}

/// Creates a new tab for one externally supplied HTTP(S) URL.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeNewTabWithUrl(
    mut env: JNIEnv,
    _class: JClass,
    url: JString,
) -> jboolean {
    env.get_string(&url)
        .map_err(|error| error.to_string())
        .and_then(|url| facade::new_tab_with_url(url.to_str().map_err(|error| error.to_string())?))
        .map_or(JNI_FALSE, |_| JNI_TRUE)
}

/// Closes one tab and persists the browser profile.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeCloseTab(
    _env: JNIEnv,
    _class: JClass,
    id: jni::sys::jlong,
) -> jboolean {
    let outcome = u64::try_from(id)
        .map_err(|_| "tab ID must be non-negative".to_string())
        .and_then(|tab_id| {
            // 回收该标签的渲染槽：transport 就地丢弃，旧 renderer 收到 EOF 自行退出
            let released = facade::tab_slot(tab_id)?;
            facade::close_tab(tab_id)?;
            #[cfg(not(target_os = "android"))]
            let _ = released;
            #[cfg(target_os = "android")]
            if let Some(slot) = released
                && let Some(slot_lock) = renderer_slot(slot)
                && let Ok(mut slot_guard) = slot_lock.lock()
            {
                *slot_guard = None;
            }
            Ok(())
        });
    outcome.map_or(JNI_FALSE, |_| JNI_TRUE)
}

/// Selects the active tab and persists the browser profile.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeSelectTab(
    _env: JNIEnv,
    _class: JClass,
    id: jni::sys::jlong,
) -> jboolean {
    u64::try_from(id)
        .map_err(|_| "tab ID must be non-negative".to_string())
        .and_then(facade::select_tab)
        .map_or(JNI_FALSE, |_| JNI_TRUE)
}

/// Moves the active tab backward in its Rust-owned navigation history.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeGoBack(_env: JNIEnv, _class: JClass) -> jboolean {
    facade::go_back().map_or(
        JNI_FALSE,
        |did_navigate| if did_navigate { JNI_TRUE } else { JNI_FALSE },
    )
}

/// Moves the active tab forward in its Rust-owned navigation history.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeGoForward(_env: JNIEnv, _class: JClass) -> jboolean {
    facade::go_forward().map_or(
        JNI_FALSE,
        |did_navigate| if did_navigate { JNI_TRUE } else { JNI_FALSE },
    )
}

/// Toggles the active page bookmark and persists the browser profile.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeToggleBookmark(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    facade::toggle_bookmark().map_or(JNI_FALSE, |_| JNI_TRUE)
}

/// Removes a bookmark identified by its HTTP(S) URL and persists the profile.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeRemoveBookmark(
    mut env: JNIEnv,
    _class: JClass,
    url: JString,
) -> jboolean {
    env.get_string(&url)
        .map_err(|error| error.to_string())
        .and_then(|url| facade::remove_bookmark(url.to_str().map_err(|error| error.to_string())?))
        .map_or(JNI_FALSE, |_| JNI_TRUE)
}

/// Clears all persisted history records.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeClearHistory(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    facade::clear_history().map_or(JNI_FALSE, |_| JNI_TRUE)
}

fn jni_string(env: &mut JNIEnv, result: Result<String, String>) -> jstring {
    let json = match result {
        Ok(snapshot) => snapshot,
        Err(error) => format!(r#"{{"error":{}}}"#, serde_json::to_string(&error).unwrap_or_default()),
    };
    env.new_string(json)
        .map_or(std::ptr::null_mut(), |value| value.into_raw())
}

/// Validates a Service role before its Android process reports itself ready.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeStartRole(
    mut env: JNIEnv,
    _class: JClass,
    role: JString,
) -> jboolean {
    let Ok(role) = env.get_string(&role) else {
        return JNI_FALSE;
    };

    if is_known_role(role.to_str().unwrap_or_default()) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// Starts an Android role with ownership of a detached socket FD.
///
/// All three roles run their shared Rust role loops. `slot` identifies the
/// predeclared renderer Service slot (RFC §6.3，0-7）；非 renderer 角色忽略该值。
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeRunRole(
    mut env: JNIEnv,
    _class: JClass,
    role: JString,
    _slot: jni::sys::jint,
    fd: jni::sys::jint,
) -> jboolean {
    let Ok(role) = env.get_string(&role) else {
        close_android_fd(fd);
        return JNI_FALSE;
    };
    match role.to_str().ok() {
        #[cfg(feature = "android-renderer")]
        Some("renderer") => match renderer_slot_id(_slot) {
            Ok(renderer_id) => std::thread::Builder::new()
                .name("android-renderer".to_string())
                .spawn(move || zero_renderer::run_android_role(renderer_id, fd))
                .map_or(JNI_FALSE, |_| JNI_TRUE),
            Err(_) => {
                close_android_fd(fd);
                JNI_FALSE
            }
        },
        Some("image-decoder") | Some("compositor") => {
            let Ok(mut transport) = zero_protocol::android_socket_transport_from_fd(fd) else {
                return JNI_FALSE;
            };
            let name = if role.to_str().ok() == Some("image-decoder") {
                "android-image-decoder"
            } else {
                "android-compositor"
            };
            std::thread::Builder::new()
                .name(name.to_string())
                .spawn(move || {
                    if name == "android-image-decoder" {
                        zero_image_decoder::run_role(&mut transport);
                    } else {
                        zero_compositor::run_role(&mut transport);
                    }
                })
                .map_or(JNI_FALSE, |_| JNI_TRUE)
        }
        _ => {
            close_android_fd(fd);
            JNI_FALSE
        }
    }
}

/// Attaches the browser-side endpoint for the compositor Service and registers
/// a long-lived UI surface. The compositor Service owns the peer endpoint.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeAttachCompositor(
    _env: JNIEnv,
    _class: JClass,
    fd: jni::sys::jint,
    width: jni::sys::jint,
    height: jni::sys::jint,
) -> jboolean {
    let Ok((width, height, _)) = compositor_pixel_len(width, height) else {
        close_android_fd(fd);
        return JNI_FALSE;
    };
    let Ok(mut transport) = zero_protocol::android_socket_transport_from_fd(fd) else {
        return JNI_FALSE;
    };
    let register = IpcMessage {
        id: 1,
        kind: IpcMessageKind::CompositorRegisterUiSurface(CompositorUiSurfaceInfo {
            surface_id: ANDROID_COMPOSITOR_SURFACE_ID,
            width,
            height,
        }),
    };
    if transport.send(register).is_err()
        || !matches!(
            transport.recv(),
            Ok(IpcMessage {
                id: 1,
                kind: IpcMessageKind::Ok
            })
        )
    {
        return JNI_FALSE;
    }
    let Ok(mut slot) = android_compositor().lock() else {
        return JNI_FALSE;
    };
    if slot.is_some() {
        return JNI_FALSE;
    }
    *slot = Some(transport);
    JNI_TRUE
}

/// Attaches the browser-side renderer endpoint and starts forwarding renderer
/// compositor frames through the already attached compositor Service channel.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeAttachRenderer(
    _env: JNIEnv,
    _class: JClass,
    slot: jni::sys::jint,
    fd: jni::sys::jint,
) -> jboolean {
    use std::os::unix::io::FromRawFd;

    let Ok(slot) = renderer_slot_id(slot).map(|id| id as usize).map_err(|_| {
        close_android_fd(fd);
    }) else {
        return JNI_FALSE;
    };
    let Some(slot_lock) = renderer_slot(slot) else {
        close_android_fd(fd);
        return JNI_FALSE;
    };
    if fd < 0 {
        return JNI_FALSE;
    }
    let stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(fd) };
    let Ok(reader) = stream.try_clone() else {
        return JNI_FALSE;
    };
    let Ok(writer) = stream.try_clone() else {
        return JNI_FALSE;
    };
    let Ok(mut slot_guard) = slot_lock.lock() else {
        return JNI_FALSE;
    };
    if slot_guard.is_some() {
        return JNI_FALSE;
    }
    *slot_guard = Some(zero_protocol::PipeTransport::new(stream, writer));
    drop(slot_guard);

    if std::thread::Builder::new()
        .name(format!("android-renderer-frames-{slot}"))
        .spawn(move || {
            let mut inbound = zero_protocol::PipeTransport::new(reader, std::io::sink());
            while let Ok(message) = inbound.recv() {
                match message.kind {
                    IpcMessageKind::CompositorFrame {
                        surface_id,
                        navigation_epoch,
                        frame_id,
                        paint,
                    } => {
                        let _ = forward_renderer_frame(surface_id, navigation_epoch, frame_id, *paint);
                    }
                    IpcMessageKind::FetchRequest(params) => {
                        let _ = proxy_renderer_fetch(slot, params);
                    }
                    IpcMessageKind::TitleChanged(title) => {
                        let _ = facade::page_loaded(&title);
                    }
                    _ => {}
                }
            }
            // renderer 进程死亡/断连：清除该槽僵尸 transport，允许 Kotlin 重新走
            // attach 协议（renderer slot 恢复，android-browser goal M3 切片 1）。
            if let Some(slot_lock) = renderer_slot(slot)
                && let Ok(mut slot_guard) = slot_lock.lock()
            {
                *slot_guard = None;
            }
            tracing::warn!("android renderer transport detached: slot {slot}");
        })
        .is_err()
    {
        if let Ok(mut slot_guard) = slot_lock.lock() {
            *slot_guard = None;
        }
        return JNI_FALSE;
    }

    let handshake = send_renderer_to_slot(slot, IpcMessageKind::SetViewport(SetViewportParams {
        width: ANDROID_PAGE_VIEWPORT_WIDTH,
        height: ANDROID_PAGE_VIEWPORT_HEIGHT,
        device_scale_factor: 1.0,
    }))
    .and_then(|_| {
        send_renderer_to_slot(slot, IpcMessageKind::SetFramePublishMode(FramePublishMode::Compositor))
    })
    .and_then(|_| {
        send_renderer_to_slot(slot, IpcMessageKind::LoadHtml(LoadHtmlParams {
            html: "<html><body style='margin:0;background:#0c2238;color:white'><h1>ZeroWeb Android renderer</h1><p>renderer → compositor → Compose</p></body></html>".to_string(),
            css: None,
            url: Some("zero://android-renderer-smoke".to_string()),
            navigation_epoch: 1,
        }))
    });
    // 握手失败说明刚 attach 的 transport 已不可用：清除避免僵尸槽位。
    if handshake.is_err() {
        if let Ok(mut slot_guard) = slot_lock.lock() {
            *slot_guard = None;
        }
        return JNI_FALSE;
    }
    JNI_TRUE
}

/// Reports whether a renderer transport is currently attached on the slot.
/// Kotlin 在导航失败/快照驱动绑槽时据此判断（renderer-less 构建恒为 false）。
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeIsRendererAttached(
    _env: JNIEnv,
    _class: JClass,
    slot: jni::sys::jint,
) -> jboolean {
    let Ok(slot) = renderer_slot_id(slot).map(|id| id as usize) else {
        return JNI_FALSE;
    };
    let Some(slot_lock) = renderer_slot(slot) else {
        return JNI_FALSE;
    };
    let Ok(slot_guard) = slot_lock.lock() else {
        return JNI_FALSE;
    };
    if slot_guard.is_some() { JNI_TRUE } else { JNI_FALSE }
}

/// Returns the latest renderer page frame of the slot after compositor rasterization.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeLatestPageFrame(
    env: JNIEnv,
    _class: JClass,
    slot: jni::sys::jint,
) -> jbyteArray {
    let Ok(slot) = renderer_slot_id(slot).map(|id| id as usize) else {
        return std::ptr::null_mut();
    };
    let Some(frame_lock) = android_page_frame(slot) else {
        return std::ptr::null_mut();
    };
    let Ok(frame) = frame_lock.lock() else {
        return std::ptr::null_mut();
    };
    frame
        .as_ref()
        .and_then(|rgba| env.byte_array_from_slice(rgba).ok())
        .map_or(std::ptr::null_mut(), |frame| frame.into_raw())
}

#[cfg(target_os = "android")]
fn navigate_renderer(url: &str) -> Result<(), String> {
    let epoch = ANDROID_NAVIGATION_EPOCH
        .fetch_add(1, Ordering::Relaxed)
        .saturating_add(1);
    android_security()
        .lock()
        .map_err(|_| "Android security context lock poisoned".to_string())?
        .set_page_origin(url);
    // 活动标签的渲染槽：无槽则按 LRU 分配；被逐出的旧租户 transport 立即丢弃
    // （旧 renderer 收到 EOF 自行退出，其槽交由 Kotlin 重新 bind）。
    let (slot, evicted) = facade::assign_active_tab_slot()?;
    if evicted.is_some()
        && let Some(slot_lock) = renderer_slot(slot)
        && let Ok(mut slot_guard) = slot_lock.lock()
    {
        *slot_guard = None;
    }
    if !renderer_attached(slot) {
        return Err(format!("renderer slot {slot} is not attached"));
    }
    send_renderer_to_slot(
        slot,
        IpcMessageKind::Navigate(NavigateParams {
            url: url.to_string(),
            referrer: None,
            navigation_epoch: epoch,
        }),
    )
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeScroll(
    _env: JNIEnv,
    _class: JClass,
    delta_y: jfloat,
) -> jboolean {
    if !delta_y.is_finite() || delta_y.abs() > 4_096.0 {
        return JNI_FALSE;
    }
    // 滚动作用于活动标签的渲染槽
    let Ok(Some(active_slot)) = facade::active_tab_slot() else {
        return JNI_FALSE;
    };
    if send_renderer_to_slot(
        active_slot,
        IpcMessageKind::ScrollEvent(ScrollEventParams {
            delta_x: 0.0,
            delta_y,
            cursor_x: ANDROID_PAGE_VIEWPORT_WIDTH as f32 / 2.0,
            cursor_y: ANDROID_PAGE_VIEWPORT_HEIGHT as f32 / 2.0,
        }),
    )
    .is_err()
    {
        return JNI_FALSE;
    }
    compositor_scroll(active_slot, delta_y).map_or(JNI_FALSE, |_| JNI_TRUE)
}

#[cfg(target_os = "android")]
fn compositor_scroll(slot: usize, delta_y: f32) -> Result<(), String> {
    let Some(meta_lock) = android_page_meta(slot) else {
        return Err("renderer slot is out of range".to_string());
    };
    let mut meta = meta_lock
        .lock()
        .map_err(|_| "Android page metadata lock poisoned".to_string())?;
    let (surface_id, navigation_epoch, frame_id, scroll_y) = meta
        .as_mut()
        .ok_or_else(|| "Android page frame is not ready".to_string())?;
    *scroll_y = (*scroll_y + delta_y).max(0.0);
    let mut compositor = android_compositor()
        .lock()
        .map_err(|_| "Android compositor socket lock poisoned".to_string())?;
    let transport = compositor
        .as_mut()
        .ok_or_else(|| "Android compositor socket is not attached".to_string())?;
    transport
        .send(IpcMessage {
            id: 12,
            kind: IpcMessageKind::CompositorSetScroll {
                surface_id: *surface_id,
                scroll_x: 0.0,
                scroll_y: *scroll_y,
            },
        })
        .map_err(|e| e.to_string())?;
    if !matches!(
        transport.recv(),
        Ok(IpcMessage {
            id: 12,
            kind: IpcMessageKind::Ok
        })
    ) {
        return Err("Android compositor rejected scroll".to_string());
    }
    transport
        .send(IpcMessage {
            id: 13,
            kind: IpcMessageKind::GetCompositorFrame {
                surface_id: *surface_id,
                navigation_epoch: *navigation_epoch,
                frame_id: *frame_id,
            },
        })
        .map_err(|e| e.to_string())?;
    let frame = match transport.recv().map_err(|e| e.to_string())? {
        IpcMessage {
            id: 13,
            kind: IpcMessageKind::CompositorFrameData { rgba, .. },
        } if !rgba.is_empty() => rgba,
        _ => return Err("Android compositor returned no scrolled frame".to_string()),
    };
    let Some(frame_lock) = android_page_frame(slot) else {
        return Err("renderer slot is out of range".to_string());
    };
    *frame_lock
        .lock()
        .map_err(|_| "Android page frame lock poisoned".to_string())? = Some(frame);
    Ok(())
}

#[cfg(target_os = "android")]
fn proxy_renderer_fetch(slot: usize, params: FetchParams) -> Result<(), String> {
    const MAX_RESPONSE_BYTES: usize = 64 * 1024 * 1024;
    let resource_type = params
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("x-zero-resource-type"))
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| "document".to_string());
    let url = match android_security()
        .lock()
        .map_err(|_| "Android security context lock poisoned".to_string())?
        .check_resource_url(&params.url, &resource_type)
    {
        zero_security::ResourceCheckResult::Allow => params.url,
        zero_security::ResourceCheckResult::Upgraded(url) => url,
        zero_security::ResourceCheckResult::Blocked(reason) => {
            return send_fetch_response(
                slot,
                params.request_id,
                0,
                Vec::new(),
                format!("resource blocked: {reason}").into_bytes(),
            );
        }
    };
    let method = match params.method.to_ascii_uppercase().as_str() {
        "POST" => zero_net::HttpMethod::Post,
        "PUT" => zero_net::HttpMethod::Put,
        "DELETE" => zero_net::HttpMethod::Delete,
        "PATCH" => zero_net::HttpMethod::Patch,
        "HEAD" => zero_net::HttpMethod::Head,
        "OPTIONS" => zero_net::HttpMethod::Options,
        _ => zero_net::HttpMethod::Get,
    };
    let headers = params
        .headers
        .into_iter()
        .filter(|(name, _)| !name.to_ascii_lowercase().starts_with("x-zero-"))
        .collect();
    match zero_net::HttpClient::new().send(zero_net::HttpRequest {
        method,
        url,
        headers,
        body: params.body,
    }) {
        Ok(response) if response.body.len() <= MAX_RESPONSE_BYTES => {
            let mut headers = response.headers;
            headers.push(("X-Zero-Resource-Type".to_string(), resource_type.to_string()));
            headers.push(("X-Zero-Final-Url".to_string(), response.url));
            send_fetch_response(slot, params.request_id, response.status_code, headers, response.body)
        }
        Ok(_) => send_fetch_response(
            slot,
            params.request_id,
            0,
            Vec::new(),
            b"resource exceeds Android IPC frame limit".to_vec(),
        ),
        Err(error) => send_fetch_response(slot, params.request_id, 0, Vec::new(), error.to_string().into_bytes()),
    }
}

#[cfg(target_os = "android")]
fn send_fetch_response(
    slot: usize,
    request_id: u64,
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> Result<(), String> {
    // fetch 响应必须回到请求所在的槽（reader 线程上下文），不能路由到活动槽。
    send_renderer_to_slot(
        slot,
        IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id,
            status_code,
            headers,
            body,
        }),
    )
}

#[cfg(target_os = "android")]
fn renderer_attached(slot: usize) -> bool {
    renderer_slot(slot)
        .and_then(|slot_lock| slot_lock.lock().ok().map(|slot_guard| slot_guard.is_some()))
        .unwrap_or(false)
}

#[cfg(target_os = "android")]
fn send_renderer_to_slot(slot: usize, kind: IpcMessageKind) -> Result<(), String> {
    let Some(slot_lock) = renderer_slot(slot) else {
        return Err(format!("renderer slot {slot} is out of range"));
    };
    let mut slot_guard = slot_lock
        .lock()
        .map_err(|_| "Android renderer socket lock poisoned".to_string())?;
    slot_guard
        .as_mut()
        .ok_or_else(|| format!("renderer slot {slot} is not attached"))?
        .send(IpcMessage { id: 1, kind })
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "android")]
fn forward_renderer_frame(
    surface_id: u64,
    navigation_epoch: u64,
    frame_id: u64,
    paint: zero_protocol::paint_snapshot::PaintSnapshotParams,
) -> Result<(), String> {
    let mut slot = android_compositor()
        .lock()
        .map_err(|_| "Android compositor socket lock poisoned".to_string())?;
    let transport = slot
        .as_mut()
        .ok_or_else(|| "Android compositor socket is not attached".to_string())?;
    transport
        .send(IpcMessage {
            id: 10,
            kind: IpcMessageKind::CompositorFrame {
                surface_id,
                navigation_epoch,
                frame_id,
                paint: Box::new(paint),
            },
        })
        .map_err(|error| error.to_string())?;
    if !matches!(
        transport.recv(),
        Ok(IpcMessage {
            id: 10,
            kind: IpcMessageKind::CompositorFrameResult { .. }
        })
    ) {
        return Err("Android compositor rejected renderer frame".to_string());
    }
    transport
        .send(IpcMessage {
            id: 11,
            kind: IpcMessageKind::GetCompositorFrame {
                surface_id,
                navigation_epoch,
                frame_id,
            },
        })
        .map_err(|error| error.to_string())?;
    let frame = match transport.recv().map_err(|error| error.to_string())? {
        IpcMessage {
            id: 11,
            kind: IpcMessageKind::CompositorFrameData {
                width, height, rgba, ..
            },
        } if width == ANDROID_PAGE_VIEWPORT_WIDTH
            && height == ANDROID_PAGE_VIEWPORT_HEIGHT
            && rgba.len()
                == usize::try_from(width)
                    .unwrap_or(usize::MAX)
                    .saturating_mul(usize::try_from(height).unwrap_or(usize::MAX))
                    .saturating_mul(4) =>
        {
            rgba
        }
        _ => return Err("Android compositor returned an unexpected page frame".to_string()),
    };
    // 帧按其来源 surface 归档（renderer_id == slot）：多标签各回各的帧缓冲
    let Some(frame_lock) = android_page_frame(surface_id as usize) else {
        return Err("renderer frame surface is out of range".to_string());
    };
    *frame_lock
        .lock()
        .map_err(|_| "Android page frame lock poisoned".to_string())? = Some(frame);
    let Some(meta_lock) = android_page_meta(surface_id as usize) else {
        return Err("renderer frame surface is out of range".to_string());
    };
    *meta_lock
        .lock()
        .map_err(|_| "Android page metadata lock poisoned".to_string())? =
        Some((surface_id, navigation_epoch, frame_id, 0.0));
    Ok(())
}

/// Publishes a deterministic compositor frame and reads it back from the
/// independent compositor process for Android UI verification.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeCompositorTestFrame(
    env: JNIEnv,
    _class: JClass,
    width: jni::sys::jint,
    height: jni::sys::jint,
) -> jbyteArray {
    let Ok(rgba) = compositor_test_frame(width, height) else {
        return std::ptr::null_mut();
    };
    env.byte_array_from_slice(&rgba)
        .map_or(std::ptr::null_mut(), |frame| frame.into_raw())
}

#[cfg(target_os = "android")]
fn compositor_test_frame(width: jni::sys::jint, height: jni::sys::jint) -> Result<Vec<u8>, String> {
    let (width, height, len) = compositor_pixel_len(width, height)?;
    let mut rgba = vec![0; len];
    // MSRV 1.85：clippy 建议的 as_chunks_mut 1.88 才稳定，保留 chunks_exact_mut
    #[allow(clippy::chunks_exact_to_as_chunks)]
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[12, 34, 56, 255]);
    }
    let mut slot = android_compositor()
        .lock()
        .map_err(|_| "Android compositor socket lock poisoned".to_string())?;
    let transport = slot
        .as_mut()
        .ok_or_else(|| "Android compositor socket is not attached".to_string())?;
    transport
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::CompositorUiFrame {
                surface_id: ANDROID_COMPOSITOR_SURFACE_ID,
                width,
                height,
                rgba: rgba.clone(),
                shm_name: None,
            },
        })
        .map_err(|error| error.to_string())?;
    if !matches!(
        transport.recv(),
        Ok(IpcMessage {
            id: 2,
            kind: IpcMessageKind::Ok
        })
    ) {
        return Err("Android compositor rejected UI frame".to_string());
    }
    transport
        .send(IpcMessage {
            id: 3,
            kind: IpcMessageKind::GetCompositorUiFrame {
                surface_id: ANDROID_COMPOSITOR_SURFACE_ID,
            },
        })
        .map_err(|error| error.to_string())?;
    match transport.recv().map_err(|error| error.to_string())? {
        IpcMessage {
            id: 3,
            kind:
                IpcMessageKind::CompositorFrameData {
                    surface_id: ANDROID_COMPOSITOR_SURFACE_ID,
                    width: returned_width,
                    height: returned_height,
                    rgba: returned_rgba,
                    ..
                },
        } if returned_width == width && returned_height == height && returned_rgba == rgba => Ok(returned_rgba),
        _ => Err("Android compositor returned an unexpected UI frame".to_string()),
    }
}

#[cfg(target_os = "android")]
fn compositor_pixel_len(width: jni::sys::jint, height: jni::sys::jint) -> Result<(u32, u32, usize), String> {
    validate_compositor_dimensions(width, height)
}

// 尺寸校验是纯逻辑，cfg 放宽到 test：bounds 契约保持宿主可测
// （android-browser goal M2 P3——JNI 桥接测试覆盖）。
#[cfg(any(target_os = "android", test))]
fn validate_compositor_dimensions(width: jni::sys::jint, height: jni::sys::jint) -> Result<(u32, u32, usize), String> {
    let width = u32::try_from(width).map_err(|_| "compositor width must be non-negative".to_string())?;
    let height = u32::try_from(height).map_err(|_| "compositor height must be non-negative".to_string())?;
    if width == 0
        || height == 0
        || width > MAX_COMPOSITOR_SURFACE_DIMENSION
        || height > MAX_COMPOSITOR_SURFACE_DIMENSION
    {
        return Err("compositor dimensions are outside Android bounds".to_string());
    }
    let len = usize::try_from(width)
        .unwrap_or(usize::MAX)
        .checked_mul(usize::try_from(height).unwrap_or(usize::MAX))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "compositor frame size overflow".to_string())?;
    Ok((width, height, len))
}

#[cfg(target_os = "android")]
fn close_android_fd(fd: jni::sys::jint) {
    if fd >= 0 {
        // SAFETY: this branch rejects ownership transferred by detachFd().
        unsafe { libc::close(fd) };
    }
}

/// Sends one malformed image through the decoder socket and verifies its error reply.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeProbeDecoder(
    _env: JNIEnv,
    _class: JClass,
    fd: jni::sys::jint,
) -> jboolean {
    let Ok(mut transport) = zero_protocol::android_socket_transport_from_fd(fd) else {
        return JNI_FALSE;
    };
    let request = IpcMessage {
        id: 1,
        kind: IpcMessageKind::ImageDecodeRequest(ImageDecodeParams {
            request_id: 1,
            mime: "image/png".to_string(),
            bytes: vec![0, 1, 2],
        }),
    };
    if transport.send(request).is_err() {
        return JNI_FALSE;
    }

    match transport.recv() {
        Ok(IpcMessage {
            id: 1,
            kind: IpcMessageKind::ImageDecodeResult(result),
        }) if result.request_id == 1 && result.error.is_some() && result.rgba.is_empty() => JNI_TRUE,
        _ => JNI_FALSE,
    }
}

/// Verifies a compositor UI-surface round trip through the Android socket.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeProbeCompositor(
    _env: JNIEnv,
    _class: JClass,
    fd: jni::sys::jint,
) -> jboolean {
    let Ok(mut transport) = zero_protocol::android_socket_transport_from_fd(fd) else {
        return JNI_FALSE;
    };
    let register = IpcMessage {
        id: 1,
        kind: IpcMessageKind::CompositorRegisterUiSurface(CompositorUiSurfaceInfo {
            surface_id: 1,
            width: 1,
            height: 1,
        }),
    };
    if transport.send(register).is_err()
        || !matches!(
            transport.recv(),
            Ok(IpcMessage {
                id: 1,
                kind: IpcMessageKind::Ok
            })
        )
    {
        return JNI_FALSE;
    }
    let frame = IpcMessage {
        id: 2,
        kind: IpcMessageKind::CompositorUiFrame {
            surface_id: 1,
            width: 1,
            height: 1,
            rgba: vec![12, 34, 56, 255],
            shm_name: None,
        },
    };
    if transport.send(frame).is_err()
        || !matches!(
            transport.recv(),
            Ok(IpcMessage {
                id: 2,
                kind: IpcMessageKind::Ok
            })
        )
    {
        return JNI_FALSE;
    }
    if transport
        .send(IpcMessage {
            id: 3,
            kind: IpcMessageKind::GetCompositorUiFrame { surface_id: 1 },
        })
        .is_err()
    {
        return JNI_FALSE;
    }
    matches!(
        transport.recv(),
        Ok(IpcMessage {
            id: 3,
            kind:
                IpcMessageKind::CompositorFrameData {
                    surface_id: 1,
                    width: 1,
                    height: 1,
                    rgba,
                    ..
                },
        }) if rgba == vec![12, 34, 56, 255]
    )
    .then_some(JNI_TRUE)
    .unwrap_or(JNI_FALSE)
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_COMPOSITOR_SURFACE_DIMENSION, NATIVE_VERSION, is_known_role, renderer_slot_id,
        validate_compositor_dimensions,
    };

    #[test]
    fn renderer_slots_map_to_u64_ids_within_eight() {
        assert_eq!(renderer_slot_id(0), Ok(0));
        assert_eq!(renderer_slot_id(7), Ok(7));
        assert!(renderer_slot_id(8).is_err());
        assert!(renderer_slot_id(-1).is_err());
    }

    #[test]
    fn only_declared_process_roles_are_accepted() {
        assert!(is_known_role("renderer"));
        assert!(is_known_role("compositor"));
        assert!(is_known_role("image-decoder"));
        assert!(!is_known_role("browser"));
        assert!(!is_known_role("renderer0"));
    }

    #[test]
    fn compositor_dimensions_reject_out_of_bounds() {
        assert!(validate_compositor_dimensions(0, 100).is_err());
        assert!(validate_compositor_dimensions(100, 0).is_err());
        assert!(validate_compositor_dimensions(-1, 100).is_err());
        assert!(validate_compositor_dimensions(100, -1).is_err());
        assert!(validate_compositor_dimensions(MAX_COMPOSITOR_SURFACE_DIMENSION as i32 + 1, 100).is_err());
        assert!(validate_compositor_dimensions(100, MAX_COMPOSITOR_SURFACE_DIMENSION as i32 + 1).is_err());
    }

    #[test]
    fn compositor_dimensions_accept_bounded_size_and_compute_rgba_len() {
        assert_eq!(validate_compositor_dimensions(1, 1), Ok((1, 1, 4)));
        assert_eq!(
            validate_compositor_dimensions(
                MAX_COMPOSITOR_SURFACE_DIMENSION as i32,
                MAX_COMPOSITOR_SURFACE_DIMENSION as i32
            ),
            Ok((
                MAX_COMPOSITOR_SURFACE_DIMENSION,
                MAX_COMPOSITOR_SURFACE_DIMENSION,
                4_096 * 4_096 * 4
            ))
        );
    }

    #[test]
    fn native_version_exposes_product_prefix() {
        assert!(NATIVE_VERSION.starts_with("ZeroWeb Android"));
    }
}
