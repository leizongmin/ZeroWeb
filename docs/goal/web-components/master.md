# Web Components — 运行时控制面板（master.md）

**入口文档**: [../web-components.md](../web-components.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-11（M3 切片 6 落地——slottable 规范过滤（Comment/PI 排除，
HTMLSlotElement-interface 18/18）+ 纯文本 innerHTML 的文本 slottable 可见性，净 +3；
余项：slotchange 尾 12 案（dispatch/observer 层）+ DC-5 终判）

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
| P1 | WPT 用例覆盖为零（fetch 脚本 + 三目录导入 + 基线） | ✅ 2026-09-10（265 案导入，基线 9%，见 evidence/2026-09-10-wc-baseline.md） |
| P2 | CE 三缺口（upgrade no-op / whenDefined 假 resolve / adoptedCallback 无） | ✅ 2026-09-10 切片 4（adopted 切片 3；upgrade shim 真实现 + is-aware；whenDefined define 前 pending/define 后 microtask resolve——probe 断言） |
| P3 | template DOM 层占位（parser + R145 规则收敛） | ✅ 2026-09-10 M2 收口（DC-3 三项全满足；XHTML 悬空 body 5 案为 pre-existing 查询域问题，记挂账） |
| P4 | slot 全链路（IDL → 分配接线 → slotchange → assignedNodes） | ◐ M3 切片 1-4 ✅（IDL + flatten/find-a-slot + plain slotchange + imperative slot API 全簇 36→36 绿）；余 name-mode slotchange diff 化 + Rust 接线 |

## 基线（2026-09-10 M1 切片 1）

- 执行通道：`make testharness-web-components`（fetch 脚本 `fetch-web-components-subset.sh`，pin 同版）
- 分母：265 案（custom-elements 175 / shadow-dom 62 / the-template-element 28），
  4730 subtests——**Pass 437（9%）**（CE 9% / shadow-dom 13% / template 4%）
- 基线失败聚类 10 类见 `evidence/2026-09-10-wc-baseline.md`。定向修复顺序按 ROI：
  ① shadow 树 appendChild 断裂（~112）② CustomElementRegistry 接口对象（~88）
  ③ reactions/ per-API 反应（59 案整簇）④ slot 全链路（M3 主线）⑤ template DOM 层（M2 主线）
- skip 域：渲染级 composed tree / 几何命中 / 交互面（fetch 脚本头注释与 runner
  `wc_case_skipped` 同一规则集），等用户点名 Shadow DOM 渲染级专项

## 下一步计划

1. **DC-4/DC-5 收尾评估**：slot 语义观测面（assignedNodes/flatten/assignedSlot/
   slotchange/imperative assign）已由 shim 满足且三目录通过率 66%；剩余 slotchange 尾
   12 案为 dispatch/observer 层工作（wrapper 身份二相、MutationObserver 集成、嵌套
   retarget、Chrome async host apply 语义差）——属 event/dispatch 基建，评估是否并入
   event-loop-spec 流或等专项点名；Rust `resolve_slots` 接线的消费面为渲染级查询
   （等用户点名后按 run-rules §9 协调）。若判定 DC-4 观测面已满足，则整理 DC-1~5
   逐项核对材料做终判。
2. **DC-5 全量门禁**（`make test` + clippy + reftest 持续全绿基础上）终判

### 已知挂账（2026-09-11 M3 切片 6 更新）

- ~~嵌套 template 装配的 childNodes 回指环~~ ✅ M2 结构性消除
- ~~slots-fallback 混合树身份贯通~~ ✅ M3 切片 3 收口
- ~~imperative-slot-api 全簇~~ ✅ M3 切片 4 收口
- ~~name-mode slotchange diff 化~~ ✅ M3 切片 5 收口（flatten 口径 marked-transition 链）
- ~~slottable 规范过滤（Comment/PI）+ 文本 slottable 可见性~~ ✅ M3 切片 6 收口
- slotchange 尾 12 案（dispatch/observer 层——非 slot 语义）：
  innerHTML 尾计数（Chrome async host apply 语义差，upstream 测试自带 FIXME 同域）、
  transient slot（wrapper 身份二相——dispatch/retarget 层）、mutation-observer 交错
  （需 MutationObserver 集成）、嵌套 retarget（composed-path 深化）
- builtin-coverage 的 innerHTML 解析簇（~108F）：sel 容器 innerHTML 后
  getElementById 的 pending 融合（R380 查询融合域，非 CE 域）
- reactions/ 表格族（table-scoped 解析升级）与 customized-builtins 的 iframe/reparse 面
- XHTML 悬空 body 查询域（`additions-to-parsing-xhtml-documents/node-document.html`
  5 案）——pre-existing（M2 前同 Fail），与 template 真实化无因果
- ElementData 加字段曾致 dom 微基准全线 +2-3x（分配/layout 阈值效应）——contents 已改
  Document 侧表规避；后续给 ElementData 加字段前必须过 bench-gate

**碰撞管理**：碰 engine/dom 前先 `git log --since="14 days ago" -- crates/engine/
crates/dom/` 核对渲染流域活跃面；碰 part01.js 前与 event-loop-spec 流互相核对。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + Custom Elements 收口 | ✅ 2026-09-10（切片 1/2a/2b/3/4 全清——DC-1 基线 + DC-2 upgrade/whenDefined/adoptedCallback/双路径全收口） |
| M2 — template 真实化 | ✅ 2026-09-10 收口（切片 1 + iframe/detached 工厂 content 视图——DC-3 全满足） |
| M3 — slot 全链路 + 收尾 | ◐ 切片 1-6 ✅（HTMLSlotElement 接口 + IDL + slotchange flatten-diff 化 + find-a-slot 仲裁 + flatten spec 形 + imperative slot API 全簇 + slottable 规范过滤）；余 DC-4/DC-5 收尾评估（见下一步计划） |

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| Shadow DOM 渲染级 composed tree 专项 | ⬜ 等点名 | shadow 树进样式/布局/绘制、:host/::slotted——与 rendering-compat 交界，按 run-rules §9 协调 |
| adoptedStyleSheets / Constructable Stylesheets | ⬜ 待议 | 依赖 CSSOM 深化 |

## 验证基线

- 测试基线：立项时点全绿（`make test` / `make reftest` 入口，经 test-guard 包裹；
  禁止裸跑 cargo test）
- WC 用例面：**3145 Pass / 4762 subtests（66%）**（2026-09-11 M3 切片 6，
  evidence/2026-09-11-wc-m3s6.{md,json}。历史：基线 437=9% → 2a 689=15% → 2b 2565=55% →
  3 2720=58% → 4 3008=64% → M2 3014 → M3s1 3046 → M3s2 3050 → M3s3 3105 → M3s4 3137 →
  M3s5 3142 → M3s6 3145）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  dom 结构变更轮跑 `make reftest` 作渲染面守卫
