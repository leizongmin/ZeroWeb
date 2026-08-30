# R379 — pa1 移除标记生命周期审计（pending-apply RFC 首片，零行为探针轮）

**日期**: 2026-08-30
**切片**: pending-apply RFC（`docs/specs/js-dom-pending-apply-lifecycle-rfc.md`）pa1——
`_zwRemovedSels`/`_zwRemovedHandles` 全写点/清除点枚举 + 双身份矩阵 + R377「清除点」定位
**改动面**: 零（探针插桩临时注入，审计后 `git checkout` 全量恢复，工作树零残留）

---

## 1. 审计方法

1. **静态枚举**：grep 全 shim（part01–part07）的 `_zwMarkRemoved`/`_zwUnmarkRemoved`/
   `_zwMarkRemovedHandle`/`_zwUnmarkRemovedHandle`/`_zwIsRemoved(Node)`/`_zwRemovedSels`/
   `_zwRemovedHandles` 全部出现点，逐点读上下文归类。
2. **动态探针**：临时插桩四个 mark/unmark 函数注入 `__zwPa1Trace` 调用链 + 表 dump
   钩子，跑 `remove-next-sibling-during-replace-with`（探针变体页面，forced-fail
   带出 trace），三轮递进定位（mark/unmark 调用链 → 表状态 → R377 钩子内部状态）。
   探针经 `--wpt-data /tmp/pa1-wpt` 隔离跑，不污染仓库 wpt-data。

## 2. 双身份标记矩阵（静态，全部站点）

### 2.1 sel 维度（`_zwRemovedSels`）

| 站点 | 位置 | 操作 | 语境 |
|------|------|------|------|
| M1 | part04.js:4252 | mark | removeChild sel 分支（`__zw_remove(sel)` 后） |
| M2 | part04.js:4917 | mark | replaceChild sel 分支（移除 oldChild） |
| M3 | part04.js:5038 | mark | `remove()` else 分支（无 handle 的 sel 元素） |
| M4 | part04.js:6842 | mark | outerHTML setter sel 路径（自身替换移除） |
| M5 | part06.js:2880 | mark | `document.removeChild`/document 级移除 |
| M6 | part04.js:5119 | **缺失** | **`replaceWith` sel 路径只调 `__zw_remove(sel)`，无 mark（见 §4）** |
| U1 | part04.js:3816–3817 | unmark | appendChild 尾部（container sel + child sel） |
| U2 | part04.js:4003–4008 | unmark×2 | appendChild fragment 展开的 ceAdded 循环（handle+sel 双维度） |
| U3 | part04.js:4662–4668 | unmark×2 | insertBefore 的 ceAdded 循环（handle+sel 双维度） |

### 2.2 handle 维度（`_zwRemovedHandles`）

| 站点 | 位置 | 操作 | 语境 |
|------|------|------|------|
| H1 | part04.js:4164 | mark | removeChild handle 分支 |
| H2 | part04.js:5000 | mark | `remove()` 的 handle 分支 |
| H3 | part04.js:5051 | mark | `remove()` 元素路径 handle 存在时 |
| H4 | part04.js:6861 | mark | outerHTML setter handle 路径 |
| H5 | part04.js:5810±（`_zwRemovedHandles` 消费） | — | part03:6795/6850、part05:1446 均为 **unmark**（工厂域 appendChild/insertBefore 入树清除） |
| — | part03.js:6795、6850；part05.js:1446 | unmark | **只清 handle 维度，无对应 sel unmark**（工厂域 sel 身份节点回插不清 sel 标记——当前工厂域节点多为 handle-only，未暴露，但属矩阵不对称点） |

### 2.3 消费面（读点）

| 读点 | 位置 | 语义 |
|------|------|------|
| `_zwIsRemoved(sel)` | part03.js:5429 | `_parentNodeFor` 开头——标记中节点的 parentNode 立即返 null |
| radio group 校验 | part03.js:3016 | required radio 组遍历跳过已移除节点 |
| `_zwIsRemovedNode(node)` | part03.js:5436–5451 | 自身 sel/handle 双查 + 沿 parentNode 上行 64 跳查祖先标记 |
| part06.js:494–496、620 | 迭代器 order 扫描 | NodeIterator/TreeWalker 跳过已移除节点 |
| part04.js:4058、4088 | closest/matches 类查询门 | 移除节点不参与 |
| R352 `_zwDeadContainer352` | part03.js:7117–7147 | live-range 注册表扫描的 removed 容器快道（自身+反链上行查表） |

### 2.4 与快照换代（`__zw_reset_pending_state`）的关系

- 挂钩位置：`apps/browser/src/tab_js_worker.rs:367`、`apps/renderer/src/js_worker.rs:504`
  （两处 `SetDomSnapshot` 后）。
- 钩子体（part05.js:7608）清：live 集合/pending added/removed/桶/id 覆盖表/child+sibling
  基底缓存——**不清移除标记**。
- RFC §1.1「移除标记不在清桶范围、无快照换代绑定」**属实**。
- 另一空白：`execute_script` 每次尾部 `apply_pending_shared_mutations`（webview.rs:2134）
  **不触发任何 shim 换代钩子**——host apply 完成对 shim 完全无感（pa2 要补的正是这条
  「apply 完成 → 补偿状态失效」回调链；SetDomSnapshot 只覆盖导航/快照整体替换形态）。

## 3. 动态探针发现（R377 §1.2 勘误）

