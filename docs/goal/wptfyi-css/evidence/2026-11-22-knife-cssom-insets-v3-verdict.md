# 刀 CSSOM-INSETS-V3 收账证据（2026-11-22）

## 刀面（r2 终版＝提交字节，patch md5 `ed1c8a67f20c01ffe6a5aaaca8973a59`，commit 104db1607）

两文件（V2 残池根因① RTL 过约束锚定的引擎布局臂）：

1. `crates/layout-engine/src/engine/abspos.rs`
   - `rtl_overconstrained_right_anchor` 扩 width:auto 非 replaced 负 slack 形（css-position-3 §abspos-non-replaced）：width:auto 方程解 `width = CB−left−ml−mr−right` 为负、钳 0 后（left+ml+mr+width+right）定和≠CB → 同属过约束，rtl CB 忽略 left 按 right 锚（盒右缘 = CB−right−mr，x+w 恒等保 right used 正确）；正/零 slack 方程成立维持左锚（inset-stretch 域既有口径不动）。replaced 全 definite 过约束（R5378）与非 replaced 正 slack（R5376/R5377 既有）两臂零改动。
   - root-CB pass 与 nested-CB pass 的 left 定位臂入口 `resolve_abspos_calc_inset` → `resolve_abspos_inset_px` 统一解析：%/calc/px 同通道，使 rtl 过约束选边覆盖 % 形（V2 后 % 值在 left 臂仍走 Calc-only 通道的残留缺口）。
2. `crates/layout-engine/src/engine/tests/r5377_abspos_rtl_overconstrained_tests.rs`
   - 钉测 3 新案：width:auto 负 slack 右锚（650−300−400−17−19 = −186 → 钳 0，x = 650−400−19−0 = 231）；四边 % 过约束右锚（left/right 100%，x = 0）；ltr CB 同 % 形对照左锚（x = 650）。既有 width:auto 正 slack（603 ≥ 0）左锚案断言口径同步（注释更新，断言不变）。

## 电池谱系（2 轮，判读全同值）

| 轮 | patch md5 | 基 face | 结果 |
|---|---|---|---|
| r1 | `4256820e…`（fmt 前） | 2983cdf47 + face（pre-KC-1，12:38:33 BUILD → 12:48:09 GATE PASS） | F2P=30 P2F=0 同值（`results-knife-cssom-insets-v3/` 首轮，log `…-battery-122132.log`） |
| r2 | `ed1c8a67…`（rustfmt 终版） | 0f184e0bf + face（post-KC-1 rebase 后，14:26:51 BUILD → 14:35:47 GATE PASS） | **GATE PASS 终版**（`results-knife-cssom-insets-v3/`，log `…-battery-142903.log`） |

轮间事件：上 session 12:51:26 限额终断（r1 GATE PASS 已落但刀未 commit）；本 session rebase（8 commit 并入，blob 恒 `b7c7a545d`/`ef1d77922`）、门禁 fmt 检出 abspos.rs `let ml_px` 折行偏差（r1 面未过 rustfmt）→ rustfmt 后 patch md5 变更＝判读域失效重跑（V2 r2→r3 同教训再现），r2 以 fmt 终版字节重放并复现同值。

配方：`knife-cssom-insets-v3-battery.sh`（chmod 400；PATCH_MD5 字节绑定 `ed1c8a67` + patch 双向校验 + 树面白名单 2 文件 + 前置静默闸 + b0=HEAD 剥面/b1 重放 + M2 双二进制 md5 异硬门 + test-guard 包裹 build per-proc 6/total 14/1800s、leg per-proc 4/total 8/3000s CASE_WALL_S=60；trap 归 b1 态）。

## r2 判读（14:26:51 PREFLIGHT-QUIET → 14:35:47 GATE-DONE，rc=0）

- M2 双二进制 md5 异：b0 `0e11b6971f45be22e2d85ead7bbbe728` ≠ b1 `449c54d4be647a00db2aa7c4cf940b84`。
- 主靶 cssom：keys 3459 恒，P 2263→2293，**F2P=30 P2F=0 new=0 gone=0 excluded=1**（css-values random-computed 已知 flake 排除域）。
  - getComputedStyle-insets-absolute：F2P=18（42→24F）
  - getComputedStyle-insets-fixed：F2P=12（96→84F）
  - insets-relative/static/sticky/sticky-container-for-abspos：零漂移（F2P=0）
- 四哨 SET-IDENTICAL：css-position（0 键）/cssom-view（565）/css-sizing（3441——KC-1 +89 已在基，V3 对 sizing 零位移）/css-values（b0 991/b1 990＝random flake 单键双向域，排除不计）。
- GRAND GATE：GATE-PAGE-F2P absolute+fixed=30（expect ≥1），INSETS-V3-GATE-PASS。

## V3 兑现恰集（30 案）

RTL 过约束锚定（v2 判读根因①）中「horizontal-tb 内层」组全翻：absolute 页 ltr/rtl × 百分比 absolutize/overconstrained 形 18 案＋fixed 页同族 12 案。钉测三案与 WPT 主判据「Percentages absolutize the computed value when overconstrained」同型。

## 残池（gate 两页 138→108F，家族定谳）

| 页 | 残 F | 家族 |
|---|---|---|
| absolute | 24 | both-auto used-value 族（`If opposite sides are 'auto'…`）：inner vertical-lr ×12 ＋ vertical-rl ×12（inner horizontal-tb 组已全过） |
| fixed | 84 | 垂直写向 nested 域：both-auto（lr/rl 各 12）＋ pct/calc/overconstrained 绝对化（vertical-rl 36）＋ single-auto used（vertical-rl 24）——vertical-rl 块轴向右起坐标换算为主根因 |

挂后续刀：fixed nested 垂直写向（vertical-rl/lr）坐标换算臂 ~72F ＋ both-auto used-value 垂直写向回读臂 ~36F（与 USEDECHO 刀的 gBCR echo 垂直写向域衔接）；static/relative/sticky 族在 gate 页外维持前账（relative 180F 等静态位域未入本轮判读）。

## 提交前门禁序列

- fmt：workspace 既有债维持（非本刀面；`results-gate-maketest-insets-v3/fmt-check-head-debt.diff`），face 2 文件 rustfmt 后零 diff（r1 检出 `let ml_px` 折行 → 修正 → patch md5 变更 → r2 重跑闭环）。
- clippy（14 crate quickjs 配方同 V2）：错误文件集与 V2 基线恒同（engine 4 文件既有债，`CLIPPY-BASELINE-SAME`）。
- make test 配方腿（`gate-maketest-insets-v3.sh`，test-guard 包裹）：build-standalone rc=0；test-workspace rc=101 但失败集与 V2 基线恒同（`WORKSPACE-FAILSET-BASELINE-SAME`：mis_ref_page_still_wraps / nowrap_page_renders_single_line 2 案既有债）；results 聚合 diff 纯 timing 抖动 ＋ 上游新增 2 个 1-test ok 二进制（并行流 docs commit 随带测试），FAILED 行恒同。
- 提交门禁 lei-pre-commit-guard：PASS（0 critical / 0 warning / 0 advisory）。
