# Selection 兼容 — window.getSelection / Selection API

**版本**: v1.0
**日期**: 2026-10-08
**状态**: ✅ Completed（M3 收口 2026-10-15——终态 2968/2970，99.93%）
**执行模式**: WPT 驱动（上游 selection corpus 为验收标尺）+ 语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——文本交互面）

> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：js-dom
> 做过 Range 三件套、editing-contenteditable 做过编辑语义，但 Selection API 面
> （getSelection/collapse/extend/modify/方向语义）从未 goal 化度量。
>
> **▶ 基线事实（2026-10-14 M1 实测）**：`make testharness-selection` 全量
> **2965/2970（99.83%）**；Selection 对象存在性/方向语义/isCollapsed 面已大
> 体收敛，余缺口为 toString rendered text 语义（已修，2966/2970）、Actions
> 拖选选区构建 + live-range 边界吸附（已修，M2 切片 2）、
> document.open 依赖（跨域挂账）。证据：
> `docs/goal/selection-compat/evidence/2026-10-14-m1-baseline.md`、
> `evidence/2026-10-14-m2s2-anchor-removal.md`。
>
> **▶ 终态（2026-10-15 M3 收口）**：**2968/2970（99.93%）**，分级通过率
> 连续两轮不下行；make test 全绿、make reftest 100.0% 零不一致、
> product-smoke 15.35% ≤ 20%。残余 2 失败为跨域挂账（Document-open，待
> document.open 落地复测）与排除域（editing/event）。证据：
> `evidence/2026-10-15-m3-close.md`。

## Mission

以 WPT selection corpus 为验收标准，收敛 Selection 对象语义与 DOM Range 的
桥接。排除：contenteditable 编辑行为（editing-contenteditable 已归档域）、
execCommand 命令语义（editing 域）、剪贴板交互（web-api-batch2 已归档域）、
选择高亮绘制（渲染面归 rendering-compat）。

## Done Criteria

- **DC-1** 导入 + 基线 + csv planned 行回填 ✅（2026-10-14，evidence/2026-10-14-m1-baseline.md）
- **DC-2** Selection 语义对齐（含与 Range 边界换算、方向/collapse 语义），分级
  通过率不再下行两个连续轮次 ✅（2026-10-15，2966 → 2968 → 2968）
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值 ✅（2026-10-15，evidence/2026-10-15-m3-close.md）

## 里程碑

M1 导入与基线 → M2 语义收敛 → M3 收口归档。

## 依赖约束（run-rules §9）

与已归档 js-dom（Range 遗产）/editing（编辑面）边界清晰；渲染面（选区高亮）
缺口记账回流 rendering-compat，不在本 goal 处理。
