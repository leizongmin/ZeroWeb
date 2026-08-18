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
    assert_eq!(
        sandbox
            .execute(
                "['source','direction','key','primaryKey','request'].every(function (name) {\
                   var descriptor = Object.getOwnPropertyDescriptor(IDBCursor.prototype, name);\
                   return descriptor && typeof descriptor.get === 'function' && descriptor.set === undefined;\
                 })"
            )
            .unwrap()
            .value,
        "true"
    );
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
                     __cursorMutations.push('key-only-value:' + String('value' in event.target.result));\
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
        "key:1|value:updated|deleted:undefined|key-only-value:false"
    );
}

#[test]
fn test_indexeddb_cursor_stepping_guards_and_compound_keys() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__cursorStepping = [];\
             var setup = indexedDB.open('cursor-stepping-guards', 1);\
             setup.onupgradeneeded = function () {\
               var db = setup.result;\
               var compound = db.createObjectStore('compound', {keyPath:['id','nested.key']});\
               compound.createIndex('byCompound', ['id','nested.key']);\
               compound.add({id:'a', nested:{key:'b'}, value:'stored'});\
             };\
             var guardSetup = indexedDB.open('cursor-stepping-deleted', 1);\
             guardSetup.onupgradeneeded = function () {\
               var db = guardSetup.result;\
               var deleted = db.createObjectStore('deleted');\
               deleted.put('value', 'key');\
               var deletedCursor = deleted.openKeyCursor();\
               deletedCursor.onsuccess = function () {\
                 var cursor = deletedCursor.result;\
                 db.deleteObjectStore('deleted');\
                 guardSetup.transaction.abort();\
                 try { cursor.advance(0); } catch (error) { __cursorStepping.push('zero:' + error.name); }\
                 try { cursor.advance(1); } catch (error) { __cursorStepping.push('inactive:' + error.name); }\
               };\
             };\
             setup.onsuccess = function () {\
               var store = setup.result.transaction('compound').objectStore('compound');\
               store.get(['a','b']).onsuccess = function (event) {\
                 __cursorStepping.push('compound:' + event.target.result.value);\
               };\
               var request = store.index('byCompound').openCursor();\
               var continued = false;\
               request.onsuccess = function () {\
                 var cursor = request.result;\
                 if (!cursor || continued) return;\
                 continued = true;\
                 cursor.continue(undefined);\
                 __cursorStepping.push('undefined:ok');\
                 try { cursor.continue(); } catch (error) {\
                   __cursorStepping.push('iterating:' + error.name);\
                 }\
               };\
             };",
        )
        .unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox.execute("__cursorStepping.sort().join('|')").unwrap().value,
        "compound:stored|inactive:TransactionInactiveError|iterating:InvalidStateError|undefined:ok|zero:TypeError"
    );
}

#[test]
fn test_indexeddb_metadata_tasks_and_utf16_name_wire() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let captured = Arc::clone(&requests);
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.register_callback(
        "__zw_idb",
        Box::new(move |args: &[String]| {
            let request = args[0].clone();
            captured.lock().unwrap().push(request.clone());
            let response = if request.contains("\"op\":\"connection_capabilities\"") {
                r#"{"crossRenderer":false,"transactionScheduling":false}"#
            } else if request.contains("\"op\":\"inspect\"") {
                r#"{"database":null}"#
            } else if request.contains("\"op\":\"begin_transaction\"") {
                r#"{"transaction":1}"#
            } else {
                "{}"
            };
            format!("__zw_idb_ok:{response}")
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__metadataOrder = [];\
             var setup = indexedDB.open('metadata-wire', 1);\
             setup.onupgradeneeded = function () {\
               setup.result.createObjectStore('\\uD800');\
               Promise.resolve().then(function () { __metadataOrder.push('microtask'); });\
             };\
             setup.onsuccess = function () {\
               __metadataOrder.push('success');\
               setup.result.close();\
             };",
        )
        .unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox.execute("__metadataOrder.join('|')").unwrap().value,
        "microtask|success"
    );
    let requests = requests.lock().unwrap();
    let schema = requests
        .iter()
        .find(|request| request.contains("\"op\":\"sync_schema\""))
        .expect("schema request");
    assert!(schema.contains(r#""name":"__zw_utf16_name__:d800""#));
}

#[test]
fn test_indexeddb_keypath_extraction_edge_cases() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__keyPathEdges = [];\
             var setup = indexedDB.open('keypath-edges', 1);\
             setup.onupgradeneeded = function () {\
               var db = setup.result;\
               var whole = db.createObjectStore('whole', {keyPath:''});\
               whole.add('whole-key');\
               try { whole.createIndex('invalid', 'not valid'); }\
               catch (error) { __keyPathEdges.push('index:' + error.name); }\
               var generated = db.createObjectStore('generated', {keyPath:'a.b', autoIncrement:true});\
               var getterCount = 0;\
               Object.defineProperty(Object.prototype, 'a', {\
                 configurable:true,\
                 get:function () { getterCount++; throw new Error('unexpected getter'); }\
               });\
               var generatedRequest;\
               try { generatedRequest = generated.put({}); }\
               finally { delete Object.prototype.a; }\
               __keyPathEdges.push('getter:' + getterCount);\
               __keyPathEdges.push('request:' + Object.prototype.toString.call(generatedRequest));\
               var out = db.createObjectStore('out');\
               try { out.add('value', new Proxy([1], {})); }\
               catch (error) { __keyPathEdges.push('proxy:' + error.name); }\
               var files = db.createObjectStore('files', {keyPath:'name'});\
               files.put(new File(['x'], 'file.txt', {lastModified:123}));\
             };\
             setup.onsuccess = function () {\
               var db = setup.result;\
               db.transaction('whole').objectStore('whole').get('whole-key').onsuccess = function (event) {\
                 __keyPathEdges.push('whole:' + event.target.result);\
               };\
               db.transaction('generated').objectStore('generated').get(1).onsuccess = function (event) {\
                 __keyPathEdges.push('generated:' + event.target.result.a.b);\
               };\
               db.transaction('files').objectStore('files').get('file.txt').onsuccess = function (event) {\
                 __keyPathEdges.push('file:' + event.target.result.name + ':' + event.target.result.lastModified);\
               };\
             };",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox.execute("__keyPathEdges.sort().join('|')").unwrap().value,
        "file:file.txt:123|generated:1|getter:0|index:SyntaxError|proxy:DataError|request:[object IDBRequest]|whole:whole-key"
    );
}

