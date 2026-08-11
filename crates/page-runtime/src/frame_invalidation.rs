//! Page frame invalidation and event-transaction coalescing.

/// Composable page-frame invalidation flags.
///
/// Flags are expanded when inserted so callers cannot accidentally request a
/// layout without also repainting and publishing the resulting frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameInvalidation(u8);

impl FrameInvalidation {
    /// No frame work is pending.
    pub const CLEAN: Self = Self(0);
    /// Recompute computed style.
    pub const NEEDS_STYLE: Self = Self(1 << 0);
    /// Recompute layout geometry.
    pub const NEEDS_LAYOUT: Self = Self(1 << 1);
    /// Re-record paint primitives.
    pub const NEEDS_PAINT: Self = Self(1 << 2);
    /// Rebuild compositor state without page layout.
    pub const NEEDS_COMPOSITE: Self = Self(1 << 3);
    /// Publish the newest frame to the browser/compositor.
    pub const NEEDS_PUBLISH: Self = Self(1 << 4);
    /// Rebuild the hit-test snapshot.
    pub const NEEDS_HIT_TEST: Self = Self(1 << 5);

    /// Returns whether no work is pending.
    pub fn is_clean(self) -> bool {
        self.0 == 0
    }

    /// Returns whether all `other` work is pending.
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Adds work and all work implied by it.
    pub fn insert(&mut self, other: Self) {
        self.0 |= Self::expanded(other.0);
    }

    fn expanded(mut bits: u8) -> u8 {
        if bits & Self::NEEDS_STYLE.0 != 0 {
            bits |= Self::NEEDS_LAYOUT.0;
        }
        if bits & Self::NEEDS_LAYOUT.0 != 0 {
            bits |= Self::NEEDS_PAINT.0 | Self::NEEDS_HIT_TEST.0;
        }
        if bits & (Self::NEEDS_PAINT.0 | Self::NEEDS_COMPOSITE.0 | Self::NEEDS_HIT_TEST.0) != 0 {
            bits |= Self::NEEDS_PUBLISH.0;
        }
        bits
    }
}

/// Coalesces invalidations produced by one platform event and its callbacks.
#[derive(Debug, Default)]
pub struct FrameTransaction {
    depth: u16,
    pending: FrameInvalidation,
}

impl FrameTransaction {
    /// Starts an event transaction. Transactions may be nested.
    pub fn begin(&mut self) {
        self.depth = self.depth.saturating_add(1);
    }

    /// Returns whether a transaction is active.
    pub fn is_active(&self) -> bool {
        self.depth != 0
    }

    /// Adds invalidation work to the active transaction.
    pub fn invalidate(&mut self, invalidation: FrameInvalidation) {
        self.pending.insert(invalidation);
    }

    /// Finishes one nesting level and returns coalesced work at the outer edge.
    pub fn finish(&mut self) -> Option<FrameInvalidation> {
        if self.depth == 0 {
            return None;
        }
        self.depth -= 1;
        if self.depth != 0 {
            return None;
        }
        Some(std::mem::take(&mut self.pending))
    }

    /// Discards pending work while preserving the current nesting depth.
    ///
    /// Navigation uses this when an event replaces the active document before
    /// the transaction reaches its publish edge.
    pub fn discard_pending(&mut self) {
        self.pending = FrameInvalidation::CLEAN;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_implies_paint_hit_test_and_publish() {
        let mut invalidation = FrameInvalidation::CLEAN;
        invalidation.insert(FrameInvalidation::NEEDS_LAYOUT);

        assert!(invalidation.contains(FrameInvalidation::NEEDS_LAYOUT));
        assert!(invalidation.contains(FrameInvalidation::NEEDS_PAINT));
        assert!(invalidation.contains(FrameInvalidation::NEEDS_HIT_TEST));
        assert!(invalidation.contains(FrameInvalidation::NEEDS_PUBLISH));
        assert!(!invalidation.contains(FrameInvalidation::NEEDS_STYLE));
    }

    #[test]
    fn paint_only_does_not_upgrade_to_layout() {
        let mut invalidation = FrameInvalidation::CLEAN;
        invalidation.insert(FrameInvalidation::NEEDS_PAINT);

        assert!(invalidation.contains(FrameInvalidation::NEEDS_PAINT));
        assert!(invalidation.contains(FrameInvalidation::NEEDS_PUBLISH));
        assert!(!invalidation.contains(FrameInvalidation::NEEDS_LAYOUT));
    }

    #[test]
    fn nested_event_transaction_flushes_once_with_highest_invalidation() {
        let mut transaction = FrameTransaction::default();
        transaction.begin();
        transaction.invalidate(FrameInvalidation::NEEDS_PAINT);
        transaction.begin();
        transaction.invalidate(FrameInvalidation::NEEDS_STYLE);

        assert_eq!(transaction.finish(), None);
        let work = transaction.finish().expect("outer transaction flush");
        assert!(work.contains(FrameInvalidation::NEEDS_STYLE));
        assert!(work.contains(FrameInvalidation::NEEDS_LAYOUT));
        assert!(work.contains(FrameInvalidation::NEEDS_PUBLISH));
        assert!(transaction.finish().is_none());
    }

    #[test]
    fn navigation_can_discard_stale_work_without_breaking_nesting() {
        let mut transaction = FrameTransaction::default();
        transaction.begin();
        transaction.invalidate(FrameInvalidation::NEEDS_PAINT);
        transaction.discard_pending();

        assert_eq!(transaction.finish(), Some(FrameInvalidation::CLEAN));
    }
}
