# M2-S5 — pagereveal reveal 事件面（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案迭代 + 全量
per-subtest 精确 diff vs S4Z 终态）
**前序**: [2026-10-10-m2-s4z-traverse-scrollrestoration-gate.md](2026-10-10-m2-s4z-traverse-scrollrestoration-gate.md)

## 切片内容

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| testharness runner 路径 pagereveal 派发：① 页面脚本前包一层 rAF——首次调用先派 pagereveal（trusted，`__zwPagerevealFired` 门）再透传原 rAF；② 生命周期 timer 任务内 pageshow 后兜底派发（同门防双发——页面从不调 rAF 的形态） | `testharness.rs`（runner 单一 chokepoint；零 shim/产品面） | spec reveal steps（新文档显示时 window 派 pagereveal，persisted=false 新载入形；早于该渲染机会的 rAF 回调批）；WPT pagereveal/order-in-new-document-navigation「pagereveal,rAF」 |

## 定位要点

页面脚本 `addEventListener('pagereveal')` → `requestAnimationFrame(push 'rAF')`：runner
默认 rAF 是**同步 stub**（part01 reftest 兼容模型，注册即执行回调）——'rAF' 在页面脚本
任务内同步入 log，任何 timer 期派发（首版放生命周期 timer）都晚于断言点，恒 'rAF'。
spec 语义（pagereveal 先于首个渲染机会的 rAF 回调批）映射到本 runner 模型 = 首个 rAF
调用前派发——包 rAF 实现。bfcache-restore 兄弟案在 helper.sub.js 基建处先行失败
（runBfcacheTest 未定义——bfcache 挂账面），与本派发无交互；pageswap/prerender 同目录
案不在本地运行集（语料 grep 全树仅 pagereveal/pageswap 家族 listen 此事件）。

## 挂账口径修订

M4 收口评估把 pagereveal ×2 整目录归入 bfcache 族挂账；本轮定谳：
**new-document 形不依赖 bfcache**（fresh-load reveal），单文档 runner 可达——收口；
**bfcache-restore 形**（persisted=true 恢复语义）仍挂 bfcache（helper 基建 + restore
管线，M3 后续立项）。

## 数字（按 evidence txt 原始 Pass 行）

| corpus 域 | S4Z 后 | 本轮 | Δ |
|---|---|---|---|
| 全量 | 398 P / 476 = 83.6% | **399 P / 476 = 83.8%** | +1 |
| navigation-api | 237 / 255 | 237 / 255（92.9%） | 0（本案 html/browsers 域） |

**零回归**：全量 diff 恰一行（order-in-new-document-navigation Fail→Pass）。rAF 包裹
触达每个 testharness 页面，语料全树 grep 证明仅 pagereveal 家族 listen 该事件——惰性
新增面对其余 475 案零观察面（diff 实证）。

## 质量门禁

- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning（testharness.rs
  变更轮）；`cargo fmt --all -- --check`：零 diff。
- `make test`：全绿（数字见本轮终态行）。
