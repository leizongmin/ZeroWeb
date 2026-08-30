# RFC：JS-DOM pending-apply 生命周期（host apply 异步滞后与 JS 同步视图的边界收口）

**版本**: v0.1（立项——材料收集完成，切片分解待评审）
**日期**: 2026-08-30
**状态**: Draft（立项材料齐备，待评审切片分解）
**goal**: `docs/goal/js-dom.md` M4 基线维护延伸（已知 Fail 深项同根归因）
**证据锚点**: `docs/goal/js-dom/evidence/2026-08-30-r373-abort-domain-import.md`（parse-time 评估）/ `2026-08-30-r377-inserted-script-execution.md`（fused innerHTML 实验 + 探针）/ R371（replaceWith 重键 + 探针链）

---

## 0. 执行摘要

**问题**：shim 的 DOM mutation 经 `DomMutation` 队列**异步 apply** 到 host
（渲染权威），而 JS 侧同步视图靠**多层补偿机制**（pending 桶 / 移除标记 /
融合 childNodes / R309-R322 归并）追赶。三层已知 Fail（约 8 个 subtest / 3
文件）与一影子缺陷共享同一根因：**补偿机制各自为政，且其生命周期与
host apply 的代际边界（快照换代）不一致**。

**三类实证**：
1. **parse-time MO**（MutationObserver-document 3F）：解析器建树早于脚本
   执行——注册在文档中段的 observer 收不到「注册点之后」的解析插入 record。
   本仓架构整树解析完再执行脚本，解析与执行不交错（R373 备档）。
2. **fused innerHTML**（remove-next-sibling 剩余件）：sel 容器 pending 桶非
   空时 innerHTML 须从融合 childNodes 序列化；R377 实验实现后探针揭示
   `_childNodeList` 不剔除 sel 移除标记子（`_zwRemovedSels` 读时已空——清除
   链路未定位），实验代码已回退（零残留）。
3. **window-extends-event-target 2F**：window 事件路由是定制实现
   （`_globalAddEventListener`），非 EventTarget.prototype 继承——同一
   「JS 视图与 spec 结构不对齐」家族的另一种表现（转档观察，非本 RFC 范围）。

**方案方向**（待评审）：单一 **pending-apply 生命周期层**——
1. **代际令牌**（generation token）：每次 host apply 完成发布新代；补偿状态
   （pending 桶/移除标记/融合缓存）挂代，换代统一失效——消除「补偿状态比
   host 真相更旧/更新」的窗口。
2. **移除标记生命周期审计**：枚举 `_zwRemovedSels/_zwRemovedHandles` 的全部
   写点/清除点，标记清除必须与 apply 代际绑定（apply 后标记失效是**正常**语
   义，但失效时补偿视图必须同步重算——当前缺失这一联动）。
3. **parse-segment 回放**（可选独立片）：runner 侧按脚本文档序分段，
   host 报告段间 body 子树 delta，shim 合成 MO record 回放给中段注册的
   observer——不改解析架构即可服务 parse-time MO 用例的主要断言面。

**切片序草案**（每片独立 land、全量双路径 net≥0）：
pa1 移除标记全写点/清除点审计（零行为变化，产出清单）→ pa2 apply 代际令牌
（host 侧 apply 完成回调 + shim 换代钩子）→ pa3 融合 innerHTML 重落（依赖
pa1/pa2 的标记语义修正）→ pa4 parse-segment 回放（独立，可并行）。

**边界**：不动 host 解析架构（整树解析保持）；不动 native 路径；不改
`__zw_*` wire 协议（代际令牌新增一个轻量回调，不改动既有）。实验教训
（R377）：fused 视图实现前必须先落 pa1/pa2——补偿语义不修正，融合序列化
会基于错误前提。

---

## 1. 背景与实证细节

### 1.1 pending-apply 现状

shim mutation 面分两类：
- **wire 即时型**（`__zw_append_child` 等）：推入 `DomMutation` 队列，脚本
  turn 结束后统一 apply；
- **host 直调型**（`__zw_set_inner_html` 等）：部分直接改 host。

apply 后 host 发布新快照（`SetDomSnapshot`），shim 经 R358
`__zw_reset_pending_state` 清桶——但**移除标记（`_zwRemovedSels`/
`_zwRemovedHandles`）不在清桶范围**，也**没有与快照换代绑定的失效**。

### 1.2 R377 探针记录（remove-next-sibling 流程）

- `replaceWith` 前插 + `__zw_remove` + `_zwMarkRemoved` 后，
  `_childNodeList('#container')` 产物 `[target, b, span, span]` 全部
  `rm=false`（`_zwIsRemovedNode` 全 false）——标记在读时已空；
- 清除点未定位：`_zwUnmarkRemoved` 仅 4 处（appendChild 系的回插清除），不
  在本流程路径上；`__zw_reset_pending_state` 不清标记；
- 候选解释（待 pa1 审计）：R368 盖章使 b/target 获得双身份（sel+handle），
  某一 unmark 点以 handle 维度清了 sel 标记；或 `_childNodeList` 内部的
  identity 反查（R331）触发了标记外的路径。

### 1.3 影响面

- remove-next-sibling-during-replace-with（1F，fused innerHTML + 标记语义）
- parse-time MO 3F（pa4 可选面）
- 未来任何「同步视图 vs apply 滞后」新补偿逻辑（R309/R310/R322 模式的每个
  新增点都在累积跨代不一致风险）

---

## 2. 切片草案

| 片 | 内容 | 验证门 |
|----|------|--------|
| pa1 | 标记审计：全写点/清除点枚举 + 双身份（sel/handle）矩阵 + 清除点定位（探针法：标记表 Object.keys 周期 dump） | 审计文档（零行为变化） |
| pa2 | apply 代际令牌：host apply 完成回调 → shim 换代钩子（清标记 + 清桶 + 失效融合缓存，语义 = 「host 真相已更新，补偿作废」） | 全量 sweep net≥0 + R309/R310/R322 哨兵 |
| pa3 | fused innerHTML 重落：pending 桶非空时从 `_childNodeList` 序列化（依赖 pa1/pa2 标记语义） | remove-next-sibling 全绿 + innerHTML 哨兵（Range-mutations 全族） |
| pa4 | parse-segment 回放：runner 分段 + host delta + shim record 合成 | MutationObserver-document 3 断言面 |

## 3. 修订历史

- v0.1（2026-08-30 R378）：立项——R373/R377/R371 材料入册 + 探针记录
  （§1.2）+ 切片草案（§2）。切片分解待评审；pa1 审计可先行（零风险）。
