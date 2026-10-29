use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use zero_engine::fetch_bridge::FetchResponse;

use crate::{IndexedDbOwner, WebView, WebViewConfig};

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "zeroweb-webview-cache-storage-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        let _ = fs::remove_file(&path);
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
        let _ = fs::remove_file(&self.path);
    }
}

fn webview_with_owner(owner: IndexedDbOwner, page_url: &str) -> WebView {
    let mut webview = WebView::new_with_indexed_db_owner(WebViewConfig::default(), owner);
    webview.prepare_document_state(page_url);
    webview.execute_script("0").unwrap();
    webview
}

fn pump_microtasks(webview: &mut WebView) {
    for i in 0..8 {
        webview
            .execute_script(&format!("globalThis.__cachePump = {i};"))
            .unwrap();
    }
}

#[test]
fn page_cache_api_put_and_match_roundtrip() {
    let mut webview = webview_with_owner(IndexedDbOwner::in_memory(), "https://cache.example/app/page.html");

    webview
        .execute_script(
            r#"
            (async function () {
              try {
                const cache = await caches.open('v1');
                await cache.put(
                  'https://cache.example/app/data.txt',
                  new Response('cached text', {
                    status: 201,
                    statusText: 'Created',
                    headers: {'content-type': 'text/plain'}
                  })
                );
                const matched = await cache.match(new Request('https://cache.example/app/data.txt'));
                const matchedAll = await cache.matchAll('https://cache.example/app/data.txt');
                const requests = await cache.keys();
                const keys = await caches.keys();
                globalThis.__cacheResult = [
                  String(matched instanceof Response),
                  String(matchedAll.length),
                  String(matchedAll[0] instanceof Response),
                  String(requests.length),
                  String(requests[0] instanceof Request),
                  requests[0].method,
                  String(matched.status),
                  String(matched.statusText),
                  String(matched.headers.get('content-type')),
                  keys.join(','),
                  await matched.text()
                ].join('|');
              } catch (error) {
                globalThis.__cacheResult = 'error:' + String(error && error.message ? error.message : error);
              }
            })();
            "#,
        )
        .unwrap();
    pump_microtasks(&mut webview);

    assert_eq!(
        webview.execute_script("globalThis.__cacheResult").unwrap(),
        "true|1|true|1|true|GET|201|Created|text/plain|v1|cached text"
    );
}

#[test]
fn page_cache_api_match_preserves_cached_response_type() {
    let mut webview = webview_with_owner(IndexedDbOwner::in_memory(), "https://cache.example/app/page.html");

    webview
        .execute_script(
            r#"
            (async function () {
              try {
                const openResult = JSON.parse(__zw_cache_storage(JSON.stringify({
                  op: 'open',
                  name: 'typed'
                })).slice('__zw_cache_ok:'.length));
                __zw_cache_storage(JSON.stringify({
                  op: 'put',
                  cache_name: 'typed',
                  cache_id: openResult.cache_id,
                  request: {
                    url: 'https://cache.example/app/typed.txt',
                    method: 'GET'
                  },
                  response: {
                    status: 200,
                    statusText: 'OK',
                    type: 'basic',
                    headers: 'content-type\x1etext/plain',
                    body: 'typed body'
                  }
                }));
                const cache = await caches.open('typed');
                const matched = await cache.match('https://cache.example/app/typed.txt');
                globalThis.__cacheResponseType = [
                  String(matched instanceof Response),
                  matched.type,
                  matched.clone().type,
                  await matched.text()
                ].join('|');
              } catch (error) {
                globalThis.__cacheResponseType = 'error:' + String(error && error.message ? error.message : error);
              }
            })();
            "#,
        )
        .unwrap();
    pump_microtasks(&mut webview);

    assert_eq!(
        webview.execute_script("globalThis.__cacheResponseType").unwrap(),
        "true|basic|basic|typed body"
    );
}

