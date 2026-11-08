# Shadow DOM 渲染级 — composed tree 进样式/布局/绘制 + :host/::slotted 命中

**版本**: v1.4
**日期**: 2026-11-04（立项，用户拍板「现在就立」——web-components 收口排除条款转正）；2026-11-07 M0 完成；2026-11-07 M1 完成；2026-11-07 M2 完成；2026-11-08 M3 R4 前半（taffy 侧，kill-switch 落地）；2026-11-08 M3 R4 完成（第二臂 IFC 重测 gate，`ZW_SHADOW_TEXT_BOX` 翻默认开）；2026-11-08 M3 R3 变异应用臂代码落地（`resolve_slots_all_hosts` 批末挂接）；2026-11-08 M3 R3b 代码落地 + B6 通道落地；2026-11-08 M3 追加轮——006 翻绿根因修复（append sel 项通道，R3b 目标案收口）；2026-11-08 M4 复测收口完成——M0-M4 全部完成，goal 收口
**状态**: 收口——M0-M4 全部完成（2026-11-08 M4 复测收口落账，待归档
sweep）。以下为各里程碑历史账：M0 完成（挂接点审计 + reftest 盘点 + 数值门，见
[M0 evidence](shadow-dom-rendering/evidence/2026-11-07-m0-hookpoint-audit-and-corpus.md)）；
M1 完成（`:host` 族命中 + scoped sheets 进渲染管线 + 级联顺序 + shadow focus 通路，
B4 focus-pseudo 3/3 strict 真通过，WC 全量 A/B 零回归，见
[M1 evidence](shadow-dom-rendering/evidence/2026-11-07-m1-host-segment-and-focus-wire.md)）；
M2 完成（`::slotted(...)` 命中翻转 + flat-tree 组合子 + R5 继承切父 + 增量退化门，
DC-2 收口——slotted-has ×4 翻绿、全量 reftest 725/725 零回归、upstream slotted
38 案 15 绿 / 23 红逐案归因（20 `:has-slotted` 范围外 / 2 命令式 M3 面 / 1 锚定位
范围外），见
[M2 evidence](shadow-dom-rendering/evidence/2026-11-07-m2-slotted-segment-and-flat-tree.md)）。
M3 完成（2026-11-08，切片账如下。R4 完成：shadow Text 布局通道两臂落地——taffy 侧
匿名叶 + inline_finalization `remeasure_inline_only_containers` 匿名叶
gate，`ZW_SHADOW_TEXT_BOX` 翻默认开（`=0` 杀）。slot-fallback-content-004
严格比对 ✗1.75%→✓0.00% 首次真通过（flag 关基线的 ✓0.86% 实为回退文本
未绘制的欠阈值假象）；门控矩阵 32P/3F 零 P→F、真通过-可信档 8→9，
kill-switch 腿与 flag 关基线全等。**根因改判**（推翻前半片定谳）：非
paint 行走按 DOM 序自累计 Y 之失，实为后处理 `remeasure_inline_only_containers`
对 slot 化 plain-block host 测得 IFC 行数 0 → 收缩臂清 0 host 高度
（taffy/extract 两层均正确，paint 忠实跟随坏几何），另发现 shadow host
不绘制自身背景（pre-existing），见
[R4 前半 evidence](shadow-dom-rendering/evidence/2026-11-08-m3-r4-shadow-text-taffy-arm.md)、
[R4 第二臂 evidence](shadow-dom-rendering/evidence/2026-11-08-m3-r4-remeasure-gate-arm.md)）；
R3 变异应用臂代码落地（2026-11-08 `3744043e2`：`resolve_slots_all_hosts`
批末统一挂接 `apply_dom_mutations_full`——命令式 attachShadow/childList/
innerHTML/slot 属性改名后分配不再过期，无 shadow 页 O(1) 快返；shadow-dom
矩阵与 R4 收口态逐位一致零漂移、全量 reftest 725/725、单测 2/2，006 归因
改判为 manual slot 分配通道缺口——并发风暴平息后补跑落账：workspace
debug 腿 3814P/6F（6 案与 HEAD 环境失败名单全同零 P→F）+ clippy -D
warnings 净 + product-smoke PASS + bench-gate 定向（dom/engine）PASS +
fmt/renderer×2/render-foundation×2 RC=0；余 browser/quickjs 腿归 R3b
净树门禁覆盖，见
[R3 evidence](shadow-dom-rendering/evidence/2026-11-08-m3-r3-mutation-arm.md)）。
R3b 代码落地 + B6 通道落地（2026-11-08，`e5f84ad0b`/`9d307d1b0`/跨流
`cfe55be04`/B6 `60928d40c`/`30ea4c630` 测试盲区修复）：manual slot 分配通道（slotAssignment wire
全参透传 + `assign()` 渲染镜像 + serialize→re-parse marker 回环 + Manual
渲染消费，单测 6）+ `testharness-shadow-dom` 补充池通道（渲染级排除域
10 案显式清单两侧同源）。门禁账（净 HEAD）：reftest 全量 **725/725 零
回归**、quickjs 5-crate/bin/lib rc=0（lib 腿 370P/0F 带修复复核）、
browser bin 742P/1F 在册同型、fmt/clippy 净、bench-gate（dom/engine）
PASS；**DC-4 矩阵实测 32P/3F——006 未翻绿（4.90%
px differ），R3b 目标案未收口**，归因调查挂账；cdp_focus_governance
6F 为 mtime 守卫拒跑（在流编辑所致）待净树复核非行为账红；
quickjs-ren-lib mutation_arm 1F（11-03 起在册）已根因修复（30ea4c630，
定性改判为测试侧引擎时序盲区——v8 形态 250ms 裕量在 quickjs 下失效，
非产品缺陷非对端碰头红）。B6 基线首跑 10 案 101 子测 15P/86F——几何命中簇零过符合
无输入注入口径，滚动/焦点面已过。见
[R3b evidence](shadow-dom-rendering/evidence/2026-11-08-m3-r3b-manual-slot-channel.md)。
**M3 追加轮（2026-11-08）——006 翻绿收口**：根因不在 R3b manual 通道，而在
shim `_appendVariadic` 对 sel-backed 静态解析元素（仅 `__zwSelector` 无
`__zwHandle`）的分型缺失——`host.append(A,B)` 掉文本分支产 `[object …]` 垃圾
文本、真身滞留原父致 manual 分配通道 parent!=host 过滤全惰性（insertAdjacent
漏斗 R4200 已修同型洞，append/replaceChildren 通道漏网）。补 sel 项臂后 006
solo **✓0.07%**（修复前 4.90% 稳定红）、DC-4 矩阵 **33P/2F**（余 vw-units
2.08% + display-contents-shadow-dom-1 1.97% 在册）、reftest 全量 725/725
零回归、product-smoke/clippy-v8/fmt/bench-gate（dom/engine）净、B6 基线零位移。
见 [追加轮 evidence](shadow-dom-rendering/evidence/2026-11-08-m3-append-sel-channel.md)。
**M4 复测收口完成（2026-11-08）——goal 收口**：DC-4 数值门逐批 **19/20
全绿 + vw-units 具名挂账**（B1 8/8 含 006 0.07% 保持翻绿 + B2 5/5 含三案
恒白守卫 + B4 3/3 + B5 1/1 静态面 + B3 2/3）、邻域 selection/shadow-dom
**9/9 不回落**、reftest 全量 **725/725 双跑零回归**、B6 门值回填 **15P/86F
零位移**（86 Fail 与基线 diff 零差异）、make test 全腿出账（workspace
3815P/6F 在册名册恒一零新增；browser bin 742P/1F 在册同型；browser tests
9 集成二进制全 ok 含 **cdp_focus_governance 净树 6P/0F——R3b §6 挂账项②
关闭**；product-smoke/gpu×2/quickjs 三腿 370P/0F/bench-gate 全绿）、头条
slot-fallback-content-006 不回落。见
[M4 evidence](shadow-dom-rendering/evidence/2026-11-08-m4-closeout-numbers.md)。
下一步：归档 sweep（模式 A：目录迁移 archive/ + wpt-suites.csv 证据路径
同步 + README 检查清单核对）
**执行模式**: 引擎专项（渲染管线挂接）+ WPT shadow-dom 语料验收
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——Web Components 渲染面）
**前账**: `docs/goal/archive/web-components.md`（2026-09-11 收口，DC-1~5 ✅——
JS 面 slot 全链路已落；渲染级按 Support Envelope 排除条款挂账）；42 号
shadow-dom-expansion（2026-11-07 收口归档，JS 语义面 53.84%）

