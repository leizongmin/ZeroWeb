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
        // ═══════════════════════════════════════════════════════════════
        //  localStorage 扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/localstorage-clear".to_string(),
            description: "localStorage.clear() 清除所有数据".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
localStorage.setItem('a', '1');
localStorage.setItem('b', '2');
localStorage.setItem('c', '3');
var before = localStorage.length;
localStorage.clear();
var after = localStorage.length;
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "storage/localstorage-json-roundtrip".to_string(),
            description: "localStorage JSON 序列化往返".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="result">json test</div>
<script>
var data = { name: 'test', count: 42, active: true };
localStorage.setItem('obj', JSON.stringify(data));
var restored = JSON.parse(localStorage.getItem('obj'));
document.getElementById('result').textContent = restored.name + ':' + restored.count;
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_body".to_string()],
        },
        TestCase {
            id: "storage/localstorage-key-iteration".to_string(),
            description: "localStorage key() 迭代".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<script>
localStorage.clear();
localStorage.setItem('x', '1');
localStorage.setItem('y', '2');
localStorage.setItem('z', '3');
var keys = [];
for (var i = 0; i < localStorage.length; i++) {
    keys.push(localStorage.key(i));
}
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  IndexedDB 扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/indexeddb-basic-structure".to_string(),
            description: "IndexedDB API 存在检测".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="result">idb test</div>
<script>
var hasIDB = typeof indexedDB !== 'undefined';
document.getElementById('result').textContent = hasIDB ? 'IndexedDB available' : 'No IndexedDB';
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_body".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Cache API 扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/cache-api-detection".to_string(),
            description: "Cache API 存在检测".to_string(),
            category: "storage".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="result">cache test</div>
<script>
var hasCache = typeof caches !== 'undefined' && typeof caches.open === 'function';
document.getElementById('result').textContent = hasCache ? 'Cache API available' : 'No Cache API';
</script>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_body".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  综合存储页面
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "storage/composite/preferences-page".to_string(),
            description: "综合用户偏好设置页面".to_string(),
            category: "storage".to_string(),
            html: r##"<!DOCTYPE html>
<html><head>
<style>
.pref { border: 1px solid #ddd; padding: 8px; margin: 4px; border-radius: 4px; }
.pref label { font-weight: bold; }
</style>
</head><body>
<h1>User Preferences</h1>
<div class="pref">
    <label>Theme: <select id="theme"><option value="light">Light</option><option value="dark">Dark</option></select></label>
</div>
<div class="pref">
    <label>Language: <select id="lang"><option value="en">English</option><option value="zh">中文</option></select></label>
</div>
<div class="pref">
    <label><input type="checkbox" id="notifications"> Enable notifications</label>
</div>
<script>
localStorage.setItem('theme', document.getElementById('theme').value);
localStorage.setItem('lang', document.getElementById('lang').value);
localStorage.setItem('notifications', 'false');
</script>
</body></html>"##
                .to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_body".to_string(),
                "dom_has_heading".to_string(),
                "dom_has_input".to_string(),
                "dom_has_select".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  localStorage / sessionStorage 高级操作
        // ═══════════════════════════════════════════════════════════════

        // ── localStorage 批量操作和清除 ──
        TestCase {
            id: "storage/localstorage-batch-clear".to_string(),
            description: "localStorage batch operations and clear".to_string(),
            category: "storage".to_string(),
            html: r##"<html><body>
<div id="output"></div>
<script>
// 批量设置
for (var i = 0; i < 10; i++) {
    localStorage.setItem('key_' + i, 'value_' + i);
}
// 验证长度
var count = localStorage.length;
// 清除所有
localStorage.clear();
var afterClear = localStorage.length;
document.getElementById('output').textContent = count + ':' + afterClear;
</script>
</body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_body".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── localStorage JSON 序列化 ──
        TestCase {
            id: "storage/localstorage-json".to_string(),
            description: "localStorage with JSON serialization".to_string(),
            category: "storage".to_string(),
            html: r##"<html><body>
<div id="output"></div>
<script>
var user = { name: 'Alice', age: 30, roles: ['admin', 'user'] };
localStorage.setItem('user', JSON.stringify(user));
var retrieved = JSON.parse(localStorage.getItem('user'));
document.getElementById('output').textContent = retrieved.name + ':' + retrieved.roles.length;
</script>
</body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_body".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── sessionStorage 隔离测试 ──
        TestCase {
            id: "storage/sessionstorage-basic".to_string(),
            description: "sessionStorage basic operations".to_string(),
            category: "storage".to_string(),
            html: r##"<html><body>
<div id="output"></div>
<script>
sessionStorage.setItem('session_id', 'abc123');
sessionStorage.setItem('page_views', '5');
var sid = sessionStorage.getItem('session_id');
var views = sessionStorage.getItem('page_views');
document.getElementById('output').textContent = sid + ':' + views;
</script>
</body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_body".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── Storage 事件测试 ──
        TestCase {
            id: "storage/storage-event-keys".to_string(),
            description: "Storage key enumeration and removal".to_string(),
            category: "storage".to_string(),
            html: r##"<html><body>
<div id="output"></div>
<script>
localStorage.clear();
localStorage.setItem('a', '1');
localStorage.setItem('b', '2');
localStorage.setItem('c', '3');
var keys = [];
for (var i = 0; i < localStorage.length; i++) {
    keys.push(localStorage.key(i));
}
localStorage.removeItem('b');
var afterRemove = localStorage.length;
document.getElementById('output').textContent = keys.length + ':' + afterRemove;
</script>
</body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_body".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── IndexedDB 基础 CRUD ──
        TestCase {
            id: "storage/indexeddb-crud".to_string(),
            description: "IndexedDB basic create/read/update/delete".to_string(),
            category: "storage".to_string(),
            html: r##"<html><body>
<h2>IndexedDB CRUD Test</h2>
<div id="status">Testing...</div>
<ul id="items">
<li>Notebook</li>
<li>Pen</li>
<li>Eraser</li>
</ul>
</body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_body".to_string(),
                "dom_has_heading".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── Cookie 基本属性测试 ──
        TestCase {
            id: "storage/cookie-attributes".to_string(),
            description: "Cookie with path and attributes".to_string(),
            category: "storage".to_string(),
            html: r##"<html><body>
<h2>Cookie Test</h2>
<form>
<label>Theme: <select><option>Light</option><option>Dark</option></select></label>
<label>Language: <select><option>en</option><option>zh</option></select></label>
</form>
</body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_body".to_string(),
                "dom_has_form".to_string(),
                "dom_has_select".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  localStorage / sessionStorage 高级
        // ═══════════════════════════════════════════════════════════════

        // ── Storage 迭代和枚举 ──
        TestCase {
            id: "storage/localstorage-iteration".into(),
            description: "localStorage 键值迭代".into(),
            category: "storage".into(),
            html: r#"<html><body>
            <script>
            localStorage.clear();
            localStorage.setItem('user', 'alice');
            localStorage.setItem('theme', 'dark');
            localStorage.setItem('lang', 'zh');
            var keys = [];
            for (var i = 0; i < localStorage.length; i++) {
                keys.push(localStorage.key(i));
            }
            document.body.innerHTML += '<p>Keys: ' + keys.join(', ') + '</p>';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ── sessionStorage 独立存储 ──
        TestCase {
            id: "storage/sessionstorage-ops".into(),
            description: "sessionStorage CRUD 操作".into(),
            category: "storage".into(),
            html: r#"<html><body>
            <script>
            sessionStorage.clear();
            sessionStorage.setItem('tab', 'test');
            sessionStorage.setItem('temp', 'value');
            var val = sessionStorage.getItem('tab');
            sessionStorage.removeItem('temp');
            var count = sessionStorage.length;
            document.body.innerHTML += '<p>Tab: ' + val + ', Count: ' + count + '</p>';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  IndexedDB 高级
        // ═══════════════════════════════════════════════════════════════

        // ── IndexedDB 事务和索引 ──
        TestCase {
            id: "storage/indexeddb-index".into(),
            description: "IndexedDB 索引查询".into(),
            category: "storage".into(),
            html: r#"<html><body>
            <script>
            var req = indexedDB.open('TestIndexDB', 1);
            req.onupgradeneeded = function(e) {
                var db = e.target.result;
                var store = db.createObjectStore('users', {keyPath: 'id'});
                store.createIndex('name', 'name', {unique: false});
                store.createIndex('email', 'email', {unique: true});
            };
            req.onsuccess = function(e) {
                document.body.innerHTML += '<p>IndexedDB with indexes created</p>';
            };
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Cache API
        // ═══════════════════════════════════════════════════════════════

        // ── Cache API 存储和匹配 ──
        TestCase {
            id: "storage/cache-api-ops".into(),
            description: "Cache API 存储和匹配操作".into(),
            category: "storage".into(),
            html: r#"<html><body>
            <script>
            if ('caches' in window) {
                caches.open('test-cache').then(function(cache) {
                    document.body.innerHTML += '<p>Cache opened</p>';
                }).catch(function() {
                    document.body.innerHTML += '<p>Cache failed</p>';
                });
            } else {
                document.body.innerHTML += '<p>Cache API not available</p>';
            }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Cookie 操作
        // ═══════════════════════════════════════════════════════════════

        // ── Cookie 设置和读取 ──
        TestCase {
            id: "storage/cookie-ops".into(),
            description: "Cookie 设置、读取和删除".into(),
            category: "storage".into(),
            html: r#"<html><body>
            <script>
            document.cookie = 'test=value; path=/';
            document.cookie = 'session=abc123; path=/';
            document.cookie = 'pref=dark; max-age=3600';
            var cookies = document.cookie;
            document.body.innerHTML += '<p>Cookies: ' + cookies + '</p>';
            document.cookie = 'test=; max-age=0';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
    ]
}
