//! Storage 和 Web Worker 标准合规性测试。
//!
//! 覆盖 localStorage、sessionStorage、IndexedDB 基本操作、
//! Cookie 处理、Cache API、Service Worker 生命周期、
//! Web Worker 通信和 Dedicated Worker 行为。

use super::TestCase;

/// 返回 Storage 和 Web Worker 标准合规性测试用例。
pub fn storage_compliance_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  localStorage / sessionStorage
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/localstorage-basic".to_string(),
            description: "localStorage setItem/getItem/removeItem".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
localStorage.setItem('key1', 'value1');
var v = localStorage.getItem('key1');
localStorage.removeItem('key1');
var removed = localStorage.getItem('key1');
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/localstorage-multi-keys".to_string(),
            description: "localStorage with multiple keys and clear".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
localStorage.setItem('a', '1');
localStorage.setItem('b', '2');
localStorage.setItem('c', '3');
var len = localStorage.length;
localStorage.clear();
var after = localStorage.length;
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/sessionstorage-basic".to_string(),
            description: "sessionStorage setItem/getItem".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
sessionStorage.setItem('session', 'data');
var v = sessionStorage.getItem('session');
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/storage-json-roundtrip".to_string(),
            description: "Store and retrieve JSON data".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
var data = {name: 'test', count: 42, items: [1, 2, 3]};
localStorage.setItem('json', JSON.stringify(data));
var parsed = JSON.parse(localStorage.getItem('json'));
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  IndexedDB
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/indexeddb-open".to_string(),
            description: "IndexedDB open database".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
var request = indexedDB.open('testdb', 1);
request.onupgradeneeded = function(e) {
    var db = e.target.result;
    if (!db.objectStoreNames.contains('items')) {
        db.createObjectStore('items', {keyPath: 'id'});
    }
};
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/indexeddb-crud".to_string(),
            description: "IndexedDB add/get/put/delete operations".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
var request = indexedDB.open('crudtest', 1);
request.onupgradeneeded = function(e) {
    var db = e.target.result;
    db.createObjectStore('data', {keyPath: 'id'});
};
request.onsuccess = function(e) {
    var db = e.target.result;
    var tx = db.transaction('data', 'readwrite');
    var store = tx.objectStore('data');
    store.add({id: 1, name: 'item1'});
    store.add({id: 2, name: 'item2'});
    store.put({id: 1, name: 'updated'});
    store.delete(2);
};
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/indexeddb-index".to_string(),
            description: "IndexedDB index and cursor".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
var request = indexedDB.open('indextest', 1);
request.onupgradeneeded = function(e) {
    var db = e.target.result;
    var store = db.createObjectStore('users', {keyPath: 'id'});
    store.createIndex('name', 'name', {unique: false});
    store.createIndex('email', 'email', {unique: true});
};
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Cookie
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/cookie-basic".to_string(),
            description: "document.cookie read/write".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
document.cookie = 'test=value; path=/';
document.cookie = 'session=abc123';
var cookies = document.cookie;
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/cookie-attributes".to_string(),
            description: "Cookie with Secure, HttpOnly, SameSite".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
document.cookie = 'secure1=val; Secure';
document.cookie = 'samesite=val; SameSite=Strict';
document.cookie = 'lax=val; SameSite=Lax';
document.cookie = 'maxage=val; Max-Age=3600';
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Cache API
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/cache-api-basic".to_string(),
            description: "Cache API open/put/match".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
if ('caches' in window) {
    caches.open('v1').then(function(cache) {
        var response = new Response('cached data');
        cache.put('/test.html', response);
        cache.match('/test.html').then(function(r) {
            return r.text();
        });
    });
}
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Web Worker
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/worker-create".to_string(),
            description: "Web Worker constructor exists".to_string(),
            category: "web-workers".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
if (typeof Worker !== 'undefined') {
    var hasWorker = true;
}
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/worker-message-channel".to_string(),
            description: "Worker postMessage/onmessage channel".to_string(),
            category: "web-workers".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
if (typeof Worker !== 'undefined') {
    var blob = new Blob([
        'self.onmessage = function(e) { self.postMessage(e.data * 2); };'
    ], {type: 'application/javascript'});
    var url = URL.createObjectURL(blob);
    var w = new Worker(url);
    w.onmessage = function(e) {
        var result = e.data;
        URL.revokeObjectURL(url);
    };
    w.postMessage(21);
}
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/worker-error-handling".to_string(),
            description: "Worker error event".to_string(),
            category: "web-workers".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
if (typeof Worker !== 'undefined') {
    var blob = new Blob([
        'throw new Error("worker error");'
    ], {type: 'application/javascript'});
    var url = URL.createObjectURL(blob);
    var w = new Worker(url);
    w.onerror = function(e) {
        var handled = true;
    };
}
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/worker-terminate".to_string(),
            description: "Worker terminate lifecycle".to_string(),
            category: "web-workers".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
if (typeof Worker !== 'undefined') {
    var blob = new Blob([
        'var i = 0; setInterval(function() { i++; }, 100);'
    ], {type: 'application/javascript'});
    var url = URL.createObjectURL(blob);
    var w = new Worker(url);
    w.terminate();
    URL.revokeObjectURL(url);
}
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Fetch API
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/fetch-api-exists".to_string(),
            description: "fetch() function is available".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
var hasFetch = typeof fetch === 'function';
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/fetch-request-response".to_string(),
            description: "Request and Response constructors".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
if (typeof Request !== 'undefined') {
    var req = new Request('https://example.com/api', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({key: 'value'})
    });
    var resp = new Response('ok', {status: 200, headers: {'X-Custom': 'yes'}});
}
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  综合场景
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/offline-page".to_string(),
            description: "Offline-capable page with Storage and Cache API".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Offline App</h2>
<form id="noteForm">
<textarea id="noteText" placeholder="Enter note..."></textarea>
<button type="button" id="saveBtn">Save</button>
</form>
<div id="savedNotes"></div>
<script>
var notes = JSON.parse(localStorage.getItem('notes') || '[]');
function saveNote(text) {
    notes.push({text: text, time: Date.now()});
    localStorage.setItem('notes', JSON.stringify(notes));
}
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_heading".to_string(),
                "dom_has_form".to_string(),
            ],
        },
        TestCase {
            id: "storage/session-dashboard".to_string(),
            description: "Dashboard using sessionStorage for session data".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<h2>Session Dashboard</h2>
<div id="stats">
<p>Sessions: <span id="count">0</span></p>
<p>Last visit: <span id="lastVisit">never</span></p>
</div>
<script>
var count = parseInt(sessionStorage.getItem('visits') || '0') + 1;
sessionStorage.setItem('visits', count.toString());
sessionStorage.setItem('lastVisit', new Date().toISOString());
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
        },
    ]
}
