use serde::Deserialize;
use serde_json::{Value, json};
use zero_storage::{IdbKey, StorageManager};

use super::{
    IndexedDbKeyWire, IndexedDbQueryWire, IndexedDbTransactionRegistry, active_database_mut, active_transaction_mut,
    storage_error,
};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum IndexedDbCursorDirection {
    Next,
    Nextunique,
    Prev,
    Prevunique,
}

impl IndexedDbCursorDirection {
    fn is_reverse(self) -> bool {
        matches!(self, Self::Prev | Self::Prevunique)
    }

    fn is_unique(self) -> bool {
        matches!(self, Self::Nextunique | Self::Prevunique)
    }
}

pub(super) struct ActiveIndexedDbCursor {
    direction: IndexedDbCursorDirection,
    entries: Vec<IndexedDbCursorEntry>,
    position: usize,
}

struct IndexedDbCursorEntry {
    key: IdbKey,
    primary_key: IdbKey,
    value: Option<Value>,
}

pub(super) enum CursorStep {
    Continue(Option<IdbKey>),
    Advance(usize),
}

// https://w3c.github.io/IndexedDB/#iterate-a-cursor
#[allow(clippy::too_many_arguments)]
pub(super) fn open_transaction_cursor(
    storage: &mut StorageManager,
    transactions: &mut IndexedDbTransactionRegistry,
    origin: &str,
    transaction: u64,
    store: &str,
    index: Option<&str>,
    query: Option<IndexedDbQueryWire>,
    direction: IndexedDbCursorDirection,
    key_only: bool,
) -> Result<Value, String> {
    let active = active_transaction_mut(transactions, origin, transaction)?;
    let query = query.map(IndexedDbQueryWire::into_storage_query).transpose()?;
    let database = active_database_mut(storage, active)?;
    let mut entries = if let Some(index) = index {
        database
            .tx_get_all_from_index(&active.transaction, store, index)
            .map_err(storage_error)?
            .into_iter()
            .filter(|entry| query.as_ref().is_none_or(|query| query.matches(&entry.index_key)))
            .map(|entry| IndexedDbCursorEntry {
                key: entry.index_key,
                primary_key: entry.primary_key,
                value: (!key_only).then_some(entry.value),
            })
            .collect::<Vec<_>>()
    } else {
        database
            .tx_get_all(&active.transaction, store)
            .map_err(storage_error)?
            .into_iter()
            .filter(|record| query.as_ref().is_none_or(|query| query.matches(&record.key)))
            .map(|record| IndexedDbCursorEntry {
                primary_key: record.key.clone(),
                key: record.key,
                value: (!key_only).then_some(record.value),
            })
            .collect::<Vec<_>>()
    };
    if direction.is_reverse() {
        entries.reverse();
    }
    if direction.is_unique() {
        entries.dedup_by(|next, current| next.key == current.key);
    }
    if entries.is_empty() {
        return Ok(json!({"cursor": null, "entry": null}));
    }

    active.next_cursor_id = active
        .next_cursor_id
        .checked_add(1)
        .ok_or_else(|| "UnknownError: IndexedDB cursor id overflow".to_string())?;
    let cursor_id = active.next_cursor_id;
    let entry = cursor_entry_json(&entries[0]);
    active.cursors.insert(
        cursor_id,
        ActiveIndexedDbCursor {
            direction,
            entries,
            position: 0,
        },
    );
    Ok(json!({"cursor": cursor_id, "entry": entry}))
}

// https://w3c.github.io/IndexedDB/#dom-idbcursor-continue
// https://w3c.github.io/IndexedDB/#dom-idbcursor-advance
pub(super) fn step_transaction_cursor(
    transactions: &mut IndexedDbTransactionRegistry,
    origin: &str,
    transaction: u64,
    cursor: u64,
    step: CursorStep,
) -> Result<Value, String> {
    let active = active_transaction_mut(transactions, origin, transaction)?;
    let cursor = active
        .cursors
        .get_mut(&cursor)
        .ok_or_else(|| "InvalidStateError: IndexedDB cursor does not exist".to_string())?;
    if cursor.position >= cursor.entries.len() {
        return Err("InvalidStateError: IndexedDB cursor has no current value".to_string());
    }

    let next_position = match step {
        CursorStep::Advance(count) => cursor.position.saturating_add(count),
        CursorStep::Continue(None) => cursor.position.saturating_add(1),
        CursorStep::Continue(Some(key)) => {
            let current = &cursor.entries[cursor.position].key;
            let valid = if cursor.direction.is_reverse() {
                key < *current
            } else {
                key > *current
            };
            if !valid {
                return Err("DataError: cursor continue key must move in its direction".to_string());
            }
            cursor
                .entries
                .iter()
                .enumerate()
                .skip(cursor.position.saturating_add(1))
                .find(|(_, entry)| {
                    if cursor.direction.is_reverse() {
                        entry.key <= key
                    } else {
                        entry.key >= key
                    }
                })
                .map_or(cursor.entries.len(), |(position, _)| position)
        }
    };
    cursor.position = next_position;
    let entry = cursor.entries.get(cursor.position).map(cursor_entry_json);
    Ok(json!({"entry": entry}))
}

fn cursor_entry_json(entry: &IndexedDbCursorEntry) -> Value {
    let mut object = serde_json::Map::from_iter([
        ("key".to_string(), json!(IndexedDbKeyWire::from(&entry.key))),
        (
            "primaryKey".to_string(),
            json!(IndexedDbKeyWire::from(&entry.primary_key)),
        ),
    ]);
    if let Some(value) = &entry.value {
        object.insert("value".to_string(), value.clone());
    }
    Value::Object(object)
}
