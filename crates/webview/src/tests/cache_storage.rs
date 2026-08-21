use crate::{IndexedDbOwner, WebView, WebViewConfig};

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
                const keys = await caches.keys();
                globalThis.__cacheResult = [
                  String(matched instanceof Response),
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
        "true|201|Created|text/plain|v1|cached text"
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
