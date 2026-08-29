# RFC：P1b L2-d3 统一查询匹配器 + identity 桥（polyfill-live 合一收口设计）

**版本**: v0.4（v0.3→v0.4：d3d-r2 达成裁定 + iframe realm timer 子片——探针普查确认树源前提已消、d3d-r3 两前置均满足——2026-08-29 R356）
**日期**: 2026-08-22（v0.1）/ 2026-08-28（v0.2）/ 2026-08-29（v0.3）
**状态**: Partially-landed（d3a/d3b/d3c/d3d-r1 已落；**d3d-r2 达成**[R356 探针裁定：树源前提已由 R296–R310 收口]；d3d-r3 重启条件满足未启动；d3e 未启动。doc 上下文已收口 = M1 实用收口点[R171]；全量收口按 §6 总装推进）
**父 RFC**: `docs/specs/p1b-v8-native-bindings-rfc.md` §3.7 L2（本文件是 L2 的 JS 侧细化）
**goal**: `docs/goal/js-dom.md` M1（L2 polyfill-live 合一）

---

## 0. 执行摘要

**问题**：shim 的 DOM 元素在 JS 侧存在**四个互不相同的对象域**，同一逻辑节点经不同
API 面返回不同对象——`querySelector` 结果 !== `createElement` 产物 !== 遍历面节点。
这是 R159–R165 实证的三类 fail 簇（`#any-namespace` 族 / `[*|TiTlE]` 第 3 命中 /
duplicate-id identity 分歧 / tree order 断言）的共同根因，也是 L2「查询读 live 树」
按形态逐个 gate 迁移失败（R165 compound 首版 1072P/902F 大回归）的结构性原因。

**方案**：两件事——
1. **identity 桥**（双向映射表）：任何 API 面首次暴露一个逻辑节点时登记到全局
   `node ↔ exposed-object` 表；此后**所有**面再产出该节点一律返回已登记对象。
   不改任何一面的数据源，只改产物归一——identity 分歧类 fail 从根上消失。
2. **统一匹配器**：把 doc 上下文已验证的「本树查询」（L2-d1 纯 tag →
   `_queryTreeByCompound`）逐步扩展到 element/fragment 上下文与 compound/组合器
   形态，最终全部查询面遍历各自 live 树（`_tree` / `_zwMEl` 子树 / handle
   registry 树——handle 面已有完整先例 `_handleSubtreeNodes`+`_matchComplexAgainst`）。

**切片序**（每片独立 land、全量双路径 WPT net≥0）：
d3a 桥基建登记（零行为变化）→ d3b 查询产物归一 → d3c doc compound gate →
d3d element/fragment 本树化 → d3e 组合器/伪类本树化。

**边界**：本 RFC 只动 JS shim 内部（part02/03/05 的查询产物层），不动 host 回调
协议（`__zw_*` wire 不变）、不动 native 路径、不删 JSON 往返（作为不支持形态的
回落永续保留）。R43/R77/R165 三次「最小切片绕不开完整方案」的教训在此收口：
完整方案 = 本 RFC，但它自身仍按可独立 land 的切片分解。

---

## 1. 背景：四对象域与实证证据

### 1.1 对象域盘点（2026-08-22 代码核实）

| 域 | 工厂/形态 | 产生面 | 身份载体 |
|----|-----------|--------|----------|
| **A. sel proxy** | `_wrapSelector(sel)`（part05） | `document.querySelector` 等 host 快照命中 | 唯一选择器串（host DOM 元素） |
| **B. handle proxy** | `_makeProxy`（part04，`_proxyCache['@'+handle]`） | `createElement`/`createDocumentFragment`/shadow root | handle 串（host arena 节点） |
| **C. mutTree 节点** | `_zwMEl(snap, parent)`（part03） | detached doc 树、cloneNode(deep)、`_zwMBuildBodyTree` | JS 对象本体（childNodes/parentNode 真链接） |
| **D. 解析 wrapper** | `_zwParseEl(info)`（part02） | `__zw_parse_html_query` JSON 往返产物 | tag+id+outer 快照键（只读） |

同域内部已有 identity 缓存（B：`_proxyCache`；C：对象本体；D：`_zwQWrapCache`/
`_zwMWrapMap`/fragment `_zwQWrapMap` 三条 per-root 键缓存）。**跨域**只有三个局部
桥接点：R100 `_zwQueryWrapIdentity`（sel→handle 反查，限 query 返回点）、R163
fragment 真实节点优先（D 产物按键匹配回 C 本体）、R164 doc 纯 tag 本树直出 C。

