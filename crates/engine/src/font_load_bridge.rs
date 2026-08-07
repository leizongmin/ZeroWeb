//! R2949 `FontFace.load()` 宿主桥——JS `new FontFace(family, src).load()` 经 `__zw_load_font` 回调
//! 把加载请求投递到共享队列，runtime（renderer/browser 主线程）drain 后 fetch_get 字节 + load_font +
//! register_family_alias + set_font_resolver + request_rerender，再经 `AsyncResolver::resolve` 解析 Promise。
//!
//! 桥本身**不抓取**（fetch 在 runtime 线程，复用既有 fetch_get + 字体加载代码）——worker 线程回调仅
//! push 请求（同 mutations 队列模式），保持 FontLoader / font_resolver 的单线程归属不变。

use std::sync::{Arc, Mutex};

use zero_script_sandbox::Sandbox;

/// 一个 `FontFace.load()` 请求（worker 投递 → runtime 消费）。
#[derive(Debug, Clone)]
pub struct FontLoadRequest {
    /// @font-face family（注册键基础）。
    pub family: String,
    /// 字体源 URL（runtime 经 fetch_get 取字节）。
    pub src: String,
    /// `__zw_pending` 回调 id（runtime 完成后 `async_resolver.resolve(id, "ok"/"err")` 解析 Promise）。
    pub resolve_id: String,
    /// weight（数字；None→默认）。runtime 按 ≥600 构粗体键（R2417）。
    pub weight: Option<u16>,
    /// italic 标志（style=italic/oblique）。runtime 构 italic 键（R2493）。
    pub is_italic: bool,
}

/// `__zw_load_font` 回调 → 共享队列桥。worker 线程 push，runtime 线程 drain。
pub struct FontLoadBridge {
    queue: Arc<Mutex<Vec<FontLoadRequest>>>,
}

impl FontLoadBridge {
    /// 构造（空队列）。
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 共享队列句柄（runtime 经此 drain——`mem::take` 取出全部请求）。
    pub fn queue(&self) -> Arc<Mutex<Vec<FontLoadRequest>>> {
        Arc::clone(&self.queue)
    }

    /// 注册 `__zw_load_font(family, src, id, weightNum, isItalic)` 回调。worker 线程调，仅 push 请求后返 ""。
    /// 实际 fetch+register+resolve 由 runtime drain 时完成（async_resolver 解析 Promise）。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let queue = Arc::clone(&self.queue);
        sandbox.register_callback(
            "__zw_load_font",
            Box::new(move |args: &[String]| -> String {
                let family = args.first().cloned().unwrap_or_default();
                let src = args.get(1).cloned().unwrap_or_default();
                let resolve_id = args.get(2).cloned().unwrap_or_default();
                let weight = args.get(3).and_then(|s| s.trim().parse::<u16>().ok());
                let is_italic = args.get(4).map(|s| s == "true").unwrap_or(false);
                if let Ok(mut q) = queue.lock() {
                    q.push(FontLoadRequest {
                        family,
                        src,
                        resolve_id,
                        weight,
                        is_italic,
                    });
                }
                String::new()
            }),
        );
    }
}

impl Default for FontLoadBridge {
    fn default() -> Self {
        Self::new()
    }
}
