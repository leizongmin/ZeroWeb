# R5376 证据 — css2.1 §10.3.7 约束方程 margin 侧（auto margin 解方程）

日期：2026-11-01（R5376 轮；基线 = main@583ebe1df 干净树 **755F** 稳定交集）

## 0. 结论速览

| 案 | 修复前 | 修复后 | 定谳 |
|---|---|---|---|
| `css/CSS2/csswg-issues/submitted/css2.1/abspos-non-replaced-width-margin-000.xht` | 2.96% | **1.58%** | 残差③（单 auto margin）收口；残差①（rtl ignore-left）②（all-auto 静态位 +29）仍挂账 |
| `css/CSS2/csswg-issues/submitted/css2.1/abspos-replaced-width-margin-000.xht` | 3.90% | **2.49%** | 同域同收口（replaced 定宽形态同方程） |

corpus 同树 A/B 双跑（R5376 树 RUN1 758F / RUN2 756F，稳定交集 **755F** 与 R5375 基线
持平）：FIXED=0 REGRESSED=∅——RUN1/RUN2 超出稳定集的
`table-anonymous-objects-127/128/150/152` + `intrinsic-percent-replaced-dynamic-005`
全部隔离复跑 PASS（jitter 面，与 R5374/R5375 轮同族记录）。两目标案 diff 收敛但仍 F
（残差①②未清），同 R5372 轮「760F 持平、diff 收敛」先例。真通过-可信 10366。

## 1. 归因（R5372 残差③收口）

R5372 定谳的三类残差中的 **③单 auto margin 不吸收方程松弛**：spec §10.3.7 约束方程
`left + ml + bb + mr + right = CB`，left/width/right 全 definite 且恰一个 margin auto
→ 解该 margin（可负）；双 auto → 均分 slack/2（负 → ltr ml=0）。旧三写点（viewport
`adjust_absolute_pct_to_viewport` / root-CB / nested-CB）对 auto margin 一律取 0 →
"only ml auto" 组落过约束左锚 x=CB+left=53，应 496（ml=650−3−177−19−8=443）。

## 2. 修复

`abspos.rs` 新增 `solved_auto_margin_left` helper，三个定位写点同臂：

- 门：`margin-left:auto` + `width` 非 auto（shrink-to-fit 域不入）+ `right` definite
  （右缘吸收域不入）。
- 单 auto：`ml = CB − left − right − mr − bb`（`child.width` 为 border-box 即方程
  width+border+padding 项，模块头注）；双 auto：`slack/2`，负 slack → ml=0（既有
  口径）；definite margin / width:auto / right:auto → None 回退旧口径。
- 无新增 kill-switch（R5372 同裁决：nested 臂整体有 `ZW_ABSPOS_NESTED_CB`；臂窄，
  auto margin + 全 definite inset 在 corpus 为 rare 群）。
- 垂直轴不动：R2068 已证 taffy 正确解垂直单 auto（max-height-004 0.06% PASS），
  水平轴是 taffy 不解的缺口（R5372 实证）。

## 3. 门禁

- corpus 同树 A/B 双跑：755F 稳定持平，FIXED=0 REGRESSED=∅（jitter 隔离复跑全过）。
- layout-engine lib：**2316P/0F**（+6 `r5376_abspos_auto_margin_tests`：嵌套单 auto
  解方程 / 双 auto 均分 / 负解 / width:auto 回退守卫 / definite margin 守卫 /
  root-CB 臂）。R5372 既有 7 pin 全过。
- make test workspace：**3664P/1F**——1F = `ua_display_li`（D4 既有账，与
  R5372–R5375 轮同位同签名）。
- clippy `--workspace --all-targets`（guarded）0 告警 + fmt 清零。
- product-smoke：struct-check PASS（welcome + article @320 双面）。
- bench-gate：定向 zero-layout-engine 静窗 **GATE PASS（25 指标全预算内 NEW=0）**——
  改动面为 abspos auto-margin rare 群的 6 行算术。前序三跑 GATE FAIL/ABORT 均环境：
  ①ABORT（loadavg 20.9 守卫拒跑）②4 指标 FAIL（1min 11.8 起跑、5min 74 未排空）
  ③2 指标 FAIL（1min 5.8 但 5min 13.8 余波）；**stash 干净树同窗对照更差**（clean
  树 5 指标 FAIL、block_layout 4× 基线）证非本改；终跑静窗（1min 3.7 且对端流
  rustc/cargo 零进程）全绿。
- pre-commit guard：三提交全 PASS（独立扫描）。

## 4. 方法注记

- **残差① 的下一轮抓手**：taffy 直挂（parent 即 CB）对 rtl 双锚过约束已正确
  ignore-left（`r5372_direct_hang_rtl_double_anchor_ignores_left` pin 470），root/nested
  两 pass 恒左锚——修法 = CB direction 穿线（nested walk cb 元组扩 rtl 位 +
  root pass 查根元素 style），过约束臂改按 CB direction 选边。与 taffy 直挂值
  对齐即自检。
- **残差② 不动**：all-auto 静态位 x +29（中间层 border-left）域在 taffy 静态悬挂
  基，未归因到写点；本轮修复后复测 +29 仍在（1.58% 主导成分之一）。
- 残差③修复后 replaced 兄弟案同步收敛（3.90→2.49）证两案同根（R5372 已判），
  replaced 残差 = ①同 rtl 组 + replaced 自身形态条目。
- jitter 面轮换谱系（R5374: -127 → R5375: 149/150/152/153 → 本轮:
  127/128/150/152 + dynamic-005）：`table-anonymous-objects` 家族 8 job 并行下
  的边际抖动 + 偶发外溢案——稳定交集比对 + 隔离复跑两件套足以洗清，无需逐案
  kill-switch 归因。
