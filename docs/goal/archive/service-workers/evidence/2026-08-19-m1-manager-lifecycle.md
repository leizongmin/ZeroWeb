# M1-2 ServiceWorkerManager 生命周期协调

**日期**：2026-08-19
**状态**：M1-2 complete
**前置**：[M1-1 typed runtime](2026-08-19-m1-threaded-runtime.md)

## 0. 交付

- `zero-page-runtime::ServiceWorkerManager` 成为 registration/version/runtime 的单一可变
  owner。
- registration key 为同一 storage partition 内的 `(origin, normalized scope)`。
- 每个 key 维护独立 `installing` / `waiting` / `active` version slot。
- manager 持有自己的 `ServiceWorkerRegistry` 与 `ServiceWorkerRuntime` map，只向调用方暴露
  immutable registration/slot 查询。
- typed evaluate 结果驱动 manager 事件：
  - success：版本保持 `Installing`，等待真实 install event；
  - compile/runtime/timeout/closed：版本转 `Redundant`，清 installing slot 并回收 runtime。
- install/activate 完成入口按宿主提供的 lifetime outcome 推进状态：
  - install fulfilled：`Installing -> Installed` 并进入 waiting；
  - install rejected：新版本 redundant，旧 active 保持；
  - activate fulfilled：`Activating -> Activated`，仅替换同 key active；
  - activate rejected：新版本 redundant，旧 active 保持。

## 1. Scope 与并发

- 同一 origin/scope 同时只允许一个 installing job。
- 不同 scope 可以并行 evaluate。
- active lookup 先过滤 origin，再按最长匹配 scope 选择版本。
- `/` 与 `/app/` 可同时 active；替换 `/app/` 不影响 `/`。

manager 不使用 legacy registry 的“一 origin 一 active”映射作为权威；后续 bridge/controller
必须消费 manager slot。

## 2. 信任边界

- origin/scope/script URL：非空且各自最多 64 KiB；
- script source：最多 16 MiB；
- live runtime：默认最多 32；
- capacity/oversize 拒绝发生在创建 runtime 和写入 registry 之前；
- manager error/event 不携带 script source。

URL secure-context、same-origin、scheme、Service-Worker-Allowed 与真正 script fetch 属于 M1-3
host adapter，当前 API 明确接收已经过 host 安全校验和规范化的输入。

## 3. 验证

manager conformance 共 10 项，两后端运行相同测试：

- 完整 installing -> waiting -> active slot 序列；
- evaluate compile failure 清理；
- install/activate rejection 保留旧 active；
- 同 scope active 替换、跨 scope 隔离和最长 scope lookup；
- 同 key job 串行、不同 key 并行；
- evaluate 完成前禁止 install completion；
- runtime capacity 拒绝零副作用；
- oversized input 拒绝零副作用。

| 矩阵 | 结果 |
|------|------|
| `zero-page-runtime` 默认 V8 | 56/56 |
| QuickJS-only | 56/56 |
| V8 + QuickJS feature union | 56/56 |
| 三种 `--all-targets -D warnings` clippy | 全部通过 |

## 4. 未完成边界

- manager 仍接收已抓取 script 字节，没有网络 host adapter；
- runtime 尚未派发真实 install/activate event 或聚合 `waitUntil()`；
- 页面 R3318 shim、WebView adapter、browser owner IPC 均未接入；
- 当前 lifecycle completion 由 Rust 测试直接喂 outcome，只证明状态协调算法；
- 未触碰 M2 fetch bridge。

M1-3 必须把 script fetch、ServiceWorkerGlobalScope bootstrap、真实 lifecycle event outcome 和
in-process bridge 接到该 manager，不能在 WebView 另写第二份状态机。
