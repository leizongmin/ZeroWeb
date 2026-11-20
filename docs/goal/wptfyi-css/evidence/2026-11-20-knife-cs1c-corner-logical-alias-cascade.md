# 刀 CS-1c：corner 逻辑 shape longhand 级联序消解（2026-11-20）

## 目标

css-borders-4 corner 逻辑 shape longhand（corner-start-start-shape /
corner-start-end-shape / corner-end-end-shape / corner-end-start-shape）与同槽
物理名（corner-top-left-shape 等）的 computed 竞争此前未走 css-cascade-4
§overlapping-aliases 的 CascadeOrder 消解，而是两臂都进胜者 map 按 HashMap
迭代序裁决——确定性 hasher 下每键集固定但任意，违反后声明胜。driving:
corner-shape-computed :46/:48/:49。

## 根因（三探针 + 代码考古）

- cs1c-probe1：基线 census，corner-shape-computed b0 形态序噪声 4-6F 漂移。
- cs1c-probe2（A-F 槽竞争隔离）：同页内 4 槽逻辑-物理竞争，槽 0 物理**恒胜**、
  槽 1-3 逻辑**恒胜**，与声明序无关——胜者 map 迭代序裁决的直接实证；跨进程
  （probe2 vs probe3 旧 bin）同输入不同结果，页面上下文/进程相关。
- 代码考古：4 个逻辑名既不在 `is_logical_property_name`（不进
  deferred_logical 延迟臂）也不在 `logical_alias_physical_slot`（无物理映射）
  → 主循环与物理名竞争。

## 修复（face，2 文件 3 hunk）

- `crates/style-system/src/inheritance.rs`：
  1. `#[used] static CS1C_FACE_MARKER`（ASCII 代码字面落 binary）；
  2. `is_logical_property_name` 收录 4 个 corner 逻辑 shape longhand →
     进 deferred_logical 延迟臂，按 CascadeOrder 与物理槽胜者消解
     （R4449 overlapping-alias 既有路径复用）。
- `crates/style-system/src/property/apply_advanced.rs`：
  `logical_alias_physical_slot` 增 corner 分支物理 longhand 映射
  （start-start=TL、start-end=TR、end-end=BR、end-start=BL，
  horizontal-tb ltr 同 apply 臂既有约定；FIXME(cs1-writing-modes) 垂直
  书写模式另账）。

## 电池（results-knife-cs1c，链 knife-cs1c-chain-r1.log）

- 发射：systemd `zw-cs1c-chain.service` 19:08:28，TREE-COMPILABLE round=1
  （clipshape fork WIP 修复后）；B0 BUILD 19:08:29（1m07s）；B1 BUILD
  19:09:37（1m02s）。
- FACE-MARKER：b0=0 b1=1 ✓；M2：b0 d48d7265… / b1 4023e796… 异 ✓。
- 腿：borders masking shapes ui typed-om × b0/b1，test-guard
  --per-proc-mem 4 --total-mem 8 --time-limit 3000，CASE_WALL_S=60。
- RECONCILE：（待填）

### 臂间差登记（rides 不对称，收账分解）

apply_advanced.rs 为共享文件，battery patch 含 1 外流 hunk；另有 1 ride 文件
在 b0 build 窗内被改（S1/S2/S3 census 实证）：

1. clipshape fork：shape-outside computed 门加 `ClipPathValue::Url(_) =>
   return false`（apply_advanced.rs @@ -3107，其 CLIP-URL parse 枚举消费面）
   ——b0 剥除/b1 在场，shapes 腿 shape-outside url() 相关页 delta 归因
   CLIP-URL 不入本刀账。
2. clipshape fork：parse_extended_border.rs 19:09:32 保存新版（S1 efde5c →
   S2/S3 e0eb50，S2=S3 稳定）——b0 bin 含旧版/b1 含新版，borders 腿额外页动
   按页名分解（corner-logical 签名页 vs parse 面）。
3. 其余 rides（types.rs、computed_style.rs、animation.rs、border_shape.rs、
   es_module.rs、apply.rs）S2=S3 零扰动 ✓。

