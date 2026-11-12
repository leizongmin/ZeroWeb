# R5510 轮次证据：表内盒 content_height 补写收窄到 r109 载体域——单格 tr 78→60，tao-087/088 真回归由 corpus A/B 拦截后门控收口

日期：2026-11-13
轮次：R5509 下一步②（挂账①「单格态 tr 高 78＝cell 60＋一行 18」归因修复）
案：单格态（唯一 cell 含 block-in-inline 拆分载体；corpus 无专案，真渲染不可见域）
常驻锚：`crates/layout-engine/src/engine/tests/r5510_table_row_backfill_invariant_tests.rs`（3 test）
修复：`crates/layout-engine/src/table.rs`（position_cells 行盒 + update_row_group_positions 行组盒）、
  `crates/layout-engine/src/table/table_vertical.rs`（position_cells_vertical 行盒/行组盒孪生同款）

## 一、缺陷签名与归因

R5509 收账挂账①：单格态 tr 高 78＝cell 60＋一行 18（stash 新旧同值非 R5509 引入），
真渲染不可见（v2 单格 cell 画 59px 正确）。本轮证伪跑复现签名：
- 单格锚：`row must equal cell height, got row=78 cell=60`
- 双行锚：`table height must equal sum of rows, got table=144 rows=78+24`（行 0 膨胀、
  行 1 被多推、表高复利）

根因＝`backfill_r109_anon_block_heights`（engine/postprocess.rs）arm② 的 delta-add 依赖
`height = content_height + frame` 不变式。行/行组盒高由表格 pass 直接写入而 content_height
留 taffy 旧值 → 行高吃 r109 拆分载体陈旧值时（真范围要等 engine 步骤 12/12.1 回填），
`delta = max_bottom − content_height` 叠加在已含真值的 height 上：tr 60+18=78，行组/表/body
逐级复利（前 session 探针复利 96）。行盒无自身框（ZW 网格模型 collapse 边框在 paint 侧
半带、separate 边框在 cell 上），不变式应为 `height = content_height`。

## 二、首轮无条件补写 → corpus A/B 拦截真回归（本 guarding round 的主事件）

首轮实现＝4 写点无条件 `content_height = height`（水平行/行组 + vertical 孪生），锚 3/3 绿、
全包 2657P/0F、reftest 725/725、product-smoke 6/6（welcome 17.23% 基线同值）。

**corpus 全量 A/B（16594，基线 /tmp/r5509/new-fails.txt 684F——HEAD 与 R5509 测量树
05c3e2b4f 代码零差异已核）翻出 tao-087/088 确定性真回归**：

| 判据 | base 二进制 | 无条件修复二进制 |
|---|---|---|
| tao-087 solo×3 | ✓ 0.00% ×3 | ✗ 2.68% ×3 |
| tao-088 solo×3 | ✓ 0.00% ×3 | ✗ 3.49% ×3 |
| 双格参照（tao 族其余/锚） | — | 全绿 |

**机制诊断**（PNG 像素带 forensics + 临时盒树 dump/表格 pass 探针，均用后撤净）：
- ref 臂（真 `<table>` 页）base/new 逐像素相同——真表路径不受影响；test 臂三条文本行带
  全移位（y=116..145/152..191/198..228）。
- tao-087/088（infer-cells-2）畸形表生成：`display:table-row` span 行盒含 **unwrapped
  Inline 子**（y=−9.6 基线上悬）与嵌套 TableRow——行子树 max_bottom（138.4）≫ 网格行高
  R（46），taffy content_height 陈旧 −0。
- base 的 arm② delta = maxB − (−0) = 138.4 行膨胀 + **兄弟位移臂**把 unwrapped Inline
  推到与参照网格文本对齐的 y——**arm② 恰为其绿面承担者**（guidelines #29「绿案可能是
  另一条回填/位移 pass 间接撑住」活例）；无条件补写后 delta 缩为 maxB − R，兄弟位移
  差一行高，文本带移位 → 与参照失配。
