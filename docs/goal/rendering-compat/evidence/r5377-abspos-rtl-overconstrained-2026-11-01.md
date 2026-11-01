# R5377 证据 — css2.1 §10.3.7 过约束臂按 CB direction 选边（rtl CB 忽略 left）

日期：2026-11-01（R5377 轮；基线 = main@517ccefda，R5376 轮稳定交集 **755F**）

## 0. 结论速览

| 案 | R5376 后 | R5377 后 | 定谳 |
|---|---|---|---|
| `css/CSS2/csswg-issues/submitted/css2.1/abspos-non-replaced-width-margin-000.xht` | 1.58% | **1.09%** | 残差①（rtl CB 过约束 ignore-left）收口；残差②（all-auto 静态位 +29）主导，案仍 F |
| `css/CSS2/csswg-issues/submitted/css2.1/abspos-replaced-width-margin-000.xht` | 2.49% | **1.95%** | 同域同收口（R5372 已判同根） |

corpus 同树 A/B 三跑（基线 RUN0 760F / 修复树 RUN1 758F、RUN2 756F，稳定交集
**754F**）：FIXED=0 REGRESSED=∅——REGRESSED 候选 6 案（table-anonymous-objects-
151/152/153 + intrinsic-percent-replaced-dynamic-007 + insert-inline-middle-003 +
conic-gradient-repaint）隔离复跑全 0/3 F = 并行抖动面。稳定集对 R5376 755F 唯一
差异 = table-anonymous-objects-150 缺席（家族轮换成员，本轮双跑均过，非归因）。
真通过-可信 10365（16592 分母，与 R5376 的 10367 同抖动构成域）。

## 1. 归因（R5372 残差①收口）

§10.3.7 过约束规则（left/margins/width/right 全 definite 方程不可满足，或双 auto
margin 归零后仍不可满足）：ltr CB 忽略 `right`、**rtl CB 忽略 `left`**。root/nested
两 pass 对 `left` definite 一律左锚 → rtl 的 nothing-auto 组落 left+ml（WPT adir 组
spec 496 = 700−8−19−177，ZW 70 = 50+3+17）；taffy 直挂已按方向选边（R5372 pin 470）
——嵌套/根路径与直挂劈叉。方向来源是 **CB 自身** direction：WPT 页 `adir`（CB rtl）
组换边，`sdir`（static 中间层 rtl）/`edir`（abs 自身 rtl）组不换边。

## 2. 修复

- `rtl_overconstrained_right_anchor` helper（与 R5376 `solved_auto_margin_left` 并列）：
  返回 `Some(CB − right − mr − 盒宽)`（盒 border 缘 x，相对 CB 原点）。门 =
  `width` definite（inset-stretch 域不选边）+ `right` definite +（双 definite margin
  ｜ 双 auto 负 slack——归零后过约束）；单 auto 已被 R5376 方程精确解 → 方程成立
  不属过约束 → None 维持左锚。
- 方向穿线：nested pass cb 元组 4→5 位（递归建 child_cb 时查 positioned 子
  computed `direction`）；root pass 新增 `cb_is_rtl` 参数（engine.rs 调点查根元素
  style）。root/nested 左锚写点同臂 `unwrap_or(px + ml_used)`。
- 无新增 kill-switch（nested 臂整体有 `ZW_ABSPOS_NESTED_CB`；臂窄——rtl + 全
  definite inset 在 corpus 为 rare 群，与 R5375/R5376 裁决同口径）。
- taffy 直挂对齐自检：嵌套 rtl nothing-auto pin 期望 470 与 R5372 直挂 pin 同值。

## 3. 门禁

- corpus 同树 A/B 三跑：稳定交集 754F 持平，FIXED=0 REGRESSED=∅（jitter 隔离复跑
  6/6 洗清）；两目标案 diff 收敛仍 F（残差②未清，同 R5372/R5376「760F 持平、diff
  收敛」先例）。
