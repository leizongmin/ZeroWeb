//! Android JNI entry points for the ZeroWeb browser host.

mod facade;

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jstring};

#[cfg(target_os = "android")]
use zero_protocol::CompositorUiSurfaceInfo;
#[cfg(target_os = "android")]
use zero_protocol::IpcChannel;
#[cfg(target_os = "android")]
use zero_protocol::message::{ImageDecodeParams, IpcMessage, IpcMessageKind};

const NATIVE_VERSION: &str = "ZeroWeb Android M2";

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
