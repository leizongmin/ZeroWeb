# M1 证据：`:host` 族命中 + scoped sheets 进渲染管线 + 级联顺序 + shadow focus 通路

**日期**: 2026-11-07
**范围**: M1（[goal](../../shadow-dom-rendering.md) 里程碑 §M1；M0 计划 §6 五面 + B4 验收钉焦点通路）
**基线**: M0 [evidence](2026-11-07-m0-hookpoint-audit-and-corpus.md)——20 案地址池 16P/4F
（slot-fallback-004/006、shadow-style-invalidation-vw-units、focus-pseudo-on-shadow-host-1）

## 1. 变更面（R1–R5 + focus 通路）

| 面 | 文件 | 内容 |
|---|---|---|
| R1 管线 | `crates/engine/src/pipeline/mod.rs` | 渲染链 `compute_styles_with_shadow` 收集 shadow scopes 装配进样式计算（shadow 作者样式渲染面生效）；`inject_focus_state` 改走 `js_dom_bridge::find_by_selector` 单漏斗（原 `query_selector` 被封装门拦 + 不识 zwsh 地址） |
| R2 命中 | `crates/style-system/src/matcher/mod.rs` | `:host`/`:host()`/`:host-context` 真命中（`:host` 段宿主匹配 + `:host-context` 祖先链），配套 zwsh 快径 |
| R3 合法化 | `crates/css-parser/src/{ast.rs,parser/mod.rs,selector.rs}` | `Host(Option<Vec<Selector>>)`/`HostContext(Vec<Selector>)` AST 变体 + 解析臂（CSS Scoping §3）；`PseudoClassSelector` 序列化两处（css-parser tests helper、`css_wire.rs` best-effort 口径）——`::slotted` 仍非法，属 M2 |
| R4 级联 | `crates/style-system/src/cascade.rs` | `tree_context` 轴（cascade 顺序语义：document → host → shadow），级联装配带轴参数 |
| R5 宿主段 | `crates/style-system/src/tests/shadow_host_segment.rs`（新） | 宿主段匹配单测 |
| focus 通路 | `crates/engine/src/js_dom_bridge.rs`（`FocusChangedInShadow` + `FOCUS_SELECTOR_KEY` + `resolve_focus_shadow_address`）、`js_dom_bridge/callbacks.rs`（`__zw_focus_changed_shadow_child`）、`js_dom_shim/part04.js`（focus 陷阱 shadow 分支）、`apps/renderer/src/page_scripts.rs`（queue 翻译）、`crates/dom/src/document/focus.rs`（`:focus` composed 链匹配）、`computed_style_cache.rs`（无样式效应臂）、`tests/wpt-runner/src/reftest/reftest_scripts.rs`（保留键消费） | shadow 树内 `focus()` 稳定地址通路，见 §3 |

DomMutation 为 engine 内部类型（无 serde、不过 IPC）——新变体无需协议端同步。

## 2. B4 验收：focus-pseudo 三案 3/3 strict 真通过

`make reftest FILTER=focus-pseudo`（800×600，strict 门）三案全 ✓0.00%：

- `focus-pseudo-on-shadow-host-1`（closed + `shadowRoot.firstChild.focus()`）
- `focus-pseudo-on-shadow-host-2`（closed + slot，`:host:focus span`）
- `focus-pseudo-on-shadow-host-3`（closed + slot，`:host:focus` 负向臂不触发）

修前 `-1`/`-2` 是阈值伪象（红带 ≈0.083% < 默认 2%）——PNG dump 法证
（`REFTEST_DUMP=1` → `target/reftest-dump/`，诊断产物留 .acceptance/ 不入库）
证明修前三案 `:focus` 均未命中（-2 黑 FAIL 条带）；修后三案 0.00% 为
strict 真通过，非阈值擦过。DC-4 基线 4F 中 focus-pseudo-1 翻绿。

## 3. focus 缺口与通路

**缺口 1**：shim `focus()` 落在 shadow 树内元素时，元素是 handle-only
代理（deep entry JSON 无 `s` 字段）→ wire 上无任何焦点消息。
**缺口 2**：`inject_focus_state` 用 `query_selector`——封装门拦 shadow 内
地址，也不识别 zwsh。

**通路**（最小面，仅直接子级）：

