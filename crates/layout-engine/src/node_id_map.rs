//! Hash collections specialized for trusted DOM slot-map keys.

use std::collections::hash_map::{DefaultHasher, RandomState};
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};
use std::sync::LazyLock;

use zero_dom::NodeId;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// A [`HashMap`] keyed by trusted DOM node identifiers.
pub type NodeIdMap<V> = HashMap<NodeId, V, NodeIdBuildHasher>;

/// A [`HashSet`] containing trusted DOM node identifiers.
pub type NodeIdSet = HashSet<NodeId, NodeIdBuildHasher>;

/// Selects fast `NodeId` mixing or the standard randomized fallback.
///
/// Must stay ≤ 8 bytes: `NodeIdMap` fields live inside every `LayoutBox`, and an
/// oversized hasher inflates `LayoutBox`, slowing the compositing walk by ~25%
/// (R3441-F regression, ZRG-2026-08-18-01). Kept to a mode tag only — the FNV
/// seed is a constant and the fallback `RandomState` is shared.
#[derive(Clone)]
pub struct NodeIdBuildHasher(BuildHasherMode);

#[derive(Clone)]
enum BuildHasherMode {
    Fast,
    Random,
}

/// Shared randomized state for the fallback mode (avoids per-map allocation).
static RANDOM_STATE: LazyLock<RandomState> = LazyLock::new(RandomState::new);

impl Default for NodeIdBuildHasher {
    fn default() -> Self {
        static DIRECT: LazyLock<bool> = LazyLock::new(|| std::env::var("ZW_NODE_ID_FAST_HASH").as_deref() != Ok("0"));
        if *DIRECT {
            Self(BuildHasherMode::Fast)
        } else {
            Self(BuildHasherMode::Random)
        }
    }
}

impl BuildHasher for NodeIdBuildHasher {
    type Hasher = NodeIdHasher;

    fn build_hasher(&self) -> Self::Hasher {
        match &self.0 {
            BuildHasherMode::Fast => NodeIdHasher(NodeIdHasherMode::Fast(FNV_OFFSET)),
            BuildHasherMode::Random => NodeIdHasher(NodeIdHasherMode::Random(RANDOM_STATE.build_hasher())),
        }
    }
}

/// Hashes the single `u64` emitted by slotmap's `NodeId`.
pub struct NodeIdHasher(NodeIdHasherMode);

enum NodeIdHasherMode {
    Fast(u64),
    Random(DefaultHasher),
}

impl Hasher for NodeIdHasher {
    fn finish(&self) -> u64 {
        match &self.0 {
            NodeIdHasherMode::Fast(hash) => *hash,
            NodeIdHasherMode::Random(hasher) => hasher.finish(),
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        match &mut self.0 {
            NodeIdHasherMode::Fast(hash) => {
                for byte in bytes {
                    *hash ^= u64::from(*byte);
                    *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
            NodeIdHasherMode::Random(hasher) => hasher.write(bytes),
        }
    }

    fn write_u64(&mut self, value: u64) {
        match &mut self.0 {
            NodeIdHasherMode::Fast(hash) => *hash = mix_node_id(*hash ^ value),
            NodeIdHasherMode::Random(hasher) => hasher.write_u64(value),
        }
    }
}

#[inline]
fn mix_node_id(value: u64) -> u64 {
    let mut mixed = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^ (mixed >> 31)
}

#[cfg(test)]
mod tests {
    use super::{NodeIdBuildHasher, NodeIdMap, NodeIdSet};
    use std::collections::HashMap;
    use zero_dom::{Document, NodeId};

    #[test]
    fn specialized_collections_distinguish_and_overwrite_node_ids() {
        let mut doc = Document::new();
        let first = doc.create_element("div");
        let second = doc.create_element("span");

        let mut map = NodeIdMap::default();
        map.insert(first, 1);
        map.insert(second, 2);
        map.insert(first, 3);
        assert_eq!(map.get(&first), Some(&3));
        assert_eq!(map.get(&second), Some(&2));

        let set = NodeIdSet::from_iter([first, second, first]);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn hasher_size_does_not_inflate_layoutbox() {
        // ZRG-2026-08-18-01: R3441-F's oversized hasher (24 B) grew every
        // LayoutBox by 80 B and slowed the compositing walk ~25%. The hasher
        // must stay small enough that NodeIdMap is no larger than a plain
        // HashMap, which is what layout performance was measured against.
        use std::collections::hash_map::RandomState;
        assert!(std::mem::size_of::<NodeIdBuildHasher>() <= std::mem::size_of::<RandomState>());
        assert!(
            std::mem::size_of::<NodeIdMap<f32>>() <= std::mem::size_of::<HashMap<NodeId, f32, RandomState>>(),
            "NodeIdMap must not be larger than a plain HashMap"
        );
    }
}
