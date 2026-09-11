# Web Components — 运行时控制面板（master.md）

**入口文档**: [../web-components.md](../web-components.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**状态**: ✅ **DONE（2026-09-11 收口判定）**——DC-1~5 全部满足，Final Output Protocol
输出 DONE。收口记录见 [archive/2026-09-11-goal-closure.md](archive/2026-09-11-goal-closure.md)。

---

## 最终状态（收口判定）

- **成绩**：WPT 三目录 265→271 案，**3619 / 4764 subtests（76.0%）**——基线
  437/4730（9%）累计 **+3182 Pass**，全程 54 份 evidence（md+json 成对，逐步可溯）。
- **交付面**：
  - M1——WPT 三目录导入 + 基线 + Custom Elements 收口（upgrade 真语义 / whenDefined
    真 Promise / adoptedCallback / v8 native + quickjs 双路径一致）。
  - M2——template DOM 层真实化（parser inert fragment contents + content 属性 +
    R145 查询规则收敛）。
  - M3——slot 全链路（IDL → 分配 → slotchange → assignedNodes({flatten}) → 基础
    flattened tree 查询）+ shadow-dom JS 语义增修（composed path / shadow root 站 /
    per-station retarget / relatedTarget / eventPhase / clearTargets / disabledFeatures
    gate）。
  - 切片 8 第二增量 13 小步（2026-09-11 单日）：slotchange 尾 12 案全清（notify
    mutation observers 步骤 4-7 spec 直译 + 接收者 identity + diff 双 map 口径）、CE
    reactions 反射面、CE markup 构造管线（innerHTML/outerHTML/insertAdjacent 构造 +
    连接 + 跨文档 adopted）、width/height 数值反射。
- **门禁（收口时点）**：make test 67 套件全绿 / clippy -D warnings 零警告 /
  reftest 687/687（100%）/ cargo build 经 compile-first 覆盖。

## DC-1~5 判定（逐项证据见 archive 收口记录）

| DC | 判定 |
|---|---|
| DC-1 WPT 导入 + 基线 + 报告 + 账本 + 持久化 | ✅ |
| DC-2 upgrade / whenDefined / adoptedCallback / 双路径 | ✅ |
| DC-3 template 三项（inert contents / content 属性 / 资源不加载） | ✅ |
| DC-4 slot 四项（IDL / assignedNodes / slotchange / flattened tree） | ✅ |
| DC-5 make test / clippy / 单测+WPT 资产化 / reftest | ✅ |

## 遗留深项（收口后记录，非 DC 范围、非本 goal 义务）

- reactions/ 剩余 fail（~207F）：media 播放联动、table insertRow/deleteRow（table-scoped
  解析升级）、Range/cloneContents 交叉面——独立深项域。
- builtin-coverage（~126F）：customized built-ins innerHTML 解析实例化（R380 查询融合域
  交界）。
- XHTML 悬空 body 查询域（5 案）——pre-existing（M2 已判无因果）。
- customized-builtins iframe/reparse 面——深结构。

## 渲染级排除（等用户点名专项，不算未满足 DC）

- **Shadow DOM 渲染级 composed tree**（shadow 树进样式/布局/绘制、`:host`/`::slotted`）
  ——与 rendering-compat 交界，按 Support Envelope 排除条款。
- **Rust `resolve_slots` 接线**——JS 面 slot 全链路已完成；渲染级消费属上项专项。

## 碰撞管理（存档备查）

共享面 crates/engine、crates/dom 碰前 `git log --since="14 days ago"` 核对；part01.js
（event-loop-spec 流主力）碰前互相核对——本 goal 全程未发生共享面冲突。

## 资产位置

- 收口记录：[archive/2026-09-11-goal-closure.md](archive/2026-09-11-goal-closure.md)
- 逐步证据：[evidence/](evidence/)（2026-09-10 基线 → 2026-09-11 收口，54 文件）
- 常驻断言集账本：`tests/wpt-runner/imported-testharness.txt`（WC-M1-baseline 批次）
- 执行通道：`make testharness-web-components`（`scripts/fetch-web-components-subset.sh`，
  pin 同版；271 案每轮全量复跑）
