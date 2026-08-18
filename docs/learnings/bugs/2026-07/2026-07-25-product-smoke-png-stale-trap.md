---
date: 2026-07-25
modules: tests/wpt-runner（cmd_product_smoke）, legacy/product smoke 诊断流程
---

# product-smoke 输出 PNG 路径陷阱（stale 文件致假 bug 误判）

**触发轮**：R2077（false alarm）

## 问题描述

R2077 调查 legacy smoke fixture `19-testpage-minimal`（diff=22.39%）时，用 PIL 读取
`docs/goal/rendering-compat/evidence/product-static/legacy-html/fixtures/product-smoke-cpu.png`
做 post-hoc 像素分析，发现整图全白（255,255,255），据此判定 `<body bgcolor="#c0c0c0">`
presentational hint 未绘到 canvas（系统性 bug），并加了 painter 诊断插桩深查。

插桩显示 body computed `background_color = Rgba(192,192,192,255)` 正确、canvas 传播
`add_fill` 银色也正确执行——但读取的 PNG 仍是全白。最终发现：**读取的是 stale 文件**。

## 根因分析

`cmd_product_smoke`（`tests/wpt-runner/src/main.rs:390`）把输出 PNG 写到 **CWD 相对路径**
`product-smoke-cpu.png`（`out_path = out.as_deref().unwrap_or("product-smoke-cpu.png")`）：

- `make product-smoke-legacy` 经 `run-all.sh` 从 **repo root** 跑 → 写到 **repo root**
  `./product-smoke-cpu.png`（gitignored，见 `.gitignore:70`）。
- 手动从 repo root 跑单个 fixture → 同样写到 repo root。
- `fixtures/product-smoke-cpu.png` 是 **orphan stale 文件**（疑似早期 run-all.sh 版本
  cd 到 fixtures 跑、或人工从 fixtures 目录跑遗留），与当前渲染结果无关。

读取 orphan stale 文件 → 拿到旧的/空白像素 → 误判渲染 bug，浪费一轮深查插桩。

## 解决方案 / 如何避免

**post-hoc 像素分析 product-smoke 渲染结果时，读 repo root `./product-smoke-cpu.png`
（命令实际写入处），勿读 `fixtures/product-smoke-cpu.png`（orphan stale）。**

判定 fixture 是否真有渲染 bug 的权威信号（按可信度排序）：

1. **run-all.sh 报告的 diff%**（渲染时新鲜计算，权威）。若 body bg 坏（白 vs 彩色），
   diff 必然 ~99%；diff 仅个位数%~20% → bg 正确，diff 在内容（font-wall）。
2. **struct-check**（sibling overlap / text concatenation / collapsed）= 真结构 bug 信号。
3. **LAYOUT_DUMP**（`cmd_product_smoke` 经 `LAYOUT_DUMP=1` env）= 布局盒几何，定位结构性问题。
4. **REFTEST_DEBUG=1** = dump `result.primitives` fills/images/...（图元级）。
5. post-hoc PIL 读 PNG 时，**确认读的是新鲜文件**（先跑一次再立刻读 repo root 文件）。

## 验证（R2077 收尾）

读 repo root 新鲜 PNG 复测：

- `19-testpage-minimal`：corner(5,5)=(192,192,192) 银 ✓、navbar=(0,0,128) ✓；
  PIL diff 区域分析：内容区（y<260）51% diff、背景区（y>260）**0.3% diff** →
  body bg 银色完全正确，22.39% diff 全在内容区（font-wall：sans-serif 文本 + 表格）。
- `01-body-attrs`：corner(5,5)=(255,255,238) 淡黄 ✓（`bgcolor="#ffffee"` 正确）。

body bgcolor → canvas 传播（`painter/mod.rs:309` `paint()` §14.2）正常工作，无 bug。
R2077 为 false alarm，插桩已回退。