1. shim focus 陷阱 else-if 分支：目标 handle 的 parent 是 `_shadowHandles`
   root 且在 `_handleChildren[root]` 中 → 按元素序发
   `__zw_focus_changed_shadow_child(rootHandle, childIdx)`（深链逐跳
   `FIXME(45-M2+)`，语料无钉面先最小通路）。
2. 回调推 `DomMutation::FocusChangedInShadow { root_handle, child_path }`；
   apply 臂 `resolve_focus_shadow_address` 从 handle→ShadowRoot 节点按
   child_path 逐级解析到目标元素，铸 zwsh 选择器
   （`shadow_scoped_selector`，形 `zwshN:host|local`）。
3. `handle_selectors` 以保留键 `FOCUS_SELECTOR_KEY = "__zw_focus"` 回带；
   runner/renderer 两侧读键 → `pipeline.focused_selector` →
   `inject_focus_state` 经 `find_by_selector`（zwsh 快径）→
   `doc.set_focus_element` → `:focus` 经 `dom::document::focus` 的
   composed 链匹配到 host。

`FocusManager::scan_node` 递归 shadow 树 + `is_focusable_selector` zwsh
门同通路工作（runtime 可聚焦性）。

## 4. B3 归因修正：vw-units 属 iframe 载体面

M0 归因「R1/R2 面」勘误：`shadow-style-invalidation-vw-units` 用例跑在
iframe data: URL 文档里，runner 将 iframe 渲染为占位 replaced element、
无嵌套文档/几何管道（`wc_case_skipped` 同因注释）——样式工作不可翻，
具名遗留改记**载体面**。M1 样式面本身（shadow 内联 sheet 样式 shadow 内
元素 → 绘制输出）由常驻单测证明：
`crates/engine/src/tests/pipeline.rs::test_shadow_inline_sheet_styles_shadow_inner_element`
（`<template shadowrootmode=open><style>div{background:...}` → shadow 内
div 出绿色 fill 图元）。

## 5. 附带红面归因（非本池）

`make reftest FILTER=shadow-dom` 批跑见 `display-contents-shadow-dom-1`
✗2.04%：FILTER 子串附带命中，不在 20 案地址池。页面无 `focus()`/`:focus`
——本轮改动机制性无关（session delta 惰性）；diff 内容为 `::slotted`
伪元素内容（M2 面）+ slot 顺序/回退（M3 面），修前既红。

## 6. B1/B2 零回落

定向复扫 vs M0 基线：`shadow-dom/` 9/9、`focus/` 3/3 保持；B1/B2 除具名
遗留（slot-fallback-004/006、vw-units）与 §5 附带面外零新红。

## 7. WC testharness 全量回归（shim 变更 A/B）

`make testharness-web-components`（test-guard 4G/8G/1800s）vs 2026-11-06
`m3b2-wc-full-run.json` 基线（393 文件）逐文件 A/B（对照脚本内联于本轮
session，产物 `.acceptance/m1-wc-ab/`，不入库）：

- **新红 0、恶化 0**（247 失败文件全部为在册基线红；
  `shadowrootadoptedstylesheets` 族 = 42 号归口 45 号划界面、
  `reference-target` tentative 族 = 42 号在册、adopted-callback /
  ElementInternals / AriaMixin / reflect-html-for 大簇 = 42 号在册）；
- **计数级对账**：本轮 P=8365/F=1403/other=8（总 9776）——**F 与 42 号 M4
  冻结二进制精确相等（1403=1403**，构建提交 `d7155cef5`，M4 recap 表，
  彼侧 P=8437/F=1403/other=16〔8 Timeout+8 Not run〕/总 9856）；差 80 =
  P 差 72 + other 差 8，为语料宇宙差（M4 全语料 365 文件 vs 本 target
  web-components fetch 子集）；
- 4 文件好转 + 29 文件痊愈（declarative-shadow-dom-attachment 170F→0 等）
  全部落在 11-06 m3b2 基线 → d7155cef5 之间 42 号 M3 批三/M4 时段（面域：
  declarative 附件格式 + adoptedstylesheets 载入链，与本轮 focus/样式面
  无交集），归因 42 号正向漂移；
- make 退出 1 为在册红质量预期（1403 子测试 F / 247 文件），与 M4 全语料
  同态。

**shim 变更（part04.js focus 分支）零回归定谳（计数级）。**

## 8. 共享面留痕（run-rules §9）