#[test]
fn page_cache_api_query_options_match_delete_and_keys() {
    let mut webview = webview_with_owner(IndexedDbOwner::in_memory(), "https://cache.example/app/page.html");

    webview
        .execute_script(
            r#"
            (async function () {
              try {
                const cache = await caches.open('v1');
                await cache.put(
                  new Request('https://cache.example/app/data?version=1', {method: 'GET'}),
                  new Response('cached text')
                );
                await cache.put(
                  new Request('https://cache.example/app/other?version=1', {method: 'GET'}),
                  new Response('other text')
                );
                const strict = await cache.match(
                  new Request('https://cache.example/app/data?version=2', {method: 'HEAD'})
                );
                const matched = await cache.match(
                  new Request('https://cache.example/app/data?version=2', {method: 'HEAD'}),
                  {ignoreSearch: true, ignoreMethod: true}
                );
                const storageMatched = await caches.match(
                  new Request('https://cache.example/app/data?version=3', {method: 'POST'}),
                  {cacheName: 'v1', ignoreSearch: true, ignoreMethod: true}
                );
                const keys = await cache.keys(
                  new Request('https://cache.example/app/data?version=4', {method: 'POST'}),
                  {ignoreSearch: true, ignoreMethod: true}
                );
                const deleted = await cache.delete(
                  new Request('https://cache.example/app/data?version=5', {method: 'POST'}),
                  {ignoreSearch: true, ignoreMethod: true}
                );
                const remaining = await cache.keys();
                globalThis.__cacheOptionsResult = [
                  String(strict === undefined),
                  String(matched instanceof Response),
                  String(storageMatched instanceof Response),
                  String(keys.length),
                  keys[0].url,
                  String(deleted),
                  String(remaining.length),
                  remaining[0].url,
                  await matched.text()
                ].join('|');
              } catch (error) {
                globalThis.__cacheOptionsResult = 'error:' + String(error && error.message ? error.message : error);
              }
            })();
            "#,
        )
        .unwrap();
    pump_microtasks(&mut webview);

    assert_eq!(
        webview.execute_script("globalThis.__cacheOptionsResult").unwrap(),
        "true|true|true|1|https://cache.example/app/data?version=1|true|1|https://cache.example/app/other?version=1|cached text"
    );
}

#[test]
fn page_cache_api_add_and_add_all_fetch_then_store() {
    let config = WebViewConfig {
        fetch_handler: Some(Arc::new(|request| match request.url.as_str() {
            "https://cache.example/app/add-a.txt" => Ok(FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers: vec![("content-type".to_string(), "text/plain".to_string())],
                body: "alpha".to_string(),
                body_bytes: None,
            }),
            "https://cache.example/app/add-b.txt" => Ok(FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers: vec![("content-type".to_string(), "text/plain".to_string())],
                body: "beta".to_string(),
                body_bytes: None,
            }),
            "https://cache.example/app/missing.txt" => Ok(FetchResponse {
                status: 404,
                status_text: "Not Found".to_string(),
                headers: Vec::new(),
                body: "missing".to_string(),
                body_bytes: None,
            }),
            other => Err(format!("unexpected fetch URL: {other}")),
        })),
        ..Default::default()
    };
    let mut webview = WebView::new_with_indexed_db_owner(config, IndexedDbOwner::in_memory());
    webview.prepare_document_state("https://cache.example/app/page.html");
    webview.load_html(
        r#"
        <html><body><script>
            (async function () {
              try {
                const cache = await caches.open('assets');
                await cache.add('https://cache.example/app/add-a.txt');
                await cache.addAll(['https://cache.example/app/add-b.txt']);
                const one = await cache.match('https://cache.example/app/add-a.txt');
                const two = await cache.match('https://cache.example/app/add-b.txt');
                let postRejected = 'no';
                try {
                  await cache.add(new Request('https://cache.example/app/post.txt', {method: 'POST'}));
                } catch (error) {
                  postRejected = String(error instanceof TypeError) + ':' + String(error.message);
                }
                let missingRejected = 'no';
                try {
                  await cache.add('https://cache.example/app/missing.txt');
                } catch (error) {
                  missingRejected = String(error instanceof TypeError) + ':' + String(error.message);
                }
                const missing = await cache.match('https://cache.example/app/missing.txt');
                globalThis.__cacheAddResult = [
                  await one.text(),
                  one.headers.get('content-type'),
                  await two.text(),
                  postRejected,
                  missingRejected,
                  String(missing === undefined)
                ].join('|');
              } catch (error) {
                globalThis.__cacheAddResult = 'error:' + String(error && error.message ? error.message : error);
              }
            })();
        </script></body></html>
        "#,
        None,
    );
    webview.run_page_scripts_strict().unwrap();

    assert_eq!(
        webview.execute_script("globalThis.__cacheAddResult").unwrap(),
        "alpha|text/plain|beta|true:Cache.add only supports GET requests|true:Cache.add fetch response is not ok|true"
    );
}

