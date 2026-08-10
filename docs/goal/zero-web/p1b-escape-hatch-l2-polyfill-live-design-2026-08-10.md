# P1b L2 polyfill-live 实施设计——escape-hatch 收敛（native 成生产路径）

**日期**：2026-08-10
**关联轮次**：R3178（设计 doc；实施为 gated 切片，需用户点名 rule 11）
**父目标**：`docs/goal/zero-web.md` P1 DOM/JS Bridge 原生化 → P1b L2 polyfill-live
**关联**：`docs/specs/p1b-v8-native-bindings-rfc.md` §3.7（L1/L2/L3 分阶段）；`p1b-mutationobserver-host-trigger-design-2026-08-10.md`（MO 伴随）
**状态**：设计就绪，**实施需用户点名**（rule 11 深结构门禁——L2 = 🔴 高风险，polyfill 热路径 + 14137 测试依赖）。本文档把 RFC §3.7 的 L2 单行 sketch 展开为可实施切片计划 + 正面解决阻塞 crux，使 L2 从「需用户拍板」变为「有清晰实施路径」。

---

## 0. 执行摘要

- **一句话目标**：让 polyfill 桥（B）的 `__zw_*` 回调读/写**共享 live Document C**（renderer 的 `cached_doc`），而非每次 re-parse 序列化 `dom_html` String——使 native(A)=polyfill(B)=renderer(C) **三方合一**，polyfill 不再经 String round-trip，native 成生产路径（escape-hatch 收敛，P1b 终极价值）。
- **为何 gated**：L2 触碰 polyfill 热路径 + 14137 测试依赖的 String 行为；属 rule 11 深结构。本文档给出去风险路径：**全默认 OFF kill-switch + A/B 行为对照门 + 单回调渐进路由**，使每片可独立 land（零回归）+ 可即时回退。
- **核心 crux（阻塞 22 轮的根因）**：`render_html`（pipeline/mod.rs:434）每次 re-parse `cached_html` **替换** `cached_doc`（C）。若 polyfill 直接写 C，re-render 覆盖 C → mutation 丢失；若 polyfill 读 C，re-render 前 in-flight mutation 不可见。**L2 必须解决 C↔cached_html 的一致性**——本文档的解决方案 = C 成唯一真源（single source of truth），cached_html 降为 C 的派生序列化。
- **推荐方案**：三阶段（L2a 读路由 / L2b 写路由 / L3 清理），每阶段 kill-switch `ZW_NATIVE_DOM=polyfill-live-<scope>` 默认 OFF；L2a 先把**只读、无 in-flight 依赖**的回调（如 `matches`/`closest` 纯选择器判定）路由到 C 验证管线，再扩到写回调。
- **首个落地步骤（L2a-1）**：选一个**幂等只读**回调（`__zw_matches`）经 `with_dom` 读 C（OFF 时仍 re-parse String），加 A/B 对照门断言两路径同结果 + make test 零回归（OFF）。

---

## 1. 现状（A/B/C 三方分裂 + L1 已闭合 A↔C）

经全量侦察（`p1b-v8-native-bindings-rfc.md` §3.7 + 本轮核查 `gc.rs` / `js_dom_bridge/*` / `pipeline/mod.rs`）：

| Document | 持有者 | 来源 | 随 mutation 同步？ |
|----------|--------|------|-------------------|
| **A. native** | native bindings（`gc.rs` `DOM_SOURCE` thread_local） | L1 后 = C 共享句柄（`with_dom`/`with_dom_mut`） | ✅ 是（L1 闭合） |
| **B. polyfill 瞬态** | polyfill 桥（`__zw_*` 回调内） | 每操作 `parse_html(dom_html)` re-parse String | ❌ 否（每次重新解析；写经 `DomMutation`→`apply_dom_mutations`→序列化回 String） |
| **C. renderer live** | `RenderPipeline.cached_doc: Option<Rc<RefCell<Document>>>` | `render_html` 解析 `cached_html` | ✅ 是（渲染用此） |

- **L1（R3106–R3108）已完成**：`cached_doc` 改 `Rc<RefCell<Document>>`；`DOM_SOURCE`（gc.rs:27）+ `set_dom_source`（:110）+ `with_dom`/`with_dom_mut`（:330/:343）；webview `run_page_scripts_impl` 经 escape-hatch 把 `pipeline.cached_doc_shared()` 注 `install_dom_bindings`。**native(A) 读/写 = C**（去 inert）。
- **L2 未做**：polyfill(B) 仍 re-parse String。`js_dom_bridge/selector_match.rs`/`css_wire.rs`/`callbacks.rs` 共 ~10+ `parse_html(html)` 站点；写经 `DomMutation`（`js_dom_bridge.rs:50` 枚举 ~30 变体）→ `apply_dom_mutations`（:427）→ 序列化回 `dom_html`。

**结论**：A=C（L1），但 B≠C。polyfill 经 String round-trip（parse→mutate→serialize→re-parse-on-render），native 经 C 直读/写。两路径语义对绝大多数操作等价，但：① 性能（parse/serialize 开销，大页面 ~30%）；② 一致性边例（A/B 行为细微差）；③ 维护双份逻辑。

