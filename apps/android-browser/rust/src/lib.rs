//! Android JNI entry points for the ZeroWeb browser host.

mod facade;

use jni::JNIEnv;
use jni::objects::{JClass, JString};
#[cfg(target_os = "android")]
use jni::sys::jbyteArray;
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jstring};

#[cfg(target_os = "android")]
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "android")]
use zero_protocol::CompositorUiSurfaceInfo;
#[cfg(target_os = "android")]
use zero_protocol::IpcChannel;
#[cfg(target_os = "android")]
use zero_protocol::message::{ImageDecodeParams, IpcMessage, IpcMessageKind};

const NATIVE_VERSION: &str = "ZeroWeb Android M2";

#[cfg(target_os = "android")]
const ANDROID_COMPOSITOR_SURFACE_ID: u64 = 1;
#[cfg(target_os = "android")]
const MAX_COMPOSITOR_SURFACE_DIMENSION: u32 = 4_096;
#[cfg(target_os = "android")]
type AndroidCompositorTransport =
    zero_protocol::PipeTransport<std::os::unix::net::UnixStream, std::os::unix::net::UnixStream>;
#[cfg(target_os = "android")]
static ANDROID_COMPOSITOR: OnceLock<Mutex<Option<AndroidCompositorTransport>>> = OnceLock::new();

#[cfg(target_os = "android")]
fn android_compositor() -> &'static Mutex<Option<AndroidCompositorTransport>> {
    ANDROID_COMPOSITOR.get_or_init(|| Mutex::new(None))
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
        .and_then(|url| facade::navigate(url.to_str().map_err(|error| error.to_string())?))
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
    u64::try_from(id)
        .map_err(|_| "tab ID must be non-negative".to_string())
        .and_then(facade::close_tab)
        .map_or(JNI_FALSE, |_| JNI_TRUE)
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
/// Decoder and compositor already use their shared Rust role loops. Renderer
/// keeps its Service topology while its Android transport adapter is completed
/// in a subsequent M1 slice.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_leizm_zeroweb_NativeBridge_nativeRunRole(
    mut env: JNIEnv,
    _class: JClass,
    role: JString,
    fd: jni::sys::jint,
) -> jboolean {
    let Ok(role) = env.get_string(&role) else {
        close_android_fd(fd);
        return JNI_FALSE;
    };
    match role.to_str().ok() {
        #[cfg(feature = "android-renderer")]
        Some("renderer") => std::thread::Builder::new()
            .name("android-renderer".to_string())
            .spawn(move || zero_renderer::run_android_role(0, fd))
            .map_or(JNI_FALSE, |_| JNI_TRUE),
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
    use super::is_known_role;

    #[test]
    fn only_declared_process_roles_are_accepted() {
        assert!(is_known_role("renderer"));
        assert!(is_known_role("compositor"));
        assert!(is_known_role("image-decoder"));
        assert!(!is_known_role("browser"));
        assert!(!is_known_role("renderer0"));
    }
}
