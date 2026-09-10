# Web Components — 运行时控制面板（master.md）

**入口文档**: [../web-components.md](../web-components.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-11（M3 切片 8 第二增量第二小步——R114/R167 事件路径同构化
（slot detour + shadow root 站 + per-station retarget）+ relatedTarget 全语义面
（retarget/停止规则/空路径 guard/clearTargets）+ composedPath closed 隐藏，净 +67
零回归；余：every-slots 的 shadow 身份分裂 + event-with-related-target + DC-5 终判）

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

1. **M3 切片 8 第二增量续（2026-09-11 二小步实测后余项）**：
   - ~~R114 重设计 ②③（slot detour + shadow root 站 + retarget）~~ ✅ 2026-09-11
     spec 直译落地（handle `_dispatchWithBubble` + plain R167 双世界），净 +67 零回归，
     见 evidence/2026-09-11-wc-m3s8c.md
   - ~~relatedTarget 全语义面~~ ✅ 同日（每站 retarget / 停止规则 / 空路径 guard /
     clearTargets；composedPath closed 隐藏 `__zwCpClosedFilter`）
   - **every-slots 2F 根因已定位**：plain 克隆宿主 `.shadowRoot` 读值 ≠ attachShadow
     注册对象（own data property 值是二次克隆 B-shadow——detour 落 B 宇宙站无页面
     listener）。修法：detour 改走 identity 稳定源（A 宇宙 `_zwForceParentLink` 链
     上溯，或排查 getter 写回序）。
   - **event-with-related-target 18F**：自有 log 形态（labels 全 undefined）——专用
     helper/树构建的世界归属未查，独立小切片。
   - capturing-and-bubbling 余 3F、event-composed-path-after-dom-mutation 1F。
2. slotchange 尾 12 案 + disabledFeatures×attachShadow registry 集成（尾 2 案）。
3. Rust `resolve_slots` 接线（渲染级消费等用户点名专项）+ **DC-5 终判**
   （make test + clippy + reftest 持续全绿基础上）。

### 已知挂账（2026-09-11 M3 切片 8 更新）

- ~~嵌套 template 装配的 childNodes 回指环~~ ✅ M2 结构性消除
- ~~slots-fallback 混合树身份贯通~~ ✅ M3 切片 3 收口
- ~~imperative-slot-api 全簇~~ ✅ M3 切片 4 收口
- ~~name-mode slotchange diff 化~~ ✅ M3 切片 5 收口
- ~~slottable 规范过滤 + 文本 slottable 可见性~~ ✅ M3 切片 6 收口
- ~~attachShadow 规范校验（ns + safelist + 异常 realm）~~ ✅ M3 切片 7 收口
- ~~plain 世界 composed path/shadow root 站/retarget~~ ✅ M3 切片 8 第一增量收口
  （event-composed 9/9）
- ~~R114/R167 事件路径同构化（slot detour + shadow root 站 + per-station retarget）
  + relatedTarget 全语义面~~ ✅ M3 切片 8 第二增量第二小步收口（event-composed-path
  11/11、event-inside-shadow-tree 12/12、event-inside-slotted-node 20/20、
  event-post-dispatch-no-listeners 5/5）
- every-slots 2F：plain 克隆宿主 `.shadowRoot` getter 读值与 attachShadow 注册对象
  身份分裂（B-shadow 二次克隆，detour 站无页面 listener）——修法见下一步计划
- event-with-related-target 18F：自有 log 形态 labels 全 undefined——helper/树构建
  世界归属未查
- capturing-and-bubbling 余 3F、event-composed-path-after-dom-mutation 1F
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
| M3 — slot 全链路 + 收尾 | ◐ 切片 1-7 ✅ + 切片 8 第一/二增量 ✅（composed path/shadow root 站/per-station retarget/relatedTarget 全语义面——净 +67）；余 every-slots 身份分裂 2F + event-with-related-target 18F + 尾簇 + DC-5 终判 |

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| Shadow DOM 渲染级 composed tree 专项 | ⬜ 等点名 | shadow 树进样式/布局/绘制、:host/::slotted——与 rendering-compat 交界，按 run-rules §9 协调 |
| adoptedStyleSheets / Constructable Stylesheets | ⬜ 待议 | 依赖 CSSOM 深化 |

## 验证基线

- 测试基线：立项时点全绿（`make test` / `make reftest` 入口，经 test-guard 包裹；
  禁止裸跑 cargo test）
- WC 用例面：**3494 Pass / 4763 subtests（73.4%）**（2026-09-11 M3 切片 8 第二增量
  第二小步，evidence/2026-09-11-wc-m3s8c.{md,json}。历史：基线 437=9% → 2a 689=15% →
  2b 2565=55% → 3 2720=58% → 4 3008=64% → M2 3014 → M3s1 3046 → M3s2 3050 → M3s3
  3105 → M3s4 3137 → M3s5 3142 → M3s6 3145 → M3s7 3413 → M3s8 3425 → M3s8b 3427 →
  M3s8c 3494）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  dom 结构变更轮跑 `make reftest` 作渲染面守卫
