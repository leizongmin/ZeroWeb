//! Isolated IndexedDB JS worker regression test.

use std::time::Duration;
use zero_renderer::js_worker::RendererJsWorker;

#[test]
fn renderer_js_worker_registers_indexed_db_host() {
    fn wait_for_value(worker: &RendererJsWorker, expression: &str) -> String {
        let mut value = String::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            value = worker.execute_script_direct(expression).unwrap();
            if value != "undefined" {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        value
    }

    let mut worker = RendererJsWorker::spawn(45);
    worker.set_dom_snapshot("<html><body></body></html>", "https://storage.example/page");

    worker
        .execute_script_direct(r#"__zw_idb(JSON.stringify({op:"open",name:"app",version:1}))"#)
        .unwrap();
    worker
        .execute_script_direct(
            r#"var created = indexedDB.open("app", 2);
               created.onupgradeneeded = function () {
                 var store = created.result.createObjectStore("items", {keyPath:"id"});
                 store.createIndex("by_label", "label");
                 store.createIndex("by_identity", ["profile.first", "profile.last"]);
                 store.put({
                   id: new Date(10),
                   label: "stored",
                   profile: {first: "Ada", last: "Lovelace"},
                   bytes: new Uint8Array([1, 2, 3])
                 });
                 store.put({
                   id: new Date(20),
                   label: "alpha",
                   profile: {first: "Grace", last: "Hopper"},
                   bytes: new Uint8Array([4])
                 });
                 var shared = {value: 7};
                 var graph = {
                   id: new Date(30),
                   label: "graph",
                   profile: {first: "Katherine", last: "Johnson"},
                   left: shared,
                   right: shared
                 };
                 graph.self = graph;
                 store.put(graph);
               };
               created.onsuccess = function () { globalThis.__created = true; };"#,
        )
        .unwrap();
    assert_eq!(wait_for_value(&worker, "String(globalThis.__created)"), "true");

    worker.reset_document_state();
    worker.set_dom_snapshot("<html><body></body></html>", "https://storage.example/next");
    worker
        .execute_script_direct(
            r#"var reopened = indexedDB.open("app");
               reopened.onsuccess = function () {
                 var tx = reopened.result.transaction("items");
                 var index = tx.objectStore("items").index("by_label");
                 var read = index.get("stored");
                 var compoundRead =
                   tx.objectStore("items").index("by_identity").get(["Ada", "Lovelace"]);
                 var graphRead = index.get("graph");
                 var graphCompoundRead =
                   tx.objectStore("items").index("by_identity").get(["Katherine", "Johnson"]);
                 var storeCursor = tx.objectStore("items").openCursor();
                 var labels = [];
                 read.onsuccess = function () {
                   globalThis.__restoredRecord =
                     reopened.result.version + ":" +
                     reopened.result.objectStoreNames.contains("items") + ":" +
                     read.result.label + ":" + read.result.id.getTime() + ":" +
                     read.result.bytes[2];
                 };
                 compoundRead.onsuccess = function () {
                   globalThis.__compound = compoundRead.result.label;
                 };
                 graphRead.onsuccess = function () {
                   globalThis.__graph =
                     (graphRead.result.self === graphRead.result) + ":" +
                     (graphRead.result.left === graphRead.result.right) + ":" +
                     graphRead.result.left.value;
                 };
                 graphCompoundRead.onsuccess = function () {
                   globalThis.__graphCompound = graphCompoundRead.result.label;
                 };
                 storeCursor.onsuccess = function () {
                   if (!storeCursor.result) return;
                   if (storeCursor.result.key.getTime() === 10) {
                     var currentCursor = storeCursor.result;
                     currentCursor.advance(2);
                     try {
                       currentCursor.continue();
                       globalThis.__cursorPending = "missing";
                     } catch (error) {
                       globalThis.__cursorPending = error.name;
                     }
                   } else {
                     globalThis.__cursorAdvance = storeCursor.result.key.getTime();
                     storeCursor.result.continue();
                   }
                 };
                 var cursor = index.openCursor();
                 cursor.onsuccess = function () {
                   if (cursor.result) {
                     labels.push(cursor.result.key);
                     cursor.result.continue();
                   } else {
                     globalThis.__restored =
                       globalThis.__restoredRecord + ":" + labels.join(",") + ":" +
                       globalThis.__compound + ":" + globalThis.__graph + ":" +
                       globalThis.__graphCompound + ":" + globalThis.__cursorAdvance + ":" +
                       globalThis.__cursorPending;
                   }
                 };
               };"#,
        )
        .unwrap();
    assert_eq!(
        wait_for_value(&worker, "String(globalThis.__restored)"),
        "2:true:stored:10:3:alpha,graph,stored:stored:true:true:7:graph:30:InvalidStateError"
    );
    worker.shutdown();
}