---

## 2. 核心 crux——C↔cached_html 一致性（阻塞根因）

### 2.1 re-render 覆盖 C

`render_html`（pipeline/mod.rs:434）：`self.cached_doc = Some(Rc::new(RefCell::new(doc)))`，其中 `doc = parse_html(cached_html)`。**每次渲染 re-parse `cached_html` 替换 C**。

- 若 polyfill 写直接改 C（`with_dom_mut`），下一次 render re-parse `cached_html`（不含该 mutation）→ **C 被覆盖，mutation 丢失**。
- L1 native 写为何存活：webview 检测 native 改 C 后触发重渲染，且 native 写经 polyfill 路径同步到 `cached_html`（webview.rs:1180「native 绑定直接改 live cached_doc，polyfill 增量路径」——实际 native 写亦走 DomMutation→cached_html 双轨，re-render 保 mutation）。

### 2.2 in-flight mutation 可见性

polyfill 读若改用 C：同一脚本内 `setAttribute` 后 `getAttribute`，C 反映**上次 render**（pre-mutation），**不**含 in-flight 写 → 读见旧值 → 破坏基本 JS 模式。当前 String 路径：写更新 dom_html，读 re-parse dom_html → **in-flight 可见**。

> `getBoundingClientRect` 已接受此 stale（design p1a-layout-geometry-feedback：「rect 反映上次 render，stale-but-non-zero，force-reflow-on-demand 为 follow-up」）——因 gBCR 返值多弃用（作 reflow 触发）。但 `getAttribute`/`querySelector` 等**语义读**不可 stale。

### 2.3 crux 总结

L2 必须**同时**满足：① polyfill 写即时反映到 C（in-flight 可见）；② C mutation 在 re-render 后存活。当前架构二者冲突（写 C → re-render 覆盖）。

---

## 3. 解决方案——C 成 single source of truth

**核心转变**：把 `cached_doc`(C) 从「render 产物（re-parse cached_html）」升格为**唯一真源**；`cached_html` 降为 C 的**派生序列化**（仅供仍需 String 的边角路径 + 测试 + reftest 单渲染）。

### 3.1 写路径（L2b）

polyfill `__zw_*` 写回调经 `with_dom_mut` 直接改 C（同 native）。`cached_html` 不再是写的目标——改为**按需从 C 派生**（serialize C → cached_html，供 reftest/测试/String 边角路径）。

### 3.2 render 路径（关键变更）

`render_html` 不再无条件 re-parse `cached_html` 替换 C：
- **C 存在且 live（L2 开）**：render 直接用 C（`cached_doc` 已是最新——polyfill/native 写即时反映）；**跳过 re-parse**。仅在首次渲染（C=None）re-parse `cached_html` 建 C。
- **C=None 或 L2 关**：维持现状（re-parse cached_html）。
- `cached_html` 改由 C 序列化维护（`doc.outer_html(root)`），供：reftest 单渲染（apply_scripted_dom_mutations 后取最终 HTML）、String 边角回调、debug。

### 3.3 读路径（L2a）

polyfill 读回调经 `with_dom` 读 C。因写即时改 C（3.1），**in-flight 可见**（同 native）。

### 3.4 不变量（安全前提，继承 L1）

单进程 webview 脚本执行（`run_page_scripts`）与渲染（`render`）**顺序**进行（同线程）；`with_dom`/`with_dom_mut` 把 borrow 限定单操作闭包，无跨回调 borrow。L2 不引入新并发，`RefCell` 嵌套 panic 前提不变（任何 V8 回调不持 borrow 跨另一回调）。

---

## 4. 分阶段切片（每片 kill-switch 默认 OFF + A/B 对照门 + make test 零回归）

kill-switch：env `ZW_NATIVE_DOM=polyfill-live-<scope>`（unset/OFF = 现 String 路径，14137 测试不变；ON = 路由 C）。每片附 **A/B 对照门**（同输入两路径结果断言一致），确保 ON 路径行为 = OFF 路径。

| 阶段 | 范围 | 风险 | 验证 | 收益 |
|------|------|------|------|------|
| **L2a 只读路由** | 幂等只读回调（`matches`/`closest`/`children`/`query_*_sub`）经 `with_dom` 读 C（无 in-flight 依赖，最安全） | 🟡 中（读路径，行为须 = String 路径） | A/B 对照门（同 selector 两路径同结果）+ 既有 dom_bridge 测试 + make test（OFF 零回归） | 验证 C 读管线 + handle/selector→NodeId 解析 |
| **L2b-1 写路由（属性/文本）** | `SetAttr`/`RemoveAttr`/`ToggleAttribute`/`SetText`/`SetStyle` 等写回调经 `with_dom_mut` 改 C + render 改用 C（跳过 re-parse） | 🔴 高（写热路径 + render 覆盖 crux 在此正面解决） | 全量 dom_bridge 测试 + WPT + reftest（reftest 取最终 HTML 须 = 现状）+ product-smoke | in-flight 写可见 + C 成真源（核心突破） |
| **L2b-2 写路由（结构）** | `AppendChild`/`InsertBefore`/`CreateElement`/`Remove`/`SetInnerHtml` 等结构写（handle 系统） | 🔴 高（handle→NodeId 映射 + 结构 mutation） | 同 L2b-1 + handle 边例全覆盖 | 写全路由 C |
| **L3 清理** | 移除 `parse_html(html)` re-parse 站点 + `cached_html` String 写路径降为 C 派生 + A/B 分支删除 | 🟢 低（删除已验证冗余） | 全量回归 | 维护体量降 + 性能（去 round-trip） |

