# svg 二期 — SMIL 时钟模型专项 + 语料滚动扩面

**版本**: v1.0
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）
**状态**: Active——立项，M0 未启动
**执行模式**: WPT 驱动（上游 svg corpus 为验收标尺）+ 专项语义收敛 + 语料扩面
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——SVG 面）
**一期**: `docs/goal/archive/svg-compat.md`（80 号，2026-10-22 收口，corpus
70.3%、七项挂账定稿）

## coverage 对账（coverage-2026-11-02 基线，全量头条 67.54%）

svg topdir 全量分母 **15,342 子测试 / 1,465 文件**，zw 覆盖 486 文件
1,181P = 7.70%，未覆盖 **14,161**。一期收口时 corpus 70.3% 入账、基线
8.8%——上游语料滚动 + 一期排除域（SMIL 时钟模型、渲染面）是缺口主体。

一期七项挂账定稿清单（master.md + evidence/2026-10-21-m4-smil-ledger-
and-closeout.md，本 goal 承接底稿）：

| # | 挂账项 | 体量 | 归口 |
|---|---|---:|---|
| 1 | SMIL 时钟模型（动画时序/同步/事件时基） | 199 子 Fail + 29 案 Timeout | **独立立项面——本 goal 即承接载体** |
| 2 | getCTM/getScreenCTM 族 | 22 子 | rendering-compat 协同排期 |
| 3 | SVGLength 上下文单位（cap/ch/ic/lh/rlh + viewport/percentage） | S2a FIXME 批 | 与 getBBox 几何上下文同批评估 |
| 4 | 文本几何 / image 内在尺寸 / CSS cascade 命中 / CSS d property | 既有挂账 | style-system/rendering 协同 |
| 5 | `.py` 动态端点（script-load-error-events 2F） | 2F | runner/infra 域 |
| 6 | a.ping / a.rel-\* | 小 | 导航域 |

关键事实：SMIL 分类账（199F + 29T 案级量化）是一期 M4 交付物——本 goal
M0 直接以分类账为底稿落批次，不需从零归因。

## Mission

1. **SMIL 时钟模型专项**（M1/M2）：一期分类账逐簇推进——时钟原点/
   begin-end 时序/同步基（syncbase、event-value、repeat-value）/动画事件
   时序语义；落 `zero-media`/engine 动画泵边界（SMIL 是 SVG 文档级时间
   线，不进 compositor 动画泵）。
2. **语料滚动扩面**（M0/M3）：FORCE 续拉对齐 pin + 一期收口后上游差集
   甄别；渲染面（reftest）不入账照旧，testharness 新面按簇导入。
3. **挂账 #2/#3 归口重估**（M3）：getCTM 族与 rendering-compat 碰头归口；
   SVGLength 上下文单位批评估结论落档。
4. **复测收口**（M4）：svg topdir 落数值门；一期已收口面零回归。

## Done Criteria

- **DC-1（M0 对账）**：语料续拉对齐 pin（差集清单 + missing=0）+ SMIL
  分类账对照上游现势的批次划分落 `docs/goal/svg-compat-v2/evidence/`；
  DC-4 数值门按决策规则落定并回填本文件。
- **DC-2（SMIL 通道）**：SMIL 面入常驻通道（runner 子命令或 svg 通道
  扩面，M0 定）+ skip 规则 + Makefile target；分类账基线复核落档。
- **DC-3（语义收敛）**：SMIL 批次簇逐批收敛或具名挂账（每批 evidence：
  FIXED/REGRESSED 记账）；挂账 #2/#3 归口结论落档。
- **DC-4（coverage 复测）**：svg topdir 从 7.70% 升至数值门；一期已收口
  corpus 账逐 subtest 零回归；头条不回落。

**DC-4 数值门决策规则（立项即定）**：SMIL 分类账地址池（199F + 29T 中
定性可修簇）+ 滚动新面导入即得，两项合计为地址池；数值门 = 地址池 ×
一期同类簇通过率（M0 实测后填入）；渲染级耦合簇不进门，具名挂账。

## 划界

- **渲染面 reftest 不入账**：svg reftest 归 rendering-compat（一期口径
  不变）。
- **动画泵边界**：SMIL 时钟模型落文档级时间线语义，与
  rendering-compat/compositor-animation 的 CSS/WAAPI 动画泵不混写；
  共享时钟原点改动须双方控制面留痕。
- **挂账 #2/#4 渲染耦合项**：getCTM/getBBox/文本几何归 rendering-compat
  协同，本 goal 只做归口结论不双改。
- **`.py` 端点（#5）与 a.ping（#6）**：分别归 runner/infra 与导航域，
  不进本 goal DC。

## 里程碑

- **M0**：语料续拉 + 分类账批次划分 + 数值门（DC-1）。
- **M1**：SMIL 通道落地 + 时序基簇第一批（DC-2 起步）。
- **M2**：SMIL 同步基/事件时基簇收敛（DC-3 主体）。
- **M3**：滚动新面导入 + 挂账 #2/#3 归口（DC-3 收尾）。
- **M4**：coverage 复测收口（DC-4）。

## 依赖约束（run-rules §9）

- svg 通道（80 号已收口面）每批次逐 subtest 零回归复验；SMIL 时序改动
  额外复验 web-animations/timing 通道（timing-animation-compat 已收口
  台账）不受文档级时间线改动波及。
- 与 rendering-compat（活跃 rally）交界：getCTM/几何上下文/动画泵共享
  面先碰头归口再动，双方控制面留痕。
- 上游滚动走 pin 对齐 + blob-hash 对账（goal-35/80 先例）。
- SMIL 长动画案（Timeout 面族）跑 runner 时按 test-guard 内存与墙钟
  口径包裹，防无人值守超预算。
