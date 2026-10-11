# M3 收口判定 — StorageEvent 构造/init + DC-1~4 终验

**日期**: 2026-10-11
**判定**: **goal Completed（DC-1~4 全部满足）**

## M3 改动（StorageEvent 接口保真）

`crates/engine/src/js_dom_shim/part05.js`——StorageEvent wrapper 构造器 +
`initStorageEvent` 原型成员（webstorage-compat M3，WPT event_constructor /
event_initstorageevent 口径）：

1. 接口构造器无 [[Call]]（无 new 抛 TypeError，PopStateEvent 先例）；
2. `type` 必填（0 参 TypeError，`StorageEvent.length === 1`）；
3. `url` 非空 USVString：dict 显式 null → `'null'`（基类 `_defineEventSubclass`
   泛型 `!= null` 回落把 null 吞成 `''`；undefined/缺省仍 `''`）；
4. `initStorageEvent`：8 参全反射、type 必填、重置语义（bubbles/cancelable
   Boolean 化；key/oldValue/newValue/storageArea 可空缺省 null；url 不解析相对
   URL）、`initStorageEvent.length === 1`。

**A/B**: 1243/1251 = 99.4%（M2-P3 后）→ **1251/1251 = 100%**（+8P：event_constructor
3/6→6/6、event_initstorageevent 0/5→5/5）。`make testharness-webstorage` 退出码
0（全绿）。

## DC 判定

| DC | 项 | 判定 |
|---|---|---|
| DC-1 | corpus 导入 + 甄别 + 基线 + csv 回填 | ✅ 2026-10-11（[m1-baseline](2026-10-11-m1-baseline.md)，25 案可执行子集） |
| DC-2 | 存取/配额语义收敛 + origin 隔离/键序收口出数字 | ✅ M2-P1/P2/P3 三切片（named property 语义、WebIDL arity/ToUint32、5MiB 配额 + QuotaExceededError 接口）；轨迹 94.9%→97.6%→99.0%→99.4% 单调无下行轮次；键序（enumerate/key/indexing）与 origin 单文档面在可执行子集内全绿 |
| DC-3 | storage 事件甄别切片 + 收口 | ✅ 甄别结论落档（可执行子集 = 构造/init 2 案，本切片全绿；派发面挂账见下）；不可执行面挂账理由成立 |
| DC-4 | 每修复附回归测试 + 门禁 | ✅ 每切片 A/B + `WEBSTORAGE_CASES` 常驻断言集（runner 固定清单 + `make testharness-webstorage` 可重建，encoding 先例——corpus gitignored 按需重拉，不占 reftest 账本）；门禁见下 |

## 收口门禁（DC-4，2026-10-11 实测）

- **make test**（分解逐腿复跑，全部 0 failed）：workspace 71 套件 + quickjs clippy +
  renderer --bin 685P + renderer --lib 239P + zero-browser --bin 488P + GPU adapter
  96P + surface 一致性 1P + quickjs 五 crate 767P + renderer quickjs 双腿 685P/239P。
  归因注记：完整 `make test` 连跑三次均于 zero-browser xvfb 腿静默终止（无错误输出、
  进程树消失）；该腿单独复跑 LEG-EXIT=0（488P），renderer onload 失败案单测三次
  全 Pass 且全 lib 套件 241P 全绿——判定为并行流（另一 clone 的 test-webui/browser
  任务）资源争用杀树，非本 goal 变更回归（§10 归因纪律，A/B 经 git stash 隔离）。
- **make reftest**：708/708 = 100%，0 不一致（零回归）。
- **make product-smoke**：welcome vs chromium Oracle diff **15.40% ≤ 20%**；6/6
  struct-check PASS（welcome 三 viewport + article 三 viewport）；form-input-perf
  gate PASS。产品 fixture 资产零改动（welcome.html 字节不变）。
- fmt 干净 + clippy `-D warnings` 零告警（zero-engine/zero-wpt-runner 包级）。
- zero-engine lib 2977P/0F（三轮复跑；`--skip test_pipeline_deeply_nested_html`
  为干净 main 预存 stack overflow，git stash A/B 归因非本轮）。

## 最终账本（分级通过率）

| 域 | 案数 | subtests | 通过率 |
|---|---|---|---|
| Storage 存取/枚举/配额语义（`.window.js`） | 23 | 1234/1234 | 100% |
| StorageEvent 构造/init（可执行子集） | 2 | 17/17 | 100% |
| **合计** | **25** | **1251/1251** | **100%** |

排除面（DC-3 挂账，全部有归属）：

| 排除域 | 案数 | 理由 | 重入条件 |
|---|---|---|---|
| `event_*.html` 派发族 | 19 | eventTestHarness.js 经 iframe 跨文档派发 + setTimeout 轮询，runner 无多 frame 管道 | **navigation-compat M3**（frame 树）落地后评估 |
| window.open 双上下文（storage_*_window_*.window.js） | 4 | 第二 browsing context（会话历史/持久性） | 同上 |
| 跨源 iframe（localstorage-cross-origin-iframe） | 1 | 跨源双文档 | 同上 |
| document-domain / 分区 / 跨源共享（3+1 案） | 4 | document.domain 松弛 / 第一方-第三方分区 / 跨源对——跨上下文 origin 面，单文档静态 runner 无对应域 | 同上（分区面另涉 net/storage 分区域） |
| manual（storage_local/session-manual） | 2 | 需真实用户操作 | 不重入 |

余量诚实注记：100% 是**可执行子集**口径；上游全语料 26 `.html` + 30 `.window.js`
中 30 案挂账（上表），派发面语义（storage 事件触发/属性填充/跨文档同步）未在本
goal 度量，不宣称全语料 100%。

## 数字维护

全量复跑 `make testharness-webstorage` 后向 `docs/compat/trends/wpt-suites.csv`
追加行；本次收口行 = completed，1251/1251。
