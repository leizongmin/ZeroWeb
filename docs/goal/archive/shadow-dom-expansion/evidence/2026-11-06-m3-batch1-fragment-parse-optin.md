# M3 Batch 1：片段解析 opt-in 语义收敛（簇 B 首刀，2026-11-06）

## 批次范围

declarative-shadow-dom-opt-in.html（120 子测试）片段解析路径语义收敛：
innerHTML / insertAdjacentHTML / createHTMLDocument / document.write(fresh)
路径 `<template shadowrootmode>` 以普通 template 存活（不物化，
whatwg/dom#831）、`shadowrootmode` 属性值还原源码小写形态、
`template.content` 可读。

## 根因链（IAH/createHTMLDocument 失败 `expected "open" got "Open"` 同根）

1. **shim 查询源模板内容剥离**（WC-M3 既定行为）：`_zwMOuterHtml` 把嵌套
   template 序列化为空形态（内容不入查询源）→ 引擎查询路径对**空模板
   形态**输入 re-parse（`parse_html_element_json_full` 物化解析——快照
   回环闭包刻意保留）。
2. **引擎 skip 路径值不可观测污染**：`materialize_declarative_shadow_roots`
   对 contents 缺失（空模板）的 template skip 时留下 s173 预变换产物
   `Open`/`Close`（html5ever 0.29.1 值精确匹配集出表机制），经
   getAttribute/序列化可观测。
3. **shim 身份桥键失配**：walk 键（真实节点 outer，template 内容内联形态）
   vs cand 键（引擎 JSON outer，空模板形态）不一致 → `_zwMFindRealNode`
   miss → wrapper 按 JSON 重建丢失 content 视图（`tmpl.content.querySelector`
   MISS）。

## 修复（5 文件，+170/-8）

| 文件 | 修复 |
|---|---|
| `crates/dom/src/document/shadow.rs` | materialize 循环起点对**全部** template 先还原 shadowrootmode 预变换值（消费路径随后摘除 template 无观测面；skip 路径存活值正确小写） |
| `crates/dom/src/parser/declarative_shadow.rs` | （上轮已落）`restore_shadowrootmode_values` 全树还原 + 片段路径预变换接线 + 2 单测（fragment 不物化保留 / unmaterialized 文档解析保留） |
| `crates/dom/src/parser.rs` | （上轮已落）`parse_html_with_scripting` 物化接线（`declarative_shadows=true`）；`parse_html_fragment_with_scripting` 预变换+还原；新增 `parse_html_with_scripting_unmaterialized` |
| `crates/engine/src/js_dom_bridge.rs` | （上轮已落）`insert_adjacent_html` 走 `parse_html_fragment_with_scripting`；`parse_html_element_json_full` 保持物化（快照回环，DOMParser/XHR 已知余量） |
| `crates/engine/src/js_dom_shim/part03.js` | `_zwQueryKey` template 键**内容不敏感**剥离（`(<template\b[^>]*>)[\s\S]*?(</template>)` → `$1$2`）——walk 键与 cand 键对齐，身份桥命中 → WC-M1 content 别名生效 |

## opt-in 页结果

基线（M2）117F+1P+1T → **108P/10F + 1T（2 未报告）**。

- 翻转代表（三连 `'Open'` 断言 → P）：createHTMLDocument with innerHTML、
  insertAdjacentHTML on element、document.write disallowed on fresh document。
- 剩余 10F 归因：canvas 1（wrapper 查找失败，canvas 专属路径待查）；
  DOMParser 1 + XHR 3（查询路径物化——快照回环闭包刻意保留，unsafe API
  族落地后重审）；createContextualFragment 1（轻量 fragment 缺
  querySelector）；parseHTMLUnsafe 1（Batch 2 API）；document.write
  allowed 1（主文档同步脚本路径应物化，待接）；iframe 2（iframe 环境）。
- 1T 同基线：2 个 sandboxed iframe async 子测试挂起未报告（环境受限）。

## 全通道 A/B（392 案 corpus vs M2 after 基线，逐案逐子测试）

| 面 | 基线 P/F | 本轮 P/F | 判定 |
|---|---|---|---|
| custom-elements | 2,821/1,018 | 2,821/1,018 | 逐案一致（1 文件 30 名含 `\n`/`\x00` 的子测试名文本截断差异——行式输出解析伪差，状态/计数恒同） |
| template | 583/84 | 583/84 | 逐案一致 |
| sd-top | 622/43 | 622/43 | corpus 逐案一致；+6P/+4F 全为本轮 2 个临时 probe 页（已删） |
| untriaged | 153/97 | 153/97 | 逐案一致 |
| reference-target | 0/139 | 0/139 | 逐案一致 |
| decl-top | 6/5,326 | 114/5,712 | **+108P**；池 +494 = gethtml.html +440 子测试展开（早抛解除→生成更多，全部命中缺 unsafe API 簇 A——Batch 2 目标面），F 增量全为新池子测试 |

**P→F 回归：0**。probe 页（zz-m3-probe / sd-probe-fragment-materialize）为
gitignored 临时件，已删除，不入库不占池。

## 证据文件

- opt-in 终态（修复后 release 二进制完整输出）：
  `2026-11-06-m3-batch1-optin-fresh.txt`
- 全通道 A/B 原始输出（gzip）：`2026-11-06-m3-batch1-wc-channel.txt.gz`
- 基线对账源：`2026-11-06-m2-baseline-after.json.gz`

## 门禁

fmt clean；zero-dom 883 单测 0F（含新增 2）；clippy
`-p zero-dom -p zero-engine --all-targets -D warnings` 0 告警
（workspace 全量门按 M2 先例因共享树 rally 活跃收窄，本批改动面
恰为该两 crate）；lei-pre-commit-guard PASS（见同提交）。
