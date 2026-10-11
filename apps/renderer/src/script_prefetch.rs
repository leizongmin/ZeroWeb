//! 异步脚本预取 — 替代阻塞式 `build_script_fetch_cache`。

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::Receiver;

use zero_engine::{PageScript, extract_page_scripts, resolve_document_url};
use zero_page_runtime::{AsyncFetchHost, ResourceFetchMeta};
use zero_script_sandbox::{ImportMap, extract_static_module_import_specifiers};

use crate::js_worker::parse_page_import_map;

/// 进行中的脚本预取。
pub struct PendingScriptPrefetch {
    queue: VecDeque<String>,
    cache: HashMap<String, String>,
    seen: HashSet<String>,
    inflight: Vec<(String, Receiver<Result<String, String>>)>,
    import_map: Option<ImportMap>,
}

impl PendingScriptPrefetch {
    /// 从 HTML 构造脚本预取队列。
    pub fn from_html(base_url: &str, html: &str) -> Self {
        let import_map = parse_page_import_map(html, base_url);
        let mut queue = VecDeque::new();
        let mut seen = HashSet::new();
        for script in extract_page_scripts(html) {
            match script {
                PageScript::External(src) | PageScript::ExternalModule(src) => {
                    let url = resolve_document_url(base_url, &src);
                    if seen.insert(url.clone()) {
                        queue.push_back(url);
                    }
                }
                _ => {}
            }
        }
        Self {
            queue,
            cache: HashMap::new(),
            seen,
            inflight: Vec::new(),
            import_map,
        }
    }

    /// 是否仍有工作（队列或 in-flight）。
    pub fn is_active(&self) -> bool {
        !self.queue.is_empty() || !self.inflight.is_empty()
    }

    /// 推进预取（每 tick 最多 `max_parallel` 个新请求）。
    pub fn tick(&mut self, host: &mut dyn AsyncFetchHost, max_parallel: usize) -> bool {
        while self.inflight.len() < max_parallel {
            let Some(url) = self.queue.pop_front() else {
                break;
            };
            tracing::info!(url = %url, "page load: prefetch script");
            self.inflight
                .push((url.clone(), host.fetch_text_meta(&url, ResourceFetchMeta::SCRIPT)));
        }

        let mut changed = false;
        self.inflight.retain(|(url, rx)| {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(text) => {
                            // t8j：只预取**静态** import 依赖（R3093：动态 import() 留给运行时
                            // `__zw_compile_module` fetch）——全量提取器会把 Vite `import("_")`
                            // 能力探测当依赖网络预取（404 噪声 + cache 污染）。
                            // P6：裸说明符先经本代页面 import map 映射（仅实际命中映射键时改写，
                            // 与 js_worker `collect_module_deps` 同判据）；未命中保持
                            // `resolve_document_url` 原路径（URL 形态说明符零漂移）。
                            for spec in extract_static_module_import_specifiers(&text) {
                                let dep = self
                                    .import_map
                                    .as_ref()
                                    .and_then(|m| m.resolve_mapped(&spec, url))
                                    .unwrap_or_else(|| resolve_document_url(url, &spec));
                                if self.seen.insert(dep.clone()) {
                                    self.queue.push_back(dep);
                                }
                            }
                            self.cache.insert(url.clone(), text);
                        }
                        Err(e) => tracing::warn!("script prefetch {url}: {e}"),
                    }
                    changed = true;
                    false
                }
                // Sender 随导航边界 / StopLoading 的 `inflight_fetches.clear()` 被丢弃 →
                // 通道关闭。必须视作 Err 完成并移除条目：否则条目永滞 inflight，
                // `is_active()` 恒真，队列永不收口。
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    tracing::warn!("script prefetch {url}: fetch channel closed");
                    changed = true;
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
            }
        });
        changed
    }

    /// 预取完成后取出脚本 cache。
    pub fn finish(self) -> HashMap<String, String> {
        self.cache
    }
}

