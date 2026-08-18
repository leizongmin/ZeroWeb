//! Process-lifetime IFC feature switches with a live rollback path.

use std::sync::LazyLock;

static SNAPSHOT_ENABLED: LazyLock<bool> = LazyLock::new(|| default_on("ZW_IFC_ENV_SNAPSHOT"));
static RESIDUAL_SNAPSHOT_ENABLED: LazyLock<bool> = LazyLock::new(|| default_on("ZW_LAYOUT_RESIDUAL_ENV_SNAPSHOT"));

pub(super) fn plaintext_line_direction() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_PLAINTEXT_LINE_DIRECTION"));
    selected(*VALUE, || default_on("ZW_PLAINTEXT_LINE_DIRECTION"))
}

pub(super) fn debug_ifc() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| std::env::var("ZW_DEBUG_IFC").is_ok());
    selected(*VALUE, || std::env::var("ZW_DEBUG_IFC").is_ok())
}

pub(super) fn br_ifc_line() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_BR_IFC_LINE"));
    selected(*VALUE, || default_on("ZW_BR_IFC_LINE"))
}

pub(super) fn skip_oof() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_IFC_SKIP_OOF"));
    selected(*VALUE, || default_on("ZW_IFC_SKIP_OOF"))
}

pub(super) fn line_clamp() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_LINE_CLAMP"));
    selected(*VALUE, || default_on("ZW_LINE_CLAMP"))
}

pub(super) fn inline_box_recurse() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_INLINE_BOX_RECURSE"));
    selected(*VALUE, || default_on("ZW_INLINE_BOX_RECURSE"))
}

pub(super) fn prewrap_hang() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_PREWRAP_HANG"));
    selected(*VALUE, || default_on("ZW_PREWRAP_HANG"))
}

pub(super) fn cjk_contiguous() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| opt_in("ZW_CJK_CONTIGUOUS"));
    selected(*VALUE, || opt_in("ZW_CJK_CONTIGUOUS"))
}

pub(super) fn bidi_fragment_source() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_BIDI_FRAGMENT_SOURCE"));
    selected(*VALUE, || default_on("ZW_BIDI_FRAGMENT_SOURCE"))
}

pub(super) fn bidi_mirroring() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_BIDI_MIRRORING"));
    selected(*VALUE, || default_on("ZW_BIDI_MIRRORING"))
}

pub(super) fn bidi_override() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_BIDI_OVERRIDE"));
    selected(*VALUE, || default_on("ZW_BIDI_OVERRIDE"))
}

pub(super) fn font_size_adjust_normal_line() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_FONT_SIZE_ADJUST_NORMAL_LINE"));
    residual_selected(*VALUE, || default_on("ZW_FONT_SIZE_ADJUST_NORMAL_LINE"))
}

pub(super) fn font_face_size_adjust_normal_line() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_FONT_FACE_SIZE_ADJUST_NORMAL_LINE"));
    residual_selected(*VALUE, || default_on("ZW_FONT_FACE_SIZE_ADJUST_NORMAL_LINE"))
}

pub(super) fn content_visibility() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| default_on("ZW_CONTENT_VISIBILITY"));
    residual_selected(*VALUE, || default_on("ZW_CONTENT_VISIBILITY"))
}

pub(super) fn shaped_advance_trace() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| opt_in("ZW_SHAPED_ADVANCE_TRACE"));
    residual_selected(*VALUE, || opt_in("ZW_SHAPED_ADVANCE_TRACE"))
}

pub(super) fn shaped_fallback() -> bool {
    static VALUE: LazyLock<bool> = LazyLock::new(|| opt_in("ZW_SHAPED_FALLBACK"));
    residual_selected(*VALUE, || opt_in("ZW_SHAPED_FALLBACK"))
}

fn default_on(name: &str) -> bool {
    std::env::var(name).as_deref() != Ok("0")
}

fn opt_in(name: &str) -> bool {
    std::env::var(name).as_deref() == Ok("1")
}

#[inline]
fn selected(cached: bool, live: impl FnOnce() -> bool) -> bool {
    select_value(*SNAPSHOT_ENABLED, cached, live)
}

#[inline]
fn residual_selected(cached: bool, live: impl FnOnce() -> bool) -> bool {
    select_value(*RESIDUAL_SNAPSHOT_ENABLED, cached, live)
}

#[inline]
fn select_value(snapshot_enabled: bool, cached: bool, live: impl FnOnce() -> bool) -> bool {
    if snapshot_enabled { cached } else { live() }
}

#[cfg(test)]
mod tests {
    use super::select_value;

    #[test]
    fn snapshot_skips_live_lookup_and_fallback_uses_it() {
        let calls = std::cell::Cell::new(0);
        assert!(select_value(true, true, || {
            calls.set(calls.get() + 1);
            false
        }));
        assert_eq!(calls.get(), 0);

        assert!(!select_value(false, true, || {
            calls.set(calls.get() + 1);
            false
        }));
        assert_eq!(calls.get(), 1);
    }
}
