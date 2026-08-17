#[derive(Clone, Copy)]
pub(super) struct TreeRuntimeFlags {
    snapshot_enabled: bool,
    margin_trim: bool,
    br_lineheight: bool,
    content_visibility: bool,
    content_replace: bool,
    phasea_multi_inline: bool,
    br_inline_no_node: bool,
    inline_box_model_coherence: bool,
}

impl TreeRuntimeFlags {
    pub(super) fn from_env() -> Self {
        Self {
            snapshot_enabled: enabled("ZW_TREE_ENV_SNAPSHOT"),
            margin_trim: enabled("ZW_MARGIN_TRIM"),
            br_lineheight: enabled("ZW_BR_LINEHEIGHT"),
            content_visibility: enabled("ZW_CONTENT_VISIBILITY"),
            content_replace: enabled("ZW_CONTENT_REPLACE"),
            phasea_multi_inline: enabled("ZW_PHASEA_MULTI_INLINE"),
            br_inline_no_node: enabled("ZW_BR_INLINE_NO_NODE"),
            inline_box_model_coherence: enabled("ZW_INLINE_BOX_MODEL_COHERENCE"),
        }
    }

    pub(super) fn margin_trim(self) -> bool {
        self.value("ZW_MARGIN_TRIM", self.margin_trim)
    }

    pub(super) fn br_lineheight(self) -> bool {
        self.value("ZW_BR_LINEHEIGHT", self.br_lineheight)
    }

    pub(super) fn content_visibility(self) -> bool {
        self.value("ZW_CONTENT_VISIBILITY", self.content_visibility)
    }

    pub(super) fn content_replace(self) -> bool {
        self.value("ZW_CONTENT_REPLACE", self.content_replace)
    }

    pub(super) fn phasea_multi_inline(self) -> bool {
        self.value("ZW_PHASEA_MULTI_INLINE", self.phasea_multi_inline)
    }

    pub(super) fn br_inline_no_node(self) -> bool {
        self.value("ZW_BR_INLINE_NO_NODE", self.br_inline_no_node)
    }

    pub(super) fn inline_box_model_coherence(self) -> bool {
        self.value("ZW_INLINE_BOX_MODEL_COHERENCE", self.inline_box_model_coherence)
    }

    #[inline]
    fn value(self, name: &str, cached: bool) -> bool {
        select_value(self.snapshot_enabled, cached, || enabled(name))
    }
}

fn enabled(name: &str) -> bool {
    std::env::var(name).as_deref() != Ok("0")
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
