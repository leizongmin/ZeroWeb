use super::*;

fn call(handler: &IndexedDbHandler, origin: &str, request: Value) -> Result<Value, String> {
    handler(origin, &request.to_string())
        .and_then(|response| serde_json::from_str(&response).map_err(|error| format!("invalid test response: {error}")))
}

fn key(value: u64) -> Value {
    json!({"type": "number", "value": value.to_string()})
}

fn open_cursor(handler: &IndexedDbHandler, transaction: u64, index: Option<&str>, direction: &str) -> Value {
    let mut request = json!({
        "op": "transaction_open_cursor",
        "transaction": transaction,
        "store": "items",
        "direction": direction
    });
    if let Some(index) = index {
        request["index"] = json!(index);
    }
    call(handler, "https://app.example", request).unwrap()
}

#[test]
fn transaction_cursor_continues_to_index_and_primary_key_pair() {
    let handler = indexed_db_handler(Arc::new(Mutex::new(StorageManager::new())));
    call(
        &handler,
        "https://app.example",
        json!({
            "op": "sync_schema",
            "name": "app",
            "version": 1,
            "stores": [{
                "name": "items",
                "keyPath": null,
                "autoIncrement": false,
                "indexes": [{"name": "by_group", "keyPath": "group"}]
            }]
        }),
    )
    .unwrap();
    let transaction = call(
        &handler,
        "https://app.example",
        json!({
            "op": "begin_transaction",
            "database": "app",
            "stores": ["items"],
            "mode": "readwrite"
        }),
    )
    .unwrap()["transaction"]
        .as_u64()
        .unwrap();
    for (primary_key, group) in [(1, "a"), (2, "a"), (3, "b"), (4, "b")] {
        call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_add",
                "transaction": transaction,
                "store": "items",
                "value": {"group": group},
                "key": key(primary_key)
            }),
        )
        .unwrap();
    }

    let next = open_cursor(&handler, transaction, Some("by_group"), "next");
    let next_cursor = next["cursor"].as_u64().unwrap();
    let same_index_key = call(
        &handler,
        "https://app.example",
        json!({
            "op": "transaction_cursor_continue_primary_key",
            "transaction": transaction,
            "cursor": next_cursor,
            "key": {"type": "string", "value": "a"},
            "primary_key": key(2)
        }),
    )
    .unwrap();
    assert_eq!(same_index_key["entry"]["primaryKey"], key(2));
    let next_index_key = call(
        &handler,
        "https://app.example",
        json!({
            "op": "transaction_cursor_continue_primary_key",
            "transaction": transaction,
            "cursor": next_cursor,
            "key": {"type": "string", "value": "b"},
            "primary_key": key(1)
        }),
    )
    .unwrap();
    assert_eq!(next_index_key["entry"]["primaryKey"], key(3));

    let prev = open_cursor(&handler, transaction, Some("by_group"), "prev");
    let prev_cursor = prev["cursor"].as_u64().unwrap();
    let previous = call(
        &handler,
        "https://app.example",
        json!({
            "op": "transaction_cursor_continue_primary_key",
            "transaction": transaction,
            "cursor": prev_cursor,
            "key": {"type": "string", "value": "a"},
            "primary_key": key(9)
        }),
    )
    .unwrap();
    assert_eq!(previous["entry"]["primaryKey"], key(2));

    let invalid_pair = call(
        &handler,
        "https://app.example",
        json!({
            "op": "transaction_cursor_continue_primary_key",
            "transaction": transaction,
            "cursor": prev_cursor,
            "key": {"type": "string", "value": "b"},
            "primary_key": key(4)
        }),
    )
    .unwrap_err();
    assert!(invalid_pair.starts_with("DataError:"));

    for cursor in [
        open_cursor(&handler, transaction, None, "next")["cursor"]
            .as_u64()
            .unwrap(),
        open_cursor(&handler, transaction, Some("by_group"), "nextunique")["cursor"]
            .as_u64()
            .unwrap(),
    ] {
        let error = call(
            &handler,
            "https://app.example",
            json!({
                "op": "transaction_cursor_continue_primary_key",
                "transaction": transaction,
                "cursor": cursor,
                "key": {"type": "string", "value": "b"},
                "primary_key": key(4)
            }),
        )
        .unwrap_err();
        assert!(error.starts_with("InvalidAccessError:"));
    }
}