本轮触碰通用样式面：`style-system` matcher（`:host` 族臂）、cascade
（`tree_context` 轴参数）、engine pipeline 渲染入口（shadow scopes 装配
+ focus 注入漏斗）、css-parser 选择器合法化。rendering-compat 控制面
[留痕](../../../rendering-compat.md)（跨 goal 碰头块），义务面：`make reftest`
全量零回归 + workspace `make test`（本轮门禁 §9）。

## 9. 门禁记录

**fmt/clippy**：`cargo fmt --check` 绿；workspace clippy `-D warnings` 绿
（含 matcher `Rule::Supports` 臂 collapsible-match 改 match-guard 修复）。

**make test**（workspace 腿 Error 101 停止点后按 R5442 判例对余腿用
Makefile 原命令 + `--no-fail-fast` 补跑，test-guard 包裹，rebase 后树 solo）：

- workspace `--no-fail-fast`：**engine lib 6F 全为在册**
  （img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 /
  fetch_filtered_response_type_matrix / iframe_service_worker_messages /
  t8k_websocket_error_nonterminal / t8k_websocket_event_flow——42 号同日
  stash 对拍 9F 的子集），其余 crate 腿零新增红；
- renderer bin/lib、GPU 探针+条件腿、quickjs 三 crate 腿：全绿；
- browser bin（xvfb，`--test-threads=1`）：**741P/2F/1I**——2F 归因见下；
- browser tests（xvfb）：**740P/3F**——3F 归因见下；其中
  `cdp_focus_governance` **6P/0F**（含 `ZW_HOST_FOCUS_STATE_SYNC`
  kill-switch 面，shadow focus 通路与其无冲突）；
- quickjs renderer lib：**369P/1F** =
  `page_scripts::mutation_arm_does_not_apply_across_document_reset_window`
  （在册对端碰头红第 6 轮观察，apps/renderer 面非本流域；r2 重跑轮再复现
  同型 1F，与前轮交替通过/失败＝负载敏感 flaky 非回归）。

**browser 面 4 红（非在册 2 红）机器态污染定谳（隔离探针法）**：
`autocomplete_url_match_segment_uses_text_color` +
`bookmark_context_menu_open_in_new_tab_loads_url` 在干净隔离
`ZERO_CONFIG_DIR=/tmp` 探针下 **2P/0F**——根因：浏览器面测试无
`ZERO_CONFIG_DIR` 隔离，读写真实 `~/.config/zeroweb/`（bookmarks.json 被
既往测试写入 t62-* 书签，URL 时间戳含 "127" 子串 → 下拉 4 行撑宽 scheme
冒号灰字形基线带 → 失败输出 "127127127127" 逐字符吻合＝4 行各一段）。归
browser 流域既有
基建缺陷，本轮只归因不修（非本流域面）；另 2 红
`default_actions_work_without_javascript`（bin+tests 各一次）为在册账红。

**基建留痕**：cdp 套件依赖 `target/debug/zero-renderer` 二进制在场（缺位
时报 5F insta-panic——部分补腿必须先按 Makefile 顺序 build renderer）；
quickjs 腿首跑 test-guard 相对路径 ENOENT → 绝对路径重跑通过。

**make reftest 全量**：**725/725、不一致 0、rc=0**（真通过-可信 558 +
可疑 113 + 近似 54）——rendering-compat 零回归门落定；B1/B2/B4 定向面
见 §2/§6。

**make product-smoke**：rc=0 全 PASS——welcome.html diff **17.23%**
（与 R5444–R5449 在册基线逐字节同值）+ 三 viewport（800/375/320）
struct-check 全 PASS + form-input-perf-gate 过（DC-13 义务）。

**bench-gate**（§12 定向 `ZERO_WEB_BENCH_CRATES=zero-style-system,
zero-css-parser`）：待静窗回填（对端 knife11 编辑面 testharness 窗占核，
静窗纪律候跑）。

**rebase 重验（2026-11-07 17:07，合入前基线同步）**：`git pull --rebase`
进两提交——04fb3e133（s332 媒体 Illegal constructor + WC-M1 super 注册门，
part01b/03/09）与 f8b317b70（svg-compat-v2 B6b SMIL 类型域）——与 M1 改动
文件面零交叠（comm 对账空集）。合并树全量 `make reftest` 重跑：**725/725、
不一致 0、rc=0**，与 fbaa34b1e 基线账逐位一致；workspace `make test` 维持
原账（新提交文件面不相交，且其自带门禁在册：engine lib 3797P/6F 红基线
不变，见 04fb3e133 提交信息）。
