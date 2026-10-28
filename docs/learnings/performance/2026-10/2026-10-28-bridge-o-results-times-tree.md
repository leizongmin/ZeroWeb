---
date: 2026-10-28
modules: engine
---

# 桥查询路径的 O(结果数 × 树) 三形态——per-result 全局量必须惰性/缓存

## 问题

encoding-legacy-mb 语料（单页 13.5k span、qsa 千元命中）把三处共享路径的
「每结果重算全局量」放大到 13.9s/查询与 4GB 堆 OOM（详见
`docs/goal/encoding-legacy-mb/evidence/2026-10-28-m1-runner-channel-perf.md`）。

## 三形态与修复模式

1. **eager 全树/全文档序列化挂在每结果上**（`_zwRootHtml = outerHtml(root)`
   ×2 处）：修复 = `Object.defineProperty` 非枚举惰性 accessor（读时才算，
   语义仍新鲜）。前提核查唯一读者：`_zwParseEl.prototype.matches` 不在真节点
   原型链上，真节点零开销；cached 臂（`_zwParseEl` 本体）保留 eager。
2. **索引上限悬崖**（`size > 512 → clear()` 后仍回存 → 索引恒空、每次查找
   全量重建）：修复 = 提额至真实容量量级（13.5k 键 ≈ 1MB 级），失效语义
   （写点置 null）不变。上限型缓存「清空后回存」等于无缓存，review 时查
   clear 与回存的相对位置。
3. **建树每元素两次全文档重解析**（浅臂 per-element query + child_nodes
   回调）：修复 = host 单次解析整子树深 JSON 一次下发，本地 O(n) 建树；
   解析失败落旧臂零行为收敛。

## 通用判据

写循环内的表达式时问：它依赖循环变量，还是只依赖循环外状态？后者必须
提到循环外（惰性/缓存/一次下发）。放大器 = 语料树规模 × 结果数，小语料
测试不可见，语料页一上量即 OOM。