### 页级 delta（RECONCILE 19:19:54，gained=53 lost=11）

| 腿 | 页 | P | 初判归因 |
|---|---|---|---|
| borders | corner-shape/corner-shape-computed.html | 33→38 (+5) | **本刀 face**（4 逻辑竞争臂翻正，与 probe2/3 预测签名一致） |
| borders | corner-shape/corner-shape-interpolation.html | 134→123 (−11) | 待定谳：子测级 gained61/lost72（物理名插值、superellipse 期望值形态），疑似 clipurl parse 层；b2 孤立臂分解 |
| masking | animations/clip-path-interpolation-001.html | 358→394 (+36) | **clipurl face**（b0 parse v1/b1 最终形臂间差，其披露归因约定） |
| masking | clip-path/animations/clip-path-interpolation-discrete.html | 135→147 (+12) | **clipurl face**（同上） |

- shapes 腿零位移：shape-outside-010/011（url 面，clipurl hunk 风险页）双臂
  恒 1F，其 apply_advanced @@3107 hunk 实测零影响。
- ui/typed-om 哨兵零位移。

### clipurl fork 双向污染与处置（其披露 NOTES-clipurl-fork-contamination-cs1c-window.md，已采纳）

1. 其 7 文件 face 与本刀共享 apply_advanced.rs；其电池 patch 含本刀 corner
   映射 hunk（grep 实证），剥/回加双向扫过——其电池窗（19:23 发射，串行
   让位）b1 与本刀 b1 全等 → borders b1 页级 P 逐页比对＝免费确定性交叉验证。
2. 本刀收账口径：masking +48 不入账（其 face 效果）；interpolation −11 待
   b2 孤立臂（现树留 clipurl 全 face、仅剥本刀 3 hunk）定谳——b2≈b1 则
   parse 层所伤（不入本刀账），b2≈134P 则本刀 face 所伤（须修后再收）。
   **终谳（22:2x 续开轮）：b2 免发射，−11 按 flake 域双向不入账**——其
   回执点 6（19:38）先期终证：cs1c-b1 vs clipurl-b1 同页翻转 69F→P/52P→F
   且含同语义断言双向各现（两 b1 仅差 OM-4 cursor shim、与 corner 页零机制
   交集＝行为等价构建），b0 基线自漂 ±20；±17~20 噪声带（probe2/3 迭代序
   机制）内 −11/−14 均不可归因任何 face hunk，双向不入账不记回归。b2
   strip patch 因树演化（clipurl 落库 + motion1 WIP）apply 失败
  （knife-cs1c-b2-solo-r1.log B2-STRIP-FAILED），按回执点 6 同判免重发。
3. commit 双向 hunk 拆分：本刀 knife-cs1c-commit.patch（3 hunk 零
   CLIPURL 内容）apply --cached；其回执（同 NOTES 回执节）已警告其整文件
   add 风险。

## 收账（2026-11-20 22:2x 续开轮落账）

- 本刀净 delta：**css-borders +5**＝corner-shape-computed 33→38 恰集
  （probe2/3 预测签名一致）；interpolation −11 flake 域双向不入账；masking
  +36/+12 让渡 CLIP-URL 账；shapes/ui/typed-om 哨兵零位移。局部账累计
  +9807→+9812。
- 孤立复核（solo 复跑）：免发射——同页噪声带已由双电池互证＋行为等价构建
  同语义断言双向翻转终证（clipurl 回执点 6）；b2 孤立臂 strip patch 因树
  演化 apply 失败（B2-STRIP-FAILED），按同判免重发。
- commit：face `5e330d1b8`（hunk 级 knife-cs1c-commit.patch 3 hunk
  apply --cached，零 CLIPURL/motion 内容卷入；pre-commit-guard PASS）。
- 已知域外残：corner-top-left-shape :23 supports 通道 'superellipse( -1)'
  C4 残（CSS.supports 解析缺口，非级联，本刀域外）；corner-block-start
  对逻辑竞争（CS-1b 面，FIXME 另刀）。
