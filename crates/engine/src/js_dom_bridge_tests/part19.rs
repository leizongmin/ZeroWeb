// IndexedDB transaction scheduling regressions.

#[test]
fn test_indexeddb_transactions_schedule_across_connections() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__schedule = [];\
             var setup = indexedDB.open('connection-scheduling', 1);\
             setup.onupgradeneeded = function () { setup.result.createObjectStore('items'); };\
             setup.onsuccess = function () {\
               var first = setup.result;\
               var secondOpen = indexedDB.open(first.name);\
               secondOpen.onsuccess = function () {\
                 var second = secondOpen.result;\
                 var write = first.transaction('items', 'readwrite');\
                 var read = second.transaction('items', 'readonly');\
                 write.objectStore('items').put('new', 'key').onsuccess = function () {\
                   __schedule.push('write');\
                 };\
                 write.oncomplete = function () { __schedule.push('write-complete'); };\
                 read.objectStore('items').get('key').onsuccess = function (event) {\
                   __schedule.push('read:' + event.target.result);\
                 };\
                 read.oncomplete = function () { __schedule.push('read-complete'); };\
               };\
             };",
        )
        .unwrap();
    sandbox.execute("0").unwrap();

    assert_eq!(
        sandbox.execute("__schedule.join('|')").unwrap().value,
        "write|write-complete|read:new|read-complete"
    );
}

#[test]
fn test_indexeddb_deferred_operations_wait_for_transaction_start() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__deferred = [];\
             var setup = indexedDB.open('deferred-operations', 1);\
             setup.onupgradeneeded = function () {\
               var store = setup.result.createObjectStore('items');\
               store.createIndex('byValue', 'value');\
             };\
             setup.onsuccess = function () {\
               var db = setup.result;\
               var blocker = db.transaction('items', 'readwrite');\
               blocker.objectStore('items').put({value:'new'}, 'key');\
               blocker.oncomplete = function () { __deferred.push('blocker-complete'); };\
               var queued = db.transaction('items', 'readwrite');\
               var store = queued.objectStore('items');\
               var index = store.index('byValue');\
               store.delete('missing').onsuccess = function () { __deferred.push('delete'); };\
               store.clear().onsuccess = function () { __deferred.push('clear'); };\
               store.count().onsuccess = function () { __deferred.push('count'); };\
               store.getAll().onsuccess = function () { __deferred.push('getAll'); };\
               index.get('new').onsuccess = function () { __deferred.push('index-get'); };\
               index.count().onsuccess = function () { __deferred.push('index-count'); };\
               index.getAll().onsuccess = function () { __deferred.push('index-getAll'); };\
               store.openCursor().onsuccess = function () { __deferred.push('store-cursor'); };\
               index.openCursor().onsuccess = function () { __deferred.push('index-cursor'); };\
             };",
        )
        .unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox.execute("__deferred.join('|')").unwrap().value,
        "blocker-complete|delete|clear|count|getAll|index-get|index-count|index-getAll|store-cursor|index-cursor"
    );
}

#[test]
fn test_indexeddb_detached_binary_keys_throw_data_error() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "function detachKey() {\
               var view = new Uint8Array([1, 2, 3, 4]);\
               var channel = new MessageChannel();\
               channel.port1.postMessage('', [view.buffer]);\
               return view;\
             }\
             var detachedView = detachKey();\
             var detachedBuffer = detachKey().buffer;\
             try { indexedDB.cmp(detachedView, 1); }\
             catch (error) { globalThis.__viewError = error.name; }\
             try { indexedDB.cmp(detachedBuffer, 1); }\
             catch (error) { globalThis.__bufferError = error.name; }",
        )
        .unwrap();

    assert_eq!(
        sandbox
            .execute("String(globalThis.__viewError) + '|' + String(globalThis.__bufferError)")
            .unwrap()
            .value,
        "DataError|DataError"
    );
}

#[test]
fn test_indexeddb_get_key_and_cursor_mutations() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__cursorMutations = [];\
             var setup = indexedDB.open('cursor-mutations', 1);\
             setup.onupgradeneeded = function () {\
               setup.result.createObjectStore('items', {keyPath:'id'});\
             };\
             setup.onsuccess = function () {\
               var db = setup.result;\
               var seed = db.transaction('items', 'readwrite');\
               seed.objectStore('items').put({id:1, value:'a'});\
               seed.objectStore('items').put({id:2, value:'b'});\
               seed.oncomplete = function () {\
                 var mutate = db.transaction('items', 'readwrite');\
                 var request = mutate.objectStore('items').openCursor();\
                 request.onsuccess = function () {\
                   var cursor = request.result;\
                   if (!cursor) return;\
                   if (cursor.primaryKey === 1) {\
                     cursor.update({id:1, value:'updated'});\
                     cursor.continue();\
                   } else {\
                     cursor.delete();\
                   }\
                 };\
                 mutate.oncomplete = function () {\
                   var verify = db.transaction('items', 'readonly');\
                   var store = verify.objectStore('items');\
                   store.getKey(IDBKeyRange.lowerBound(1)).onsuccess = function (event) {\
                     __cursorMutations.push('key:' + event.target.result);\
                   };\
                   store.get(1).onsuccess = function (event) {\
                     __cursorMutations.push('value:' + event.target.result.value);\
                   };\
                   store.get(2).onsuccess = function (event) {\
                     __cursorMutations.push('deleted:' + String(event.target.result));\
                   };\
                   store.openKeyCursor().onsuccess = function (event) {\
                     __cursorMutations.push('key-only:' + String(event.target.result.value));\
                   };\
                 };\
               };\
             };",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox.execute("__cursorMutations.join('|')").unwrap().value,
        "key:1|value:updated|deleted:undefined|key-only:undefined"
    );
}

#[test]
fn test_indexeddb_blocked_upgrade_waits_for_connection_close() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__connectionEvents = [];\
             var initial = indexedDB.open('blocked-upgrade', 1);\
             initial.onsuccess = function () {\
               var db = initial.result;\
               db.onversionchange = function () { __connectionEvents.push('versionchange'); };\
               var upgrade = indexedDB.open(db.name, 2);\
               upgrade.onblocked = function () {\
                 __connectionEvents.push('blocked');\
                 db.close();\
               };\
               upgrade.onupgradeneeded = function () {\
                 __connectionEvents.push('upgrade');\
               };\
               upgrade.onsuccess = function () {\
                 __connectionEvents.push('success');\
                 upgrade.result.close();\
               };\
             };",
        )
        .unwrap();
    sandbox.execute("0").unwrap();

    assert_eq!(
        sandbox.execute("__connectionEvents.join('|')").unwrap().value,
        "versionchange|blocked|upgrade|success"
    );
}