/// 运行中页面插入的经典外链脚本（动态 `<script src>`）的宿主侧异步取回队列。
///
/// https://html.spec.whatwg.org/multipage/scripting.html#the-script-element — script 元素
/// 连入 document 后须取回并执行；script 取回是资源取回（no-cors 语义），不经页面
/// `fetch()`（cors 语义 + CORS 检查）——无 ACAO 的脚本源在页面 fetch 下永远失败（baidu
/// a.js live）。经与初始脚本相同的宿主通路（`fetch_text_meta`，`ResourceFetchMeta::SCRIPT`）
/// 取回；完成结果逐 URL 回调 `on_complete`（Ok(源码)/Err(原因)），执行与 load/error 事件
/// 由调用方（runtime `tick_dynamic_scripts`）处理。并行上限与 [`PendingScriptPrefetch`] 同型。
pub struct PendingDynamicScripts {
    queue: VecDeque<String>,
    inflight: Vec<(String, Receiver<Result<String, String>>)>,
}

impl PendingDynamicScripts {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            inflight: Vec::new(),
        }
    }

    /// 入队一个绝对 URL 脚本（去重由调用方负责）。
    pub fn push(&mut self, url: String) {
        self.queue.push_back(url);
    }

    /// 是否仍有工作（队列或 in-flight）。
    pub fn is_active(&self) -> bool {
        !self.queue.is_empty() || !self.inflight.is_empty()
    }

    /// 推进取回（每 tick 最多 `max_parallel` 个新请求）；完成的 (url, 结果) 逐个回调
    /// `on_complete`（Ok(源码)/Err(原因)）。
    pub fn tick(
        &mut self,
        host: &mut dyn AsyncFetchHost,
        max_parallel: usize,
        mut on_complete: impl FnMut(&str, Result<&str, &str>),
    ) {
        while self.inflight.len() < max_parallel {
            let Some(url) = self.queue.pop_front() else {
                break;
            };
            tracing::info!(url = %url, "dynamic script: host fetch");
            self.inflight
                .push((url.clone(), host.fetch_text_meta(&url, ResourceFetchMeta::SCRIPT)));
        }

        self.inflight.retain(|(url, rx)| {
            match rx.try_recv() {
                Ok(result) => {
                    match &result {
                        Ok(text) => on_complete(url, Ok(text)),
                        Err(e) => on_complete(url, Err(e)),
                    }
                    false
                }
                // Sender 随导航边界 / StopLoading 的 `inflight_fetches.clear()` 被丢弃 →
                // 通道关闭。视作 Err 完成回调并移除条目，避免 `is_active()` 恒真、
                // 队列永久卡死（renderer 进程生命周期内 tick 分支常开）。
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    on_complete(url, Err("fetch channel closed"));
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
            }
        });
    }
}

#[cfg(test)]
mod dynamic_scripts_tests {
    use super::*;
    use std::sync::mpsc::{Sender, channel};

    /// 测试宿主：预置逐 URL 应答（Err 优先），未命中 URL 即刻 Err。
    struct ScriptStubHost {
        responses: HashMap<String, Result<String, String>>,
    }
    impl AsyncFetchHost for ScriptStubHost {
        fn fetch_text_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
            let (tx, rx) = channel();
            let _ = tx.send(
                self.responses
                    .get(url)
                    .cloned()
                    .unwrap_or_else(|| Err("not stubbed".into())),
            );
            rx
        }

