# css-gaps-1 rule 下一刀设计（交接稿，未实施）

日期：2026-11-16（本轮只侦察+设计，零源码改动——共享树编译破树风险让位）
基线产物：`.acceptance/wptfyi-css/results-gaps-b0/`（css-gaps P356/F3792 75 页 +
css-multicol P237/F1376 91 页，二进制 bin-posp-b1 ≡ 4427b121e 落地态）。

## 语料面账（results-gaps-b0 实测）

- css-gaps 3,792F：parsing 1,396F（50 页）+ animation 2,396F（26 页）。
- css-multicol rule 页 311F（column-rule shorthand 门 + color/width 插值 278F）。
- animation 页普查：row/column-rule-width/color 39 页用（paint 面）；
  inset-cap/junction 56 页用（row-rule-inset-interpolation 672F +
  column-rule-inset-interpolation 672F 单页两大头）。
- 三大插值页全部为 **WAAPI testcommon 通道**（interpolation-testcommon.js +
  el.animate）——backdrop 刀 f80e4a8b8 刚验证此通道解锁形态（+253/0/0）。

## 关键文法发现（决定刀深度的根因）

css-gaps-1 把 rule 族 color/width **重定义为 repeat() 列表文法**（非
css-multicol-3 的纯值）——gap-decorations-color-valid 页实证：

```
column-rule-color: repeat(4, blue)                    # <repeat-line-color>
row-rule-color: red, repeat(3, blue), blue            # <line-color-list>
rule-color: repeat(auto, red, green), red             # <auto-line-color-list>
```

文法（css-gaps-1 §column-row-rule-color）：
`<line-color-list> | <auto-line-color-list>`，段 =
`<color> | repeat([<integer>]|auto, <color>#)`。width 同构
（rule-width-interpolation-repeaters 页 252F）。**纯色 = 单段列表**（多列
corpus 兼容面：css-multicol 现有纯色子测 serialization 不变）。

## 解锁通道（两通道探查已完成，沿 e5112287f 学习法）

1. setProperty：shim part05:3584 宽收 ✓ 零改动。
2. gCS camelCase：shim `_stylePropName` 通用 kebab 路由 ✓ 零改动。
3. CSS.supports 门：`decl_supported`（js_dom_bridge.rs:6174）= 白名单 contains
   + Rust parse——parsing-testcommon 每测内部有 supports 前置门，**这是全部
   parsing 页死因**；Rust 侧需接受列表文法。
4. WAAPI 插值页：underlying/interpolate/写回三臂（mirror offset-distance/
   animation.rs:449 color 臂先例）。

## 建议分刀

**刀 A（color-list 模型，估 +600~900F）**：
- 类型：`enum RuleColorSegment { Plain(ColorValue), Repeat { count:
  Option<u32>, colors: Vec<ColorValue> } }`（auto = count None）；ComputedStyle
  三字段 row/column_rule_color 换 `Vec<RuleColorSegment>`（column 现有
  ColorValue 迁移：单值 → 单段 Vec，multicol gCS 臂取首段平色保形）。
- 白名单 +4 串：row-rule-color、rule-color、row-rule、rule（column-rule 简写
  也缺——见刀 C）。注意 row-rule-width/style 若同轮入清单需同轮给 parse 臂
  （decl_supported 的 apply 检查会真跑）。
- parse 臂（apply_advanced.rs）：段级解析 + 逗号分层（split_top_level_
  whitespace 不够——需括号感知逗号切分，参考 background layer 切分
  split_bg_layer_list 先例）。
- gCS：段序列化（canonical = 原形小写化）；插值臂：**离散**（列表插值不
  在本刀，css-values-4 §8.5.6 ≥0.5 翻转即可过关键字端点组）。
- animation 页解锁面：row-rule-color ×9 + column-rule-color ×10 + rule-color
  repeaters 276F 页内 supports 门组。

**刀 B（width-list + 简写家族，估 +400~600F）**：同构 width 列表
（<line-width-list>：LengthValue 段）+ row-rule/rule/rule-width 简写展开
（shorthand/mod.rs expand_column_rule 先例 mod.rs:2488 改造参数化）。

**刀 C（multicol 简写门补齐，一行级）**：`column-rule` 简写不在白名单——
multicol corpus 现有实现（shorthand 展开 mod.rs:190/914 + gCS 读臂
computed_style.rs:805 全在）只差白名单一串 + supports 钉；可独立先行。

**刀 D（inset-cap/junction 几何族，1.5k+ 最大残面）**：row/column ×
inset-{start,end,cap-start,cap-end,junction-start,junction-end} 长手 +
rule-inset 双轴简写族；单长度插值（row-rule-inset-interpolation 页
from:'initial'→'15px' 线性中点 5/8/11/15px 实证）。依赖刀 A/B 的列表
底座或独立纯长度模型（inset 本身非列表）。

## 残面修正（对 master.md 十三轮残面桶的勘误）

十三轮「offset-position ~386」已由 4427b121e 部分兑现（+108）；本轮新增：
ray()/基本形状 1,830 桶按 7a0fe89ba 勘误为 supports 门死因（peer 认领实施中）；
gaps 桶按本文法勘误——「gap 装饰属性 ≈1.4k」实为列表文法模型面（本文档），
inset-cap/junction 1.5k+ 独立成桶。