- layout-engine lib：**2325P/0F**（+9 `r5377_abspos_rtl_overconstrained_tests`：
  嵌套 rtl nothing-auto 右锚 470（bug 复现主 pin，旧口径 20）/ widewidth 负解 −37 /
  双 auto 负 slack 右锚 −18 / 双 auto 正 slack 维持居中 246 / right:auto、width:auto
  左锚守卫 / sdir+edir 不换边 20 / ltr CB 左锚不变 20 / root-CB rtl 右锚 530）。
  R5372 7 pin、R5376 6 pin 全过。既有 3 个直调测试点补 ltr 参数（行为不变）。
- workspace 腿：**8960P/1F**——1F = `ua_display_li`（D4 既有账，R5372–R5376 同位同
  签名）；`compositor_gpu_*` ×3 跳过 = 对端域环境账（见 §4，净树实锤）。renderer
  lib 365P/0F；quickjs 腿 script-sandbox/webview/demo/runner 全绿。
- clippy `--workspace --all-targets`（guarded）0 告警 + fmt 清零。
- product-smoke：struct-check PASS（welcome + article @320 双面）。
- bench-gate：定向 zero-layout-engine 静窗（1min 0.41、零构建进程）**GATE PASS
  （25 指标全预算内 NEW=0）**。前序 8 分钟负载排空轮询（9.74→0.41）。

## 4. 对端碰头记（非本域，全部 IPC/fd 形态，main@517ccefda 净树复现）

| 套件 | 案 | 签名 | 归属 |
|---|---|---|---|
| apps/compositor frame_flow | compositor_gpu_dmabuf_browser_import / texture_export_dma_buf / partial_dirty_draws_glyphs（3F 稳定） | `SCM_RIGHTS 未携带 fd` | apps/compositor + zero-protocol（4752a00df fd 信任收口面） |
| apps/browser bin | default_actions_work_without_javascript（1F，净树复现实锤） | 表单 name 值丢失 + `ipc reader terminated: 读取帧头失败` | apps/browser |
| apps/browser tests | bookmark_context_menu_open_in_new_tab / gpu_compositor_path_dispatches（跑间 1F↔3F 波动） | 同上 IPC 族 | apps/browser |
| tests/integration (quickjs) | image_batch_e2e concurrent_in_flight（1F） | renderer publish/fetch IPC | apps/renderer + protocol |

与昨日 R5376 轮 make test（同 commit 1F）相比环境面劣化：全日重 corpus/构建负载
后 IPC 通道族稳定/波动失败。净树复现 → 与 R5377 改动（纯 layout 算术）无关。
apps/* + zero-protocol 归对端流，按碰头条款记账，待对端流核证。

## 5. 方法注记

- **viewport pass 同病第三写点未动**：`adjust_absolute_pct_to_viewport` 全 definite
  过约束注释明示「（LTR）忽略 right」左锚。ICB direction = 根元素 direction
  （css-position-3），同构修法可行；本轮按 R5372 记录口径只修 root/nested 两 pass
  （记录在案，corpus 若现 viewport 级 rtl abspos 过约束案再启动——保守面：viewport
  pass 面最大，且双侧自洽案（learning #33）下盲改有翻绿案风险）。
- **垂直轴无对应缺口**（learning #29 对应面审计）：§10.6.4 垂直过约束恒忽略
  `bottom`（无方向依赖），现有 top 锚臂（top definite → y=top+mt）即正确口径，
  无 vertical 写入模式需要审计。
- 残差② 不动：all-auto 静态位 x +29（中间层 border-left，taffy 静态悬挂基域）——
  本轮后 non-replaced 1.09% 的主导成分，仍待独立归因轮。
- R5376 方程解序与过约束选边的臂序：helper（单 auto 精确解）优先于 rtl 选边臂——
  方程可满足的组永不换边，两轮修复叠加语义 = §10.3.7 完整解序。
