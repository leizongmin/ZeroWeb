# g37：anchor()/anchor-size() calc 嵌入叶 + serialize canonical 三修

日期：2026-11-14。承接 g36（evidence 2026-11-14-g36-border-coord-s423.md）
后 serialize canonical 刀。12F 名单源自 ab4c 组合账逐案签名（见 §6 基线
仲裁——ab4c/comb3-x7 两账均非真态基线，HEAD 反证定谳）。

## 1. 12F 三类签名（anchor-parse-valid 4F/2359T + anchor-size-parse-valid 8F/4305T）

1. **number 除数转乘**：`calc((anchor(--foo top) + anchor(--bar bottom)) / 2)`
   → 期望 `calc(0.5 * (anchor(--foo top) + anchor(--bar bottom)))`（§10.6
   number-first，4F 位点 top×1/width/max-width/margin-left×3）。
2. **anchor 叶节点排序**：`calc(anchor(--foo left, 1px) + 10%)` → 期望
   `calc(10% + anchor(--foo left, 1px))`（节点殿后，4F）。
3. **fallback 无单位数字补 px + calc 单参解包**：`anchor(--foo left, 0)` →
   `anchor(--foo left, 0px)`；`calc(anchor(--foo left, 0))` →
   `anchor(--foo left, 0px)`（4F）。

## 2. 根因

- CalcExpr 无 anchor() 叶表示——CalcParser 主叶分发无 anchor 臂，落
  parse_atom 失败 → 整 calc parse None → fold_decl_specified None → 写链
  存储原文（canonicalize_anchor_functions 只做参数序换位）。
- 裸 anchor 形不在 parse_math_function 分派（calc/min/max/clamp/trig/
  sibling/progress 名单外）→ fold None。
- fold_decl_specified 白名单无 top/max-width 槽（即便 parse 通也不折叠）。

## 3. 修复三件套（css-parser）

1. **`CalcExpr::Anchor { is_size, args }` 叶**（types.rs）：CalcParser
   `try_parse_anchor_fn`（平衡段扫描 + 实参 canonical 归一委托
   `canonicalize_anchor_call_args`——序换位 + fallback 无单位数字补 px +
   `anchor()` 空参拒/`anchor-size()` 隐式形合法 + 段数 ≤2 粗门）；side/
   fallback 细拒面仍由写文法门 `__zw_anchor_func_check` 保底。叶文本
   parse 期已 canonical，serialize 直发。型别 `<length-percentage>`
   （MTLength）；求值/掩码/维度后缀等 25 处 exhaustive match 补臂（编译器
   驱动），求值面一律 None fail-closed（FIXME: used 值面桥接挂账）。
2. **Divide 臂除法转乘**（serialize_math.rs）：number 除数 →
   `product_term_from_factors`（自 canonical_product_term 重构共用——
   和差树因子 `c * (…)`、函数/anchor 节点因子 `c * node`、维度叶系数并入
   标量）；零除/维度除数保原文。殿首唯一乘积项外括号剥除
   `strip_outer_parens`（`calc(0.5 * (A+B))` 裸发射——`(a) + (b)` 多段形
   不剥；sum 语境括号化只在项间）。
3. **canonicalize_anchor_functions fallback 补 px**（anchor.rs）：写链兜底
   链（merge_style_property / native CSD）对裸 anchor 形 fallback 无单位
   数字补 px（`canonicalize_fallback_chunk`，发射保留逗号后原空白）。

配套：
- fold_decl_specified 白名单扩 **top / max-width**（12F 槽位驱动；
  right/bottom/left 无语料驱动不接入——left 既有 store 原文零漂移钉面
  sibling_tree_counting_tests::tree_counting_fold_decl_dispatch）；
- Length 位 dimension_suffix 门放行纯 anchor 树（
  `calc_expr_has_anchor` 谓词，parse_extended.rs）；
- `canonical_math_specified` 顶层 Anchor 臂直发（calc 单参解包）。

## 4. invalid 页 4 REG 定谳与 parse_length_or_math guard

首射（guard 前）anchor-invalid-fallback 4 NEWFAIL：「Invalid anchor
function, nested min-width/min-height/max-width/max-height」
（`min-width: calc(anchor-size(width) + 10px)` 等，页 L189-192「Invalid
anchor*() deeper within calc()」段）。

- 机制：anchor-size() 在 sizing 槽 calc 内——写面域内放行（
  anchor_size_domain 含 min-/max- 族，parse-valid 页 sizeProperties 12 项
  同集）→ 此前 parse 失败 = 声明无效 → computed 初始值 auto/none（Pass）；
  本刀 parse 成功 → Calc(anchor 叶) 存活到 gCS 读出面 →
  calc_computed_to_css eval None → `unwrap_or_default()` = ""（REG）。
