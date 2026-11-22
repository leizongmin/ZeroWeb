---
date: 2026-11-22
modules: engine
---

# shim sel 域同 turn 可观测性：元素种子面优先于镜像 MO 桶链

## 问题

`Element.setHTMLUnsafe()` 的 sel 域臂（宿主为静态解析元素，`hostHandle == null`）写后同步读（`firstElementChild`/`childNodes`/`innerHTML`）恒空：解析产物不挂父链、不挂任何视图账本，引擎 mutation 又是异步 apply。WPT reference-target 矩阵的 setup（`host_container.setHTMLUnsafe(...)` 后 `host_container.firstElementChild`）整个面因此不可用。

## 根因分析

innerHTML setter 的同 turn 可见性靠数百行尾部 machinery（R304 空基底 + MO 汇流入桶 + R380 融合 + CE 断连/连接 + slotchange），其中与「子节点可见」直接相关的通路有两条，且都对 setHTMLUnsafe sel 臂失效：

1. **MO 桶面**：`_mo_notify` 推 childList record 期望 R51c 分桶。但对无 MutationObserver 的场景与无 `__zwHandle` 的解析代理（`_zwMBuildBodyTree` 产物是工厂代理），桶写入不发生或条目无人消费——首版修复（镜像 setter 推 `_mo_notify`）实测零效果（探针 innerHTML readback 仍 `[]`）。
2. **种子面**（R5356b）：`_zwRegisterFragEl(el, handle, sel, nodes)` 把解析顶层代理注册进 `_zwTextElsBySel`，读路径 `_zwLocalChildNodes` 按序返回整个 `nodes` 列表，childNodes/firstElementChild 融合视图自动并入；innerHTML 需另补 `_zwFusedInnerHtmlSel` 的触发臂（该函数只认桶/子树脏/属性脏三门，种子不在内）。

## 解决方案

- setHTMLUnsafe sel 臂尾部：`_zwRegisterFragEl(_makeProxy(sel, handle), handle, sel, _uAdded.slice())` + `_zwChildBaseCache.set(sel, [])`（旧基底剔除失配防残留）。
- `_zwFusedInnerHtmlSel` 加第四臂：种子条目带 `nodes` 且 `gen === _zwApplyGeneration()` 才激活融合序列化——R381 同款代际门，apply 换代后自动回落 host 快照，不遮蔽引擎树推进（setter 的 R5356b 叶内容门防的是 contenteditable 场景种子遮蔽，setHTMLUnsafe 恒整体替换语义，容器形片段正是主形态，不需要该门）。

## 通用教训

- **镜像热路径全链是坑**：setter 尾部 machinery 有语料敏感门（叶内容门、R4100、PA id……），整体镜像回归面大。先找已有读侧消费面（种子注册表），写侧只补注册，是低风险通路。
- **同 turn 可见性机制有三套**（MO 桶/R380、文本/frag 种子/R5273+R5356b、detached 缓存/R86），写路径选择哪套决定读路径走查序；新写 API 时先核对目标读 API 的消费面归属，别假定「推了 record 就可见」。
- **proxy expando 遮蔽原型 setter**：`_makeProxy` 的 set trap 对未派发属性落 `_expando[key]` 侧表，原型 setter（IDL String 化/null 清除）永不可达且 expando 读先于原型走查——探针表现为 `sr.referenceTarget = 42` 读回数字 42 而非 "42"。修复模式：名称门控原型走查（R-t2qq/R5432 形态），set 侧命中即 `return true`，get 侧描述符按属性名查。
