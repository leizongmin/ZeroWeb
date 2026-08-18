use serde::Deserialize;
use serde_json::{Value, json};
use zero_storage::{IdbKey, StorageManager};

use super::{
    IndexedDbKeyWire, IndexedDbQuery, IndexedDbQueryWire, IndexedDbTransactionRegistry, active_database_mut,
    active_transaction_mut, storage_error,
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
    index_cursor: bool,
    store: String,
    index: Option<String>,
    query: Option<IndexedDbQuery>,
    key_only: bool,
    entries: Vec<IndexedDbCursorEntry>,
    position: usize,
}

#[derive(Clone)]
struct IndexedDbCursorEntry {
    key: IdbKey,
    primary_key: IdbKey,
    value: Option<Value>,
}

pub(super) enum CursorStep {
    Continue(Option<IdbKey>),
    ContinuePrimaryKey(IdbKey, IdbKey),
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
    let mut entries = collect_cursor_entries(database, &active.transaction, store, index, query.as_ref(), key_only)?;
    normalize_cursor_entries(&mut entries, direction);
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
            index_cursor: index.is_some(),
            store: store.to_string(),
            index: index.map(str::to_string),
            query,
            key_only,
            entries,
            position: 0,
        },
    );
    Ok(json!({"cursor": cursor_id, "entry": entry}))
}

fn collect_cursor_entries(
    database: &mut zero_storage::IdbDatabase,
    transaction: &zero_storage::IdbTransaction,
    store: &str,
    index: Option<&str>,
    query: Option<&IndexedDbQuery>,
    key_only: bool,
) -> Result<Vec<IndexedDbCursorEntry>, String> {
    Ok(if let Some(index) = index {
        database
            .tx_get_all_from_index(transaction, store, index)
            .map_err(storage_error)?
            .into_iter()
            .filter(|entry| query.is_none_or(|query| query.matches(&entry.index_key)))
            .map(|entry| IndexedDbCursorEntry {
                key: entry.index_key,
                primary_key: entry.primary_key,
                value: (!key_only).then_some(entry.value),
            })
            .collect::<Vec<_>>()
    } else {
        database
            .tx_get_all(transaction, store)
            .map_err(storage_error)?
            .into_iter()
            .filter(|record| query.is_none_or(|query| query.matches(&record.key)))
            .map(|record| IndexedDbCursorEntry {
                primary_key: record.key.clone(),
                key: record.key,
                value: (!key_only).then_some(record.value),
            })
            .collect::<Vec<_>>()
    })
}

fn normalize_cursor_entries(entries: &mut Vec<IndexedDbCursorEntry>, direction: IndexedDbCursorDirection) {
    if direction.is_unique() {
        entries.dedup_by(|next, current| next.key == current.key);
    }
    if direction.is_reverse() {
        entries.reverse();
    }
}

// https://w3c.github.io/IndexedDB/#dom-idbcursor-continue
// https://w3c.github.io/IndexedDB/#dom-idbcursor-advance
pub(super) fn step_transaction_cursor(
    storage: &mut StorageManager,
    transactions: &mut IndexedDbTransactionRegistry,
    origin: &str,
    transaction: u64,
    cursor: u64,
    step: CursorStep,
) -> Result<Value, String> {
    let active = active_transaction_mut(transactions, origin, transaction)?;
    let cursor_state = active
        .cursors
        .get(&cursor)
        .ok_or_else(|| "InvalidStateError: IndexedDB cursor does not exist".to_string())?;
    if cursor_state.position >= cursor_state.entries.len() {
        return Err("InvalidStateError: IndexedDB cursor has no current value".to_string());
    }
    let current = cursor_state.entries[cursor_state.position].clone();
    let direction = cursor_state.direction;
    let index_cursor = cursor_state.index_cursor;
    let store = cursor_state.store.clone();
    let index = cursor_state.index.clone();
    let query = cursor_state.query.clone();
    let key_only = cursor_state.key_only;
    let database = active_database_mut(storage, active)?;
    let mut entries = collect_cursor_entries(
        database,
        &active.transaction,
        &store,
        index.as_deref(),
        query.as_ref(),
        key_only,
    )?;
    normalize_cursor_entries(&mut entries, direction);

    let after_current = |entry: &IndexedDbCursorEntry| {
        let compared = if index_cursor {
            (&entry.key, &entry.primary_key).cmp(&(&current.key, &current.primary_key))
        } else {
            entry.key.cmp(&current.key)
        };
        if direction.is_reverse() {
            compared.is_lt()
        } else {
            compared.is_gt()
        }
    };
    let next_position = match step {
        CursorStep::Advance(count) => entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| after_current(entry))
            .nth(count.saturating_sub(1))
            .map_or(entries.len(), |(position, _)| position),
        CursorStep::Continue(None) => entries.iter().position(after_current).unwrap_or(entries.len()),
        CursorStep::Continue(Some(key)) => {
            let valid = if direction.is_reverse() {
                key < current.key
            } else {
                key > current.key
            };
            if !valid {
                return Err("DataError: cursor continue key must move in its direction".to_string());
            }
            entries
                .iter()
                .enumerate()
                .find(|(_, entry)| {
                    if direction.is_reverse() {
                        entry.key <= key
                    } else {
                        entry.key >= key
                    }
                })
                .map_or(entries.len(), |(position, _)| position)
        }
        CursorStep::ContinuePrimaryKey(key, primary_key) => {
            if !index_cursor || direction.is_unique() {
                return Err("InvalidAccessError: continuePrimaryKey requires a non-unique index cursor".to_string());
            }
            let target = (&key, &primary_key);
            let current_pair = (&current.key, &current.primary_key);
            let valid = if direction.is_reverse() {
                target < current_pair
            } else {
                target > current_pair
            };
            if !valid {
                return Err("DataError: cursor continue primary key must move in its direction".to_string());
            }
            entries
                .iter()
                .enumerate()
                .find(|(_, entry)| {
                    let pair = (&entry.key, &entry.primary_key);
                    if direction.is_reverse() {
                        pair <= target
                    } else {
                        pair >= target
                    }
                })
                .map_or(entries.len(), |(position, _)| position)
        }
    };
    let cursor = active.cursors.get_mut(&cursor).expect("IndexedDB cursor checked above");
    cursor.entries = entries;
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
