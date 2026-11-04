# Shadow DOM 渲染级 — composed tree 进样式/布局/绘制 + :host/::slotted 命中

**版本**: v1.0
**日期**: 2026-11-04（立项，用户拍板「现在就立」——web-components 收口排除条款转正）
**状态**: Active——立项，M0 未启动
**执行模式**: 引擎专项（渲染管线挂接）+ WPT shadow-dom 语料验收
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——Web Components 渲染面）
**前账**: `docs/goal/archive/web-components.md`（2026-09-11 收口，DC-1~5 ✅——
JS 面 slot 全链路已落；渲染级按 Support Envelope 排除条款挂账）

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

**DC-4 数值门决策规则（立项即定）**：M0 reftest 盘点后按地址池分档——
渲染级可验案 ≥ 60 案：全量推进，数值门 = 地址池 × 验收通过率（M0 实测
后填入）；< 60 案：按子域分批验收，不足部分挂账具名。testharness 面的
渲染耦合簇（42 号甄别出的）按归口结论并入本 goal 或留 42 号。

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

- **M0**：挂接点审计 + reftest 盘点 + 数值门（DC-1）。
- **M1**：`:host` 族命中 + 级联顺序（DC-2 前半）。
- **M2**：`::slotted(...)` 命中翻转（DC-2 收口）。
- **M3**：slotted 布局/绘制消费 + reftest 分批验收（DC-3）。
- **M4**：复测收口（DC-4）。

## 依赖约束（run-rules §9）

- 渲染管线挂接改动与 rendering-compat（活跃 rally）同窗碰头：凡触通用
  级联入口/布局树遍历的改动，双方控制面留痕后合入。
- 每批合入跑 `make reftest` 全量零回归 + workspace `make test`；
  test-guard 内存与墙钟口径包裹无人值守运行。
- 与 42 号（shadow-dom-expansion）语料共享 shadow-dom 目录——两 goal
  的 fetch 子集清单分域记账（JS 语义面 / 渲染级 reftest 面），skip 规则
  两侧同源。
