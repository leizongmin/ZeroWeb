# R5470 证据档：c5525 双缺陷定谳与修复——taffy collapse-through 高度不传播 + §10.6.3 高度门翻默认 + 步 6 兄弟重锚（c5525 ✗2.90%→✓0.28%）

日期：2026-11-09。R5469 §2 残余定域接续——c5525 布局域残差根因链完整定谳并修复。

## 0. 轮次结论

- **双缺陷机制链定谳**：taffy 对「float 子 + inline 内容」容器给 collapse-through（行内内容对 taffy 块堆叠不可见 → h=0，且 margin 折链吃掉兄弟竖直 margin）→ 步 5 float walk 消费 taffy 陈旧帧 → 真实行盒高由步 6 `remeasure_text_with_float_exclusions` 物化但**不传播**——后续流内兄弟停驻旧帧位。签名 =「含 float 前驱之后兄弟竖直 margin 丢失」（c5525 p3-p6 累计上移 15/60/75/90px）。叠加 R5026 时代 default-off 的 §10.6.3 高度语义缺陷（普通块 auto 高含 float extent，误差 +15 与传播缺陷 −15 反号耦合）。
- **修复四件**（全部 `crates/layout-engine/src/inline_finalization.rs`）：
  1. `ZW_IFC_AUTOHEIGHT_BFC` 翻默认 on（§10.6.3：普通块 auto 高 = 行盒底，float 溢出不撑高）；
  2. 步 6 兄弟重锚臂：重测物化高度后，首个后继流内兄弟重锚到 `child.y + child.height + collapse(child.mb, sib.mt)`，其后兄弟随正差整体平移（只下推不回拉，保留 clearance/BFC 避让合法超额）；
  3. 行盒证据门：`text_height=0`（IFC 零行盒）时退回 max 旧口径——floats-149 暴露的行盒归属缺陷防御臂；
  4. multicol 子树门：BFC 门与重锚臂在 multicol 子树内双双禁用——floats-clear-multicol-001 暴露的列断流保守收窄。
- **corpus A/B（同二进制 16599，ON/OFF 双跑）**：692F/692F。solo ×3 甄别：真修复 c5525-fltmrgn-000（OFF ✗2.90% 恒 → ON ✓0.28% 恒）；floats-149、floats-clear-multicol-001 首轮真回归 → 收窄件修复后 ON solo ✓ 恒；movers 4 案（tao-127、intrinsic-percent-replaced-dynamic-007/008、insert-inline-in-blocks-n-inlines-middle-003）solo 双配置全 ✓0.00% 剔除。**净真回归 0**。
- **门禁**：内测 725/725 全绿；judgment test `r5470_float_remeasure_sibling_reanchor_tests` 2 案（min-a/min-f 形态）；layout-engine 全腿 2538P/0F；product-smoke 6 门 PASS/0 fail；bench-gate GATE PASS（见 §5）。

## 1. chromium oracle 三形态真值（最小页定谳）

最小页：body margin 0，div（w120 border1，fs15/lh15）内 p×3（margin 15px 0），p2 含 float span。

| 形态 | p2 内容 | chromium p2 h | chromium p3 y | 关键判据 |
|---|---|---|---|---|
| min-a | float 45px + 1 行（15px） | **15** | **84** | §10.6.3：float 溢出不撑高；float 带（15-60）叠入 p3 区，p3 文本绕排 |
| min-f | float 30px + 3 行（45px） | **45** | **114** | float 短于文本不参与；p3 = p2.y+45+15 |
| min-c | 无 float（对照） | 15 | 84 | 基线 |

chromium 仪器：`--headless=new --disable-gpu --dump-dom --virtual-time-budget=2000` + `</head>` 前注入 script 写 `document.title`（div>p rect）。ZW 修复后三形态 p2 h/p3 y 逐位一致。

## 2. taffy collapse-through 实证

c5525 test3 形态 p（float img + 3 行文本）taffy-raw 帧：h=0、p 自身 mt/mb（1em 折链）被吃——步 5 walk 读到 y 位缺整段行盒高；步 6 重测物化 p h=45 后 p4-p6 仍停旧位。零增长折叠（float-only 容器）= §8.3.1 合法塌穿不需修——重锚门 `|Δh| > 0.5`（高度物化）天然只修真增长块。

## 3. 修复件细节与首轮回归的二次归因

### 3.1 重锚臂初版方向性错误