### 1.2 实证失败（为什么逐形态 gate 走不通）

| 轮次 | 尝试 | 结果 | 根因 |
|------|------|------|------|
| R159 | `#any-namespace` 查询 | fail | setup 面 appendChild 落 C 域 per-element 树，doc 级查询树（A/D 源）**互不合并**——内容都不同，不止 identity |
| R163 | fragment QSA 真实节点优先 | +1P land | 键匹配（tag+id+outer）可闭合 fragment 单面，但 probe 实证查询源含 `<head>` 内容返 309 vs 遍历面 1——**查询源与活树内容不同** |
| R165 | doc+element+fragment compound gate | 1072P/902F | doc 上下文正常（docQ:1），element/fragment 上下文消费面依赖 D 域 wrapper 语义，直出 C 域真实节点破坏断言 |
| R165 | 收窄纯 `#id` | 1936P/38F（-1） | `#id-li-duplicate`：duplicate-id 树首命中（C 域对象）与 JSON 往返首命中（D 域对象）在用例变量里共存 → identity 断言失败 |

**结论**（R165 已记 master.md）：compound 迁移不能按形态逐个 gate；需要统一
匹配器 + identity 桥。本 RFC 把该结论落成可执行设计。

---

## 2. 设计

### 2.1 identity 桥（`_zwNodeBridge`）

**数据结构**（part03 模块级，全 shim 单例）：

```
_zwNodeBridge = {
  map: Map<node, object>,      // C 域真实节点 → 已暴露对象（任意域）
  rev: Map<object, node>,      // 反向（暴露对象可能是 wrapper/proxy，不能挂 own slot——
                               //   proxy 的 get trap 拦截属性写；WeakMap 不可枚举遍历，
                               //   用双向 Map + 代际清理）
  gen: 0                       // 代际（innerHTML setter / tree rebuild 时 clear）
}
```

**API**（三函数，全部幂等）：

```
_zwBridgeGet(node)        → 已暴露对象 | undefined
_zwBridgeSet(node, obj)   → 登记并返 obj（同 node 重复登记：首登记者胜，后到者被
                            _zwBridgeGet 归一——不覆盖，保证「首次暴露」语义稳定）
_zwBridgeNodeOf(obj)      → 反查真实节点 | undefined（wrapper 消费面升级用，d3e）
```

**登记点**（d3a，零行为变化——各面仍返回自己的产物，只同步登记）：

| 面 | 登记处 | node 来源 |
|----|--------|-----------|
| doc 本树查询（queryBody L2-d1 直出） | part03 queryBody 直出分支 | 树节点（C） |
| `_zwMWrapCached`（Element.prototype 查询） | part03 缓存命中/新建两分支 | **无真节点**——JSON 往返产物无 C 域对应，登记跳过（见 2.3 限制） |
| fragment QSA（R163 真实节点优先） | part03 real163 命中分支 | 子树节点（C） |
| handle proxy 暴露（`_wrapHandle`） | part05 `_proxyCache` 命中分支 | **无 C 域节点**——d3a 不登记（d3d 经 `_handleSubtreeNodes` 建联系） |
| `_zwQueryWrapIdentity`（R100 sel→handle） | part05 命中分支 | 同上 |

**归一点**（d3b）：三条查询面（queryBody / `_zwMQueryAll`→`_zwMWrapCached` /
fragment QSA）在返回前对**可定位真实节点**的产物调 `_zwBridgeGet` 命中即返已
登记对象。D 域 wrapper（无真节点）不参与归一——它们只服务无树上下文的只读面。

**失效**：与 R158 `_zwQWrapCache` 同款代际钩子（`_zwQWrapBump` 已有 set 侧）+
桥自身在 bump 时 `map.clear()`。树重建（innerHTML setter）后旧节点全部失联，
桥清空防泄漏与 stale。

### 2.2 统一匹配器（形态门 + 本树遍历）

**已存在**（无需新写，d3c–d3e 只是接线）：

- `_queryTreeByCompound(comp, all)`（part03，R165）：compound 节点局部匹配
  （tag/`*`/`#id`/`.class`×n/`[attr]`/`[attr="v"]`，class 空白分词，attr 仅 `=`；
  其余运算符显式回落）。守卫：outerHTML 为空的元素中止 → 整体回落 JSON。
