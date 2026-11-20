# 刀 optical（font-optical-sizing/kerning/variant-numeric 三性）A/B 电池收账

**2026-11-20 · 提交 `413730881`（feat(engine): 刀 optical 三性）· fonts +27 零回归**

> 姊妹主账：[evidence knife-optical-gcs-gate](evidence/2026-11-20-knife-optical-gcs-gate.md)（收账主笔，哨兵 SET-IDENTICAL 1051/840/958/12024/6244 已复核）。本文件为补充：M2/census 原始账、域拆提交边界、corpusexp 撞窗注记与过程事故 provenance。

## 电池配置

- 脚本：`.acceptance/wptfyi-css/knife-optical-battery.sh`（链 `knife-optical-chain.service`，承 knife-ffix2 配方：test-guard 包裹 + md5 census + trap 归一）
- b0 = HEAD `f89e48406` + 全部 ride（face 不入树）；b1 = 同 + face patch（`knife-optical-face.patch` 3 文件：animation.rs optical 应用臂 / transition_registry.rs DISCRETE_TRANSITIONABLE +1 / apply.rs 白名单 69→72 + entries + 分派臂）
- M2：b0-runner md5 `367c1066ee197030754838168f4b8744` ≠ b1-runner md5 `b17abf3e879fab67a5cd46c747dc2b24` ✓
- ride census：RIDE 18 文件，S1（b0 窗）/S3 全零扰动 ✓；S1/S2/S3 口径见脚本注释
- 时间线：b0 build 10:20:33 → b0 六腿 10:22:14-10:39:52 → b1 build 10:21:18 窗 → b1 六腿 10:39:52-10:56:45 → RECONCILE 10:56:45，EXIT rc=0，trap face 留树（已随 413730881 落库）

## 主靶 fonts：gained=27 lost=0 regressed=0 pages_gain=4

| 页 | b0 P | b1 P | Δ |
|---|---|---|---|
| css-fonts/discrete-no-interpolation.html | 532 | 546 | +14 |
| css-fonts/parsing/font-kerning-invalid.html | 0 | 2 | +2 |
| css-fonts/parsing/font-optical-sizing-invalid.html | 0 | 2 | +2 |
| css-fonts/parsing/font-variant-numeric-invalid.html | 0 | 9 | +9 |

- 预算 +29 中 `font-variant-numeric-computed` 2F 未动（双臂恒，级联单值存储装不下组文法的已知残面，挂账另刀），实得 +27
- discrete-no-interpolation 14F：8 expected "none " got "auto " + 6 反向，全数兑现

## 哨兵（全零差异）

- transitions / animations（registry+应用臂变更面）/ pva / typed-om / shapes 六腿 b0-b1 逐子测比对零差异（ab-compare 无输出行即零差异），TOTAL 口径下 regressed=0 pages_loss=0

## 负载窗注记

- cxprobe r3（corpusexp 库存探针，10:21:44-10:26:38）与 b1 build + fonts/transitions b0 前段并发（census 空隙竞态，双脚本互盲）；fonts/transitions 实测零 REG，无需 solo 复跑仲裁（ffix2 240s 高墙先例未触发）

## 提交集与域拆

- 提交 `413730881` = face 3 文件 + prep 9 文件（style-system types/computed_style/matcher/apply_advanced/registry/inherit/shorthand/default_impl + engine js_dom_bridge/computed_style gCS 回显臂），恰 12 文件 +165/−11，`git commit-tree` plumbing 落库（共享 index 有兄弟流 staged docs，pathspec 边界以 blob OID 锁定）
- 排除（留树归原 lane）：animation.rs @1546/@5690（SH-A1 clip round）、engine computed_style.rs @6/@3495/@3611（SH-A1 clip）、apply.rs animation-range 两处（ATP-animrange 流）、css-parser×4 + clip_interp + border_shape（SH-A1）、part05.js + es_module + part115（TEV face）
- 静态面注记：prep（gCS 回显臂+类型）在 ride 态已兑现 optical-valid/computed 2+2、inheritance 2 等静态绿（双侧同含不入 A/B 账）；随本提交落库后下周全量兑现

## 过程事故（透明记录）

- 收账 blob 手术时脚本误将 `git apply` 指向 worktree 真实路径，`git checkout --` 把 animation.rs / apply.rs 打回 HEAD 抹掉共享 WIP（sha1 lane 10:48 编辑、ATP 流 ride hunks）；已按快照终态复原（animation.rs 10:49 快照原样回填；apply.rs 以 `wip-apply-post-face.patch` HEAD 基 4 hunks 经 `patch -p1` 在 scratch tree 重建，vs HEAD 102 行=hunk 账目精确吻合）。详情与 provenance 见 `.acceptance/wptfyi-css/NOTES-knife-optical-prep.txt` 事故节
