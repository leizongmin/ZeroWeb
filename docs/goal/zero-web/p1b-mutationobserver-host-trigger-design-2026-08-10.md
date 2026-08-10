# P1b 设计片：MutationObserver host 侧触发（escape-hatch 必要伴随）

**日期**：2026-08-10
**状态**：设计片（分析 + 方案 + 切片计划，待用户点名启动实施——与 escape-hatch 收敛绑定，rule 11 决策门禁）
**承接**：R3168（native document 元数据面闭合）下一步候选②；P1b RFC §4 S6（高层 API 改调原生）的前置深化
**工作面**：engine/dom（zero-web 流）

---

## 0. 一句话问题

当前 `new MutationObserver(cb).observe(el, opts)` **仅观测 JS 经 polyfill Proxy trap 驱动的 mutation**；host 侧（native 绑定、host 事件派发、host 发起的 DOM 变更）不触发回调。escape-hatch 收敛（把生产 DOM mutation 路由到 native 绑定）会让**全部** mutation 走 native 路径 → 绕过 Proxy trap → **polyfill MO 全停**。故 MO host 侧触发不是 S6 的可选优化，而是 **escape-hatch 收敛的必要伴随**（否则 escape-hatch 回归 MO——生产核心 API）。

## 1. 当前架构事实（两套并行 MO）

### 1.1 dom 层 MO（`crates/dom/src/mutation.rs` + `document/mod.rs`）

- `MutationObserver { callback: Box<dyn Fn(&[MutationRecord])> }`——**Rust 闭包**回调，非 v8 JS 函数。
- `Document::record_mutation()` 在 `append_child` / `insert_before` / `remove_child` / `replace_child` / `set_attribute` / `remove_attribute` / `toggle_attribute` 等变更方法内 push 到 `pending_mutations: Vec<MutationRecord>`（document/mod.rs:287/328/399/439/787/814）。
- `process_mutations()` / `take_mutation_records()` 排空队列并通知 observers——**engine 从不调用**（全仓 grep `take_mutation_records` / `process_mutations` 在 engine/script-sandbox 零消费者）。
- 结论：dom 层 MO 基建**记录端可用**（host/native/dom mutation 都入队），**通知端死路**（无 JS 可见注册 + 队列永不排空）。对 JS MO 用例而言是 dead infrastructure。

### 1.2 polyfill MO（`crates/engine/src/js_dom_shim/part01.js:742+`）

- `globalThis.MutationObserver` 构造器 + `observe(target, options)` / `disconnect` / `takeRecords`，observers 存 `__zw_mo_observers`（每 observer 持 `_targets[id]` options map + `_targetProxies[id]`）。
- 元素身份 key `_mo_id(handle, sel)`：handle（JS 创建子树，`createElement` 返 `"__n{n}"`）优先，否则 selector（现有 DOM）。
- element Proxy 的 setAttribute/appendChild/textContent= 等 trap 调 `_mo_notify(sel, handle, record)` 排队；`_defer`（microtask）派发回调（spec §4 语义）。
- 已支持：attributes / childList / characterData、subtree（R3026）、attributeFilter + attributeOldValue（R3025）、characterDataOldValue（R3028）。
- **显式限制**（part01.js:749）："仅观测 JS 驱动的 mutation（host 侧 `__zw_dispatch_event` 等不触发）"。

### 1.3 native 绑定（R3095–R3168）

- native `append_child` / `set_attribute` 等经 `with_dom_mut(|d| d.append_child(...))` 直接操作 live Document——**复用 dom 方法**，故这些 mutation **已入 `pending_mutations`**（记录端生效）。
- 但 native mutation 不经 polyfill Proxy trap → **不触发 polyfill MO**。
- native_dom 默认关（kill-switch），故当前生产路径（polyfill）MO 正常，native 路径 MO 不可见。

## 2. 关键洞察：escape-hatch ↔ MO 强耦合

