# column-rule 简写门线刀——apply 简写展开臂 + 白名单串解锁 multicol/gaps 支持门

日期：2026-11-16（十七轮，wptfyi-css；gap-rules 设计轮 45b4209b5 分刀 C
「可独立先行」认领收口）。
改动面（3 文件，+42/−0）：`crates/style-system/src/property/registry.rs`
（known_properties 白名单 `"column-rule"` 一行，column-rule-style 后）+
`crates/style-system/src/property/apply_advanced.rs`（新简写展开臂 +24）+
`crates/engine/src/js_dom_bridge_tests/part02.rs`（R2785 supports 钉扩
column-rule 五向断言 +17）。

## 机制要点

**交接稿实况修正**：刀 C 设计称「multicol 全实现只差白名单门线一行 +
supports 钉」——不完整。白名单行 + 钉的中间态（B1' md5 0e3d7ab6）A/B 对
b0 **零 delta**（multicol/gaps gained 0/lost 0）= 白名单单独不足的实证。
真缺口 = **apply 链无简写展开臂**：`decl_supported`（js_dom_bridge.rs
css_supports→decl_supported）= known_properties 白名单 + apply oracle 两
conjunct，简写即便入白名单，apply oracle 对一切简写恒假（展开在 cascade
层——apply.rs:2192 注记「apply 链无展开臂」，样式表路径由 cascade 先
expand_shorthands 再逐 longhand apply，decl_supported 的 scratch-style
apply 直接进 match 尾 fallthrough 假）。

**刀体**：apply_advanced.rs 新 `"column-rule"` 臂——复用 cascade 层同一
展开器 `crate::shorthand::expand_shorthands`（→ expand_column_rule，
`<line-width> || <line-style> || <line-color>` 组合文法、重复组件拒绝、
thin/medium/thick 与颜色形裁决单点——@supports 样式表路径
matcher/supports.rs `shorthand_supported` 同判据），产出 longhand 逐条
递归 `apply_advanced_property_value` 全过才落 true；展开空/组件不合法
落 fallthrough false（非法值仍拒，钉的 bogus/重复 color 断言守此）。
递归只落 longhand 臂无环。白名单行使第一 conjunct 过，展开臂使 apply
conjunct 过——门线才算打通。gCS 重组（computed_style.rs:805
column_rule_shorthand_to_css，R2755）与 cascade 展开本在，不动。

**css-multicol-1 参考**：<https://drafts.csswg.org/css-multicol-1/#cr>；
简写展开先例 = cascade 层 expand_column_rule（shorthand/mod.rs:2482）。

## 单测

- engine `js_dom_bridge::tests::test_css_escape_supports_r2785` 扩
  column-rule 五向断言：`CSS.supports('column-rule', 'medium solid
  currentcolor'/'10px'/'dotted')` 真 + `'bogusxyz'`/'red medium green'
  （color 重复）假（门线双向 + `||` 文法双向）。定向腿 **1 passed**
  （test-guard --compile-first 包裹，peer WIP stash 窗内）。
- `cargo fmt -p zero-style-system -p zero-engine -- --check` 净（apply
  臂一处闭包换行漂移由 cargo fmt 修正后复验 0 Diff）。clippy 两 crate
  净（zero-style-system --all-targets + zero-engine --lib，零 warning——
  peer es_module lint 本次不在窗，WIP 在 stash）。

## A/B 双面账（干净二进制重放）

- B0 = `bin-crule-b0/zero-wpt-runner`（md5 c481ca1b）= 干净 HEAD
  45b4209b5 重建（pathspec stash 窗）。B1 = `bin-crule-b1/zero-wpt-runner`
  （md5 23ee88d5）：HEAD + 白名单行 + apply 臂 + 钉重建（peer WIP 持于
  stash crule-peer-window2，树净）。中间态 B1' = 0e3d7ab6（白名单 + 钉，
  无 apply 臂）——首跑批 zw-crule-ab 中 multicol/gaps 零 delta，作
  「白名单不足」对照组留档。
- 批：zw-crule-ab.service（01:15–01:21，6 腿全 EXIT=0）三 corpus × 双侧
  + zw-crule-b1.service（01:29–01:32，3 腿全 EXIT=0）apply 臂 B1 重跑；
  test-guard 包裹（--per-proc-mem 4 --total-mem 8 --time-limit 3600），
  transcript 唯一名 `/tmp/zw-crule-{side}-{corpus}.txt`，JSON 分目录
  `results-crule-{side}`。覆盖面对称：multicol 91 页、gaps 75 页、
  typed-om 同页两侧全同。

**结果（B0 → B1，三 corpus）**：

