# DC-2 · html 批三：window 变体源通道（.window.js / .any.js 窗体）

2026-11-20。批三不加语料、不加套件——把分母里 wptrunner 生成形键（`.window.html` /
`.any.html`）对应的本地源窗体（`.window.js` / `.any.js`）接入既有两通道。共享扫描器
本就收 `.window.js`（webmessaging/webstorage/fileapi 先例），但闸在 net-api 专属的
`extra_forms` 门内，且 html-syntax 通道的 `testharness.js` 引用检查把无引用的源窗体
整类漏掉。release runner 实测净 **+11 Pass**（html-syntax 63,728 → 63,737，
rendering 2,551 → 2,553），两套件共享键零回归（Pass/Fail/Timeout 逐项只加不减）。

## 变更（两文件，纯通道面）

1. `tests/wpt-runner/src/testharness.rs`——`CorpusRunOptions` 新增 `window_forms`
   独立门（只开 `.window.js` 窗体，不带 net-api 的 `.htm` 窗——html-syntax 语料
   xmldecl/support/*.htm 81 文件是 iframe 载入 fixture，`extra_forms` 敞开会误入账）；
   html-syntax 通道开启。`html_syntax_case_skipped` 的 `testharness.js` 引用检查
   收窄到 `.html` 形——`.window.js`/`.any.js` 源不引 harness（wrapper 注入）。
2. `tests/wpt-runner/src/testharness_html_rendering.rs`——rendering 通道加收
   `.window.js`（收集器窗体判定 + `run_any_js_window_case` 分派），同款检查收窄。
   单测补 window 形断言（源无 testharness 可跑、support 目录规则仍优先）。

## 结果（release runner，test-guard 包裹，全量两套件）

| 面 | 变更前 | 变更后 |
|---|---:|---:|
| html-syntax 键 | 454 | 462（+7 window.js + self-origin.any.js） |
| html-syntax Pass | 63,728 | 63,737（+9） |
| rendering 键 | 150 | 152 |
| rendering Pass | 2,551 | 2,553（+2） |

- 新键 10 个：den 107 / 执行 106 / 分母逐字对齐 9/10。唯一计数缺口
  dir-auto-dynamic-changes 20/21——断言级分支未触（引擎条件面，批三不追）。
- 零回归证明：`testharness.js` 检查收窄只放宽非 `.html` 形（.html 排除面逐字
  不变），两套件全量 Pass/Fail/Timeout 与批一/批二账严格相加（63,728+9 / 7,132+8 /
  8T 恒；2,551+2 / 7,132→7,140 同源 +8）。
- 面：`self-origin.any.js` 执行 1/1 即 Fail——断言硬编码 `http://` scheme 而
  runner 恒 `https://wpt.test`（idlharness carveout 同前提），scheme 面非通道缺口。

## 余面挂账（域内批四候选：webappapis 簇）

`.window.html` 分母全局面 = html/ 268 键 / 2,224 子测试（chrome 全过 1,918）：本批
接入在盘语料 10 键 107 子测试；余 258 键需语料拉取（webappapis 48 键 195、browsers
113、infrastructure/COEP/anonymous-iframe 等）——webappapis 簇是本 goal 五簇之一
（缺口 3,672，window 键只是其中 195 子测试），批四按簇整批接入（语料 + 通道 + 记账）。

[逐文件 JSON](2026-11-20-dc2-batch3-window-forms.json) · [批二证据](2026-11-20-dc2-rendering-batch2-channel.md) · [批一证据](2026-11-20-dc2-dom-batch1-variants-idlharness.md)