escape-hatch 收敛 = shim 把 `document`/`element` 工厂路由到 native（L2 polyfill-live）。一旦路由，生产 DOM mutation 走 native `with_dom_mut` → dom 方法 → `pending_mutations`，**不再经 polyfill Proxy trap**。

后果链：
1. 生产 mutation 全走 native → polyfill Proxy-trap MO 拿不到 mutation（trap 不触发）。
2. dom `pending_mutations` 仍记录，但无 JS 消费者 → 排空死路。
3. **净效果：escape-hatch 后 `new MutationObserver(cb)` 对生产 DOM 变更完全不触发**——MO（React/Vue/jQuery/framework 高频依赖）回归。

故：**escape-hatch 收敛必须同时落地 MO host 侧触发**，否则 escape-hatch 不可发布。两者为同一决策门禁的两个交付物。

## 3. 设计选项

### 方案 A：drain dom 队列 → polyfill MO

native/dom mutation 入 `pending_mutations`（已生效）后，engine 在 microtask 点排空队列，逐条把 `MutationRecord` 投递给 polyfill 的 `_mo_notify`（NodeId→polyfill 身份解析）。

- **优点**：复用 polyfill 成熟 MO（options 过滤、subtree 走查、attributeFilter/oldValue、microtask 派发）。
- **难点**：NodeId→polyfill 身份桥（polyfill 用 handle/selector key，native 用 NodeId）。selector-based 元素可经现有 `selector_from_node` / identity map 反查；handle-based（JS 创建子树）需 handle↔NodeId 映射（R3106 live Document 共享后 handle 与 NodeId 已关联）。
- **风险**：与 polyfill MO 双重通知（JS 经 Proxy 的 mutation 已通知一次，native drain 再通知一次）——需去重（Proxy 路径与 native 路径互斥，escape-hatch 后 Proxy 路径关闭，去重自然成立；escape-hatch 前 native_dom 关闭，无冲突）。

### 方案 B：native MO 子系统

native `new MutationObserver(cb)` 构造器（v8 FunctionTemplate）+ `observe(target, options)` 存 `(NodeId, options, JS callback)` + drain→microtask 按 registrations 过滤 `pending_mutations` 并调 JS callback。

- **优点**：干净，不耦合 polyfill 身份；escape-hatch 后 native-only MO 自然（polyfill MO 随 S7 萎缩）。
- **难点**：重复实现 options 过滤 / subtree 走查 / oldValue 捕获（polyfill 已成熟）；需 microtask 调度 host hook（当前无 `queueMicrotask` host 回调，需新增）。
- **风险**：与 polyfill MO 共存期双系统（native_dom 关时 polyfill，开时 native）——需 feature 分流清晰。

### 方案 C：hybrid（推荐）

native MO 构造器/observe **注册进 polyfill `__zw_mo_observers` 共享注册表**（不另建 native 注册表）；native mutation 经 host hook 回调 polyfill 的 `_mo_notify`（NodeId→身份解析后投递共享注册表）。

- **优点**：单注册表（polyfill + native 共用 observers），单派发逻辑（polyfill `_mo_notify` + `_defer`），identity 桥为唯一新增面。
- **难点**：同方案 A 的 NodeId→polyfill 身份桥。
- **优点 vs A**：A 是「drain 时整批投递」，C 是「native mutation 即时投递共享 notify」（更贴近 polyfill 逐 mutation 排队语义，复用更彻底）。
- **风险**：与 polyfill 路径去重（同 A，escape-hatch 后 Proxy 路径关闭自然成立）。

## 4. 推荐：方案 C（hybrid 共享注册表）

理由：① 单一 MO 注册表 + 派发逻辑（DRY，不重复 polyfill 成熟 options/oldValue/subtree 逻辑）；② identity 桥（NodeId↔handle/selector）为唯一新增面，且 R3106 live Document 共享后 handle↔NodeId 关联已存在；③ 与 escape-hatch 同期落地——native_dom 开启时 native mutation 投递共享 notify，polyfill Proxy 路径随 escape-hatch 关闭，去重自然成立。