| corpus | gained | lost | regressed | pages_gain |
|---|---|---|---|---|
| css-multicol | +6 | 0 | 0 | 1 |
| css-gaps | +3 | 0 | 0 | 1 |
| css-typed-om | 0 | 0 | 0 | 0 |
| **合计** | **+9** | **0** | **0** | **2** |

翻转子测逐名（全 `setProperty('column-rule', …)` → 支持门/计算值通道）：

- css-multicol/parsing/column-rule-computed.html +6：`'10px'`、
  `'dotted'`、`'0px none rgb(255, 0, 255)'`、`'10px dotted rgb(255, 0,
  255)'`、`'medium hidden currentcolor'`、`'medium solid currentcolor'`。
- css-gaps/parsing/gap-decorations-rule-shorthand-computed.html +3：
  `'5px solid currentcolor'`、`'rgb(0, 0, 255) 10px solid'`（乱序组）
  `'dotted'`——row-rule 页的 column-rule 案同步解锁（expand_column_rule
  对纯色单段列表天然兼容，设计轮勘误预判兑现）。

**基线完整性**：B0 same_pass 与 gap-rules 设计轮基线逐数吻合（multicol
237P/91 页 = results-gaps-b0；gaps 356P/75 页），护栏 corpus typed-om
零 delta 白名单+apply 纯增量零外溢。

**残面分解（multicol column-rule 命名 F 347，全非本刀域、已分类后置）**：
interpolation 三页 320F（width/color-interpolation + discrete 42）= 插值
语义桶；column-rule-shorthand.html 12F = shim 声明对象 longhand 投影面
（`e.style.columnRule = …` 后 `e.style.columnRuleWidth` 读空——shim
verbatim 存储不分解 longhand 槽，与 apply 链不同站）；invalid 页 10F =
shim setProperty 宽收漏拒（'reverse'/'10'/'red medium green' 原样存入，
offset 轮 parsing-invalid 24F 同族挂账）；column-rule-valid.html 2F =
canonical 序列化（缺省组件省略/规范序，gCS 重组臂域）；width-computed
2F = calc() 值支持门（calc-size 桶同型）。

## 同文件碰撞法证澄清

bdf 刀账本（master.md 未提交段 + backdrop evidence 同文件碰撞节）声明
01:14 crule 流「registry 入 column-rule 行、本刀（backdrop-filter）行被
抹、钉保留改写紧凑」——grep 法证否证：registry.rs:690 `"backdrop-filter"`
行在（f80e4a8b8 已入 HEAD，本刀 diff +1 仅 column-rule 行，插入点 661
远离 689 区）、part02.rs backdrop 断言原样（本刀只在同测试函数内追加
column-rule 断言块）。「被抹」观测疑为 stash push/pop 瞬态窗口误读
（本轮为双二进制重建在 peer WIP 上做过三轮 pathspec stash 往返）。复录
步骤可免；其预留分支「若 crule 提交已含本刀行/钉则按认领件核对其账本
描述后并入账」适用——本刀提交含 backdrop 行/钉原样（HEAD 内容，非本刀
改动）。

## 门禁

zw-crule-mkt3 拆腿（systemd unit + test-guard --compile-first 包裹，
01:34–01:4x；peer WIP（css-parser basic-shape 族 + es_module s419）全程
持于 stash `crule-peer-window2`，树 = HEAD + 本刀）：

| 腿 | 结果 | 归账 |
|---|---|---|
| companion-build | 0 | — |
| workspace-test | 14150P/**1F** | `service_worker_manager::module_update_compares_static_dependency_bytes`（page-runtime）——在册 script-sandbox 流真实红（TypeError: only-a-getter ServiceWorkerGlobalScope，本窗 peer WIP 已 stash=HEAD 态现形）；本刀域 style-system/engine 167 test bin 除该页外全绿、R2785 钉（含 column-rule 扩断言）ok |
| quickjs-clippy | 0 | —（peer es_module lint 不在窗） |
| renderer-bin | 0 | — |
| renderer-lib | 371P/**1F** | worker-fetch renderer 形（在册） |
| browser-bin | 741P/**2F** | tab 镜像 worker-fetch + default_actions（在册两形） |
| browser-tests | 741P/**2F** | 同上两形 |
| gpu-adapter / gpu-consistency | 0 / 0 | adapter 可用双绿 |
| quickjs-workspace | 775P/**1F** | image_batch_e2e 环境形红（在册，solo ×3 前轮定谳） |
| quickjs-renderer-bin | 0 | — |
| quickjs-renderer-lib | 369P/**1F** | worker-fetch quickjs 形（在册，348.49s slow test 与 bdf 351.47s 同签名） |

刀域（style-system / engine）零新增红：全部红项逐名对号在册
（worker-fetch 三形 / default_actions / SW module_update / image_batch_e2e），
无一本刀域。fmt/clippy 两 crate 净（见单测节）。
