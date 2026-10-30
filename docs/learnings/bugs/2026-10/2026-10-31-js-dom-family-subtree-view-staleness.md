---
date: 2026-10-31
modules: js_dom_shim, js_dom_bridge
---

# js_dom_shim 家族子树视图滞后：CE 动作第二动作成片 no-op 与 plain 寻址补全

## 问题描述

editing join 语料（join-different-white-space-style-*，每页 ~30 测试顺序跑、
每测试重新 setupEditingHost）首动作后全部 no-op：actual = initialHTML 逐字
不变，mutation recorded=0。与 [多族包装 identity 恒败](2026-10-14-js-dom-wrapper-family-identity.md)
（guideline #32）同域但不同面——对象 identity 都活着，死的是**子树视图**。

## 根因分析

同一 JS context 内存在两族 wrapper 对象图（canary 相同实证）：

- 页面家族：语料页 `document.querySelector` 早期产物被页面 const 持有，
  `setupEditingHost` 的 innerHTML setter 经它重建子树；
- query 家族：CE 动作（execute_dom_script 沙箱）`document.querySelector(sel)`
  产物，其子树视图**只在经由本家族的 wired 写时前进**。

页面 setup 走页面家族后，query 家族滞留上一动作形态（探针 H 实证：kid1 =
上轮动作的 shell span、孙代 plain 无写印）。CE walk 在陈旧树上早退（守卫
命中 / recorded=0）。两个放大器：①fused 序列化（innerHTML getter）读引擎
态恒 fresh——**用 innerHTML 断言看不见家族陈旧**，探针必须打 mutation 批
（recorded 的 kinds 序列）才能暴露；②多键流在 query 世界内自洽（写读同族
新鲜），使陈旧只在「页面写后首个 CE 动作」现身，散布的 pass 误导归因。

连带面：世界同步（以页面家族 innerHTML 回灌 query 家族 el）落地后，同步
产物是 innerHTML 解析的 plain 包装（无 `__zwSelector`/`__zwHandle` 印），
wired insertBefore/appendChild 的身份印分支门全不命中 → 落位静默本地化
（壳建成但引擎丢失）。plain 节点的移除/文本写早有 sel-path 变体
（RemoveChildAtBySelPath / SetChildTextBySelPath），**插入半边缺位**。

## 解决方案

1. 世界同步：`__zw_ce_caret_range` 入口检查 raw 选区锚链上的 contenteditable
   宿主是否属另一家族（非 el 本体），以其 innerHTML 回灌 el 子树（两家族
   fused 同源，内容一致时幂等重建）；caret 重解析门从「plain」拓宽为
   「plain 或 stamped 但锚落点非原对象」。
2. 插入 plain 寻址：`_zwPosSelOf` 自节点上行至首个 sel 祖先、按**元素序**
   （`:nth-child` 口径，与引擎 structural fastpath 计数一致——全节点序
   path 与元素序 nth-child 不能混用）渲染位置链，引擎 `find_by_selector`
   按当前树态解析；insertBefore 补 plain-ref 臂。
3. plain 壳（无 handle 无 sel 的迁移产物）：universal 落位方法只做本地化，
   需在调用点补序列化收敛——`InsertAdjacentHtml(位置串, beforebegin/
   beforeend, outerHTML)`，引擎侧「源位 remove（已有）+ 目标 copy」= move
   语义。

## 避免方式

- 排查「编辑动作 no-op」先打 mutation 批而非读 innerHTML；fused 读恒
  fresh，是陈旧视图的掩盖面。
- 判定节点寻址能力看三个印（`__zwSelector`/`__zwHandle`/写路径）；补偿树
  （innerHTML/DOMParser 解析产物）全 plain，任何 wired 写路径补分支时把
  plain 形态当作默认面而非边界——移除/文本/插入/属性四类都要有 plain 通道。
- 家族间同步选 innerHTML 回灌时要预判下游：同步产物必 plain，消费方的
  身份印依赖（如 insertBefore ref 门）在同批内就会踩空。
