# M1-5 Service Worker Core WPT Baseline

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；baseline green

## 0. Runner contract

新增 `testharness-service-workers` 子命令，固定执行 disposition contract 中 12 个 core source：

- Tier A：8 case / 28 subtest；
- next-wave：3 case / 4 subtest；
- static-wave：1 case / 4 subtest。

runner 不扫描目录。缺 `testharness.js` 或任一 case 时显式 Fail；Service Worker script URL
`https://wpt.test/...` 确定映射到 pinned 本地资产，外部 HTTP(S) origin fail closed。

Make 入口：

- `make testharness-service-workers-core`：全绿门禁，任一非 Pass 时非零退出；
- `make baseline-wpt-service-workers-core`：连续执行两轮，校验 12/36 与
  `(case, subtest, status)` 一致；产品 Fail 不掩盖为 runner failure；
- `OUTPUT=<path>` 可保存结构化 baseline JSON。

## 1. 当前结果

| Wave | Case | Subtest | Pass | Fail | Timeout | Unsupported |
|------|-----:|--------:|-----:|-----:|--------:|------------:|
| Tier A | 8 | 28 | 28 | 0 | 0 | 0 |
| next-wave | 3 | 4 | 4 | 0 | 0 | 0 |
| static-wave | 1 | 4 | 4 | 0 | 0 | 0 |
| **合计** | **12** | **36** | **36** | **0** | **0** | **0** |

M1-5c 后 baseline 两轮得到相同 `(case, subtest, status)`。

## 2. 收敛结果

| 缺口 | 修复前 | 当前 | 证据 |
|------|-------:|-----:|------|
| lifecycle task 与 EventTarget/slot projection | 4 Fail + 1 Timeout | Pass | transition log + cursor + task projection |
| ServiceWorkerRegistration interface brand | 2 Fail | Pass | EventTarget interface constructors |
| scope conversion/normalization/validation | 4 Fail | Pass | WebIDL null + fragment + encoded separator |
| rejection DOMException shape | 1 Fail | Pass | typed SecurityError + DOMException/Error brand |
| scriptURL fragment normalization | 1 Fail | Pass | shared URL validator strips fragment |

通过面已证明：

- default/undefined scope；
- 8/10 scope scheme/normalization case；
- 8/8 script URL validation/scheme case；
- scriptURL query 与 absolute URL；
- unregister success 与 unregister twice；
- lifecycle `updatefound`、`statechange` 与 installing/waiting/active task 顺序；
- `ServiceWorker` / `ServiceWorkerRegistration` interface brand；
- scope absent/undefined/null conversion 与最大 scope 路径限制；
- scriptURL/scope fragment normalization 与 encoded separator 拒绝；
- registration rejection 的 TypeError/SecurityError 与 DOMException/Error brand；
- 12/12 case 被发现，0 Unsupported。

## 3. Runner 修正

初跑出现 11 个基础设施假 Timeout。所有 case 的 harness state 已是 phase 4、pending 0，
结果数等于注册测试数，但 completion callback 未翻转。runner 现在以该结构化终态作为完成条件，
同时保留“0 results 不得完成”保护。修正后假 Timeout 为 0。

register JS projection 同时改为先稳定暴露 `installing`，再经 timer task 轮询 manager snapshot，
消除 `serviceworkerobject-scripturl relative` 的 Pass/Fail 抖动。

M1-5b 进一步由 manager 记录不可消费的 version transition log，renderer 以独立 cursor 查询；
页面把 `installed → activating → activated` 逐 task 投影，并在 slot 更新后派发
`statechange`。`updatefound` 位于 register Promise reaction 与首个状态变化之间。

M1-5c 把 browser 与 WebView URL 校验收敛为 page-runtime 共享实现，IPC 保留
TypeError/SecurityError 分类；页面 WebIDL conversion 区分 absent/undefined/null。

## 4. 完成门禁

- [x] 12/12 case 被 runner 发现。
- [x] 36/36 subtest 有明确结果。
- [x] 0 Unsupported。
- [x] 0 Timeout。
- [x] 连续两轮 case/subtest/status 一致。
- [x] 每个 lifecycle 中间态与事件按 task 顺序可观察。
- [x] 36/36 Pass。
- [x] 0 Fail。

M1-5 core baseline 完成；这只证明 M1 lifecycle core，不代表 M2 fetch/Cache 或 M3
controller/message 已完成。

## 5. 工程门禁

- default 与 QuickJS `zero-wpt-runner` all-targets clippy 通过；
- `make baseline-wpt-service-workers-core` 通过，release runner 两轮确定性验证；
- `make test` 通过：fresh peers、workspace V8、94 项 adapter GPU、QuickJS WebView、
  QuickJS WPT runner 113/113、QuickJS renderer；
- `make bench-gate`：16/16 microbenches；welcome/medium/morning total p95 分别为
  18.20/472.82/152.23 ms；retained form p95 0.0312 ms、jank 0；绝对预算通过。
  当前 Xeon 8260 与共享 i5-13500H baseline 硬件指纹不匹配，相对指标不作比较。