        fn fetch_bytes_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
            let (tx, rx) = channel();
            let _ = tx.send(Err("not used".into()));
            rx
        }
    }

    #[test]
    fn dynamic_scripts_complete_with_ok_and_err_callbacks() {
        let mut pending = PendingDynamicScripts::new();
        pending.push("https://zero.test/a.js".into());
        pending.push("https://zero.test/b.js".into());
        pending.push("https://zero.test/c.js".into());
        let mut host = ScriptStubHost {
            responses: HashMap::from([
                (
                    "https://zero.test/a.js".to_string(),
                    Ok("globalThis.__a = 1;".to_string()),
                ),
                ("https://zero.test/b.js".to_string(), Err("dns fail".to_string())),
            ]),
        };
        let completions: std::sync::Mutex<Vec<(String, Result<String, String>)>> = std::sync::Mutex::new(Vec::new());
        {
            let completions = &completions;
            pending.tick(&mut host, 4, |url, result| {
                completions.lock().unwrap().push((
                    url.to_string(),
                    result.map(|t| t.to_string()).map_err(|e| e.to_string()),
                ));
            });
        }
        let done = completions.into_inner().unwrap();
        assert_eq!(done.len(), 3, "stub 即刻应答 → 三条全部完成（c.js 未 stubbed 也 Err）");
        assert!(
            done.contains(&(
                "https://zero.test/a.js".to_string(),
                Ok("globalThis.__a = 1;".to_string())
            )),
            "Ok 完成回调携带源码：{done:?}"
        );
        assert!(
            done.contains(&("https://zero.test/b.js".to_string(), Err("dns fail".to_string()))),
            "Err 完成回调携带原因：{done:?}"
        );
        assert!(
            done.contains(&("https://zero.test/c.js".to_string(), Err("not stubbed".to_string()))),
            "未 stubbed URL 走 Err 完成回调：{done:?}"
        );
        assert!(!pending.is_active(), "全部完成 → 队列排空");
    }

    /// 测试宿主：永不完成——保活 Sender（rx 停留 Empty 而非 Disconnected），
    /// 验证并行上限与 inflight 停留。
    struct HangingHost {
        held: Vec<Sender<Result<String, String>>>,
    }
    impl AsyncFetchHost for HangingHost {
        fn fetch_text_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
            let (tx, rx) = channel();
            self.held.push(tx);
            rx
        }

        fn fetch_bytes_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
            // 本测试面只经 text 通路；bytes 通道不保活（即刻断开无碍）。
            let (_tx, rx) = channel();
            rx
        }
    }

    #[test]
    fn dynamic_scripts_parallel_cap_bounds_new_requests_per_tick() {
        let mut pending = PendingDynamicScripts::new();
        for i in 0..6 {
            pending.push(format!("https://zero.test/{i}.js"));
        }
        let mut host = HangingHost { held: Vec::new() };
        let mut started = 0usize;
        pending.tick(&mut host, 4, |_, _| {
            started += 1;
        });
        assert_eq!(started, 0, "无完成则无回调");
        assert_eq!(pending.queue.len(), 2, "每 tick 新请求钳到 max_parallel=4");
        assert!(pending.is_active(), "4 inflight + 2 queued");
    }

    /// t8j（site-compat bilibili laputa-home）：预取层只沿**静态** import 依赖展开——
    /// 动态 `import()` spec（Vite `import("_")` 探测形态）不得当依赖网络预取
    /// （404 噪声 + cache 污染）。与 renderer js_worker / browser tab_js_worker 同判据。
    #[test]
    fn prefetch_expands_static_module_imports_only() {
        struct RecordingHost {
            requested: std::sync::Mutex<Vec<String>>,
            responses: HashMap<String, String>,
        }
        impl AsyncFetchHost for RecordingHost {
            fn fetch_text_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
                self.requested.lock().unwrap().push(url.to_string());
                let (tx, rx) = channel();
                let _ = tx.send(
                    self.responses
                        .get(url)
                        .cloned()
                        .map(Ok)
                        .unwrap_or_else(|| Err("not stubbed".into())),
                );
                rx
            }

            fn fetch_bytes_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
                let (tx, rx) = channel();
                let _ = tx.send(Err("not used".into()));
                rx
            }
        }

        let mut host = RecordingHost {
            requested: std::sync::Mutex::new(Vec::new()),
            responses: HashMap::from([(
                "https://zero.test/m.js".to_string(),
                "import st from './t.js'\nimport('./s.js')\nimport('_')\nexport default st".to_string(),
            )]),
        };
        let mut pending =
            PendingScriptPrefetch::from_html("https://zero.test/", r#"<script type="module" src="m.js"></script>"#);
        let mut guard = 0;
        while pending.is_active() && guard < 10 {
            pending.tick(&mut host, 4);
            guard += 1;
        }
        let mut requested = host.requested.into_inner().unwrap();
        requested.sort();
        assert_eq!(
            requested,
            vec![
                "https://zero.test/m.js".to_string(),
                "https://zero.test/t.js".to_string(),
            ],
            "只预取静态 import 依赖（红态：'./s.js' 与 '_' 被动态提取进预取队列）：{requested:?}"
        );
    }

    /// P6：静态 import 依赖的裸说明符先经页面 import map 映射再预取——未命中 map 的
    /// 相对说明符保持 document 相对路径（零漂移）。与 js_worker `collect_module_deps`
    /// 同判据（github home：无 map 时 `assets/react` 裸 join 404，map 命中后取哈希 URL）。
    #[test]
    fn prefetch_applies_page_import_map_to_bare_specifiers() {
        struct RecordingHost {
            requested: std::sync::Mutex<Vec<String>>,
            responses: HashMap<String, String>,
        }
        impl AsyncFetchHost for RecordingHost {
            fn fetch_text_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
                self.requested.lock().unwrap().push(url.to_string());
                let (tx, rx) = channel();
                let _ = tx.send(
                    self.responses
                        .get(url)
                        .cloned()
                        .map(Ok)
                        .unwrap_or_else(|| Err("not stubbed".into())),
                );
                rx
            }

            fn fetch_bytes_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
                let (tx, rx) = channel();
                let _ = tx.send(Err("not used".into()));
                rx
            }
        }

        let html = r#"<script type="importmap">{"imports":{"react":"https://zero.test/react-e27d1b3e.js"}}</script>
