# R307 Evidence — tree-order 三上下文四层根因修复（工厂域 identity 全等 + R300 遗留 r223 归因修复）

**日期**: 2026-08-27
**切片**: M4/L2——R305/R306 的 tree-order 3F 深挖（上轮定性「R291 深结构」被部分推翻）
**改动面**: `part03.js`（四层修复）+ `part21.rs`（r223 对齐 R300 spec 严格化）+ `part24.rs`（+1 回归单测）

## 一、探针方法（完整复刻 WPT 执行序）

上轮遗留的 R307 探针（沙箱完整复刻 `ParentNode-querySelector-All` 的执行序：
setupSpecialElements **含 namespace div 簇**（上轮探针遗漏）→ verifyStaticList 逐上下文
append div → outOfScope 注解 → tree order 断言）逐步定位出**四层叠加根因**：

| # | 根因 | 现象 |
|---|------|------|
| ① | `_zwMFindRealNode` 的 walk167 对**非元素根**（DocumentFragment nodeType 11）直接 return 不递归子 | fragment 的 `_zwNodeIdx` 恒空（探针 `fragIdx=0`）→ fragment 查询全部回落 wrapper 域，idx0 起 identity 全断 |
| ② | 真节点 `outerHTML` 含 R174 的 `data-zw-empty-ns` 内部标记属性 vs host JSON info `.outer` 不含（host `apply_empty_ns_markers` 消费后序列化） | 含 `createElementNS('')` 后代的元素（anyNS 容器等）wrapper 键恒 miss |
| ③ | `_zwMEl` 的 `id`/`className` 是 **plain 数据字段**（无 reflect）——`.id = 'x'` 赋值不进 attrs 数组 | `_zwMSerialize` 丢 id → 查询源序列化无 id → host JSON info `.id` 恒空 → namespace 簇的 res 对象 id 全空（探针 `res=DIV[]` vs `trav=DIV[any-namespace]`） |
| ④ | `_zwMEl`/fragment 的 appendChild/removeChild **不失效**祖先的 `_zwNodeIdx`/`_zwQWrapMap` 查询缓存 | verifyStaticList append 的静态 div 在首查询建的索引里不存在 → 尾部 identity 断 |

**上轮（R305/R306）归因勘误**：R305 称三上下文 = 「R291 wrapper→视图归一深结构」。
本轮探针证明 **createHTMLDocument 工厂域**的三上下文 identity 断在上述四层轻量缺陷——
全部可修（非深结构）；**iframe contentDocument 代理域**（WPT 本体）确实仍卡在
query 产物 = `_wrapSelector` proxy vs traverse 读工厂树的两域割裂（R291 域成立）。

## 二、修复（part03.js 四件 + part21.rs 一件）

1. **walk167 非元素根递归**（`_zwMFindRealNode`）：根 nodeType 11/9 时跳过自身登记但递归 childNodes（元素根维持 descendants-only）。
2. **键 empty-ns 标记归一**：`_zwMFindRealNode` walk 键、`_zwMWrapCached` 消费键、fragment 查询键三处统一 `split(' data-zw-empty-ns=""').join('')` 剥离（序列化源不动——host 消费面零变化）。
3. **id/className IDL accessor 化**（`_zwMEl` 出口）：getter 读闭包槽，setter 同步 attrs 数组（spec `reflect id/class content attribute`）；`_zwMReflectIdl` 回写改经内部槽 `_zwIdlSet` 直写（绕过 setter 反向复活已删属性——`removeAttribute('class')` 后 `hasAttribute` 恒 true 的回归，测试实证修复）。
4. **祖先查询索引失效**：`_zwMEl` 的 appendChild/removeChild + fragment 的 appendChild，沿父链向上 `_zwNodeIdx = null` + `_zwQWrapMap.clear()`（64 层守卫）。
5. **r223 对齐 R300**（part21.rs）：R300 起 spec pre-insert 步骤 6 严格化（doc 已有元素子再插元素抛 HierarchyRequestError），r223 场景②的宽松前提失效——测试先摘 docEl 再插（bisect 归因：8f7fd8704 ok → cbce3865a FAILED，R300 引入的**既存**红灯，master.md R300 轮的「engine 2438 绿」记录与 main 实态不符——该轮 land 后 r223 已红）。

## 三、验证

| 套件 | 基线（main） | R307 | Δ |
|---|---|---|---|
| engine 单测（--lib） | 2443 + 1 红（r223，R300 遗留） | **2444 全绿** | +1 修红 +1 新增 |
| ParentNode-querySelector 全族（WPT） | hash e6ed8d9d | 同 | **恒等**（零回归零增益） |
| Element-matches（WPT） | 139F（main 既存） | 139F | 恒等（非本轮引入；R306 evidence 表记「675P/0F」与 main 实态不符——那是 R292 时代的数字，本族 iframe fragment matches 簇在后续某轮回归，待独立归因） |
| Node-cloneNode / Document-createElementNS | hash 恒等 | 同 | 恒等 |
| make test | 1F 环境项（XOpenDisplayFailed） | 同 | 持平 |
| fmt / clippy（-p zero-engine） | — | 干净 | — |

**WPT tree-order 3F 不变的原因**：WPT 本体在 iframe contentDocument 域——查询经
part04 proxy 的 `__zw_query_all_sub`/`_handleQueryAll` 返 `_wrapSelector` proxy，
traverse 读工厂树——两域对象割裂（R291 域，本轮探针实证 div=1/0/306 与上轮一致）。
本轮四层修复收益面 = createHTMLDocument 工厂域（单测覆盖 + 后续 R291 桥的消费基础）。

## 四、R291 桥的 blast radius 重估（R308+ 候选）

R305/R306 记的「R171 902F 依赖」是 R171 时代（element 上下文本树前置）的回退理由。
本轮四层修复后，工厂域三上下文已 identity 全等——**R291 桥（proxy 查询产物 → 工厂树
归一）的剩余工作收窄为 part04 `_wrapSelector`/`_handleQueryAll` 产物经
`_zwMFindRealNode` 键匹配归一到工厂真节点**。候选切片：
- part04 proxy 的 querySelectorAll 分支：产物包装前先查 `_zwBridgeGet(_zwMFindRealNode(工厂根, key))`；
- 工厂根从 proxy 的融合视图（firstChild 链上行）可达——R307 的④失效机制保证索引新鲜。

## 五、教训

1. **复刻要完整**：上轮探针漏了 namespace div 簇（setupSpecialElements 的一半），
   把根因③④（id reflect / 索引失效）整个漏掉——「复刻 WPT 执行序」须逐函数对齐。
2. **归因要分层**：「深结构」结论前先把可修的轻量层剥掉——本轮四层全是轻量修复，
   剥完后真正的深结构面（两域割裂）才暴露干净。
3. **master.md 的「全绿」记录要当轮核实**：R300 轮记录「engine 2438 绿」但其 commit
   本身把 r223 干红了（下轮才暴露）——land 前跑全量单测（--lib 全跑，非单测过滤）
   是唯一可靠凭据。