#[test]
fn page_cache_api_rejects_uncacheable_put_and_atomic_add_all() {
    let config = WebViewConfig {
        fetch_handler: Some(Arc::new(|request| match request.url.as_str() {
            "https://cache.example/app/ok.txt" => Ok(FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers: Vec::new(),
                body: "ok".to_string(),
                body_bytes: None,
            }),
            "https://cache.example/app/missing.txt" => Ok(FetchResponse {
                status: 404,
                status_text: "Not Found".to_string(),
                headers: Vec::new(),
                body: "missing".to_string(),
                body_bytes: None,
            }),
            other => Err(format!("unexpected fetch URL: {other}")),
        })),
        ..Default::default()
    };
    let mut webview = WebView::new_with_indexed_db_owner(config, IndexedDbOwner::in_memory());
    webview.prepare_document_state("https://cache.example/app/page.html");

    webview
        .execute_script(
            r#"
            (async function () {
              try {
                const cache = await caches.open('assets');
                const results = [];
                async function capture(label, fn) {
                  try {
                    await fn();
                    results.push(label + ':resolved');
                  } catch (error) {
                    results.push(label + ':' + String(error instanceof TypeError) + ':' + String(error.message));
                  }
                }
                await capture('post', () => cache.put(
                  new Request('https://cache.example/app/post.txt', {method: 'POST'}),
                  new Response('post')
                ));
                await capture('ftp', () => cache.put('ftp://cache.example/app/file.txt', new Response('ftp')));
                await capture('partial', () => cache.put(
                  'https://cache.example/app/partial-put.txt',
                  new Response('partial', {status: 206})
                ));
                await capture('vary', () => cache.put(
                  'https://cache.example/app/vary.txt',
                  new Response('vary', {headers: {'vary': 'Accept-Encoding, *'}})
                ));
                await capture('error', () => cache.put(
                  'https://cache.example/app/error.txt',
                  Response.error()
                ));
                await capture('addAll', () => cache.addAll([
                  'https://cache.example/app/ok.txt',
                  'https://cache.example/app/missing.txt'
                ]));
                const ok = await cache.match('https://cache.example/app/ok.txt');
                const error = await cache.match('https://cache.example/app/error.txt');
                const keys = await cache.keys();
                globalThis.__cacheRejectResult = results.concat([
                  'okMissing:' + String(ok === undefined),
                  'errorType:' + String(error && error.type) + ':' + String(error && error.status),
                  'keys:' + String(keys.length)
                ]).join('|');
              } catch (error) {
                globalThis.__cacheRejectResult = 'error:' + String(error && error.message ? error.message : error);
              }
            })();
            "#,
        )
        .unwrap();
    pump_microtasks(&mut webview);

    assert_eq!(
        webview.execute_script("globalThis.__cacheRejectResult").unwrap(),
        "post:true:Cache.put request method must be GET|ftp:true:Cache.put request URL must be an HTTP(S) URL|partial:true:Cache.put cannot store a 206 Partial Content response|vary:true:Cache.put cannot store a response with Vary: *|error:resolved|addAll:true:Cache.addAll fetch response is not ok|okMissing:true|errorType:error:0|keys:1"
    );
}

