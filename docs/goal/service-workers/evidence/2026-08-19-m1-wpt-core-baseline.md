# M1-5 Service Worker Core WPT Baseline

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：runner complete；baseline red

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

## 1. 首轮结果

| Wave | Case | Subtest | Pass | Fail | Timeout | Unsupported |
|------|-----:|--------:|-----:|-----:|--------:|------------:|
| Tier A | 8 | 28 | 18 | 9 | 1 | 0 |
| next-wave | 3 | 4 | 2 | 2 | 0 | 0 |
| static-wave | 1 | 4 | 3 | 1 | 0 | 0 |
| **合计** | **12** | **36** | **23** | **12** | **1** | **0** |

连续五轮 exploratory run 与 baseline 两轮均得到相同 `(case, subtest, status)`。错误文本不作为
确定性键，因为生命周期竞态可产生不同但等价的 TypeError 文本。

## 2. 红项分组

| 缺口 | Fail | Timeout | 证据 |
|------|-----:|--------:|------|
| lifecycle state task 与 EventTarget/slot projection | 4 | 1 | 缺 `addEventListener`；installed task 可被跳过 |
| ServiceWorkerRegistration interface brand | 2 | 0 | `ServiceWorkerRegistration is not defined` |
| scope conversion/normalization/validation | 4 | 0 | null、fragment、encoded slash/backslash |
| rejection DOMException shape | 1 | 0 | rejection 不是 DOMException + Error |
| scriptURL fragment normalization | 1 | 0 | fragment 当前被拒绝而非移除 |
| **合计** | **12** | **1** | |

通过面已证明：

- default/undefined scope；
- 8/10 scope scheme/normalization case；
- 8/8 script URL validation/scheme case；
- scriptURL query 与 absolute URL；
- unregister success 与 unregister twice；
- 12/12 case 被发现，0 Unsupported。

## 3. Runner 修正

初跑出现 11 个基础设施假 Timeout。所有 case 的 harness state 已是 phase 4、pending 0，
结果数等于注册测试数，但 completion callback 未翻转。runner 现在以该结构化终态作为完成条件，
同时保留“0 results 不得完成”保护。修正后假 Timeout 为 0；保留的 1 个 Timeout 是真实产品红项：
`installed event should be fired before activating service worker` 未观察到 installed task。

register JS projection 同时改为先稳定暴露 `installing`，再经 timer task 轮询 manager snapshot，
消除 `serviceworkerobject-scripturl relative` 的 Pass/Fail 抖动。

## 4. 未完成门禁

- [x] 12/12 case 被 runner 发现。
- [x] 36/36 subtest 有明确结果。
- [x] 0 Unsupported。
- [x] 连续两轮 case/subtest/status 一致。
- [ ] 36/36 Pass。
- [ ] 0 Fail / 0 Timeout。
- [ ] 每个 lifecycle 中间态与事件按 task 顺序可观察。

M1-5 尚未完成；当前 baseline 不能作为 Service Worker Done 结论。

## 5. 工程门禁

- default 与 QuickJS `zero-wpt-runner` all-targets clippy 通过；
- `make baseline-wpt-service-workers-core` 通过，release runner 两轮确定性验证；
- `make test` 通过：fresh peers、workspace V8、94 项 adapter GPU、QuickJS WebView、
  QuickJS WPT runner 113/113、QuickJS renderer；
- `make bench-gate`：16/16 microbenches；welcome/medium/morning total p95 分别为
  19.85/518.53/131.51 ms；retained form p95 0.039 ms、jank 0；绝对预算通过。