#[test]
fn test_indexeddb_index_metadata_and_failed_upgrade_error() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.register_callback(
        "__zw_idb",
        Box::new(|args: &[String]| {
            let request = &args[0];
            if request.contains("\"op\":\"sync_schema\"")
                && request.contains("\"name\":\"index-unique-failure\"")
            {
                return "__zw_idb_error:ConstraintError:duplicate index keys".to_string();
            }
            let response = if request.contains("\"op\":\"connection_capabilities\"") {
                r#"{"crossRenderer":false,"transactionScheduling":false}"#
            } else if request.contains("\"op\":\"inspect\"") {
                r#"{"database":null}"#
            } else if request.contains("\"op\":\"begin_transaction\"") {
                r#"{"transaction":1}"#
            } else {
                "{}"
            };
            format!("__zw_idb_ok:{response}")
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__indexMetadata = [];\
             var metadata = indexedDB.open('index-metadata', 1);\
             metadata.onupgradeneeded = function () {\
               var store = metadata.result.createObjectStore('items');\
               var first = store.createIndex('compound', ['a','b']);\
               var second = store.index('compound');\
               __indexMetadata.push('keyPath:' + (first.keyPath === first.keyPath ? 'same' : 'changed'));\
               __indexMetadata.push('keyPath:' + (first.keyPath === second.keyPath ? 'shared' : 'different'));\
               __indexMetadata.push('store:' + (first.objectStore === first.objectStore ? 'same' : 'changed'));\
               try { store.createIndex('compound', 'not valid'); }\
               catch (error) { __indexMetadata.push('duplicate:' + error.name); }\
               try { store.deleteIndex('missing'); }\
               catch (error) { __indexMetadata.push('missing:' + error.name); }\
             };\
             metadata.onsuccess = function () {\
               var store = metadata.result.transaction('items').objectStore('items');\
               try { store.createIndex('late', 'value'); }\
               catch (error) { __indexMetadata.push('create:' + error.name); }\
               try { store.deleteIndex('compound'); }\
               catch (error) { __indexMetadata.push('delete:' + error.name); }\
             };\
             var failed = indexedDB.open('index-unique-failure', 1);\
             failed.onupgradeneeded = function () {\
               var db = failed.result;\
               var store = db.createObjectStore('items');\
               store.put({name:'duplicate'}, 1);\
               store.put({name:'duplicate'}, 2);\
               store.createIndex('unique', 'name', {unique:true});\
               failed.transaction.onabort = function () { __indexMetadata.push('abort:transaction'); };\
               db.onabort = function () { __indexMetadata.push('abort:database'); };\
             };\
             failed.onerror = function () {\
               __indexMetadata.push('open:' + failed.error.name);\
             };",
        )
        .unwrap();
    for _ in 0..24 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox
            .execute("__indexMetadata.sort().join('|')")
            .unwrap()
            .value,
        "abort:database|abort:transaction|create:InvalidStateError|delete:InvalidStateError|\
         duplicate:ConstraintError|keyPath:same|keyPath:shared|missing:NotFoundError|\
         open:AbortError|store:same"
    );
}