先按 `child_growth` 平移后继兄弟 → min-a p3 过冲（69 vs 84）、c5525 p4 推过头（375 vs 300）。根因：taffy collapse-through 同时吃掉高度与 margin，δ 与 fold 亏欠不可分离叠加 → 重写为**目标位重锚**（push-only 正差），六案逐位吻合。

### 3.2 floats-149（首轮真回归 → 行盒证据门）

- 现象：`.block`（流内内容经 display:inline div 中间层）ON h=0 / OFF h=19 / ref 18（1px 在容差内）。
- 探针定谳：`.block` 第二遍 exclusion-aware IFC **lines=0**（行盒物化在 display:inline 中间层子树 span 层 h=19，不进本容器行流）→ `text_height=0`；`float_bottom=19` 来自**非 float 的文本 span 被 nested 臂收集 + 行盒高折返**——OFF 靠此巧合得 19，ON BFC 门丢弃 float extent 后塌 0。
- 修复：BFC 门加 `text_height > 0.0` 行盒证据门（零行盒证据不收窄，退回 max 旧口径）。**行盒归属缺陷**（display:inline 中间层行盒不进祖先 IFC 行流）单列在册，待修复后移除本臂。
- 注意：该臂是保守回退而非根因修复；R5446 时代注释已实证过同案塌 0（floats-149 div.block 绿条消失），当时修 nested float 补测。

### 3.3 floats-clear-multicol-001（首轮真回归 → multicol 子树门）

- 三配置三种死法：双开 container h=20（BFC 门）+ clear 错位第一列 (73,27)；重锚单关同前（纯 BFC 门害）；BFC 单关 container h=250 + **重锚臂把 .clear 推出列**到 (303,27)、multicol 被撑 259（单列堆叠假设不认列断）。
- 机制：multicol 列断流里 float 参与列内布局并撑列，§10.6.3 单列收窄与重锚推挤都破坏列断语义（br clear 的 clearance/列位判定链断）。
- 修复：`remeasure_text_with_float_exclusions_inner` 加 `in_multicol: bool` 递归参数；BFC 门与重锚臂均 `!in_multicol` 门（multicol 子树全退旧口径）。

## 4. corpus A/B 与 solo 甄别记录

- 双跑（同二进制，总量 16599 = 上游 +5）：ON 692F / OFF 692F（`ZW_IFC_AUTOHEIGHT_BFC=0 ZW_FLOAT_REMEASURE_SIB_SHIFT=0`）。
- 差集 solo ×3：ON 独红 {insert-003, dynamic-008}、OFF 独红 {c5525, dynamic-007}；前两者 solo 双配置全 ✓0.00%（容差边缘 movers）剔除，c5525 真修复、dynamic-007 为 mover。
- 首轮（收窄前）ON 693F：差集 {floats-149 ✗, multicol-001 ✗（真回归，bisect 定位后收窄修复）、tao-127（solo 全绿 mover）}。
- 跑批漂移带实证：OFF（=旧行为）自身 692F vs 修复前基线 690F，同代码 ±2 movers 漂移。

## 5. 验证矩阵

- c5525 solo：OFF ×3 恒 ✗2.90%、ON ×3 恒 ✓0.28%（残差 = test6 墨 z 序 paint 域，R5469 §2 条 1 结论保留）。
- floats-149 / multicol-001 修复后 ON solo ✓ 恒。
- 内测 `make reftest` 725/725 全绿（Layout 521 + Text 204）。
- judgment test 2/2 ✓（`cargo test -p zero-layout-engine --lib r5470`）。
- layout-engine 全腿（test-guard compile-first）：**2538P/0F**。
- `make product-smoke` 全腿 PASS：welcome vs chromium oracle **17.23%**（与 R5465-R5469 同值零漂移）＋ 6 站 struct-check 全 PASS ＋ form-input-perf-gate，0 fail。
- bench-gate：GATE PASS（25 gate 指标全部在预算内；另 88 条无基线新指标按 NEW/PASS 记录——bench-report 16/16 microbenches 完整跑。首两次 loadavg 62/18 繁忙中止，静窗 1min=2.6 后通过）。
- clippy/fmt 清。

## 6. 下一步

1. c5525 残余 ✓0.28% paint 域（test6 墨 z 序 + test4/blob3 bg 域）——R5469 §2 条 1/3 在册，开案前先查图元发射序。
2. **行盒归属缺陷**（display:inline 中间层行盒不进祖先容器 IFC 行流，lines=0）——行盒证据门是保守回退，根因修复后可移除该臂并复核 floats-149 域。
3. fsa 度量单 owner 切片（R5468 遗留）；缺陷 3 维持待用户决策⑥。