#[test]
fn page_cache_storage_names_are_domstrings_in_inline_scripts() {
    let mut webview = WebView::new_with_indexed_db_owner(WebViewConfig::default(), IndexedDbOwner::in_memory());
    webview.prepare_document_state("https://cache.example/app/page.html");
    webview.load_html(
        r#"
        <html><body><script>
            (async function () {
              try {
                const testCases = [
                  {
                    name: 'cache-storage/lowercase',
                    should_not_match: [
                      'cache-storage/Lowercase',
                      ' cache-storage/lowercase',
                      'cache-storage/lowercase '
                    ]
                  },
                  {
                    name: 'cache-storage/has a space',
                    should_not_match: [
                      'cache-storage/has'
                    ]
                  },
                  {
                    name: 'cache-storage/has\000_in_the_name',
                    should_not_match: [
                      'cache-storage/has',
                      'cache-storage/has_in_the_name'
                    ]
                  },
                  {
                    name: 'unpaired\uD800',
                    should_not_match: [
                      'unpaired\uFFFD'
                    ]
                  }
                ];
                const results = await Promise.all(testCases.map(async testcase => {
                  const cacheName = testcase.name;
                  await caches.delete(testcase.should_not_match[0]);
                  await caches.delete(cacheName);
                  await caches.open(cacheName);
                  const hasOriginal = await caches.has(cacheName);
                  const keys = await caches.keys();
                  const mismatches = await Promise.all(testcase.should_not_match.map(name => caches.has(name)));
                  await caches.delete(cacheName);
                  return [
                    String(cacheName.charCodeAt(17)),
                    String(hasOriginal),
                    String(keys.indexOf(cacheName) !== -1),
                    mismatches.map(String).join(',')
                  ].join(':');
                }));
                globalThis.__cacheDomStringName = results.join('|');
              } catch (error) {
                globalThis.__cacheDomStringName = 'error:' + String(error && error.message ? error.message : error);
              }
            })();
        </script></body></html>
        "#,
        None,
    );
    webview.run_page_scripts_strict().unwrap();
    pump_microtasks(&mut webview);

    assert_eq!(
        webview.execute_script("globalThis.__cacheDomStringName").unwrap(),
        "101:true:true:false,false,false|32:true:true:false|0:true:true:false,false|NaN:true:true:false"
    );
}

#[test]
fn page_cache_storage_delete_dooms_existing_cache_objects() {
    let mut webview = webview_with_owner(IndexedDbOwner::in_memory(), "https://cache.example/app/page.html");

    webview
        .execute_script(
            r#"
            (async function () {
              try {
                const cacheName = 'cache-storage/delete-dooming';
                const firstCache = await caches.open(cacheName);
                await caches.delete(cacheName);
                await firstCache.put('https://cache.example/app/old.txt', new Response('old'));
                const namesAfterDelete = await caches.keys();
                const secondCache = await caches.open(cacheName);
                const secondKeys = await secondCache.keys();
                const firstKeys = await firstCache.keys();
                await caches.delete(cacheName);
                globalThis.__cacheDoomingResult = [
                  String(namesAfterDelete.indexOf(cacheName)),
                  String(secondKeys.length),
                  String(firstKeys.length)
                ].join('|');
              } catch (error) {
                globalThis.__cacheDoomingResult = 'error:' + String(error && error.message ? error.message : error);
              }
            })();
            "#,
        )
        .unwrap();
    pump_microtasks(&mut webview);

    assert_eq!(
        webview.execute_script("globalThis.__cacheDoomingResult").unwrap(),
        "-1|0|1"
    );
}