---

## 5. handle/selector→NodeId 解析（L2 基建）

polyfill 用 **handle**（createElement 元素，path A）或 **selector**（querySelector/getElementById 元素）标识元素；C 用 **NodeId**。L2 路由须解析：

- **selector→NodeId**：复用 `find_by_selector`（zero_dom 选择器引擎，native S3 已用）。
- **handle→NodeId**：复用 `HandleSelectorMap`（js_worker.rs:12 `new_handle_selector_map`）+ L1 native 的 handle 系统（`__zw_native_element_for_id` 身份缓存）。
- L1 native 已建此解析（native 回调经 selector/handle→NodeId 读 C）；L2 polyfill 复用同一基建。

---

## 6. 风险 + 缓解

| 风险 | 等级 | 缓解 |
|------|------|------|
| **14137 测试依赖 String 行为** | 🔴 高 | kill-switch 默认 OFF → 既有测试全走 String（零回归）；A/B 对照门确保 ON 行为 = OFF；ON 路径独立测试覆盖 |
| **render 覆盖 C（crux）** | 🔴 高 | L2b-1 正面改 render：C live 时跳过 re-parse，cached_html 改 C 派生；reftest 取最终 HTML 须 = 现状（门禁） |
| **in-flight mutation 不可见** | 🟡 中 | L2b 写经 with_dom_mut 即时改 C → L2a 读即时见；L2a 先于 L2b land 时，仅路由幂等只读（无 in-flight 依赖） |
| **A/B 行为细微差** | 🟡 中 | 每片 A/B 对照门 + WPT 全量；kill-switch 即时回退 |
| **reftest 单渲染兼容** | 🟡 中 | reftest `apply_scripted_dom_mutations` 后取 C 序列化（= 现状 dom_html）；门禁 diff |

---

## 7. 首个落地步骤（L2a-1，implementation-ready）

选 `__zw_matches`（`element.matches(selector)`，幂等只读，无 in-flight 依赖，无 handle 复杂度——selector 输入）作首个路由：

1. `js_dom_bridge/selector_match.rs::element_matches_test_selector` 加 C 读分支：`if ZW_NATIVE_DOM 含 polyfill-live-matches { with_dom(|d| { /* selector→NodeId + query_selector_all 全匹配集判定 */ }) } else { 现 parse_html(html) 路径 }`。
2. A/B 对照门测试：同 (html, elem_sel, test_sel) 两路径结果断言 `==`（覆盖组合器/子树/无匹配）。
3. `make test`（OFF 零回归）+ ON 路径单测。
4. land 后扩到 `closest`/`children`/`query_*_sub`（L2a 余量）。

**L2a-1 风险 🟡 中**（只读、幂等、kill-switch OFF 零回归），是 L2 全链路最低风险的入口切片——验证 C 读管线 + selector→NodeId 解析 + A/B 对照门基建，为 L2b 写路由（🔴 crux）铺路。

---

## 8. 决策门禁说明（为何需用户点名）

L2 触 polyfill 热路径 + render 核心（crux 正面解决）+ 14137 测试依赖，属 rule 11「深结构」。即便每片 kill-switch 默认 OFF 零回归，**L2b-1 改 render 路径**（C live 时跳过 re-parse）是架构级转变（C 升格真源），需用户拍板方向。本文档使该决策基于清晰实施路径（非模糊「大改」）：

- 方向：C 成 single source of truth，cached_html 降为派生。
- 去风险：三阶段默认 OFF kill-switch + A/B 对照门 + 渐进路由。
- 入口：L2a-1（🟡 中，只读幂等）先行验证管线。

**R3177 已确认 L2 增量价值**：polyfill 核心 API（fetch/MO/rAF/getBoundingClientRect）均已真实，L2 价值 = native 性能（~15.6x getter）+ spec 忠实度（去 String 边例差）+ 去 round-trip（大页面 ~30% parse 开销）+ 维护简化（去 A/B 双份），非「补 stub」。

---

## 9. 下一步（待用户点名）

- 用户「启动 L2」→ 执行 L2a-1（首个落地步骤），后续 L2a 余量 → L2b-1（crux）→ L2b-2 → L3。
- 用户「暂不」→ 自主机械面已穷尽，转其他 goal（WPT 兼容性 / 深结构 rendering-compat 等用户点名方向）；L2 设计就绪随时可启。
