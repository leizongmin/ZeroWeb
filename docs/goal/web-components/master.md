# Web Components — 运行时控制面板（master.md）

**入口文档**: [../web-components.md](../web-components.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（立项——M1 待启动）

---

## 当前状态

**专项定位**：父目标 M12/Tier 2「Web Components」无 goal 认领项。底子分层清晰——
Custom Elements 程度较高（真 registry + 三回调端到端、双路径）、template JS 层可用
DOM 层占位、slot 接近零（Rust 有孤立数据结构、engine 零接线）。一期只做 dom/engine 侧，
Shadow DOM 渲染级 composed tree 排除（等用户点名专项）。

**与兄弟 goal 的边界**：
- rendering-compat — 渲染流域 crate 域一期零重叠；Shadow DOM 渲染级是两流域交界，
  本流明确排除、等用户点名后按 run-rules §9 专项立项
- event-loop-spec — engine 共享大文件池：其主力 part01.js（observers/时序段），本流主力
  part03/04/05（slot/template/shadow 段）；无直接共享段，碰 part01.js（slotchange 事件名
  联动）前互相 `git log` 核对
- storage-opfs / page-wasm / webdriver / android-browser — 无共享面
- 共享面：crates/engine、crates/dom——碰之前
  `git log --since="14 days ago" -- crates/engine/ crates/dom/` 核对

## 实测基线（2026-09-07 立项时）

### 现有实现

- ✅ Custom Elements（程度较高）：
  - Rust lifecycle 桥 `dom_bindings/custom_elements.rs` 318 行（connect/disconnect 真转
    + 子树 spec 触发序 + attributeChanged R3267 S5d）
  - quickjs 路径 `quickjs_dom_bindings.rs` L1928 起五件套（define 自动升级 R149）
  - v8 native 路径 `dom_bindings/factories.rs`（S5b upgrade R3265，Reflect.construct
    复用 host NodeId）
  - 测试：tests_ce.rs 164 行 + e2e_web_components.rs 374 行 8 用例 + e2e_lit_library.rs
    391 行
- ✅ template JS 层：part04.js L710+ content fragment 视图（lit-html 管线可跑）
- ✅ Shadow DOM JS API 层基础：attachShadow open/closed 校验（R2926）+ shadowRoot getter
  + 树内 DOM/查询 + composed/getRootNode retarget 基础
- ✅ Rust DOM slot 数据结构（document/shadow.rs 207 行 assign_slot/resolve_slots/
  assigned_nodes）——孤立存在
- ⚠️ CE 缺口：`customElements.upgrade` no-op（L2049）、`whenDefined` 同步简化、
  `adoptedCallback` 无
- ⚠️ template DOM 层占位：parser.rs L302-303 `get_template_contents` 返回目标节点自身
  （内容内联文档树，非 inert fragment）；R145 querySelector 例外规则规避误命中
- ⚠️ slot 接线为零：engine 零调用 Rust 分配机制；JS 层无 el.slot/assignedSlot/
  assignedNodes/slotchange
- ⚠️ Shadow DOM 不落 `Document::shadow_roots`（engine 非测试代码零调用）、不进渲染管线、
  无 `:host`/`::slotted`（渲染级，一期排除）
- ⚠️ WPT 覆盖为零：custom-elements / shadow-dom / the-template-element 目录在 wpt-data
  均不存在；imported-tests.txt 仅 1 行 R149 注记

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | WPT 用例覆盖为零（fetch 脚本 + 三目录导入 + 基线） | ⬜ M1 |
| P2 | CE 三缺口（upgrade no-op / whenDefined 假 resolve / adoptedCallback 无） | ⬜ M1 |
| P3 | template DOM 层占位（parser + R145 规则收敛） | ⬜ M2 |
| P4 | slot 全链路（IDL → 分配接线 → slotchange → assignedNodes） | ⬜ M3 |

## 下一步计划

1. **M1 切片 1**：fetch 脚本（custom-elements/shadow-dom/the-template-element）+
   用例导入 + 分类通过率基线（现有实现真水平标定；零源码改动）
2. **M1 切片 2**：`customElements.upgrade` 真语义 + `whenDefined` Promise 真等待
3. **M1 切片 3**：`adoptedCallback` 派发路径 + 失败聚类

**碰撞管理**：碰 engine/dom 前先 `git log --since="14 days ago" -- crates/engine/
crates/dom/` 核对渲染流域活跃面；碰 part01.js 前与 event-loop-spec 流互相核对。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + Custom Elements 收口 | ⬜ 待启动 |
| M2 — template 真实化 | ⬜ |
| M3 — slot 全链路 + 收尾 | ⬜ |

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| Shadow DOM 渲染级 composed tree 专项 | ⬜ 等点名 | shadow 树进样式/布局/绘制、:host/::slotted——与 rendering-compat 交界，按 run-rules §9 协调 |
| adoptedStyleSheets / Constructable Stylesheets | ⬜ 待议 | 依赖 CSSOM 深化 |

## 验证基线

- 测试基线：立项时点全绿（`make test` / `make reftest` 入口，经 test-guard 包裹；
  禁止裸跑 cargo test）
- WC 用例面：无基线（未导入/未建）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  dom 结构变更轮跑 `make reftest` 作渲染面守卫
