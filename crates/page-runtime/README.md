# ZeroWeb Page Runtime (`zero-page-runtime`)

> WPT / TabWorker / zero-renderer 三条页面路径共享的运行时契约与处理逻辑

## 概述

`ZeroWeb Page Runtime` (`zero-page-runtime`) 把三条页面路径（WPT 测试、浏览器 TabWorker、多进程 renderer）共享的页面处理逻辑抽象成统一契约，避免各路径重复实现加载、绘制、脚本执行与表单交互语义。页面加载的核心算法（`FirstPaint → FetchingStylesheets → StyledPaint → FetchingImages → Complete` 分阶段加载）经本 crate 的 trait 与具体宿主解耦，in-process（webview / net_pool）与 IPC（renderer）两种宿主实现差异只留在实现侧。

详见 `docs/specs/runtime-unification.md`。

## 主要功能

- **`PageLoadHost`** — 分阶段页面加载宿主抽象（资源抓取 + 绘制推送），WPT / TabWorker / zero-renderer 三路径统一的 spine
- **`AsyncFetchHost`** — 异步抓取宿主 trait：in-process 实现走 net_pool 线程池，IPC 实现把阻塞抓取封装到独立线程后返回可轮询 `Receiver`
- **`BlockingFetchHost`** — 把同步阻塞 fetch（如 renderer 的 IPC 抓取）适配成 `AsyncFetchHost`，每次调用立即预填一次性 `Receiver`，供 webview 的 `AsyncPageLoad` 轮询模型无缝消费
- **`FrameModel`** — 统一绘制帧契约：视口 + 文档高度 + 渲染图元 + 脏区域 + hit-test 缓存，renderer（IPC）与 tabworker（in-process）两侧共用产出结构
- **`JsExecutor`** — 统一脚本执行器契约：设 DOM 快照 → 执行脚本/module → 读回 DOM 变更，renderer 与 tabworker 走同一套脚本调度语义
- **`fetch_meta`** — `ResourceFetchMeta` 资源抓取元数据（文档 / 图片 / CSS）
- **`frame_invalidation`** — `FrameInvalidation` / `FrameTransaction` 帧失效与事务批处理
- **`form_control`** — `FormControlStateStore` / `PageInteractionState` 表单控件与页面交互状态跟踪（焦点、值基线、change-on-blur）
- **`html_actions`** — `HtmlActionPlan` / `HtmlUserAction` / `PageEffect` 等 HTML 用户动作的计划、执行与副作用（导航 / 聚焦 / 表单提交）

## 使用示例

```rust
use zero_page_runtime::{AsyncFetchHost, BlockingFetchHost, FrameModel, JsExecutor, PageLoadHost};

// 把同步阻塞 fetch 适配成异步轮询模型（renderer IPC 场景）
let mut host = BlockingFetchHost::new(|url: &str| {
    // 这里是阻塞的 IPC fetch
    Ok(format!("body:{url}").into_bytes())
});
let rx = host.fetch_bytes("http://example.com");
assert_eq!(rx.recv().unwrap().unwrap(), b"body:http://example.com");

// 分阶段加载宿主：实现抓取与绘制推送即接入三路径统一加载算法
struct MyHost;
impl PageLoadHost for MyHost {
    fn fetch_bytes(&mut self, url: &str) -> Result<Vec<u8>, String> {
        Ok(url.as_bytes().to_vec())
    }
    fn publish(&mut self, result: &zero_engine::RenderResult, title: Option<String>, is_final: bool) -> Result<(), String> {
        Ok(())
    }
}
```
