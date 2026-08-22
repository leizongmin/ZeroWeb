# R160 Evidence — `:empty` 注释/PI 语义 + `:target` fragment + 树碎片化深结构结论（M4 nodes）

**日期**: 2026-08-22
**Commit**: `87bb059dc`（rebase 后；原 `dc6169aac`）
**切片**: M4 — R159 遗留轻件（`:empty`/`:target`）+ 树碎片化评估（深结构结论）

## 修复两件

### 1. `:empty` spec 语义（CSS Selectors L4）

元素为空 = 无子节点，**或所有子节点均为注释/处理指令**——空白文本节点**非空**：

- WPT `#pseudo-empty p:empty` expect p1（零子）+ p2（仅注释）；
  p3（空白文本 `" "`）/ p4（文本）/ p5（span 子）不中
- 旧实现 `children.is_empty()` 把 p2（注释子）漏掉
- 修：`compute_element_position` 的 `is_empty` 改 all-children-are-comment-or-PI

### 2. `:target` fragment 判定（iframe 子文档查询）

- `__zw_parse_html_query` 新第 4 参 URL（可选，3 参调用零变化）→
  `parse_html_element_json_with_url` → `doc.set_url` →
  `is_target_element`（既有 target.rs 权威判定）
- iframe doc 的 fragment 经 `_zwFragmentUrl` 槽（part04 建档时从 src 注入）
  → detached factory `queryBody` 透传
- 旧版重 parse 的 doc 无 URL → `:target` 恒 miss（WPT :target 簇 expect 1）

## 深结构结论：树碎片化不可经序列化桥（记 R161+）

R159 记录的「per-element mutTree 与 doc 查询树互不合并」本轮做了桥接
**尝试**（`_zwSyncFromElement`：mutation 后以 mutTree.outerHTML 替换 doc 树
中对应段）。桥本身工作（appendChild 后 doc 查询可见），但 probe 实证
**序列化丢属性**：`root.appendChild(createElement("div")` + `.id=` 后，
root._mtree.outerHTML 序列化为 `<div><div></div></div>` —— createElement
产物（proxy 形态）挂入 `_zwMEl` 树后 **id/属性在序列化中全丢**。正解 =
L2/M1 的统一 live Document（polyfill 查询读同一棵树），非本层可修。
桥接实验已回退，结论记 master.md。

## 跨流归因（run-rules §10）

- **~25 个 error-page 文件的 summary-line 翻转**（testRanges/testNodes
  undefined 族的 1→0）：归因 **Cache API 提交**（`aaa1753c5` 改 part01/
  02/07 shim，service-workers 流）——stash 干净 HEAD（含 Cache 不含本轮）
  复现同样 fail；且这些页面读 common.js 的 setup 延迟赋值全局，本就是
  error-edge 态。归 service-workers 流处理（master.md 记录，不硬解）。
- **两个 load-order flake**（SW `repeated_registration`、IDB
  `cross_renderer_transactions`）：满载 make test 下偶发，单跑全绿——
  storage/SW 流测试。

## A/B 双路径

全量 dom WPT（双路径逐计数一致）：**9490P/373F/18T**。

计数对照说明：R159 记录的 9503P 中含 ~39 个 Cache 提交引入的 summary-line
漂移（本轮归因后剔除认知）；本轮真实净增 = ParentNode +10 + Element-matches
族 +16 = **+26P**。R156 以来五轮累计真实增益 ≈ **+3200P**。

## 验证

- `cargo test -p zero-dom`：849 全绿（+1 zz_r160_empty_semantics：p1/p2/
  p3/p4/s1 五形态 + 无 URL :target 零命中）
- `cargo test -p zero-engine`：2307 全绿；fmt/clippy 干净
- `make test`：js-dom 域套件全绿（两个跨流 flake 单跑绿，见上）
