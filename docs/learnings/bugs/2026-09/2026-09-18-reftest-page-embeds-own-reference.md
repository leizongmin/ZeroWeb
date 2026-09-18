---
date: 2026-09-18
modules: layout-engine, engine, wpt-runner
---

# reftest 测试页自带页内参照标记——探针归因链必须先读源码

## 问题描述

rendering-compat R4483–R4488 连续六轮用 DOM/taffy/布局探针追查
`insert-inline-in-blocks-n-inlines-begin-001`（diff ~40%）的「双 container」现象，
逐步构建出「shim mutation tree 复制 → engine 侧双 DOM 生成物并存（mutation 应用视图
+ 序列化重解析视图）→ re-parse 应改替换语义」的归因链，并计划对
`js_dom_bridge`（共享面）做 re-parse 替换语义修复。

实际上该 WPT 测试页**本来就有两个 `div.container`**：第一个是 JS `insertBefore`
的动态目标，第二个是页内静态参照（ref 页同样是两个静态容器）——「双 container」
是页面设计而非缺陷。六轮探针对比的 A/B 两容器本应互异（A 有 JS 插入的 span 元素子，
B 是裸文本 + div 的静态标记），「丢失无 id span」「文本 = 源码原始空白」全部是
正常差异。

## 根因分析

真实缺陷在 `layout-engine/src/inline_block_split.rs` 的
`block_container_has_mixed_content`：谓词只把**非空白文本子**算作 inline 内容，
而测试容器 1 是「span 元素子 + block 子混排、文本子全为空白」——不触发
CSS2 §9.2.1.1 匿名块拆分。R2160 Phase A default-on 后 childless plain inline 的
taffy 节点被跳过、文本流入容器 IFC，与 block 子并存时两套几何互不感知
（span 行在容器顶连续堆叠、block 子独立堆叠、互相重叠）。

对照组容器 2（裸文本子）谓词为真、正常拆分——这正是「同一页两容器渲染不同」
的真正来源，与序列化/re-parse 无关。

## 解决方案

- 谓词扩展：childless 纯文本 inline 元素（Phase A 同源安全判据）+ 非 replaced +
  非 ooflow 也算 inline 内容（kill-switch `ZW_R109_ELEM_MIXED=0`）；
- 配套：element-only 片段 measure context 回落片段项首个 Text 后代、
  片段级 IFC 测量臂（`ZW_FRAGMENT_MEASURE`）、纯 ooflow 元素片段转 Block 片段、
  atomic_children 补 replaced 元素、paint 侧 fragment 盒放行
  `has_direct_paintable_text` 门。
- 结果：9 案全 0.00%，corpus 净 +16~17 / −0（15039 → 15055 / 16594）。

## 如何避免

- **探针归因前先读测试页源码**：reftest 页常自带页内参照标记（test/ref 同页）、
  多容器/多卡布局是设计而非复制。判断「DOM 里有两个 X」是否异常的最低成本动作是
  打开 `.xht`/`.html` 数一遍源码里的 X。
- **对照组差异 ≠ 缺陷证据**：两个本应不同的节点集（动态容器 vs 静态参照）之间的
  id/子树差异不能作为「复制/未替换」的证据；归因链要有一个「与源码逐节点对账」的
  步骤。
- **跨域假设要过最小反例**：六轮探针始终没有做过「把 JS 禁掉渲染同一页」或
  「直接静态写一份同构页面」这类能一分钟证伪 re-parse 假设的对照实验。
