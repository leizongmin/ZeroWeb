# M2 s303 — 评估移交：the-select-element reftest 41 红页簇归属 rendering-compat 成立（含 active workfront 碰头信号；纯评估零源码改动 docs-only）

日期：2026-11-06。切片号 s303（承 s302 下一步候选①）。结论：41 页全部
rendering-compat 域（select 部件渲染形态 + customizable-select active
workfront），html-semantics 域零可修钉形；**双信号**（明确归属注记 +
并行流活跃施工）记入本控制面防碰撞，本轮不动。纯评估零源码改动。

## 勘根

**簇形态**（s300 b2 全树日志）：41 页 = customizable-select 族 37 页
（差幅 0.09%–6.79%，含 100% 一页）+ 经典 select 页 3 页（**0.00%**）+
listbox-sizing 1 页。

**失败模式两种，根均在渲染域**：

1. **mismatch 型 3 页**（select-dropdown-vs-listbox / select-listbox
   -selected-hscroll / select-multiple-size-1）：`rel=mismatch`——期望
   test 与 ref **不同**（>0.01% 像素差），runner 判「only 0/480000 pixels
   differ」失败。ZW 把 dropdown 外观（size=1 非默认形态）、listbox 选中
   项横向滚动可见、`field-sizing:content` 多选形态渲染成与 ref 相同——
   select 部件 appearance 形态学（下拉箭头/listbox 盒/field-sizing）全
   缺。**归属硬证据**：rendering-compat/master.md 常设注记「0.00% fail
   = mismatch 真缺口域勿动」+「mismatch 阈值 0.0001 勿回调（R3866）」
   ——rendering 流明确认领该域；全树 45 个 mismatch 案全红同域。
2. **customizable-select 37 页（rendering 流 active workfront）**：
   `appearance: base-select`/`::picker()`/`::picker-icon` 面——
   style-system 已有 appearance 属性解析/cascade（-webkit-appearance
   别名）与 matcher `::picker`/`::picker-icon` 识别（mod.rs:1399-1420，
   注记「2026-10-31 ::picker(select) 已转正——本地 chrome 探针页绿」）；
   rendering-compat.md R5359（2026-10-31）交付 picker 解析/supports
   名单修正。37 页小差幅+fuzzy 零容差声明（threshold=0.00%/Nch）说明
   ZW 已渲染至近邻形态、正在收敛——**并行流活跃施工面，rule 9 不碰**。
   本域 shim 侧 showPicker 六文件 129P 早已收编（M2 切片 3-5），37 页
   断言面无一 shim 语义。

## 归属裁决与防碰注记

- 41 页整簇归属 rendering-compat（select 部件渲染 + customizable-select
  CSS/UI 特性）；本控制面此后不再将其列施工候选（rendering 流 active
  workfront 碰头信号，rule 9/10 归因纪律：reftest 全树该簇数字随其
  goal 漂移，本域只记录不追）。
- html-semantics 域 select 面（IDDL/行为/弹出几何语义）不受影响：九域
  testharness 通道 select 族已高覆盖（s2-s5 既有切片），reftest 红页
  无 shim 断言。

## 下一步候选（本域）

1. **the-img-element reftest 8 红页簇勘根**（本域元素、施工潜力最高：
   adopt-from-image-document 16% 等 8 页，勘根立钉面后按例施工）。
2. the-bdi-element 13 红页簇归属勘根（bidi 隔离，疑渲染域移交候选）。
3. 其余小簇（textarea 4/meter 4/input 3/link 3/embed 3/object 2/ol 6/
   option 1/iframe 1/br 1/wbr 1）按簇勘根排队。

## 证据

- s300-reftest-full-b2.log（41 页态+失败原因行：mismatch 判词
  「expected > 0.01%」×45 树-wide、fuzzy threshold=0.00%/Nch 判词）；
  语料 3 mismatch 页全文核对（rel=mismatch+形态学断言面）。
- 代码锚：style-system matcher/mod.rs:1399-1420（::picker 转正注记）、
  property/registry.rs:307/:666 + supports.rs:11 + cascade.rs:286
  （appearance 面）；rendering-compat/master.md「mismatch 真缺口域勿动」
  常设注记 + rendering-compat.md R5359（2026-10-31 picker 交付）。
- shim 承继：part03 showPicker/invoker 面（M2 切片 3-5 账）。
