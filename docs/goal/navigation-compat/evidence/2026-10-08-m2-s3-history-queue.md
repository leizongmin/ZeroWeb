# M2-S3 — session history 语义收尾切片（2026-10-08）

**通道**: `make testharness-navigation`（test-guard 包裹）　**WPT pin**: `3159769338`
**全量运行日志**: [2026-10-08-m2-s3-full-corpus.txt](2026-10-08-m2-s3-full-corpus.txt)
**前序**: [2026-10-08-m2-s2-traverse-events.md](2026-10-08-m2-s2-traverse-events.md)

## 切片内容（M1 gap 归类 #2 残余：the-history-interface 81.6% → 修齐）

| 改动 | 文件 | spec 锚点 |
|---|---|---|
| `back/forward/go` **入队**到 task 末尾 FIFO 结算——位置在执行时对当期 cursor 计算、越界跳过、每条生效即派 popstate/hashchange | `crates/engine/src/js_dom_shim/part02.js`（`_hist_queueTraversal`/`_hist_applyTraversal`） | HTML spec traverse 步骤为排队算法；WPT 004「.go commands should be queued until the thread has ended」 |
| `pushState/replaceState` 空串 url ≡ 当前文档 URL（**保留 fragment**——URL parser 空 input 剥 fragment，与此特判不同） | 同上（`_histStateUrlOrNull`） | WHATWG html issue 9343 决议；WPT pushstate-replacestate-empty-string |
| `pushState/replaceState` 跨源 url 抛 SecurityError DOMException（origin 比较，globalThis.DOMException 优先；无 URL 通道保持 permissive） | 同上 | spec pushState/replaceState step「different origin → throw」；WPT history_pushstate_err / history_replacestate_err |
| `__zw_reset_history` 清 traverse 队列（跨文档导航后旧页残留 go() 不复现） | 同上 | 导航重置面自洽 |

## 数字

| corpus 域 | S2 后 | 本轮 | Δ |
|---|---|---|---|
| history/the-history-interface | 40/49 = 81.6% | 45/49 = **91.8%** | +5 子测试（004 队列 ×1 + SecurityError ×2 + 空串 URL ×2） |
| traversal/history-traversal | 42/45 = 93.3% | 42/45 = 93.3% | 0（零回归） |
| history/the-location-interface | 31/36 = 86.1% | 31/36 = 86.1% | 0（零回归） |
| 全量 | 125 P = 29.2% | **130 P = 30.4%** | +5 |

零回归：S2 全部 Pass 案（125）本轮全保持 Pass。

## 余下 3 失败归类（the-history-interface）

1. **007.html**：`/xhr/resources/delay.py` 缺失——runner fetch 通道无 CGI 执行面，helper 资产
   不可执行（goal 域外 infra，同 /common/dispatcher 记账口径）。
2. **pushstate/replacestate too_many_calls.optional ×2**：UA pushState 速率限制面
   （`.optional.html` 非规范强制语义）——挂账不修。

## 质量门禁

- `make test`：全绿——20,183 P / 0 F（含 4 个 R3004/R3005/R3006/R3009 单测按 M2-S3
  入队模型更新：traverse 后的结算断言移到 execute 边界之后，断言语义不变）
- `cargo fmt --all -- --check`：零 diff（本切片仅 .js + 单测 .rs 变更；clippy 沿用
  S2 合并态零 warning 证据——.rs 改动仅测试文件且 fmt 干净）
