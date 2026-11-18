# g48 gradient `at` 位置文法位 + canonical 序 + computed 折算刀

2026-11-15。driving: css-images gradient-position 三页 27F（invalid 5、
valid 4、computed 18）——B0（g46-b2）既有失败。

## 1. triage

oracle 三页合读定谳 csswg#2140 后 `<position>` 行为（WPT = Blink 实装）：
- **3 值混搭形全禁**：旧 `parse_position_pair` 3|4 值共用一个宽松 walk
  （缘关键字 + 可选偏移、序任意、无同轴重复检查）——`at left 4px top`、
  `at right top 5px`、`at bottom 7% left`、`at bottom right 8%` 误收。
- **2 值文法**：H 侧（left/right/center/`<lp>`）+ V 侧严格集（`right 40%`
  → H 关键字 + V lp 合法，computed `100% 40%`）；V 缘关键字 + lp（`top
  0px`）拒；双裸关键字退化 && 形任意序可换轴（`top left`/`center left`）。
- **1 值轴向**：旧实现 `x=y=该值`（`at top` → `0% 0%`、`at 10%` →
  `10% 10%`）——正确为缺省轴补 center（`top` → `50% 0%`、`10%` →
  `10% 50%`），computed 18F 的主根因。
- **4 值缘偏移对**：right/bottom 锚定丢失（旧 walk 原值入槽，FIXME 在案）
  ——`right 30%` 应 `70%`（100−30）、`bottom 10%` 应 `90%`；百分比可数折，
  长度须 `calc(100% - Npx)`（LengthValue 模型无缘锚表达，本轮留置）。
- **canonical 序**：指定值头部 as-written 透传（g42 口径）——`bottom 10%
  right 20%` 须重排 `right 20% bottom 10%`、`bottom right` → `right
  bottom`、`center left` → `left center`（as-written 形不在 soundness 集）；
  标准序保形（`right 40%`、`40px top`、`center 50px`、g42 H 对前导
  4-token 全部 as-is）。

## 2. 实现（css-parser parse_transform.rs 单文件）

- `parse_position_pair` 重写：1 值轴向归位 / 2 值
  `parse_position_pair_two`（H V 严格集 + 双裸退化 && 换轴）/ **3 值
  `None`** / 4 值 `parse_position_pair_four`（双缘偏移对、偏移必带、同轴
  重复拒、right/bottom 百分比 100−x 折算，长度偏移保原值 FIXME 留置）。
- `canonicalize_at_clause`：at 子句 canonical 重排（2 值双裸 V/center
  前导换轴；4 值 V 对前导 H 对前置；含括号 token 不重排透传；前缀段
  shape/size/`from <angle>` 保真）；接线 radial `canonicalize_radial_head`
  之后 + conic head 捕获位。
- computed 面零改动：折算后的 pair 流经既有 `radial/comic_computed_head`
  （缺省 center-center 省略 at 段既有逻辑天然命中）。

## 3. 单测与门禁

- css-parser **3,228P/1F**（+1 钉 `test_g48_gradient_position_grammar_
  and_canonical`：invalid 7 形、canonical 重排 3 形、as-written 保形 6 形、
  conic 重排、4 值折算 `right 30%`→`70%`、1 值轴向；g42
  `test_g42_position_edge_offset_pairs` 3-token 断言按新 oracle 翻转为
  None）。1F = `test_background_position_three_values_keyword_offset`
  （R2478 `left 50px center`）——**并发流 g47（object-position，peer
  化身）收紧共享属性面 `parse_position_three_four` 的 all-or-nothing
  EdgeOffset 检查所致，归 g47**（R2478 reftest 页 `left 50px center`
  形随之受累，其轮收敛）。
- engine lib **4,014P/0F**（+1 钉 `g48_position_computed_keyword_folding`：
  computed 折算 oracle 8 形含 conic；g42 computed 4-token 透传钉无损）。
- fmt `--all --check` clean；clippy css-parser + engine(--features v8)
  `--all-targets -D warnings` 净（trim_split_whitespace + doc 缩进清尾）。

## 4. 受控 A/B（B0 = results-g46-b2 g46 提交态；B1 = results-g48-b1 合并树）

| corpus | B0 | B1 | 净 |
|---|---|---|---|
| css-images | 3091P/124F | 3127P/88F | **+36P/−36F** |
| css-values | 4041P/1546F | 4054P/1533F | **+13P/−13F** |

- 逐页（ab-g42.py，案数恒同 3215/5593；grand fixed +57 regressed 8
  net **+49**），页族归因：
  - **g48（本轮，文件 parse_transform.rs/basic.rs/engine computed_style.rs
    钉）**：gradient-position-invalid 5→0、gradient-position-valid 4→0、
    gradient-position-computed 18→4（+23，零回归）。
  - **g47（并发流，文件 parse_extended_visual.rs/callbacks.rs/part06.js，
    未提交）**：object-position-invalid 13→0、css-values
    position/position-invalid.tentative 21→0（+34）；position-valid
    .tentative 16→24 **REGRESSED +8**（`x-start`/`y-end` 逻辑关键字形
    `10px y-end`/`left 10px y-start 20%` 此前 shim 零校验误收、判定站
    上线后被共享解析器拒——css-values-4 最新草稿逻辑关键字未实现，归
    g47 轮收敛）。
- g48 computed 残余 4F = 长度锚 FIXME（`conic at right 10px` 须
  `calc(100% - 10px)`——LengthValue 单值模型无缘锚表达，需锚定模型扩展）。

## 5. 残面挂账

- g48 computed 4F：位置缘锚定模型扩展（LengthValue → 锚感知或 Calc 形）。
- g47 轮：position-valid.tentative −8 逻辑关键字（x-start/x-end/y-start/
  y-end）+ R2478 pin/reftest 冲突收敛。
- css-images 余 88F：object-position valid/computed/动画族（g47 轮）、
  conic-calc-angle invalid/valid 8F、其余既有页。
- object-position 属性面 canonical 序 + computed 折算（g47 轮范围）。

## 6. 运行记录

- runner 二进制重建 42s（合并树含 g47 未提交工作，页级归因如 §4）；
  corpus systemd 单实例（flock + 同树守卫 + 唯一 transcript），双套件
  exit=0。
