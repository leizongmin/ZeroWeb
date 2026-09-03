use crate::WebViewConfig;
use std::sync::Arc;
use std::time::{Duration, Instant};
use zero_engine::fetch_bridge::FetchResponse;

#[test]
#[serial_test::serial(service_worker_runtime)]
fn iframe_get_registration_uses_iframe_url_and_realm_workers() {
    const PAGE_URL: &str = "https://example.test/service-workers/service-worker/controller-on-load.https.html";
    let mut webview = crate::WebView::new(WebViewConfig {
        service_worker_script_fetcher: Some(Arc::new(|_, script| {
            if script != "https://example.test/service-workers/service-worker/resources/empty-worker.js" {
                return Err(format!("unexpected script URL: {script}"));
            }
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: Vec::new(),
                url: script.to_string(),
                redirect_count: 0,
            })
        })),
        fetch_handler: Some(Arc::new(|request| {
            let body = if request
                .url
                .ends_with("/service-workers/service-worker/resources/blank.html")
            {
                "<!doctype html><title>blank</title>"
            } else {
                ""
            };
            Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/html".into())],
                body: body.into(),
                body_bytes: None,
            })
        })),
        ..Default::default()
    });

    webview.load_url(PAGE_URL);
    webview.complete_load(
        "<!doctype html><body>
           <script>
             globalThis.__iframeControllerOnLoad = 'pending';
             navigator.serviceWorker
               .register('resources/empty-worker.js', { scope: 'resources/blank.html' })
               .then(function(registration) {
                 return new Promise(function(resolve, reject) {
                   function wait() {
                     var worker = registration.installing || registration.waiting || registration.active;
                     if (worker && worker.state === 'activated') {
                       resolve(registration);
                     } else {
                       setTimeout(wait, 0);
                     }
                   }
                   wait();
                 });
               })
               .then(function(registration) {
                 return new Promise(function(resolve) {
                   var frame = document.createElement('iframe');
                   frame.src = 'resources/blank.html';
                   frame.onload = function() { resolve({ registration: registration, frame: frame }); };
                   document.body.appendChild(frame);
                 });
               })
               .then(function(result) {
                 var frame = result.frame;
                 var frameWindow = frame.contentWindow;
                 var controller = frameWindow.navigator.serviceWorker.controller;
                 return frameWindow.navigator.serviceWorker.getRegistration()
                   .then(function(frameRegistration) {
                     globalThis.__iframeControllerOnLoad = [
                       controller instanceof frameWindow.ServiceWorker,
                       controller.scriptURL.endsWith('/resources/empty-worker.js'),
                       controller !== result.registration.active,
                       frameRegistration.active === controller,
                       frameRegistration.scope.endsWith('/resources/blank.html')
                     ].join('|');
                     frame.remove();
                     return result.registration.unregister();
                   });
               })
               .catch(function(error) {
                 globalThis.__iframeControllerOnLoad = 'error:' + error.name + ':' + error.message;
               });
           </script>
         </body>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();

    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let value = webview.execute_script("globalThis.__iframeControllerOnLoad").unwrap();
        if value != "pending" {
            assert_eq!(value, "true|true|true|true|true");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "iframe controller-on-load regression timed out"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn iframe_register_uses_iframe_url_and_observes_statechange() {
    const PAGE_URL: &str = "https://example.test/service-workers/service-worker/registration-updateviacache.https.html";
    let mut webview = crate::WebView::new(WebViewConfig {
        service_worker_script_fetcher: Some(Arc::new(|_, script| {
            if script != "https://example.test/service-workers/service-worker/resources/empty-worker.js" {
                return Err(format!("unexpected script URL: {script}"));
            }
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: Vec::new(),
                url: script.to_string(),
                redirect_count: 0,
            })
        })),
        fetch_handler: Some(Arc::new(|request| {
            let body = if request
                .url
                .ends_with("/service-workers/service-worker/resources/blank.html")
            {
                "<!doctype html><title>blank</title>"
            } else {
                ""
            };
            Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/html".into())],
                body: body.into(),
                body_bytes: None,
            })
        })),
        ..Default::default()
    });

    webview.load_url(PAGE_URL);
    webview.complete_load(
        "<!doctype html><body>
           <script>
             globalThis.__iframeRegisterState = 'pending';
             new Promise(function(resolve) {
               var frame = document.createElement('iframe');
               frame.src = 'resources/blank.html';
               frame.onload = function() { resolve(frame); };
               document.body.appendChild(frame);
             }).then(function(frame) {
               var frameWindow = frame.contentWindow;
               return frameWindow.navigator.serviceWorker
                 .register('/service-workers/service-worker/resources/empty-worker.js', {
                   scope: 'resources/there/is/no/there/there'
                 })
                 .then(function(registration) {
                   var worker = registration.installing;
                   if (!worker) throw new Error('missing installing worker');
                   return new Promise(function(resolve, reject) {
                     function done() {
                       globalThis.__iframeRegisterState = [
                         worker instanceof frameWindow.ServiceWorker,
                         worker.state,
                         registration.scope.endsWith('/resources/there/is/no/there/there')
                       ].join('|');
                       resolve(registration);
                     }
                     if (worker.state === 'activated') {
                       done();
                       return;
                     }
                     worker.addEventListener('statechange', function() {
                       if (worker.state === 'activated') done();
                     });
                   });
                 })
                 .then(function(registration) {
                   frame.remove();
                   return registration.unregister();
                 });
             }).catch(function(error) {
               globalThis.__iframeRegisterState = 'error:' + error.name + ':' + error.message;
             });
           </script>
         </body>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();

    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let value = webview.execute_script("globalThis.__iframeRegisterState").unwrap();
        if value != "pending" {
            assert_eq!(value, "true|activated|true");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "iframe register statechange regression timed out"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn iframe_register_resolves_with_installing_worker() {
    const PAGE_URL: &str = "https://example.test/service-workers/service-worker/registration-iframe.https.html";
    let mut webview = crate::WebView::new(WebViewConfig {
        service_worker_script_fetcher: Some(Arc::new(|_, script| {
            if script != "https://example.test/service-workers/service-worker/resources/empty-worker.js" {
                return Err(format!("unexpected script URL: {script}"));
            }
            Ok(zero_net::HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/javascript".into())],
                body: Vec::new(),
                url: script.to_string(),
                redirect_count: 0,
            })
        })),
        fetch_handler: Some(Arc::new(|request| {
            let body = if request
                .url
                .ends_with("/service-workers/service-worker/resources/blank.html")
            {
                "<!doctype html><title>blank</title>"
            } else {
                ""
            };
            Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/html".into())],
                body: body.into(),
                body_bytes: None,
            })
        })),
        ..Default::default()
    });

    webview.load_url(PAGE_URL);
    webview.complete_load(
        "<!doctype html><body>
           <script>
             globalThis.__iframeInstalling = 'pending';
             new Promise(function(resolve) {
               var frame = document.createElement('iframe');
               frame.src = 'resources/blank.html';
               frame.onload = function() { resolve(frame); };
               document.body.appendChild(frame);
             }).then(function(frame) {
               var frameWindow = frame.contentWindow;
               return frameWindow.navigator.serviceWorker
                 .register('empty-worker.js', { scope: 'registration-with-valid-scope' })
                 .then(function(registration) {
                   var worker = registration.installing;
                   globalThis.__iframeInstallingImmediate = [
                     worker instanceof frameWindow.ServiceWorker,
                     worker && worker.state
                   ].join('|');
                   return new Promise(function(resolve) {
                     function done() {
                       globalThis.__iframeInstalling = [
                         globalThis.__iframeInstallingImmediate,
                         registration.scope.endsWith('/resources/registration-with-valid-scope'),
                         registration.active.scriptURL.endsWith('/resources/empty-worker.js')
                       ].join('|');
                       resolve(registration);
                     }
                     if (worker.state === 'activated') {
                       done();
                       return;
                     }
                     worker.addEventListener('statechange', function listener() {
                       if (worker.state === 'activated') {
                         worker.removeEventListener('statechange', listener);
                         done();
                       }
                     });
                   });
                 })
                 .then(function(registration) {
                   frame.remove();
                   return registration.unregister();
                 });
             }).catch(function(error) {
               globalThis.__iframeInstalling = 'error:' + error.name + ':' + error.message;
             });
           </script>
         </body>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();

    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let value = webview.execute_script("globalThis.__iframeInstalling").unwrap();
        if value != "pending" {
            assert_eq!(value, "true|installing|true|true");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "iframe register installing worker regression timed out"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[serial_test::serial(service_worker_runtime)]
fn get_registration_rejects_cross_origin_document_url() {
    const PAGE_URL: &str = "https://example.test/service-workers/service-worker/getregistration.https.html";
    let mut webview = crate::WebView::new(WebViewConfig {
        fetch_handler: Some(Arc::new(|request| {
            let body = if request
                .url
                .ends_with("/service-workers/service-worker/resources/blank.html")
            {
                "<!doctype html><title>blank</title>"
            } else {
                ""
            };
            Ok(FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/html".into())],
                body: body.into(),
                body_bytes: None,
            })
        })),
        ..Default::default()
    });

    webview.load_url(PAGE_URL);
    webview.complete_load(
        "<!doctype html><body>
           <script>
             globalThis.__crossOriginGetRegistration = 'pending';
             Promise.all([
               navigator.serviceWorker.getRegistration('https://www1.example.test/')
                 .then(
                   function() { return 'top:resolved'; },
                   function(error) { return 'top:' + error.name; }
                 ),
               new Promise(function(resolve) {
                 var frame = document.createElement('iframe');
                 frame.src = 'resources/blank.html';
                 frame.onload = function() { resolve(frame); };
                 document.body.appendChild(frame);
               }).then(function(frame) {
                 return frame.contentWindow.navigator.serviceWorker
                   .getRegistration('https://www1.example.test/')
                   .then(
                     function() { return 'iframe:resolved'; },
                     function(error) {
                       frame.remove();
                       return 'iframe:' + error.name;
                     }
                   );
               })
             ]).then(function(results) {
               globalThis.__crossOriginGetRegistration = results.join('|');
             }).catch(function(error) {
               globalThis.__crossOriginGetRegistration =
                 'error:' + error.name + ':' + error.message;
             });
           </script>
         </body>",
        None,
    );
    webview.run_page_scripts_strict().unwrap();

    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let value = webview
            .execute_script("globalThis.__crossOriginGetRegistration")
            .unwrap();
        if value != "pending" {
            assert_eq!(value, "top:SecurityError|iframe:SecurityError");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "cross-origin getRegistration regression timed out"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