- `_parseCompoundOf` / `_parseSelectorListOf` / `_matchComplexAgainst` /
  `_handleSubtreeNodes`（part05，R2928+）：handle 面的**完整**选择器引擎——
  四组合器（` `/`>`/`+`/`~` 回溯匹配）+ 结构伪类白名单 + nodeInfo（ancestors/
  parent/prevSiblings 树上下文）。**这是统一匹配器的目标形态**：handle 域查询
  已完全本树化，从不开 JSON 往返。

**目标接线**：

```
查询入口（任意上下文）
  → _zwQueryGuard（非法选择器 SyntaxError，已有）
  → 形态门：纯 tag / compound（无组合器无伪类）
      → 本树遍历（根 = 调用者：doc._tree / _zwMEl 自身 / fragment / handle registry）
      → 产物经 identity 桥归一
  → 其余形态（组合器/伪类/attr 其它运算符/转义边缘）
      → 上下文有本树 → part05 匹配器对 nodeInfo 树求值（d3e）
      → 否则/守卫中止 → 旧 JSON 往返（永续回落）
```

### 2.3 明确不做 / 已知限制

- **A 域（sel proxy）与 B 域（handle proxy）不迁 C 域**：它们是 host 权威节点的
  视图（sel/handle 持久于 host arena），强行本树化 = 重造 host 同步协议（正是
  R133 评估否决的方向）。桥只做**产物归一**（同节点同对象），不做域合并。
- **`_zwMWrapCached` 的 JSON 产物无真节点可登记**：element 上下文查询在 d3d
  本树化后才有真节点；d3b 阶段该面行为不变。
- **duplicate-id 首命中语义**：桥的「首次暴露者胜」使跨面一致，但**哪面先执行**
  决定用例拿到哪个对象。WPT `#id-li-duplicate` 断言的是 `querySelector` 与
  `querySelectorAll[0]` 同对象（R158 已保证同面一致）；d3b 后跨面也一致。
  若仍有 fail（用例断言特定域对象），按 fail 实证再定首暴露优先级——不预设。
- **quickjs 路径**：shim 双引擎共用，本 RFC 全部 JS 侧改动对 v8/quickjs 同源
  生效；验证跑双 feature（既有惯例）。

---

### 2.4 v0.2 增补：落地后新证据（R167–R171 / R331 / R338）

1. **key 双形态**（R170）：`_zwWrapCached` 缓存键原来只认 JSON info 的 `.tag`/`.outer`；
   gate 直出真实节点后 `.tagName`/`.outerHTML` 形态使 tag 段空键撞车返空壳。归一缓存的
   键构造必须双形态兼容（`.tag||.tagName`、`.outer||.outerHTML`）——任何后续本树化
   切片的**前置检查项**。
2. **iframe doc 双工厂**（R169）：src-iframe 的树/查询走 part05 iframe 工厂，
   `_makeDetachedDocument` 的 bodyHtml 对 iframe 恒空——树源统一是 element/fragment
   本树化（d3d 重启）的前置项，单变量实验曾误判其为伪根因（srcdoc 无 src 分支）。
3. **d3d 负结果**（R171）：element 上下文 compound 全形态本树前置 +2F/0 改善、收缩
   纯 tag 持平互换——R165 的「902F wrapper 依赖」在 key 修复后**部分幸存**（element
   消费面对产物形态敏感度高于 doc）。`:enabled` 形态 `querySelector !== QSA[0]` 的
   归一缓存键命中不同步是 +2 机制。**重启前置 = 产物归一路径统一**。
4. **R331 identity 反查去重联动**：QSA 归并域（pending handle 子树并入 host 结果）
   的 identity 双源已由 `_zwQueryWrapIdentity`（R100 反查）+ 归并前置反查去重收口
   （vue_reconciliation lis:A,B,A,B 两次回归教训）——本 RFC 的桥（C 域真实节点 ↔
   暴露对象）与 R100 反查（sel ↔ handle）是**两个正交面**：前者归一 C/D 域产物，
   后者归一 A/B 域产物。d3b 的 `_zwMFindRealNode` 与 R100 反查在查询返回点的组合
   已覆盖「C 域节点经 host 快照命中」形态。
5. **QSA 同 turn 返空 = R309 刻意取舍**（R338）：普通 append（Vue mount 形态）域
   的 QSA 维持 host 快照语义以规避 identity 双源双计。该域的 identity 统一**不属
   本 RFC d3d**（R309 注记「基底仍有效且 overlay 有 identity 双源风险，维持 host
   快照语义」），归 L2 主线（polyfill-live 合一 M1 完整方案）——本 RFC 只收
   C/D 域（detached/fragment/clone 树）的查询面。
