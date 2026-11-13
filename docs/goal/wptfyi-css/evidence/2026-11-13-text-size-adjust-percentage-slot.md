# text-size-adjust <percentage> 槽：valid 两腿回位 + invalid 写面门收口（size-adjust −1 桶闭环）

2026-11-13。承接 [counter-increment 文法门批](2026-11-13-counter-increment-grammar-gate.md)
同日批（g27 后 g28），v14 对账四小回归末账 size-adjust −1 收口。

## 1. 起点形态（v14）

`css-size-adjust/parsing/text-size-adjust-valid.html` 5P/2F：

- `calc(10% + 5%)` → 期望 `calc(15%)`：写入成功但串化非 canonical（% 槽
  同单位加法折叠缺臂）。
- `calc(10% * sibling-index())` → 期望恒等：**写入被拒**（got ""）——
  validate_decl_value tree-counting 前置门槽族裁决拒。

## 2. 根因（两点）

1. **槽族门缺槽**：`tree_counting_slot_valid` MTLength 臂白名单
   （width/margin-left/text-indent/transform-origin-z/left）不含
   text-size-adjust——% 叶经 LengthPercentage 归 MTLength
   （`LengthPercentage as MTLength`），text-size-adjust 为 <percentage> 槽
   （css-size-adjust-1）同族。
2. **fold 缺臂**：fold_decl_specified 尺寸组（MathFoldKind::Length）无
   text-size-adjust——`calc(10% + 5%)` store 原文。

## 3. 修复

1. `tree_counting_slot_valid` MTLength 臂 + `"text-size-adjust"`（% 槽归
   MTLength 同 width 口径）。
2. fold_decl_specified 尺寸组 + `"text-size-adjust"`：`calc(10% + 5%)` →
   `calc(15%)`（独立 crate 探针预验——plain `200%`/`100%`/`0%`/`none`/`auto`
   parse 失败 None 保留原文零漂移，sibling 树形 fold_tree_counting_calc
   恒等保留）。
3. 顺手收口（同 g27 模式）：validate_decl_value 白名单 15th + 分派臂
   （css-size-adjust-1 §text-size-adjust `auto | none | <percentage>`，
   负 % invalid）——invalid 页 4 形（`reverse`/`0`/`10px`/`-100%`）此前
   白名单外放行 store；math 形放行 store（tree-counting 前置门已裁 sibling
   槽族、通用 math 门已裁文法）。

## 4. 验证

- 单测：zero-css-parser lib **3,188 passed / 0 failed**（+1：槽族矩阵
  text-size-adjust 腿 + fold 臂钉）；zero-style-system lib **2,530
  passed / 0 failed**（+1：写面门 invalid 4 形 + valid 8 形 + css-wide
  裸形）。clippy 净。
- css-size-adjust 全套件复测（results-g28-sizeadjust + spike2）：
  - text-size-adjust-valid **5P/2F → 7P/0F**（两腿回位，白名单接入零漂移）；
  - text-size-adjust-invalid **0P/4F → 4P/0F**（首跑账新收口，非 v14 分母）。
- 残余挂账：computed 页 6F + interpolation 1F——gCS 通道无 text-size-adjust
  computed 串化臂（computed_style.rs 无该属性臂），基础形（auto/200%）
  computed 全 F，属 gCS 通道缺口批（同 counter eval 容器参数腿模式挂账）。

## 5. 对 v14 分母账

size-adjust −1 桶（四小回归末账）**闭环**：valid +2P。至此四小回归
（properties-values-api −5 / writing-modes −4 瞬态 / css-lists −2 /
size-adjust −1）全部处置完毕（writing-modes 为瞬态定性非修复）。