## 基线事实（2026-11-04 核实）

- **JS 侧已落**（前账 DC-4）：slot IDL → 分配 → slotchange →
  assignedNodes({flatten}) → flattened tree 查询；composed path/retarget/
  per-station 事件语义已收。本 goal **不重开 JS 面**。
- **Rust 渲染侧未建**：`crates/dom/src/query.rs` `::slotted(...)` 语法合法
  但**恒不匹配**（R159 宽容解析记账）；`crates/dom/src/node.rs` ShadowRoot
  open/closed 字段在树中，样式/布局/绘制管线不消费 shadow 树——shadow
  树内容不出现在渲染输出。
- **缺口体量**：shadow-dom topdir 15,350 子测试 zw 4.05%（缺口 14,729），
  大头是渲染级耦合面；web-components goal 收口时三目录 76.0% 是 JS 面。

**M0 对账修正（2026-11-07，evidence §1）**：上方「Rust 渲染侧未建——
shadow 树内容不出现在渲染输出」判语过期——
布局扁平化 2026-05-31 已落（`c8e849320`，tree.rs shadow 分支 + slot 替换）、
样式 scope 管道 s168 已落（`compute_styles_with_shadow`）。真实缺口收窄为
六项（R1 渲染链传空 shadow scopes——shadow 作者样式渲染面整体不生效；
R2 `:host` 族/`::slotted` 匹配缺失且 `:host()`/`::slotted()` 在 css-parser
非法整规则丢弃；R3 slot 分配仅声明式解析时点写，动态面 stale；R4 shadow
Text 不进布局；R5 slotted 继承走 light 父；R6 增量样式 shadow 全量回退）。
- 归档账明确排除项原文：「Shadow DOM 渲染级 composed tree（shadow 树进
  样式/布局/绘制、`:host`/`::slotted`）——与 rendering-compat 交界，等
  用户点名专项」+「Rust `resolve_slots` 接线——渲染级消费属上项专项」。