<script type="module" src="m.js"></script>"#;
        let mut host = RecordingHost {
            requested: std::sync::Mutex::new(Vec::new()),
            responses: HashMap::from([(
                "https://zero.test/m.js".to_string(),
                "import React from 'react'\nimport t from './t.js'\nexport default React(t)".to_string(),
            )]),
        };
        let mut pending = PendingScriptPrefetch::from_html("https://zero.test/", html);
        let mut guard = 0;
        while pending.is_active() && guard < 10 {
            pending.tick(&mut host, 4);
            guard += 1;
        }
        let mut requested = host.requested.into_inner().unwrap();
        requested.sort();
        assert_eq!(
            requested,
            vec![
                "https://zero.test/m.js".to_string(),
                "https://zero.test/react-e27d1b3e.js".to_string(),
                "https://zero.test/t.js".to_string(),
            ],
            "裸说明符 'react' 经 map 取哈希 URL，'./t.js' 保持相对路径（红态：'react' 裸 join 成 \
             https://zero.test/react 404）：{requested:?}"
        );
    }

    /// Sender 被丢弃（导航边界 / StopLoading 清 inflight_fetches）→ 通道关闭；
    /// tick 须把 Disconnected 收敛为 Err 完成回调并移除条目，避免队列永不收口。
    #[test]
    fn dynamic_scripts_disconnected_channel_completes_with_err() {
        // 只构造 inflight 条目（不经 push——queue 里的条目会被 fill 循环重新签发，
        // 干扰「断开收敛」断言）。Sender 端即刻丢弃，模拟导航边界 / StopLoading 的
        // inflight_fetches.clear() 语义。
        let mut pending = PendingDynamicScripts {
            queue: std::collections::VecDeque::new(),
            inflight: Vec::new(),
        };
        let rx = {
            let (tx, rx) = channel();
            drop(tx);
            rx
        };
        pending.inflight.push(("https://zero.test/gone.js".into(), rx));
        assert!(pending.is_active());
        let mut host = HangingHost { held: Vec::new() };
        let completions: std::sync::Mutex<Vec<(String, Result<String, String>)>> = std::sync::Mutex::new(Vec::new());
        {
            let completions = &completions;
            pending.tick(&mut host, 4, |url, result| {
                completions.lock().unwrap().push((
                    url.to_string(),
                    result.map(|t| t.to_string()).map_err(|e| e.to_string()),
                ));
            });
        }
        let done = completions.into_inner().unwrap();
        assert_eq!(done.len(), 1, "Disconnected → 一次 Err 完成回调：{done:?}");
        assert!(done[0].1.is_err(), "Disconnected 收敛为 Err：{done:?}");
        assert!(!pending.is_active(), "断开条目移除 → 队列收口");
    }
}
