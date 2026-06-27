//! 异步脚本预取 — 替代阻塞式 `build_script_fetch_cache`。

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::Receiver;

use zero_engine::{PageScript, extract_page_scripts, resolve_document_url};
use zero_page_runtime::{AsyncFetchHost, ResourceFetchMeta};
use zero_script_sandbox::extract_module_import_specifiers;

/// 进行中的脚本预取。
pub struct PendingScriptPrefetch {
    queue: VecDeque<String>,
    cache: HashMap<String, String>,
    seen: HashSet<String>,
    inflight: Vec<(String, Receiver<Result<String, String>>)>,
}

impl PendingScriptPrefetch {
    /// 从 HTML 构造脚本预取队列。
    pub fn from_html(base_url: &str, html: &str) -> Self {
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
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(text) => {
                        for spec in extract_module_import_specifiers(&text) {
                            let dep = resolve_document_url(url, &spec);
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
            } else {
                true
            }
        });
        changed
    }

    /// 预取完成后取出脚本 cache。
    pub fn finish(self) -> HashMap<String, String> {
        self.cache
    }
}
