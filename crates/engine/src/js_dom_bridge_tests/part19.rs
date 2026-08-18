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
