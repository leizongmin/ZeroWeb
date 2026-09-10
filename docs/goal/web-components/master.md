# Web Components — 运行时控制面板（master.md）

**入口文档**: [../web-components.md](../web-components.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-11（M3 切片 7 落地——attachShadow 规范校验（非 HTML ns +
safelist + _throwDom 异常 realm——R126 推广），净 +268；WC 面 3145→3413（72%）；
余项：event retarget 族（~110）+ slotchange 尾 12 案 + DC-5 终判）

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

1. **M3 切片 8（event retarget 族，~110 subtest——Support Envelope 覆盖范围 4）实施
   侦察已毕（2026-09-11），下轮按下列三线并进**：
   - **路径构造**：`_dispatchWithBubble`（R114）跨边界时 **shadow root 本体不入站**
     （注释明言「shadow root 本身无 listener 站，直接断链」）——链须插入 shadow root
     站（非 composed 止于它、composed 续到 host）；composedPath（R3244）同步补站
   - **plain 世界**：createTestTree 树（plain clone）派发走 R167 工厂管线
     （parentNode 链已含轻量 shadow 对象站），但 shadow 站的 listener 存储与
     fireStation 读取域（`_mEvListeners`/`_zwEvLs`/`_zwLocalListeners`/
     `_listenerStore` 四域）未接通——WPT event-composed.html 的
     「expected 2 but got 1」（shadowRoot listener 已挂不 fire）即此
   - **retarget 语义**：event.target 现单点设置（dispatch 入口），须按站 shadow-adjusted
     retarget（shadow 树内站 = target；host 及以上 = 最近边界 host；嵌套 shadow 外层
     root = 内层 host）；relatedTarget retarget（event-with-related-target 族）与
     post-dispatch 语义在路径/retarget 落地后独立小步
   - 改动域与 event-loop-spec 流的 part01.js 无直接共享段，动 `_dispatchWithBubble`
     前照例 `git log --since="14 days ago" -- crates/engine/` 核对
2. slotchange 尾 12 案 + disabledFeatures×attachShadow registry 集成（尾 2 案）。
3. Rust `resolve_slots` 接线（渲染级消费等用户点名专项）+ **DC-5 终判**
   （make test + clippy + reftest 持续全绿基础上）。

### 已知挂账（2026-09-11 M3 切片 7 更新）

- ~~嵌套 template 装配的 childNodes 回指环~~ ✅ M2 结构性消除
- ~~slots-fallback 混合树身份贯通~~ ✅ M3 切片 3 收口
- ~~imperative-slot-api 全簇~~ ✅ M3 切片 4 收口
- ~~name-mode slotchange diff 化~~ ✅ M3 切片 5 收口
- ~~slottable 规范过滤 + 文本 slottable 可见性~~ ✅ M3 切片 6 收口
- ~~attachShadow 规范校验（ns + safelist + 异常 realm）~~ ✅ M3 切片 7 收口
  （attach-shadow-non-html-namespace 266/266）
- event retarget 族（~110 subtest）——M3 切片 8
- disabledFeatures=['shadow'] × attachShadow（尾 2 案——CE registry definition 感知）；
  canvas 等 createElement 产物 _realTag pending 回落 'div' 的 safelist 误放行
- slotchange 尾 12 案（dispatch/observer 层）：innerHTML 尾计数（Chrome async host
  apply）、transient slot（wrapper 身份二相）、mutation-observer 交错、嵌套 retarget
- builtin-coverage 的 innerHTML 解析簇（~108F）：R380 查询融合域，非 CE 域
- reactions/ 表格族（table-scoped 解析升级）与 customized-builtins 的 iframe/reparse 面
- XHTML 悬空 body 查询域（5 案）——pre-existing，与 template 真实化无因果
- ElementData 加字段前必须过 bench-gate（历史 +2-3x 教训）

**碰撞管理**：碰 engine/dom 前先 `git log --since="14 days ago" -- crates/engine/
crates/dom/` 核对渲染流域活跃面；碰 part01.js 前与 event-loop-spec 流互相核对。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + Custom Elements 收口 | ✅ 2026-09-10（切片 1/2a/2b/3/4 全清——DC-1 基线 + DC-2 upgrade/whenDefined/adoptedCallback/双路径全收口） |
| M2 — template 真实化 | ✅ 2026-09-10 收口（切片 1 + iframe/detached 工厂 content 视图——DC-3 全满足） |
| M3 — slot 全链路 + 收尾 | ◐ 切片 1-7 ✅（HTMLSlotElement 接口 + IDL + slotchange flatten-diff 化 + find-a-slot 仲裁 + flatten spec 形 + imperative slot API 全簇 + slottable 规范过滤 + attachShadow ns/safelist/realm 校验）；余切片 8（event retarget 族）+ DC-5 终判 |

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| Shadow DOM 渲染级 composed tree 专项 | ⬜ 等点名 | shadow 树进样式/布局/绘制、:host/::slotted——与 rendering-compat 交界，按 run-rules §9 协调 |
| adoptedStyleSheets / Constructable Stylesheets | ⬜ 待议 | 依赖 CSSOM 深化 |

## 验证基线

- 测试基线：立项时点全绿（`make test` / `make reftest` 入口，经 test-guard 包裹；
  禁止裸跑 cargo test）
- WC 用例面：**3413 Pass / 4762 subtests（72%）**（2026-09-11 M3 切片 7，
  evidence/2026-09-11-wc-m3s7.{md,json}。历史：基线 437=9% → 2a 689=15% → 2b 2565=55% →
  3 2720=58% → 4 3008=64% → M2 3014 → M3s1 3046 → M3s2 3050 → M3s3 3105 → M3s4 3137 →
  M3s5 3142 → M3s6 3145 → M3s7 3413）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  dom 结构变更轮跑 `make reftest` 作渲染面守卫