- 深修（改 backfill 兄弟位移语义）被否：R1163 broad 臂 welcome +12.57pp 先例，blast
  radius 不可控；畸形表匿名 cell 生成本身是独立域（不在本轮范围）。

## 三、终版修复：4 写点门控补写（收窄到已证域）

| 写点 | 门控 |
|---|---|
| table.rs 行盒 | `(content_height − row_height).abs() > 0.5 && subtree_has_r109_split(row_box)` → `content_height = row_height` |
| table.rs 行组盒 | 同款（`new_h`） |
| table_vertical.rs 行盒 | 同款（`row_inline_extent`） |
| table_vertical.rs 行组盒 | 同款 |

门控语义：①taffy ch 与网格行高一致时补写为 no-op（省载体子树遍历，常见路径零开销）；
②不一致 ∧ 含 r109 载体＝R5509 已证域，补写后 arm② max_bottom ≤ 新 ch 不触发，tr=td 精确；
③不一致 ∧ 无载体（tao-087 域）＝维持 base 世界（arm② delta 原样，绿面自洽不扰）。
`subtree_has_r109_split` 为 table.rs 私有 fn，table_vertical 经 `use super::` 复用。

写点覆盖审计（guidelines #29）：table.rs/table_vertical.rs 全部直接 `.height = ` 写点逐一
核对——root/表盒臂 height 与 content_height 成对写或由 content_height 推导（不变式自洽），
cell 臂下方同步 content_height（既有注释），本轮 4 写点＝行/行组域全部直写点。无遗漏臂。

## 四、验证链（终版门控树）

| 门禁 | 结果 |
|---|---|
| 常驻锚 3 test | 3/3 PASS；R5509 锚 2/2 PASS |
| 锚 stash 证伪 | 基态 1P/2F 签名复现（row=78 cell=60；table=144 rows=78+24），因果成立 |
| tao-087/088 solo（门控树） | ✓ 0.00%（真回归消除） |
| layout-engine 全包 release | 2657 passed / 0 failed（＝R5509 账面 2654＋新增 3 锚，恰集核算） |
| make reftest | 725/725 |
| make product-smoke | 6/6 PASS，welcome 17.23%（R5501 基线同值）struct 全 PASS |
| corpus 全量 A/B | 686F vs 基线 684F；mover 全噪声定谳见 §五 |
| clippy --all-targets -D warnings | 净 |
| rustfmt --check | 净 |
| 定向 bench-gate | 见 §五（busy-abort 后静窗重跑） |

## 五、corpus A/B（16594，门控树 vs 基线 684F）

- RESOLVED=1：background-image-gradient-currentcolor-radial-repaint.html——在册噪声族
  （R5509 批跑噪声同案，本轮向绿翻转），solo ✓ 0.00%。
- REGRESSED=3：tao-147/152/154——在册 tao-145-154 互翻族成员（R5494 起多轮在册），
  solo×3 双态协议 9/9 全 ✓ 0.00% 定谳批跑噪声。
- 真态：FIXED=∅（单格 78 缺陷无 corpus 专案，真渲染不可见——锚承载）、REGRESSED=∅
  （tao-147/152/154 solo×3 全绿）；净 +2F 全为批跑噪声（重载窗跑批，load 226-246）。

## 六、外 discoveries

- **门控纪律新证**：布局不变式「补写」类修复必须先过 corpus 全量 A/B 再定形态——首轮
  无条件实现锚/包/reftest/smoke 四门全绿仍被 corpus 拦出 2 案确定性真回归（arm② 兄弟
  位移的隐性消费方）。#29（绿面另一 pass 承担）+#33（self-source 自洽）联合活例，
  收窄门控是比深修共享 pass 更安全的落点。
- bench-gate 自带 busy-abort 守卫（loadavg 1min>12 拒测）——本轮 load 246 窗触发，
  产出不可信报告的路已被门内封死；静窗重跑收账。
- 前 session pkg/reftest 日志（debug 2492P / reftest 725）与本轮 release 2657P 为不同
  profile 口径，均 0F。