6. **现状锚定测试**：`test_append_domain_identity_baseline_r338`（part24）锚定
   append/querySelector/childNodes 三路 identity 一致 + QSA host 语义；L2 主线
   落地后更新断言。

## 3. 切片计划（每片独立 land）

| 片 | 内容 | 预期 | 验证门 | 状态 |
|----|------|------|--------|------|
| **d3a** | 桥基建：`_zwNodeBridge` 三函数 + 三个既有真实节点直出点登记（queryBody 直出 / fragment real163 / cloneNode 产物） | **零行为变化**（纯登记） | 全量双路径逐计数一致 + zero-engine 全绿 | ✅ **已落**（R166 `8d40bb957`：`_zwNodeBridgeMap` part03 模块级 + 三登记点 + 5 处树代际 bump 清桥 + part22 三单测） |
| **d3b** | 查询产物归一：三查询面返回前 `_zwBridgeGet` | `#id-li-duplicate` 类跨面 identity fail 收口（±数 P，net≥0 才 land） | 全量双路径 net≥0 + ParentNode 文件级 | ✅ **已落**（R167 `5258ab632`：`_zwMFindRealNode` 键索引归一 + matches 重写 + `_zwMEl.dispatchEvent` 三阶段链派发；d3b2 bubbles 残留 → R168 `9d62b27fa` root-hit 特判收口） |
| **d3c** | doc 上下文 compound gate：queryBody 形态门扩到 `_queryTreeByCompound` 全形态（R165 已实测 doc 上下文无回归——`docQ:1` 保持） | doc 上下文 compound 消 JSON 往返 | 同上 | ✅ **已落**（R170 `0a4146465`：`_zwWrapCached` key 双形态兼容 + 组合器守卫；R169 先行修复 `[attr]` 存在性匹配） |
| **d3d** | element/fragment 上下文本树化：`_zwMQueryAll`/fragment QSA 对纯 tag+compound 形态以调用元素为根遍历 `_zwMEl` 子树（复用 `_queryTreeByCompound` 语义 + 守卫），产物经桥；R165 的 902F 回归面在此被桥消解（真实节点 = 已登记对象） | element/fragment 查询读活树（R163「查询源与活树内容不同」收口） | 全量双路径 net≥0 + Element-matches 文件级 | ⏸️ **回退定格**（R171 `8a825479b`：两轮实验 0 subtest 改善、+2F identity 时序——element 消费面对产物形态敏感度高于 doc；**重启前置 = 产物归一路径统一**；普通 append 域 QSA 同 turn 语义 = R309 刻意取舍[R338 评估]不属本片） |
| **d3e** | 组合器/伪类本树化：element/fragment/doc 上下文的组合器形态改走 part05 `_matchComplexAgainst`（nodeInfo 从各自树构——复用 `_handleSubtreeNodes` 的 DFS+info 构造，抽公共 helper）；attr 其它运算符同步（`_matchAttrOf` 已有） | 查询面 JSON 往返只剩守卫中止回落 | 同上 + traversal 文件级（walker 与查询序一致性） | ⏸️ 未启动（依赖 d3d；保留共用解析器 `_zwParseCompoundSel` 已提取[R171]） |

每片统一验证序（既有惯例）：
1. `make test`（test-guard 包裹）
2. 全量 dom WPT 双路径（polyfill + `ZW_NATIVE_DOM=1`）逐计数对比，net≥0 才 land
3. `cargo test -p zero-engine`（2310 基线）+ fmt + clippy（含 quickjs 矩阵）
4. 单测：每片带 identity 断言（同节点跨面 `===`）+ 形态门判定单测

**回滚**：每片一个 commit；任何一片 net<0 即 revert 该片（切片间无隐藏依赖——
d3b 依赖 d3a 的登记点，d3d 依赖 d3b 的归一，其余正交）。

---

## 4. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 桥 Map 泄漏（长页面大量节点暴露） | 代际清理（bump 全清）+ 上限 4096 超限全清（同 R158 的 512 上限语义，放大因桥跨 root） |
| proxy 对象作 Map key 的 trap 副作用 | 登记点全部在产物**返回前**的 plain 流程；proxy 仅作 value 不作 key（key 恒为 C 域 plain 节点） |
| 首暴露语义改变现有用例期望对象 | d3b 单独 land + ParentNode/Element-matches 文件级对比；fail 实证再调 |
| element 本树化漏守卫（R164 三守卫：容器例外/空 outer 中止/零命中落 JSON） | `_queryTreeByCompound` 已内建守卫；d3d 复用不重写；新增守卫先 probe 实证 |
| 双引擎行为漂移 | 全部改动在共用 shim；验证双 feature 双路径 |

