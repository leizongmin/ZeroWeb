---
date: 2026-11-07
modules: layout-engine
---

# taffy abs 包含块=直接父模型：后处理「只扩不缩」让错误钳制存活（R5445 D2 域图谱）

## 问题描述

abs-auto 盒的 used width 与 CSS2 §10.1 系统分叉：taffy 没有「最近 positioned 祖先 /
ICB」包含块语义，一律按**直接父 content width** 钳制 abs 子；而
`apply_abspos_shrink_to_fit_width` 后处理 pass 的两道旧闸门使错误钳制值存活——
①写侧只扩不缩（`b.width < target` 才写）；②branch-text 窄门只放行「有 inline 内容 +
有元素子」的盒（叶文本盒/纯块分支不重测）。探针矩阵实证三种分叉面：非 positioned 窄父
下的叶（96 vs chrome 796）、分支（96 vs 796）、positioned 祖先隔非 positioned 中间层
（96 vs 796）；唯「直父=CB」与「直父≈全宽≈ICB」两对照形近似一致——这就是该债 corpus-wide
潜伏却少有爆案的原因。

## 根因分析

修复走的是「spec CB ≠ 直接父时按 spec CB available 双向重测」，但第一版在 corpus 翻出
确定性真回归 `float-rgt-orthog-htb-in-vlr-003`（ON ✗5.78% ×4 / OFF ✓3.37% ×4）：ref 页的
orthogonal（htb in vlr）abs 盒被重写成 chrome 正确宽，反而**拆掉了自洽补偿**——test 侧
orthogonal float 放置是另一处既有债，双侧各自的误差恰好凑近（3.37% 近似通过），单侧修对
必然失配（guidelines #33 已成文，本例再次实证）。同时印证 #29：几何臂几乎总有 vertical
对应缺口，abs 宽语义在 vertical CB 语境是另一套轴模型（R4436 caption 先例同款）。

## 解决方案

- D2 臂三门：`!spec_cb_is_parent`（栈下行旗标：父 positioned → taffy 直父值可信）+
  `cb_horizontal`（spec CB 书写模式水平；ICB 语义随 root）+ R5236 排除（aspect-ratio/
  definite max-width）。
- CB 种子 INF→root.width 只动宽轴（% 高臂依赖 INF-skip，不顺手扩）。
- 栈新增旗标元素只在 CB 切换点做 styles 查询（#9：热路径 per-node 查表禁止）。
- self-source 案形修复前先跑双侧 A/B 单案采样（×3-4 双态），补偿型绿面在 fail 计数里
  看不出来（✓→✓ 值变），必须读百分比。

## 证据

docs/goal/rendering-compat/evidence/r5445-abspos-icb-cb-2026-11-07.md（探针矩阵 +
vlr 回归二轮收口 + corpus 真态 FIXED={s-11-1-1b-005 strict} REGRESSED=∅）。