- 探针页定谳（zz-g37probe-invalid-minmax，跑后删除）：named setter 链
  stored/computed 全正常——翻红仅在 `target.style = "…"` cssText setter
  路径的 gCS 读出面。
- 语义：css-anchor-1 §3.3/§4 anchor() 求值需布局锚几何，computed-value
  time 不可求值 → invalid → 声明无效回落初始值。修 =
  `parse_length_or_math` / `_quirks`（style-system apply.rs:30/41）对
  `calc_expr_has_anchor` 式返 None（与此前 parse 失败行为严格一致，零
  布局回归）；inset 槽 anchor() 原文承载臂（apply.rs contains("anchor(")）
  先行拦截不达本函数——anchor_pos 布局期求值不受影响。

## 5. 布局消费面核对

- resolve_anchor_inset 只认 bare `anchor(...)` 直形
  （strip_prefix("anchor(")）——calc 包裹形布局期一直 Auto 保底；canonical
  串（`calc(0.5 * (…))`）仍 contains("anchor(") → 原文承载 → 同保底。
  布局行为零变化。
- fallback `0` → `0px`：resolve_anchor_inset fallback 臂 parse_length
  两形皆可解析（等价或正向）。

## 6. 基线仲裁（ab4c/comb3-x7 均非真态）

- ab4c（11,977P/876F）：本 session 选刀依据，master.md 已标 salvage 疑似
  假账「不可作账」——误用作刀口基线（其 4F/8F 签名与 HEAD 真态一致，
  刀口本身成立）。
- comb3-x7（11,985→账面 11,989P/868F）：master.md g36 组合态权威账，
  parse-valid 0F/invalid-fallback 10F。**HEAD 反证定谳伪账**：stash 全部
  本刀改动 → HEAD 态 release 重建 → anchor-parse-valid solo 复现
  **2355P/4F**——comb3-x7 的 parse-valid 0F 不是 HEAD 代码态产物（build2
  17:09 provenance 含未入远端的 shim 实验码）；其「IMP 12 归因 4e3eea8f3
  写面通道」归因错误（4e3eea8f3 = FORM named-first nameditem 面，提交
  信息与 anchor canonical 零交集）；invalid-fallback 10F 同源伪影。
- **canonical 12F 从未被远端代码拿到**——本刀为真增量；comb3-x7 伪绿
  12 子测由本刀真清偿、其伪红 4（invalid-fallback）由本刀 guard 修正。

## 7. 验证账

- 单测：css-parser 新钉
  `anchor_func_canonical_serialize_table`（15 形：三类 canonical + 幂等 +
  既有普通 calc 零漂移 + 裸形/非 math None）+
  `fallback_unitless_number_gets_px`（anchor.rs 7 断言）；全库
  **3208 过零失败**（sibling 既有钉面保全）。
- style-system **2534 过**（guard 后）；layout-engine **2509 过**（g1f
  面含）；engine lib **3951P/7F**（7F = ip M3-A 流在途欠账名单精确一致，
  零新增）。fmt/clippy 净（--all-targets 0 警告）。
- solo 先验：anchor-parse-valid **2359T 0F**、anchor-size-parse-valid
  **4305T 0F**（12F 全清）。
- 终账 g37-final（release 全量，265 页）：**11,993P/864F/4T**——对 ab4c
  净 **+12 零 REGRESSED**（逐页 diff 仅 parse-valid 两页）；对 HEAD 真态
  （4F/8F 复现账）：净 +12；对 comb3-x7 伪账口径：净 +4（其伪绿 12 转
  真绿 + 伪红 4 修正）。
- HEAD 反证：results-g37-head-probe（anchor-parse-valid 2355P/4F）。

## 8. 挂账

- [ ] comb3-x7 账面更正落 master.md（本轮一并提交）——其 IMP 12/REG 4
      归因撤销，anchor 真态基线回到 HEAD 复现账 + 本刀增量。
- [ ] anchor()/anchor-size() used 值布局期求值刀（sizing/margin 槽
      anchor-size 布局语料面）——FIXME: 桥接挂账。
- [ ] anchor-size() 属性域表（anchor_size_domain）与 invalid 页语义对齐
      （min-/max- 族 calc 嵌套形 invalid——写面域收窄面，涉 parse-valid
      12 sizeProperties 直值形合法集，需分「直值 vs calc 嵌套」两级域）。
- [ ] invalid-fallback 残 6F（fallback 布局期求值刀域）。