---

## 5. 与父 RFC / goal 的关系

- 父 RFC §3.7 L2 定义为「polyfill 桥 `__zw_*` 回调改读共享 live Document C」——
  host 侧（R102/R103 `with_query_doc_live_aware` 已做）。本 RFC 是 **JS 侧对偶**：
  shim 内部查询面从「序列化→host re-parse→JSON 往返」改「读 JS 侧 live 树」。
  两翼合拢后 L2 完整收口（M1 完成）。
- 不触发 Mission 级单向门（不涉 default-on / kill-switch / 删 polyfill）。
- M6（QuickJS native）已在 R75 全量收口——本 RFC 不涉。

---

## 6. v0.3 增补：L2 深水区专项总装（R350–R353 性能证据 + d3 重启路线，2026-08-29 R354）

> **背景**：R350–R353 四轮对 `Range-mutations` data 族超时的性能归因（evidence/
> 2026-08-29-r350…r353）把四类「L2 消除对象」的成本量化落地，与 §3 的 d3 重启前置
> 合并后，L2 主线的重启路线首次完整。本节是执行总装：**立项证据 → 切片分解 →
> 验证门**，供后续每轮直接按片领取。

### 6.1 立项证据（全部量化，2026-08-29 实测）

| # | 成本对象 | 量化 | 来源 | L2 消除机制 |
|---|----------|------|------|-------------|
| E1 | live-range adjust 扫描的每条目键读/根比较 | trap 键读 78µs/读（R98 分支）；根 walk 每跳 ~5µs；已修后仍 0.35ms/条（形态依赖） | R350/R351/R353 | live doc 直读：注册表语义被原生路径替代，扫描整个消失 |
| E2 | `cont.parentNode` host 往返 | 每跳 ~5µs，walk 3 跳 ~15µs；textEl 键匹配分支每条目触发 | R350 W21/W15 | live doc：parentNode 为 plain 字段 |
| E3 | R98 分支（每字符串属性读） | getPrototypeOf + CE 检查 78µs/读；内部键已顶部短路[R351]，页面属性读仍付 | R351 W8/W9 | live doc 域统一后 proxy 产物面收缩 |
| E4 | 游离树堆积下的查询/文档生命周期 | setupRangeTests 每轮 querySelector+removeChild，树堆积后 qs+rm 32ms/iter | R352/R353 W2/W4 | live doc：查询直读，无游离面（真浏览器等价：游离节点 GC） |

**共同根因**：polyfill 桥的 JS↔host 双源架构（快照 re-parse + JSON 往返 + proxy trap
面）。这正是父 RFC §3.7 L2 的定义——两翼（host 侧已收口[R102/R103]；JS 侧 = 本 RFC）
合拢后 E1–E4 全部消除。

### 6.2 切片总装（含新增性能片；每片独立 land）

**路线 A——d3 重启（identity 维度，前置已明确）**：

| 片 | 内容 | 前置 | 验证门 |
|----|------|------|--------|
| **d3d-r1 产物归一路径统一** | element/fragment 查询的**归一缓存键构造**统一为单一 helper（消 R171 的 `:enabled` +2 时序机制——两路径键命中不同步的根因）；key 双形态兼容[§2.4-1]纳入 helper | 无（纯重构，行为等价） | 全量双路径逐计数一致 + Element-matches 文件级 |
| **d3d-r2 iframe 树源统一** | part05 iframe 工厂与 `_makeDetachedDocument` 的 bodyHtml 空态收口[§2.4-2]（src-iframe 树/查询单一来源） | d3d-r1 | 同上 + case/createElementNS 文件级 | ✅ **达成（R356 裁定：探针普查确认树源分裂主体已由 R296–R310 历史切片收口——iframe doc 经 `doc.body.innerHTML` 落 detached-doc 工厂同源管线，compound/组合器/identity/traverse 探针全绿；原 R169「srcdoc 0 命中」前提不复现。附带子片：iframe realm timer 面四件转发 land）** |
| **d3d-r3 element/fragment 本树化重启** | `_zwMQueryAll`/fragment QSA 纯 tag+compound 以调用元素为根遍历（复用 `_queryTreeByCompound` + 守卫），产物经桥 | d3d-r1 + d3d-r2 | 全量双路径 net≥0 + Element-matches/`#id-li-duplicate` 文件级 |
| **d3e 组合器本树化** | 组合器形态走 part05 `_matchComplexAgainst`（nodeInfo 从各自树构）；attr 其它运算符同步 | d3d-r3 | 同上 + traversal 文件级 |

