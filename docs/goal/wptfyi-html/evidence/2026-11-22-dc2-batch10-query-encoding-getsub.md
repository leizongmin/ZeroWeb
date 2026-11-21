# DC-2 · 批十：query-encoding {{GET}} 注入刀 + DC-3 preflight 三腿基线收账

2026-11-22。批九战役图「query-encoding 序列化差（139 F）」刀位收口第一层：
`inline_local_scripts` 的 `pipe=sub`/`.sub.js` 臂补 `{{GET[name]}}` 替换（替换源=
脚本自身 src 查询，wptserve 每请求语义）。infra 套件 A/B 实测净 **+29 Pass**
（scheme 面 +20、history 面 +6、eventsource#url +3），对批九基线净 **+42 Pass**
（另 +13 为 messagechannel.any.worker.js 键级截断恢复的抖动带）。零回归。

## 根因

`resolve-url.js?encoding=utf-8&pipe=sub` 经静态内联通道（`inline_local_scripts`）
交付：`apply_wpt_substitutions`（host/ports/domains 全集）在 pipe=sub 臂已生效，
但 `{{GET[name]}}` 替换只存在于 fetch 通道（M4-S22）与文档通道
（`wpt_apply_get_query_substitutions`），静态内联面缺位 → `var encoding =
'{{GET[encoding]}}'` 字面残留 → `expected_current = undefined` → scheme 面
「expected substring undefined got …」假 Fail（got 值本身已正确：utf-8 页
`%C3%A5`、windows-1252 页 `%E5`、windows-1251 页 `%26%23229%3B`——文档编码
query 序列化主链早已工作，只差期望值注入）。

## 变更（一文件）

`tests/wpt-runner/src/testharness.rs`——pipe=sub/`.sub.js` 臂改调
`wpt_apply_get_query_substitutions(apply_wpt_substitutions(&c), src_query)`
（复用既有 helper：百分号解码 + 残留占位符空串，上游缺参语义）；新增回归测试
`inline_local_scripts_pipe_sub_get_tokens_substitute_from_src_query`（替换值/
百分号解码/残留空串三臂）。

## DC-3 preflight 三腿（本地 test-guard 链，release runner）

| 腿 | 键 | 结果 | 判读 |
|---|---:|---|---|
| infra（00:11-00:15） | 132 | Pass 550 / Fail 373 / TO 152 / PF 2 | 对批九基线 -73 全落 messagechannel.any.worker.js 单键（Pass 102→29 键级超时截断，键集不变）——负载抖动带，非回归 |
| rendering（00:15-00:16） | 152 | Pass 2491 / Fail 7201 / TO 9 | 对批二基线共享键漂移 **-62 Pass**（legend-display -56、fieldset-display -11、video-aspect-ratio -1，aspect-ratio 族 +6 回冲）；+2 新键（slot-direction.window.js、input-checkbox-switch.tentative.window.js，批三 fetch 带入）净 -60 |
| syntax（00:16-01:49，93 min） | 1,049 | Pass 67,387 / Fail 10,656 / TO 286 | 与批八 after **字节级全同**（四项 totals 逐一相同）——engine 漂移不触 syntax 套件 |

rendering 漂移归因：漂移键集中在 computed display/UA 呈现域，批三后渲染流
engine 提交 3f05d6acf/21e0ea83d/22787ab21/f25940399（11-21 19:33-22:00 入
HEAD，23:42 runner 二进制内嵌）——归因渲染流域，待该流定向定谳与修复，不入
本流回归账（master 碰撞记录在案）。

## 批十 A/B（infra 套件全量，同 guarded 入口）

- before（preflight 00:15 腿）：Pass 550 / Fail 373 / TO 152 / PF 2 / 1,077 子测试
- after（批十二进制）：**Pass 665** / Fail 352 / TO 151 / PF 3 / 1,171 子测试
- 对批九基线（Pass 623）：**+42**。changed 键仅 12：恢复面 11 键 + messagechannel
  .any.worker.js（29→115 P，键执行到面：137 子测试全执行，含 1 PF 前置跳过，
  ReadableStream transfer 前置同 `messagechannel.any.js` 已知类）；css/svg/
  websocket/window-open/workers/nested-browsing/submit/xhr/loading/base-href
  各面 TO/F 集零变化（engine 刀位面未动）。
- PF 2→3：messagechannel.any.worker.js 本次跑满后暴露 1 前置跳过，非回归。

## 余量分解（批九 139 F 刀位的剩余归属）

- **非特殊 scheme 文档编码越界（~54 F）**：windows-1251/1252 页 27 个
  utf8-expected scheme 全 F——`<a>.href` 对 ws/mailto/urn 等非 http(s)/ftp/file
  scheme 也套文档编码（spec：编码覆盖仅 http(s)/ftp/file）；utf-8 页 16 个
  URL 反射缺位 + `'href' in elm` 原型面 9 F + search setter 2 F——js_dom_shim/
  bridge 面（零-web 流核心域 WIP，待碰头）。
- **UA 样式 background 属性（8 F×3）**：`getComputedStyle(backgroundImage)`
  为 none——UA 样式面（rendering 流域）。
- **.py fixture 缺位**：resource.py/blank.py/stash.py/css-tmpl.py 未仿真——
  xhr/nested-browsing/submit/loading/base-href 面依赖；其中 iframe 门
  （nested-browsing/submit/history 既有碰头项）与 doc.write XML 拒绝
  （base-href「This document is an XML document」，批八 document open/write
  同域）先挡——runner 侧 fixture 属本域可先备料，收益待碰头刀落地后兑现。

## 门禁

单元测试 5/5 过（`cargo test -p zero-wpt-runner inline_local_scripts`，
test-guard --compile-first 包裹）；scoped clippy exit 0（25 条既有警告全在
渲染流域 css-parser/style-system 文件，master 碰撞记录在案，本批文件零警告）；
`cargo fmt -p zero-wpt-runner -- --check` clean。修复前（字面残留形）与修复后
（+29）的 A/B 证据见 [逐文件 JSON](2026-11-22-dc2-batch10-query-encoding-getsub.json)。

[批九证据](2026-11-21-dc2-batch9-infra-small-clusters.md) ·
[批十一证据](2026-11-22-dc2-batch11-syntax-template-leg.md)
