# EV · 刀 CS-1b（corner 形法 C4+C1 双臂）— 2026-11-20 rally-cont

靶面 = css-borders corner-shape 残面 46F 五缺陷类（PREP-knife-cs1b-residual.md），
本轮落 C4（superellipse 参量内空白 17F）+ C1（corner mega 组号/角号解耦 14F）。
face = 2 文件：css-parser `parse_extended_border.rs`（`corner_top_level_tokens`
括号深度感知顶层切分 + canonical 参量 trim）+ engine `css_style_declaration.rs`
（`corner_family_expand_json` enumerate 解耦——k 曾同时充当组号与角号，1 组输入
四写 corner 0、TR/BR/BL 永不写＝corner-computed 14F 根因）。双钉测：
css-parser `corner_shape_superellipse_inner_whitespace_forms`（4 passed 实跑核数）
+ engine `corner_family_expand_tests`（mega 1 组 8 对/3 组镜像 2 passed，
16:44 实跑 `cargo test -p zero-engine mega_ -j 4` rc=0）。

## 电池（zw-cs1b-battery2，17:47-17:50）

判别全过：RIDE 1 文件（es_module.rs 他流 WIP，S1/S2/S3 三点 md5 零扰动）；
B0 BUILD 1m15s（b0-runner md5 `810b8a9a`＝与 APC 电池 b1 同源同值——两者源态
同为 HEAD(已提交 APC face)+es_module WIP，cargo 确定性互证）；B1 BUILD 1m21s
（b1-runner md5 `d1dbec45`）；FACE-MARKER `corner_top_level_tokens` b0=0/b1=1；
M2 md5 异。8 腿 17:50:09-17:50:45（rc=1＝含预期失败，产物完整）。

RECONCILE raw：corner-computed 8→22 (+14)、corner-shape-valid 217→233 (+16)、
corner-shape-computed 34→33 (+1/−1)、corner-shape-interpolation 152→135 (−17)
＝gained=30 lost=18。断言数双侧全对称零 ASYM 零 phantom。

## 归因（收账前 solo 同 bin 探针闭合，FONT-1b 四象限法）

| 页面 | 电池 | solo b0 | solo b1 | 判定 | 入账 |
|---|---|---|---|---|---|
| corner-computed | +14 | 8/22 | 22/22 | 与电池同值＝确定性 C1 | **+14** |
| corner-shape-valid | +16 | 217/241 | 233/241 | 与电池同值＝确定性 C4 | **+16** |
| corner-shape-computed | +1/−1 | 32/38（6F） | 35/38（3F） | +1 确定性（`superellipse( -1)` b0 电池+solo 双挂/b1 双过）；−1 flake（见下） | **+1** |
| corner-shape-interpolation | −17 | 133/478 | 150/478 | solo 反向 +17（b0 133↔152、b1 135↔150 同 bin 双跑翻转，solo b0-only 64/b1-only 81）＝FONT-1b 同型 run-to-run flake 域 | 0（双向不入账） |

- corner-shape-computed −1 判 flake 证据：solo b0 比 b1 更红（6F vs 3F），b0 solo
  含电池 b1 的同签名挂项（`inline-start 'round scoop'`、`inline-start
  'superellipse(-1) superellipse(2)'`）且额外挂 `block-start 'round scoop'`/
  `block-end 'superellipse(4)'`/`inline-end 'superellipse(-1.5) superellipse(3)'`；
  电池 b1 挂 `block-start` 而 solo b1 过、solo b1 挂 `inline-start 'round scoop'`
  而电池 b1 过——同 bin 跨跑挂项集不稳＝现存页面级 nondeterminism。
- **lost=18 全域不入账，归因后净 +31 零 lost。**

## 残面与新刀候选

- **CS-1c 候选（现存 bug 非 face 回归）**：corner 逻辑→物理角 computed stale-slot
  串扰——`corner-inline-start-shape`/`corner-block-start-shape` 写入后 computed
  slot0 读到旧物理角值（页 ：42 `corner-top-shape: superellipse(4)` 陈值泄漏，
  computed_style.rs :550 `corner-left|inline-start → corner(0),corner(3)` 读
  state 槽；声明更新未落 slot 0 的窗口期被 gCS 读到）。b0 b1 皆现、solo 更红，
  归因窗内不计入 face 账；四签名（block/inline-start/inline-end 各 'round scoop'
  与 superellipse 组合形）已在 b0 solo 6F 全集取证。
- C4 预期 17F 落 16：余 1F＝valid 页混合形某臂（`superellipse( -1) round` pin
  canonical 已过、valid 页该臂仍挂），C3/C2/C5（4F+3F+8F）按 PREP 序后续刀。
- corner-shape-interpolation −17 域：superellipse 插值数学期望面（中点
  `superellipse ( 0.36 )` 类）＝cs1-interp-math 挂账域，非本轮面。

## 账头

master.md 状态行：本轮局部账 +9443 → **+9474**（+31＝C1 +14 + C4 +16 +
computed +1）；电池 raw gained=30/lost=18 与归因口径并记。