#[test]
fn page_cache_api_uses_shared_owner_and_origin_isolation() {
    let origin = "https://shared-cache.example/app/page.html";
    let owner = IndexedDbOwner::in_memory();
    let mut first = webview_with_owner(owner.clone(), origin);
    first
        .execute_script(
            r#"
            caches.open('assets').then(async cache => {
              await cache.put(
                'https://shared-cache.example/app/app.js',
                new Response('from first', {headers: {'x-cache': 'yes'}})
              );
              globalThis.__cacheWrite = 'done';
            }, error => {
              globalThis.__cacheWrite = 'error:' + String(error && error.message ? error.message : error);
            });
            "#,
        )
        .unwrap();
    pump_microtasks(&mut first);
    assert_eq!(first.execute_script("globalThis.__cacheWrite").unwrap(), "done");

    let mut second = webview_with_owner(owner, origin);
    second
        .execute_script(
            r#"
            caches.match('https://shared-cache.example/app/app.js').then(async response => {
              globalThis.__cacheRead = response
                ? [String(response.headers.get('x-cache')), await response.text()].join('|')
                : 'missing';
            }, error => {
              globalThis.__cacheRead = 'error:' + String(error && error.message ? error.message : error);
            });
            "#,
        )
        .unwrap();
    pump_microtasks(&mut second);
    assert_eq!(
        second.execute_script("globalThis.__cacheRead").unwrap(),
        "yes|from first"
    );

    let mut isolated = webview_with_owner(IndexedDbOwner::in_memory(), origin);
    isolated
        .execute_script(
            r#"
            caches.match('https://shared-cache.example/app/app.js').then(response => {
              globalThis.__cacheRead = response ? 'hit' : 'missing';
            }, error => {
              globalThis.__cacheRead = 'error:' + String(error && error.message ? error.message : error);
            });
            "#,
        )
        .unwrap();
    pump_microtasks(&mut isolated);
    assert_eq!(isolated.execute_script("globalThis.__cacheRead").unwrap(), "missing");
}

#[test]
fn persistent_owner_restores_page_cache_storage_after_webview_rebuild() {
    let directory = TestDirectory::new();
    let origin = "https://persistent-cache.example/app/page.html";
    {
        let owner = IndexedDbOwner::persistent(directory.path()).unwrap();
        let mut first = webview_with_owner(owner, origin);
        first
            .execute_script(
                r#"
                (async function () {
                  try {
                    const cache = await caches.open('assets');
                    await cache.put(
                      'https://persistent-cache.example/app/data.txt',
                      new Response('restored body', {
                        status: 202,
                        statusText: 'Accepted',
                        headers: {'content-type': 'text/plain', 'vary': 'Accept-Language'}
                      })
                    );
                    globalThis.__cachePersistentWrite = 'done';
                  } catch (error) {
                    globalThis.__cachePersistentWrite = 'error:' + String(error && error.message ? error.message : error);
                  }
                })();
                "#,
            )
            .unwrap();
        pump_microtasks(&mut first);
        assert_eq!(
            first.execute_script("globalThis.__cachePersistentWrite").unwrap(),
            "done"
        );
    }

    let owner = IndexedDbOwner::persistent(directory.path()).unwrap();
    let mut restored = webview_with_owner(owner, origin);
    restored
        .execute_script(
            r#"
            (async function () {
              try {
                const cache = await caches.open('assets');
                const hit = await cache.match(
                  new Request('https://persistent-cache.example/app/data.txt', {
                    headers: {'Accept-Language': 'en'}
                  }),
                  {ignoreVary: true}
                );
                const keys = await caches.keys();
                globalThis.__cachePersistentRead = hit
                  ? [keys.join(','), String(hit.status), hit.statusText, hit.headers.get('content-type'), await hit.text()].join('|')
                  : 'missing';
              } catch (error) {
                globalThis.__cachePersistentRead = 'error:' + String(error && error.message ? error.message : error);
              }
            })();
            "#,
        )
        .unwrap();
    pump_microtasks(&mut restored);
    assert_eq!(
        restored.execute_script("globalThis.__cachePersistentRead").unwrap(),
        "assets|202|Accepted|text/plain|restored body"
    );

    let owner = IndexedDbOwner::persistent(directory.path()).unwrap();
    let mut isolated = webview_with_owner(owner, "https://other-persistent-cache.example/app/page.html");
    isolated
        .execute_script(
            r#"
            caches.match('https://persistent-cache.example/app/data.txt').then(response => {
              globalThis.__cachePersistentRead = response ? 'hit' : 'missing';
            }, error => {
              globalThis.__cachePersistentRead = 'error:' + String(error && error.message ? error.message : error);
            });
            "#,
        )
        .unwrap();
    pump_microtasks(&mut isolated);
    assert_eq!(
        isolated.execute_script("globalThis.__cachePersistentRead").unwrap(),
        "missing"
    );
}