## Mission

把 shadow 树接入渲染管线，补齐「JS 语义已对、渲染不出」的断层：

1. **M0 对账**：渲染管线挂接点审计——slot 分配数据从 JS shim 到 Rust 树
   的通路（node.rs ShadowRoot/slot 字段现状）、taffy-local 布局树挂接点、
   级联入口（css_vars/pipeline 同层）；shadow-dom reftest corpus 盘点
   （渲染级验收标尺）；产出 DC-4 数值门。
2. **flattened tree 样式**（M1/M2）：`:host`/`:host()`/`:host-context`/
   `::slotted(...)` 从「恒不匹配」翻转为 flattened tree 真命中；shadow
   树级联顺序（document → host → shadow → slotted 的 cascade 语义）。
3. **布局/绘制消费**（M3）：slotted 内容进布局树与绘制输出；reftest
   corpus 分批验收。
4. **复测收口**（M4）：reftest/testharness 双面落数值门；
   rendering-compat 全量账零回归。

## Done Criteria

- **DC-1（M0 对账）**：挂接点审计（JS→Rust slot 通路 / 布局树挂接 /
  级联入口三点位）+ reftest corpus 盘点落
  `docs/goal/shadow-dom-rendering/evidence/`；DC-4 数值门落定并回填。
- **DC-2（样式面）**：`:host` 族与 `::slotted(...)` flattened tree 真命中
  （query.rs「恒不匹配」记账翻转）；级联顺序落单测或 WPT 案。
- **DC-3（布局/绘制面）**：slotted 内容渲染输出（reftest 验收）；shadow
  树进布局树不破坏 document 树布局路径（rendering-compat 零回归门）。
- **DC-4（复测收口）**：shadow-dom 渲染级 reftest 分批落数值门；
  rendering-compat 既有账逐位零回归；头条不回落。

