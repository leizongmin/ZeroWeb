# M3-14 Service Worker Static Module Graph

**日期**：2026-08-20
**状态**：complete

## 实现

- `ServiceWorkerRuntime` 支持 module 主脚本，递归提取、请求、编译并执行静态 import graph。
- 每批 module fetch 携带 canonical referrer URL；browser-owned manager 按 importer
  解析相对 specifier，并拒绝跨源、凭据、fragment 与非 HTTP(S) module URL。
- renderer IPC 和 WebView adapter 共享 typed `ModuleScriptsRequested` 事件；生产网络抓取
  仍由 browser process 单一拥有。
- `ModuleRegistry` 优先按 importer-relative canonical URL 查找依赖，避免不同目录下相同
  raw specifier 错配；旧页面 module 的 raw-specifier registry 行为保持兼容。
- 完整 module graph 复用 version-local script resource map，参与 1,024 URL / 64 MiB
  资源上限、持久化恢复和 update byte-for-byte comparison。
- module dynamic `import()` 继续显式拒绝；本阶段仅声明并实现静态 import。
- evaluated installing worker 可接收 `postMessage()`，满足安装中 worker 的标准消息语义。

## WPT

- 新增固定资产：
  [2026-08-20-m3-module-assets.tsv](2026-08-20-m3-module-assets.tsv)，5/5 asset 均校验
  固定 revision、字节数和 Git blob SHA。
- `registration-scope-module-static-import.https.html`：3/3 Pass：
  - module script 作为 top-level worker；
  - 静态 import 跨出 top-level script 目录；
  - 静态 import 经 Python redirect 到另一目录。
- core baseline：
  [2026-08-20-m3-module-baseline.json](2026-08-20-m3-module-baseline.json)，
  20 case / 75 subtest，75 Pass，两轮 deterministic。
- disposition：20 core / 49 defer / 183 gated / 42 skip。

## Rust 与生产路径

- V8 `zero-script-sandbox`：180/180。
- QuickJS module runtime 定向测试：2/2。
- manager Service Worker：25/25，包含递归 URL、持久化和 module dependency bytecheck。
- protocol module request round-trip、renderer host module fetch round-trip、browser IPC
  referrer/cache policy、WebView navigator module registration 均通过。
- workspace 测试全绿；两次全量并行运行分别出现既有 localhost ETag 和 renderer
  blank-page 环境抖动，精确串行复跑均通过，最终安静模式全 workspace 通过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。
- adapter GPU 94/94，CPU/GPU consistency 1/1。

## 性能

`make bench-gate` 报告 `benchmark_20260820_184846.json`：

- 16/16 microbench 通过；
- startup 92.45 ms，peak RSS 154.71 MiB；
- page total p95：15.05 / 432.51 / 107.70 ms；
- retained form p95 0.0355 ms，jank 0；
- baseline CPU 不同，relative gate 不可比较；absolute budgets 通过。

## 下一步

- 导入 `update-bytecheck.https.html`，用上游 classic/module 矩阵验证 module main 与
  imported dependency bytes 的更新判断。
- 补齐 module graph 的 re-export 语法后，再评估更广的 module WPT。