**路线 B——性能片（E1–E4 消除；依赖路线 A 的 live doc 直读面）**：

| 片 | 内容 | 目标（量化） | 验证门 |
|----|------|--------------|--------|
| **P1 range adjust 退化** | live doc 直读后，R260/R262/R263 的注册表扫描改为**活节点直接遍历**（`doc.live_ranges` per-node 表——真浏览器等价形态），删除跨树键比对 | dataChange/replaceData 尾部 0.35ms/条 → O(节点 range 数)；五文件全绿 | data 族 declared 100% + 全量双路径 net≥0 |
| **P2 游离堆积查询** | live doc 查询天然不扫游离子树；验证 setupRangeTests 形态 qs+rm 段恒定 | qs+rm 段 32ms/iter → 恒定 <2ms | data 族文件级 + W2 探针复测 |
| **P3 trap 面收缩** | live doc 域统一后，页面属性读的 R98 分支触发面统计复测 | E3 78µs/读的触发频次下降比 | 属性读微基准 + sweep |

**顺序**：A 先（identity 维度是 d3d 历史失败的根因），B 随 A 的面自然接续；P1 依赖
live doc 基建（属 M1 完整方案的 host+JS 两翼合拢），非独立可达——在 A 完成前 P1–P3
挂起，不设轮次预期。

### 6.3 每片统一验证序（继承 §3）

`make test`（test-guard 包裹）→ 全量 dom WPT 双路径逐计数（net≥0）→ `cargo test -p
zero-engine` + fmt + clippy（含 quickjs 矩阵）→ 单测带 identity/形态门断言。**回滚**：
每片一 commit，net<0 即 revert。

### 6.4 明确不属本 RFC（边界重申）

- 普通 append 域的 QSA 同 turn 语义（R309 刻意取舍[R338]）——归 M1 完整方案，
  d3d-r3 不得触碰该域判定。
- dataChange/replaceData 尾部的 P1 前置（live doc 基建）在路线 A 完成前**不再单独立
  轮次切片**（R353 定案：WPT mega-case 特化形态，非真实页面瓶颈）。
- host 侧（`with_query_doc_live_aware` 等）已收口[R102/R103]，本 RFC 全部改动仍限
  JS shim 内部。

---

## 7. 修订历史

| 版本 | 日期 | 内容 |
|------|------|------|
| v0.1 | 2026-08-22 | R165 结论成文化：四对象域盘点 + identity 桥设计 + d3a–d3e 切片计划（js-dom R166） |
| v0.2 | 2026-08-28 | d3a–d3c 落地状态回填（R166/R167/R168/R170）+ d3d 负结果定格（R171：element 上下文回退、重启前置 = 归一路径统一）+ §2.4 落地后新证据（key 双形态/iframe 双工厂/R331 反查正交面/R309 QSA 域边界/R338 现状锚定）（js-dom R341） |
| v0.3 | 2026-08-29 | **L2 深水区专项总装（§6 新增）**：R350–R353 四轮性能证据入册（E1 adjust 扫描 0.35ms/条形态依赖 / E2 parentNode host 往返 / E3 R98 78µs / E4 游离堆积查询）→ 切片总装（路线 A = d3d-r1 归一路径统一 → d3d-r2 iframe 树源 → d3d-r3 本树化重启 → d3e；路线 B = P1–P3 性能片随 A 接续）；边界重申（R309 QSA 域/dataChange 尾部不再独立切片）（js-dom R354） |
| v0.4 | 2026-08-29 | **d3d-r2 达成裁定 + timer 子片（§3 表状态回填）**：R356 开工探针普查（12 个 WPT 同构临时 fixture）实证 iframe 工厂域 compound/组合器/identity/traverse 全绿——R169 的「srcdoc 0 命中 + bodyHtml:0」前提已被 R296–R310 历史切片消解，d3d-r2「树源统一」目标判定达成（范围重裁定）；附带 land iframe realm timer 面四件转发（`_zwMakeIframeWin` → part01 记录式 stub）。**d3d-r3 两前置（d3d-r1 + d3d-r2）均满足**（js-dom R356） |