#[test]
fn test_indexeddb_schema_rename_identity_and_abort_restore() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__schemaRename = [];\
             globalThis.__schemaDeleteRollback = '';\
             var setup = indexedDB.open('schema-rename-abort', 1);\
             setup.onupgradeneeded = function () {\
               setup.result.createObjectStore('old').createIndex('oldIndex', 'value');\
             };\
             setup.onsuccess = function () {\
               setup.result.close();\
               var upgrade = indexedDB.open('schema-rename-abort', 2);\
               upgrade.onupgradeneeded = function () {\
                 var tx = upgrade.transaction;\
                 var store = tx.objectStore('old');\
                 __schemaRename.push('store-same:' + (store === tx.objectStore('old')));\
                 store.name = 'new';\
                 __schemaRename.push('store-list:' + Array.from(upgrade.result.objectStoreNames));\
                 var index = store.index('oldIndex');\
                 __schemaRename.push('index-same:' + (index === store.index('oldIndex')));\
                 index.name = 'newIndex';\
                 __schemaRename.push('index-list:' + Array.from(store.indexNames));\
                 tx.abort();\
                 __schemaRename.push('restored:' + store.name + ':' + index.name);\
                 __schemaRename.push('scope:' + Array.from(tx.objectStoreNames));\
               };\
               upgrade.onerror = function () {\
                 __schemaRename.push('open:' + upgrade.error.name);\
               };\
             };\
             var deleted = indexedDB.open('schema-delete-abort', 1);\
             deleted.onupgradeneeded = function () {\
               deleted.result.createObjectStore('store').createIndex('index', 'value');\
             };\
             deleted.onsuccess = function () {\
               deleted.result.close();\
               var upgrade = indexedDB.open('schema-delete-abort', 2);\
               upgrade.onupgradeneeded = function () {\
                 var store = upgrade.transaction.objectStore('store');\
                 var index = store.index('index');\
                 store.deleteIndex('index');\
                 upgrade.transaction.abort();\
                 try { index.get('value'); }\
                 catch (error) { __schemaDeleteRollback = error.name + ':' + Array.from(store.indexNames); }\
               };\
             };",
        )
        .unwrap();
    for _ in 0..16 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(
        sandbox.execute("__schemaRename.join('|')").unwrap().value,
        "store-same:true|store-list:new|index-same:true|index-list:newIndex|\
         restored:old:oldIndex|scope:old|open:AbortError"
    );
    assert_eq!(
        sandbox.execute("__schemaDeleteRollback").unwrap().value,
        "TransactionInactiveError:index"
    );
}

#[test]
fn test_indexeddb_key_range_conversion_edges() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    assert_eq!(
        sandbox
            .execute(
                "(function () {\
                   var results = [];\
                   var source = new Uint8Array([1, 2]);\
                   var range = IDBKeyRange.lowerBound(source);\
                   source[0] = 9;\
                   results.push('binary:' + (range.lower instanceof ArrayBuffer) + ':' + new Uint8Array(range.lower)[0]);\
                   var thrown = new Error('getter');\
                   var key = [];\
                   key.length = 1;\
                   Object.defineProperty(key, '0', {get:function () { throw thrown; }});\
                   try { IDBKeyRange.only(key); }\
                   catch (error) { results.push('getter:' + (error === thrown)); }\
                   var secondReads = 0;\
                   var second = [];\
                   second.length = 1;\
                   Object.defineProperty(second, '0', {get:function () { secondReads++; return 1; }});\
                   try { indexedDB.cmp({}, second); }\
                   catch (error) { results.push('cmp:' + error.name + ':' + secondReads); }\
                   try { IDBKeyRange.bound(2, 1); }\
                   catch (error) { results.push('bound:' + error.name); }\
                   try { range.includes(); }\
                   catch (error) { results.push('includes:' + error.name); }\
                   return results.join('|');\
                 })()"
            )
            .unwrap()
            .value,
        "binary:true:1|getter:true|cmp:DataError:0|bound:DataError|includes:TypeError"
    );
}

#[test]
fn test_indexeddb_transaction_exception_order() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__transactionOrder = [];\
             var request = indexedDB.open('transaction-exception-order', 1);\
             __transactionOrder.push(Object.prototype.toString.call(request));\
             request.onupgradeneeded = function () {\
               request.result.createObjectStore('store');\
               __transactionOrder.push(Object.prototype.toString.call(request.result));\
               __transactionOrder.push(Object.prototype.toString.call(request.transaction));\
               try { request.result.transaction('store'); }\
               catch (error) { __transactionOrder.push(error.name); }\
             };\
             request.onsuccess = function () {\
               var db = request.result;\
               try { db.transaction('missing', 'versionchange'); }\
               catch (error) { __transactionOrder.push(error.name); }\
               try { db.transaction('store', 'versionchange'); }\
               catch (error) { __transactionOrder.push(error.name); }\
               var relaxed = db.transaction('store', 'readonly', {durability:'relaxed'});\
               __transactionOrder.push(relaxed.durability);\
               db.close();\
               try { db.transaction('missing'); }\
               catch (error) { __transactionOrder.push(error.name); }\
             };",
        )
        .unwrap();
    sandbox.execute("0").unwrap();

    assert_eq!(
        sandbox.execute("__transactionOrder.join('|')").unwrap().value,
        "[object IDBOpenDBRequest]|[object IDBDatabase]|[object IDBTransaction]|\
         InvalidStateError|NotFoundError|TypeError|relaxed|InvalidStateError"
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