探针三轮（trace → 表 dump → R377 钩子内部状态），复现流程
`target.replaceWith(template.content.cloneNode(true))` → `container.querySelector('script').remove()`：

1. **`_zwRemovedSels` 从未被写入**——全程 `sels=[]`。R377 RFC §1.2 的
   「`_zwRemovedSels` 在读时已空——标记被中间环节清除，清除点未定位」**前提不成立**：
   不存在「清除点」，replaceWith sel 路径根本不打 sel 标记（M6 缺失），b 也不是
   sel 身份（见 3）。
2. **b 的 remove 走 handle 分支且被 unmark**——trace 序列：
   `unmark-handle:__n1/__n2/__n3`（replaceWith fragment 展开三个新 handle 入树，
   part05 `_insertAdjacentVariadic` → R321 路径触发 part04:3990 U2 循环——`span×2`
   + `script` 都被 unmark，其中 script `__n2` 本无标记）→ `mark-handle:__n2`
   （`querySelector('script')` 返回 handle proxy [part06:1430 `_zwQueryWrapIdentity`],
   `script.remove()` 走 part04:5000 H2 分支正确挂 handle 标记）。
3. **R377 插入期脚本钩子在本流程 no-op**——`ran=[]` 全程空、`script-ran` trace 从未
   出现。钩子被调用（`r377-called:__n2 tag=SCRIPT`）但 **`_handleChildren['__n2']`
   为空**（`r377-kids:(empty)`）：fragment 展开把 script 入树时只推 host wire
   （`__zw_insert_adjacent_element`）+ 反链，**script 的 registry 文本子未入
   `_handleChildren`**（template clone 产物的 text 子挂 fragment 临时 registry，
   展开路径未随迁）→ 源码收集为空 → eval 不发生 → b 从未被 JS 移除。
4. ** Fail 实际形态**：`container.innerHTML` 返 host 快照旧树
   `<div id="target"></div><b></b>`（replaceWith 的 host 侧 mutation 已正确 enqueue，
   innerHTML 读的是 `__zw_get_inner_html(sel)` 快照层——pending 桶非空时未做融合
   序列化）。期望 `<span>New </span><span>content</span>`。
   **R377 evidence §2 的「script 内容全局执行探针实证 b 进 pending-removed」
   与本轮探针矛盾**——按本轮 trace，b 未被移除（handle 标记有挂但那是探针页面里
   `container.querySelector('script').remove()` 作用的 script 自身……不，是 b 的
   remove 从未发生：script 未执行）。R377 的「实证」应为探针页面变体差异或误读，
   以本轮带内部状态的 trace 为准。

## 4. 矩阵结论 → pa2/pa3 设计输入

1. **M6 缺失（replaceWith sel 路径无 mark）**：与 removeChild(M1)/replaceChild(M2)/
   remove(M3) 不对称。spec 语义 remove-then-insert 两步，replaceWith 的「remove 自身」
   步骤应与 remove() 同语义（同步视图 parentNode null、查询门剔除）。当前 host
   `__zw_remove` 已 enqueue 但同步视图无标记——依赖 R371 重键 + pending 桶补偿，
   两套机制半覆盖（本轮 Fail 即其缝隙）。
2. **清除点不对称**：U1/U2/U3 全在 part04 append/insert 路径；part03:6795/6850、
   part05:1446 工厂域只清 handle 维度。R368 盖章使工厂节点获得双身份后，双维度
   不对称是潜在 stale 源（未爆，先记录）。
3. **apply 完成回调空白**（pa2 核心输入）：`execute_script` 尾部 apply 与
   `pump_animation_clock` 内 apply（webview.rs:1631）均无 shim 通知。标记「host
   真相已更新，补偿作废」的换代点应为：apply 完成 → shim 清标记+清桶+失效融合缓存。
   当前只有导航形态的 SetDomSnapshot 挂了钩。
4. **R377 钩子的 registry 空源**是独立缺陷（fragment 展开未随迁 text 子到
   `_handleChildren`），不属标记生命周期，记 pa 切片外待办（转 pa3 fused innerHTML
   重落时的前置修正项——fused 序列化同样依赖 `_childNodeList` 融合视图读 registry）。

## 5. 验证

| 门 | 结果 |
|----|------|
| 探针轮工作树 | 零残留（part03/part05 插桩 `git checkout` 恢复，`git status` 干净） |
| 探针运行入口 | `./target/test-guard --per-proc-mem 4 --total-mem 8 -- ./target/release/zero-wpt-runner testharness-dom pa1-probe --wpt-data /tmp/pa1-wpt`（隔离 wpt-data 树） |
| 生产行为 | 零改动（纯审计轮，R354/R378 先例） |
| 基线用例复跑 | `make testharness-dom FILTER=remove-next-sibling` 维持 1F（已知 Fail 恒等） |

## 6. 对切片草案的影响

- **pa2（apply 代际令牌）** 前置事实已齐：换代点 = `apply_pending_shared_mutations`
  三调用点 + SetDomSnapshot 已挂钩；shim 侧语义 = 标记表清空 + pending 桶清空 +
  `_zwChildBaseInvalidateAll`/`_zwSiblingBaseInvalidateAll`（现有钩子体扩展）。
  **M6 补 mark** 应随 pa2 或独立小片 land（kill-switch 下零生产风险，双路径守门）。
- **pa3（fused innerHTML）** 前置修正项新增：R377 钩子 registry 空源修复
  （fragment 展开随迁 text 子）。
- **pa4（parse-segment 回放）** 不受影响。
