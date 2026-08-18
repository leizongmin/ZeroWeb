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
