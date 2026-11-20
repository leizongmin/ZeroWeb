# DC-2 · 批四：webappapis 簇整批接入（语料 + 通道 + 记账）

2026-11-20。五簇之一（11-20 快照缺口 3,672）整批落地：46 目录语料拉取（pin
31597693 git trees 显式枚举，`GOAL_PULL_ANY_JS=1`——microtask-queuing/timers/atob
等域 .any.js 唯一形态）+ html-syntax 通道加收 25 计分目录 + .https 命名面簇内解锁。
release runner 实测净 **+1,537 Pass**（html-syntax 63,737 → 65,274，462 → 669 键），
共享键零回归（462 键数与 Pass 63,737 与批三账逐字一致；carveout 以
`html/webappapis/` 目录前缀门控，簇外执行面逐行无改动）。

## 变更（三文件）

1. `tests/wpt-runner/scripts/goals/57-wptfyi-html-webappapis.sh`——46 目录显式
   拉取（fetch_dir_html 只递归一层，depth-3+ 资产目录逐个显式；幂等标记防限流）。
   `.worker.js`（4 源）不拉——runner 本通道零消费，worker.html 分母 562 子测试挂账。
2. `Makefile`——`fetch-wpt-html-webappapis` target，挂入 `testharness-html-syntax`
   依赖（fetch 幂等，套件跑前自动补拉）。
3. `tests/wpt-runner/src/testharness.rs`——`HTML_SYNTAX_CORPUS_SUBDIRS` 加收 25
   计分目录（resources/support 助资产目录只拉不扫，分母 0 键实证）；
   `html_syntax_case_skipped` 两处扩门：
   - `.https.html` 命名排除对 `html/webappapis/` 簇解锁（runner 恒
     `https://wpt.test`、isSecureContext 恒真，idlharness carveout 同前提）；
     protocol.https.html 等 8 键不落命名排除。
   - 首跑实证收口两处溢出：`-manual.https.html` 复合形补入 manual 命名门（原门只
     匹配 `-manual.html`，carveout 放行 .https 命名后 5 个 protocol-handler-*-manual
     案整案 Timeout 挂账——wpt.fyi 分母不含 manual 案）；js 源 `META: global=` 显式
     收窄对齐（requires-success.any.js = dedicatedworker,sharedworker 无 window
     全局，不生成 window 形键——window 形跑 Atomics.wait 假阳性 Pass，wpt.fyi
     分母同形无 .any.html 键；缺省无 META 行保持全全局语义，atob 23 源先例）。

## 结果（release runner，test-guard 包裹，全量套件）

| 面 | 变更前 | 变更后 |
|---|---:|---:|
| html-syntax 键 | 462 | 669（+207 webappapis） |
| html-syntax Pass | 63,737 | 65,274（+1,537） |
| Fail | — | webappapis +968 |
| Timeout | — | webappapis +23 |
| Unsupported | — | webappapis +6 |

- 执行 207 键 2,535 子测试，**207/207 全部对上分母**（0 键无分母；分母逐字对齐
  182/207）。非 Pass 面：Fail 968 / Timeout 23 / Unsupported 6 /
  PreconditionFailed 1（structured-clone.any.js 148 Pass 内 1 transfer 面前置失败）。
- Unsupported 6 键 = protocol-handler-* 非 manual 案（testdriver 依赖）+
  event-handler-onmove-02.tentative，整案诚实记账。

## 余面挂账（126 键未执行，通道内逐键归因）

通道内分母（summary-chrome 4b26db8978）333 键 2,893 子测试（chrome 全过 2,992）：
html 含 .https 复合形 267 / window.html 48 / any.html 18。已执行 207，余 126：

- **iframe 内容门 107 键**（`<iframe`/`createElement('iframe')`——runner 无多
  frame 文档管道，net-api 先例；与 master 卡点 iframe 子文档管道同根因，零-web
  流域待碰头）。document-write iframe_001..017、opening-the-input-stream
  window 源族等。
- **上游漂移 14 键**（pin 31597693 之后上游新增案——zero-settimeout.any.js 先例
  注记同源；timers/errors、event-handler-no-browsing-context 等 14 文件 ~35
  子测试，随周跑另报）。
- **worker 内容门 5 键**（`new Worker(`/SharedWorker/serviceWorker——worker 执行
  通道域外）。worker 族生成形（31 键 654 子测试）与 xhtml/svg 形（6 键）按形
  通道外挂账，不在此 126 内。

## Fail 战役图（968 Fail 集中度）

- navigator.protocol 面 244（protocol.https.html，registerProtocolHandler/testdriver）
- event-handler 反射面 ~455（windowless-body 236 + all-global-events 108 +
  body/frameset 族 ~110——WindowProxy/全局 event-handler 属性语义）
- atob 边界面 70（base64.any.js 代理对/非法码元）
- 其余 ~199 零散（microtask/unhandled-rejection/structured-clone 边角）

[逐文件 JSON](2026-11-20-dc2-batch4-webappapis.json) · [批三证据](2026-11-20-dc2-batch3-window-forms-channel.md) · [批二证据](2026-11-20-dc2-rendering-batch2-channel.md)