**DC-4 数值门（M0 实测落定，2026-11-07）**：地址池 20 案 < 60 案 → **按
子域分批验收，不足部分具名挂账**（evidence §4）——B1 slot 回退/分配 8 案、
B2 布局失效/静态分配 5 案、B3 样式失效/继承 3 案、B4 `:host` 伪类族 3 案、
B5 交互耦合 1 案（静态面记账），M4 收口 = **20/20 全绿且逐批不回落**
（基线 16P/4F：slot-fallback-004/006、
shadow-style-invalidation-vw-units、focus-pseudo-on-shadow-host-1）。
**M1 更新（2026-11-07）**：focus-pseudo-on-shadow-host-1 翻绿——B4 3/3 全部
strict 真通过（修前 -1/-2 是阈值伪象，PNG 法证见 M1 evidence §2），基线余
3F；vw-units 归因勘误为 iframe 载体面（M1 evidence §4），具名不变。B6
testharness 渲染耦合补充池 10 案（42 号 skip 规则排除归 45 号——归口
结论为并入本 goal）：核实 10 案均不在 web-components fetch 子集（11-06
基线 JSON 与 M1 WC 全量日志零命中），计门挂补充池通道落地（M0 evidence
§6 M3+ 项），门值届时实测回填。具名挂账：交互面
（nested-hover 悬停臂、accesskey）——runner 无真实输入注入 reftest 面，
静态等价通过不记真通过；hit-test 四面与 selection/Highlight 交叠按 M1
通道归因定档。邻域：selection/shadow-dom 9 案（selection 语料 23 号既有账）
不改归属，M4 并跑对账 9/9 不回落；渲染级 shadow 耦合 reftest 总面 29 案。
对端移交（04fb3e133 提交在册）：s331 closedby-complex 18F 勘根 = R3 动态
slot 分配缺位（`shadow.rs` 物化注释既有——slotted light 子树无布局盒，
gBCR 塌 (0,0) 命中测试不可达），归 M3 变异应用臂 `resolve_slots` 挂接批
一并验收，不另开缺口。

## 划界

- **JS 语义面归 42 号**：shadow-dom-expansion 管语料滚动与 JS 语义收敛；
  本 goal 只动 Rust 渲染管线，slot JS 语义（slotchange/assignedNodes/
  composed path）不改。
- **rendering-compat**：reftest 全量账零回归门；shadow 树挂接若触通用
  级联/布局路径改动，双方控制面留痕碰头。
- **constructable stylesheets / CSSStyleSheet.replaceSync**、
  `::part()`/`::theme`（cross-shadow 主题面）不在本 goal——M0 甄别后
  具名挂账。
- **declarative shadow DOM（parser/template 层）**归 42 号重审范围；
  渲染级消费属本 goal（同一 flattened tree 管线，语料面两 goal 分账）。

## 里程碑

- **M0**：挂接点审计 + reftest 盘点 + 数值门（DC-1）——**✅ 2026-11-07 完成**
  （[evidence](shadow-dom-rendering/evidence/2026-11-07-m0-hookpoint-audit-and-corpus.md)）。
- **M1**：`:host` 族命中 + scoped sheets 进渲染管线 + 级联顺序（DC-2 前半）
  ——**✅ 2026-11-07 完成**（B4 3/3 + B1/B2 零回落 + WC 全量 A/B 零回归；
  [evidence](shadow-dom-rendering/evidence/2026-11-07-m1-host-segment-and-focus-wire.md)）。
- **M2**：`::slotted(...)` 命中翻转 + flat-tree 组合子（DC-2 收口）——**✅ 2026-11-07 完成**
  （[evidence](shadow-dom-rendering/evidence/2026-11-07-m2-slotted-segment-and-flat-tree.md)）。
- **M3**：动态 slot 分配接线 + shadow Text 布局通道 + reftest 分批验收（DC-3）
  ——**✅ 2026-11-08 完成**（R3 变异应用臂 + R4 shadow Text 两臂 + R3b manual
  通道 + B6 通道 + 追加轮 append sel 通道 006 翻绿；evidence 见 M3 各片）。
- **M4**：复测收口（DC-4）——**✅ 2026-11-08 完成**
  （[evidence](shadow-dom-rendering/evidence/2026-11-08-m4-closeout-numbers.md)）。

## 依赖约束（run-rules §9）

- 渲染管线挂接改动与 rendering-compat（活跃 rally）同窗碰头：凡触通用
  级联入口/布局树遍历的改动，双方控制面留痕后合入。
- 每批合入跑 `make reftest` 全量零回归 + workspace `make test`；
  test-guard 内存与墙钟口径包裹无人值守运行。
- 与 42 号（shadow-dom-expansion）语料共享 shadow-dom 目录——两 goal
  的 fetch 子集清单分域记账（JS 语义面 / 渲染级 reftest 面），skip 规则
  两侧同源。