## 5. 切片计划（方案 C，每片 kill-switch + 独立 land）

| 切片 | 内容 | 风险 | 验证 |
|------|------|------|------|
| **MO-S1 identity 桥** | host hook `__zw_mo_notify_native(node_id, type, added[], removed[], attr_name, old_value)` + NodeId→selector/handle 反查（复用 R3106 live Document 共享 + selector_from_node）；JS 侧 `_mo_notify` 加 native 入口 | 🟡 中 | 单测：native appendChild 后 polyfill MO 收到 record（native_dom 开分支） |
| **MO-S2 microtask 派发接通** | native mutation 触发后调度 `_defer` 派发（复用 polyfill `_defer`）；spec §4 单 microtask 批派发语义 | 🟡 中 | WPT mutation-observer 子集（host-initiated 变更触发回调） |
| **MO-S3 options 过滤 + subtree + oldValue** | 复用 polyfill `_mo_deliverToId` 的 options 过滤逻辑（attributes/childList/characterData/subtree/attributeFilter/oldValue）——native record 经同一派发路径自动获益 | 🟢 低（复用既有） | WPT mutation-observer options 用例 |
| **MO-S4 escape-hatch 联动验证** | escape-hatch 开启 + native_dom 开 → 生产 mutation 全走 native → MO 经 MO-S1/S2 触发；polyfill Proxy-trap MO 路径关闭去重 | 🔴 高（escape-hatch 主干） | `make product-smoke` + 全量回归 + MO 专项集成测试 |

**前置依赖**：MO-S4 = escape-hatch 收敛（用户点名，rule 11）。MO-S1–S3 可在 escape-hatch 前以 native_dom 开分支独立 land（不影响生产 polyfill MO）。

## 6. 决策门禁

- **MO-S1–S3**（identity 桥 + 派发接通 + options 复用）：native_dom 开分支，生产零影响，可自主 land（低-中风险，escape-hatch 前置基建）。
- **MO-S4 + escape-hatch 收敛**：生产路径主干变更（深结构），**须用户点名**（rule 11）。本设计片为 MO-S4 提供方案就绪度，escape-hatch 决策时 MO 伴随交付。

## 7. 风险

| 风险 | 级别 | 缓解 |
|------|------|------|
| NodeId→polyfill 身份桥不完整（detached JS 子树 handle 无 selector） | 🟡 中 | R3106 live Document 共享后 handle↔NodeId 关联；selector-based 经 identity map；无身份 mutation 丢弃（spec：unobserved target 不通知） |
| 双重通知（polyfill + native 共存期） | 🟡 中 | native_dom 关闭时 native 路径不生效（kill-switch）；escape-hatch 后 Proxy 路径关闭，去重自然成立 |
| microtask 时序（spec §4 单 microtask 批派发） | 🟡 中 | 复用 polyfill `_defer`（已 spec 合规）；native drain 排队同一队列 |
| 性能（每 native mutation 走 identity 解析 + notify） | 🟡 中 | 无 observer 时早退（polyfill `_mo_any_wants_*` 同款 guard）；热路径仅读 observer 注册表 |

## 8. 结论

MO host 侧触发是 escape-hatch 收敛的**必要伴随**（非可选优化）——escape-hatch 把 mutation 路由到 native 会使 polyfill Proxy-trap MO 全停。推荐方案 C（hybrid 共享 polyfill 注册表 + native identity 桥），分 4 切片 land：MO-S1–S3 可 escape-hatch 前以 native_dom 开分支自主 land（前置基建），MO-S4 + escape-hatch 收敛须用户点名（rule 11）。本设计片闭合 R3168 下一步候选②的「宜先 RFC」前置，为 escape-hatch 决策提供 MO 方案就绪度。
