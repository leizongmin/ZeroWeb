//! IndexedDB types tests - testing public API.

use zero_storage::indexed_db::*;

#[test]
fn test_idb_key_array_comparisons() {
    // Test line 235 - Array key comparisons
    let key1 = IdbKey::Array(vec![IdbKey::String("a".to_string()), IdbKey::Number(1.0)]);
    let key2 = IdbKey::Array(vec![IdbKey::String("a".to_string()), IdbKey::Number(2.0)]);
    let key3 = IdbKey::Array(vec![IdbKey::String("b".to_string()), IdbKey::Number(1.0)]);

    assert!(key1 < key2); // Same first element, second element 1 < 2
    assert!(key1 < key3); // First element "a" < "b"
}

#[test]
fn test_idb_key_range_contains_edge_cases() {
    // Test lines 519-521 - Range contains with different key types
    let range = IdbKeyRange::bound(IdbKey::Number(10.0), IdbKey::String("z".to_string()), false, true);

    assert!(range.contains(&IdbKey::Number(10.0))); // Lower bound included
    assert!(!range.contains(&IdbKey::String("z".to_string()))); // Upper bound excluded
    assert!(range.contains(&IdbKey::Number(11.0))); // Between bounds
    assert!(!range.contains(&IdbKey::Number(9.0))); // Below lower bound
    assert!(!range.contains(&IdbKey::String("zz".to_string()))); // Above upper bound
}
