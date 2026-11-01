---
date: 2026-11-01
modules: style-system,engine
---

# UA display 双面分叉：host 表滞后于 JS 回落面，句柄通道 exposing 旧值

## 问题

slice25（`af677d7f0`，2026-10-03）把 JS 回落面 `part01.js _zwUaDisplay` 的 li 翻为
`list-item`、canvas 翻为 `inline` 并新增 engine reftest——当时绿，因为动态创建元素还没有
任何通道把 host 级联值暴露给 CSSOM。s161（`ee2dc953b`，2026-10-30）为 `__zwHandle`
元素加 `__zw_get_computed_style_by_handle` 通道后，`createElement` 出来的元素 gCS
display 改经 host 级联解析，host 面 `crates/style-system/src/lib.rs ua_default_display`
仍是 R1656 时代旧值（li→Block、canvas→InlineBlock）——两面分叉立即以
`test_ua_display_li_list_item_canvas_inline_s24_flip` 常驻红形式暴露（gCS(li)='block'）。

修复首跑仍红且报 `left='inline'`：第二层根因是 UA 注入路径 `display_str` match 的
`_ => "inline"` catch-all 没有 `DisplayValue::ListItem` 臂，表翻正后的 li 在回注 UA
声明时又被折回 inline。

## 根因

1. **同一名义默认值存在两个实现面**：Rust host 表（`ua_default_display`）与 JS 回落表
   （`_zwUaDisplay`）。历史上只有 selector 元素走 host 面、动态元素走 JS 面，分叉不
   可见；任何一个新通道（句柄 gCS、audit 工具、序列化路径）让「另一面」的值变得
   可观察，分叉就变成行为回归。slice25 翻转只改了 JS 面并加测试，测试绿是通道缺位
   造成的假阴性，不是双面一致性的证明。
2. **枚举→字符串注入 match 的 catch-all 臂会吞新值**：`display_str` 用 `_ => "inline"`
   兜底，新增语义变体（ListItem）不在臂内时静默折值为 inline，类型系统不报错。

## 解决

- 双面同值裁决：host 表翻 li→`ListItem`、canvas→`Inline`（HTML 渲染规范 UA sheet
  `li { display: list-item; }`，replaced 元素不在 display 集回落初始值 inline），与
  JS 面对齐；翻转前做布局域审计（ListItem 与 Block 同 taffy Block 语义、marker 门控
  或臂、paint_list_marker tag 键——零布局差才翻）。
- 注入 match 补显式臂 `DisplayValue::ListItem => "list-item"`。
- style-system 单测钉双面裁决值，兑现 slice25-D4 冻结契约（双面翻转 + reftest/
  product-smoke 门禁同提交）。

## 避免方式

- 改任何「默认值表」时先 `command grep` 枚举所有实现面（host 表、JS shim 表、
  序列化 match），一次改齐；只改一面必须显式记录另一面的滞后与暴露条件。
- 新增解析/暴露通道（handle、bridge、dump）时，把「两面默认值一致」列为该通道的
  验收项；通道缺位时的绿测不能当作双面一致证据。
- 枚举→字符串注入 match 禁止无限期依赖 catch-all：每个有独立字符串的变体都应有
  显式臂，新变体落地时同步检查。

## 后续：布局域连锁（同提交修复，t-fup-s24-flip-red）

翻转后的「零布局差」审计只覆盖了静态存量路径；三个既有布局单测（r5178/r5346/
p114）暴露了 display:Inline canvas 与 ListItem 在**边角落径**上的真实布局缺口，
均为翻转把元素送入不同机制所致，逐个修复而非回退翻转：

1. **r5346（canvas 盒消失）**：`tree.rs all_inline` leaf-path 判据把 display:Inline
   的 canvas 计为纯 inline 包裹层，内容尺寸容器走 leaf-measure 把整棵子树吞进 IFC
   测量（无子 taffy 节点）→ canvas LayoutBox 整盒消失。canvas 是 atomic inline
   （CSS2 §9.2.1.1），从 all_inline 与 `phasea_multi_inline_eligible` 两处 skip 判据
   排除（svg 及其余 replaced 类维持既有路径，控爆炸半径）。
2. **r5178（min-width:min-content 钳制失效）**：R5178 replaced 臂在 taffy min_size
   写入 100 并 relayout 后，`inline_finalization` 的 `ib_sizes` 收集只认
   display==InlineBlock，Inline canvas 不入表 → IFC collect 以声明宽 10 重建原子项，
   覆写 taffy 已解宽。修复 = `is_inline_canvas` 谓词把 Inline canvas 的 used
   border-box 喂进两处 ib_sizes 收集点，与 inline-block 家族同表。
3. **p114（li width:min-content 塌 0）**：`apply_intrinsic_content_sizing` 入口
   `is_block = matches!(display, Block)` 漏 ListItem → R1018 关键字 pass 不受理
   li 的 width:min-content。converter 本就把 ListItem 映射 taffy Block、
   intrinsic_sizing/tree.rs 既有分组均并列，补齐 gate 即可。

**避免方式（补充）**：翻 UA display 前的布局域审计要覆盖「该 display 值在
skip 判据 / 尺寸表收集 / intrinsic pass gate 的枚举位」，不只是主渲染路径；
taffy 盒路径与 IFC 路径对同一元素可能各持一份几何（taffy min_size 解 100、
IFC stored 表缺项回退声明宽 10），修复时先定位**最终盒宽的实际来源**再动手，
不要假定 relayout 结果自然生效。

## 后续 2：双审查返修（D1/D2，同任务）

首轮缺陷审查 2 项阻断发现，返修探针（build_subtree 入口/backtrace 插桩）定位后
修复，暴露 r1 修复的第 4/5/6 个枚举位：

1. **D1 三层（inline canvas + fallback 子）**：fallback 抑制在三条路径漏
   display:Inline——`is_replaced_with_fallback` display 门（补 Inline）、
   `is_inline_r109` 拆分宿主判据 `inline_has_block_child`（直读 DOM，对
   fallback 抑制元素关闭）、R5251 `inline_child_is_pure_block_carrier`（直读
   DOM，Fallback replaced 不作纯块载体，否则 p 升格容器层、canvas 整盒消失）。
   根因形态：**树构建期清空的 children_dom 挡不住下游直读 DOM 的判据/拆分
   路径**——同一抑制语义必须在每个 DOM 直读者重复施加。
2. **D2**：`vertical_block_flow.rs` 两处 block-level 子判据 `Block` 漏
   `ListItem`（:322 vrl 根右缘重锚、:644 vertical_block_child_indices_ex 块轴
   堆叠）→ vertical 容器 ≥2 个 UA li 全部缺席 restack，滞留 taffy 同列位
   （实证双 li 同位塌叠 (18,0)×2）。与同文件 R4447 先例（已并列
   Block|ListItem）不一致即信号。

**避免方式（再补）**：(a) 「children 清空」类抑制不是屏障——审计时
`command grep` 枚举该元素子树的所有**直读 DOM** 的判据与拆分器（child_nodes/
data.children 的消费者），逐一施加同一谓词，或把谓词下沉进共享判定函数；
(b) 同一文件内同一语义判据（block-level 集合）出现多种写法时，先对齐既有
先例再改；(c) 盒收集测试辅助函数用栈实现先序遍历时要 `.rev()` 入栈保 DOM 序，
否则兄弟序断言会假红/假绿。
