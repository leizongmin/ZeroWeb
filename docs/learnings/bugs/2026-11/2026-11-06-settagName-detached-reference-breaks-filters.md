---
date: 2026-11-06
modules: engine
---

# setTagName 先插后删换型使持有引用 detached，过滤判据恒假

## 问题描述

editing 流 q16 `:478` 直选跨嵌套 wholesale 换型刀（批次10 v1）在 ARM-B' unshift 成功时记录 `shellArm = shell`，并在 enable nodeList 平铺阶段的收集过滤里用 `_zwListContainsNode(shellArm, node)` 排除壳子树成员。四腿门禁（iol/iul/probe/enginelib）全部与落地基线逐位相同——刀完全 no-op，但由于「零回归」表象，一度被当作中性有效轮放行。

## 根因分析

`_zwListRetagPreservingSelection`（part01e.js）的 setTagName 实现为**先插新元素后删旧元素**。旧壳元素被删除后脱离文档树，此前持有的 `shellArm` 引用指向 detached 节点。`_zwListContainsNode(root, n)` 沿 `n.parentNode` 向上爬到文档顶，永远遇不到 detached 的 root → 返回恒 false → 过滤器从不命中 → 排除语义静默退化为恒空。方向语义（n 是否在 root 内）本身正确，问题纯在引用存活期。

## 解决方案

在发生 DOM 替换的分支内重锚持有引用：retag 分支加 `if (list === shellArm) shellArm = retagged;`（1 判断 2 行，v2）。落地后 `:478` 双镜像 compare 翻绿（iol 46→45、iul 43→42），f1b bar F2P=190≥185，P2F=0（批次11 终判，commit 81f2f4c90）。

## 避免方式

1. 跨 DOM 替换操作持有 Element 引用的判定谓词，必须在替换点之后重锚或重查询——身份判据对 detached 引用无意义。
2. 「过滤性修改零 delta」不是过滤器生效的证据：目的为排除成员的过滤器，要用成员集构成验证排除确实发生（阶段计数插桩的 toConvert/nodeList 点），不能只看最终 diff。
3. 部署计数断言「过滤器至少命中一次」可在门禁内直接暴露恒空过滤器，而非三轮之后靠语义考古发现。
