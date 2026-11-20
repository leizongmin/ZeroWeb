# DC-2 · 批五：html/browsers the-window-object + origin 簇接入（语料 + 通道 + 记账）

2026-11-21。批五落地：22 目录语料拉取（pin 31597693 子树显式枚举，
`GOAL_PULL_ANY_JS=1`——origin/api .any.js 唯一语料形态；.headers sidecar 全量——
COOP/COEP agent-clusters 响应头语义）+ html-syntax 通道加收 15 计分目录 +
`.https` 命名面簇内解锁（named-access 归 DOM 套件 slice28 既有记账域防跨套件
重复计账）。release runner 实测净 **+40 Pass**（html-syntax 65,274 → 65,314，
669 → 801 键），共享键零回归（669 键数与 Pass 65,274 / Fail 6,383 / Timeout 28 /
Unsupported 7 / PreconditionFailed 1 与批四账逐字一致）。

## 变更（三文件）

1. `tests/wpt-runner/scripts/goals/58-wptfyi-html-browsers-window.sh`——22 目录
   显式拉取（fetch_dir_html 只递归一层，resources/support 助资产目录逐个显式；
   幂等标记防限流）。`.worker.js` 不拉——runner 本通道零消费（worker 生成形
   分母挂账）。
2. `Makefile`——`fetch-wpt-html-browsers-window` target，挂入
   `testharness-html-syntax` 依赖（fetch 幂等，套件跑前自动补拉）。
3. `tests/wpt-runner/src/testharness.rs`——`HTML_SYNTAX_CORPUS_SUBDIRS` 加收
   15 计分目录（resources/support 只拉不扫，分母 0 键实证）；
   `html_syntax_case_skipped` `.https.html` 命名排除对 `html/browsers/` 簇解锁
   （runner 恒 `https://wpt.test`、isSecureContext 恒真，批四 webappapis 同前提）；
   补簇内 `.https` 解锁与 `.any.js` 放行的单元断言。另：批四遗留单测一处
   rustfmt 违例（`secure-context-face` 断言单行超宽）随本批 fmt 门修复。

## 结果（release runner，test-guard 包裹，全量套件）

| 面 | 变更前 | 变更后 |
|---|---:|---:|
| html-syntax 键 | 669 | 801（+132 browsers） |
| html-syntax Pass | 65,274 | 65,314（+40） |
| Fail | 6,383 | 6,766（+383） |
| Timeout | 28 | 121（+93） |
| Unsupported / PreconditionFailed | 7 / 1 | 7 / 1 |

- 通道分母（summary-chrome 4b26db8978）234 键 1,740 子测试（chrome 全过
  1,441）：the-window-object 自有 84 键（779 子测试，chrome 过 629）+ origin
  150 键（961 子测试，chrome 过 812）。
- 执行 132 键 516 子测试，**132/132 全部对上分母、0 溢出键**（批五首跑无命名/
  复合形溢出；批四首跑两处溢出已由批四收口）。
- 执行面 chrome 可过 867 子测试中 ZeroWeb 过 40——Fail/Timeout 集中在多全局
  语义面（见战役图），通道大头为后续刀位储备。

## 余面挂账（102 键未执行，通道内逐键归因）

- **iframe 内容门 39 键**——含 window-properties.https.html（178 子测试，批五
  最大单面；`.https` 命名门已解锁、`<iframe` 内容门拦）与 origin-from-location/
  messageevent/window 族。与 master 卡点 iframe 子文档管道同根因，零-web 流域
  待碰头。
- **query 变体形 38 键**——location-properties-smoke-test.window.html?assign 等
  （.window.js 源 URL-query 变体 runner 不展开，通道只跑 base 形；HTML
  `<meta name=variant>` 变体正常展开——open-features-is-popup-condition?single-*
  4 键已执行实证）。
- **worker 生成形 10 键**——origin/api .any.worker.html 7 + serviceworker 2 +
  sharedworker 1（worker 执行通道域外，批四同口径挂账）。
- **上游漂移 13 键**——pin 31597693 后上游新增（jsdelivr/raw 双源 404 实证）：
  agent-clusters about-blank-popup/sandboxed-iframe-keying 族 10 +
  open-close declarative-windowfeatures.tentative 等 3。随周跑另报，
  security-window（快照漂移 124 子测试）同 bucket。
- **popup 内容门 1**（popups-crash.https.html）+ **worker 内容门 1**
  （origin-from-worker.window.html）。

## Fail 战役图（383 Fail + 93 Timeout 集中度）

- origin/api 起源序列化族 215 Fail（hyperlinkelementutils.window 129 +
  origin-from-string 27 + origin-from-url 27 + origin-from-origin 25 等——
  Origin 构造器/序列化语义，独立刀位候选）
- open-features window.open 特性串族 135 Fail（tokenization 5 键 +
  is-popup-condition 变体 4 键等 22 键——特性串解析语义，独立刀位候选）
- Timeout 93 键（文件级整案）中 origin-keyed-agent-clusters 59 键
  （iframe/popups 导航等待——iframe 子文档管道根因）
- cross-origin-objects 6 Fail + 零散（proxy-getOwnPropertyDescriptor/
  Document-defaultView/window-prototype-chain 混合 Pass/Fail）

[逐文件 JSON](2026-11-21-dc2-batch5-browsers-window.json) · [批四证据](2026-11-20-dc2-batch4-webappapis-channel.md) · [批三证据](2026-11-20-dc2-batch3-window-forms-channel.md)