#[test]
fn page_execute_script_entry_parses_absolute_urls_for_request_and_url() {
    // 邻近回归钉（T26）：独立 `execute_script` 入口注入了 js_dom shim（R3295），其
    // `new URL`/`new Request` 契约依赖 `register_dom_callbacks` 武装的 host 桥
    //（`__zw_parse_url`）。绝对 https URL 经 `_zwRequestValidate`（fetch consumer
    // 同源校验链）必须解析成功——非缓存路径，证明修复对校验器全部调用方生效。
    // T26 返修（DF1-F4）：补钉裸入口相对 URL + base 解析（`new URL('/x', base)` 与
    // `new Request('/x')` 经 `__zw_get_page_url` 取 base 链路）——若未来回归只漏装
    // get_page_url 而装了 parse_url，此钉仍红。
    let mut webview = webview_with_owner(IndexedDbOwner::in_memory(), "https://cache.example/app/page.html");

    webview
        .execute_script(
            r#"
            globalThis.__urlProbe = (function () {
              try {
                const parsed = new URL('https://cache.example/app/data.txt');
                const request = new Request('https://cache.example/app/data.txt');
                const relativeBase = new URL('/app/x', location.href).href;
                const relativeRequest = new Request('/app/x');
                return [
                  parsed.href,
                  request.url,
                  String(typeof fetch === 'function'),
                  relativeBase,
                  relativeRequest.url
                ].join('|');
              } catch (error) {
                return 'error:' + String(error && error.message ? error.message : error);
              }
            })();
            "#,
        )
        .unwrap();

    assert_eq!(
        webview.execute_script("globalThis.__urlProbe").unwrap(),
        "https://cache.example/app/data.txt|https://cache.example/app/data.txt|true|https://cache.example/app/x|https://cache.example/app/x"
    );
}

// T26 返修（DF1-F1 回归钉）：execute_script 尾部 apply 与 apply_mutations_subset
// 尾部换代 bump 的中途重入。单轮 execute_script 混合批（建元素 + 写 transition 样式）
// 走 apply_pending_shared_mutations 两步切分：结构子批 apply 后的嵌套 bump
// execute_script 若把仍在队列的样式子批抢先应用（游标超前 M），外层再重复应用同批，
// 游标永久超前——随后第二条独立 SetStyle 被 min() 钳制静默跳过（同代内永不落 DOM）。
// 断言锚 host 真相快照 html_content()：`__zw_get_style_lw`/getComputedStyle 回调均对
// 全队列（含未应用尾）做 latest-wins replay，不能作 apply 真相观测点；落 DOM 与
// 游标不超前在该断言合一（被跳过 = 游标超前 = 快照无该样式）。
#[test]
fn page_execute_script_mixed_batch_two_step_split_does_not_overshoot_cursor() {
    let mut webview = webview_with_owner(IndexedDbOwner::in_memory(), "https://cache.example/app/page.html");
    webview.load_html("<html><body><main>cursor probe base</main></body></html>", None);

    // 单轮混合批：结构（建元素 + 挂载）+ 样式（transition）→ 两步切分路径。
    webview
        .execute_script(
            r#"
            (function () {
              const el = document.createElement('div');
              el.id = 'cursorProbe';
              document.body.appendChild(el);
              el.style.transition = 'opacity 1s linear';
              return 'queued';
            })();
            "#,
        )
        .unwrap();

    // 同代内第二条独立 SetStyle：护栏缺失时其被超前的游标钳制静默跳过。
    webview
        .execute_script("document.getElementById('cursorProbe').style.color = 'red';")
        .unwrap();

    let html = webview.html_content();
    assert!(html.contains("cursorProbe"), "混合批结构子批必须落 DOM：{html}");
    assert!(
        html.contains("transition"),
        "混合批样式子批必须恰好落 DOM（两步切分成立的前置）：{html}"
    );
    assert!(
        html.contains("red"),
        "第二条独立 SetStyle 必须落 DOM——apply 重入致游标超前 M 时该 mutation 被静默跳过：{html}"
    );
}
