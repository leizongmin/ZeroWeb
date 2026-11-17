# 刀 N 二期：font-family 文法写面收口（2026-11-18 凌晨；A/B 窗 01:42:04–01:48:43）

## 靶面

`css/css-fonts/test_font_family_parsing.html` 608F/1624P（刀 N 一期
CLAIM-knife-n-fonts-parsing 侦查定账；b0=bin-conv-b1v2 md5 `4ff1057d`
单页探针 /tmp/knifen2-ffp.json 复核同账）。608 Fail 全部「应收被收」
（零反向形态），按通道分双形（rule + setter 各 304）：

| 缺口类 | 形例 | 数 |
|---|---|---|
| 前导数字 dimension token（`0simple`/`quite 0simple`） | 应拒收收 | 4 形 |
| 特殊字符（`simple!`/`simple()`/`quite@simple`/`#simple`） | 同 | 4 形 |
| 引号混排（`arial, helvetica, "times" new roman, sans-serif` 四形） | 同 | 4 形 |
| CSS-wide keyword 简写 family 位（`font: 16px inherit` 等 fontonly 组） | 同 | 4 形 |
| CSS-wide keyword 列表位（`inherit, simple`/`simple, default` 等） | 同 | 8 形 |

## 通道定谳（一期门零位移根因）

一期把文法门挂在 `expand_font`/`apply.rs` cascade 面——WPT 页走
**CSSOM 写面**（insertRule body 解析 + `rule.style`/`el.style` IDL
setter），不经过 cascade，原文落库回显即 Fail（一期 fonts corpus
3914→3914 零位移的根因）。shim 通道图：

- rule 形：`sheet.insertRule` → `_makeRuleStyle` → `_parseDeclarations`
  → `_zwNormalizeDeclValue`（part06:2163 分派枢纽——grid/anchor/color
  既有臂先例）→ null=声明剔除（cssText 无声明 = 测试断言面）。
- rule style setter：`_makeRuleStyle.setProp` → 同枢纽 → null=赋值忽略。
- inline setter：part03 setProp 通用臂已挂 `__zw_validate_decl_value`
  （T54 白名单机制），`font-family` 不在白名单 → 放行。

## 实现（文法单源三站）

1. `parse.rs`：`font_family_list_has_css_wide_keyword`（六键
   inherit/initial/unset/revert/revert-layer/default；段级——单段单
   token unquoted 才命中，`default simple` 段内 ident 序列与 quoted 段
   `"inherit"` 不命中）；`parse_font_family` 本体门（**仅列表多段**
   且任段 keyword → 整条无效；单段单 keyword 是整条 CSS-wide keyword
   声明由 cascade 前置臂处理不裁）。
2. `shorthand/mod.rs expand_font`：family 位 keyword 门（**不问段数**
   ——简写带 size 位时单段 `inherit` 亦拒）。
3. `apply.rs validate_decl_value`：白名单 +2（`font-family`/`font`）
   + 分派臂置于未实现函数放行臂**之前**（`simple()` 形 function token
   会被放行臂先行误放，counter-increment `reversed(` 同序先例）。
   escape 词法（`\s imple`/`\1f4a9` 转义 ident valid 语料组）与替换
   函数形放行保形（FIXME: font-family 转义序列词法挂账）；`font` 臂
   委托 `font_shorthand_supported`（expand_font 单源；system font
   臂 expand_font:3046 既有覆盖）。
4. shim `part06.js _zwNormalizeDeclValue` 顶部 font 臂：`font-family`/
   `font` 属性名门 → `__zw_validate_decl_value`（'false' → null 拒），
   尾部 `!important` 先剥再校验、合法保形零重写（reject-only）。
   补齐 rule 声明/规则样式 setter 两通道（inline 独立 validate 臂幂等）。

## 伴生修复：extract_random_args byte-slice panic

font-family 入白名单后通用 random 门对 valid 语料 `納豆嫌い`/简写形
`16px 納豆嫌い` 崩溃（`&lower[i..i+7]` str 切片窗口**终点**落多字节
字符中间——起点 ASCII 守卫不够，`16px 納豆嫌い` 的 end=7 落 '納'
bytes 5..8 内，两次探针实证）。修复：byte 切片比较
`&bytes[i..i+7] != b"random("`（`[u8]` 切片无 char boundary 约束，
逐字节比较语义等价）。random-item 处既有 byte 切片本已安全未动。

## 门禁与账目

- style-system lib tests **2562 passed / 0 failed**（+2 新钉测：
  `parse_font_family_css_wide_keyword_list_rejected` /
  `font_family_list_css_wide_keyword_detector`）。
- clippy `-p zero-style-system --all-targets -D warnings` rc=0；
  fmt clean。
- 主靶单页（b1=bin-knifen2 md5 `8833233e`）：608F → **0F**
  （2232P，+608 F2P 全落靶页）。
- A/B 六腿账：见下节（reconcile 回填）。

## A/B 终账（01:42:04–01:48:43 六腿全 rc 腿内 Fail 惯例，reconcile 01:48:43）

b0 = results-conv-b1v2（bin-conv-b1v2 md5 `4ff1057d`，v2 终账六腿在册产物
直接作基线侧——六腿和 27841=4074+3914+3838+943+9947+5125——同 bin 同码同机 2h 窗内零环境漂移）；b1 = bin-knifen2
md5 `8833233e`（刀 N 二期 44.8s 增量构建 + panic 修复 44.1s 二次）。
逐子测 reconcile（`.acceptance/wptfyi-css/results-knifen2-b1/`）：

| corpus | b0 | b1 | 判读 |
|---|---|---|---|
| css-fonts | 3914 | 4538 | **F2P=624 / P2F=0**——主靶页 +608 清零、font-family-invalid +6（`#POUND, sans-serif`/`Ahem!`/`Hawaii 5-0`/`Red/Black`/`"Lucida" Grande` 引号混排组）、font-invalid +10（`bold bolder xx-large/1.2 cursive` 非 normal 重复组、`menu icon` 双 system keyword 等——expand_font 严格校验经新通道生效） |
| css-masking | 4074 | 4074 | SET-IDENTICAL |
| css-gaps | 3838 | 3838 | SET-IDENTICAL |
| css-multicol | 943 | 943 | SET-IDENTICAL |
| css-typed-om | 9947 | 9947 | SET-IDENTICAL |
| css-shapes | 5125 | 5125 | SET-IDENTICAL |

**TOTAL 27841 → 28465 净 +624**；`gained=624 lost=0 regressed=0
pages_gain=3 pages_loss=0`（ab-compare.py 逐子测）——P2F=0 零容忍
满足、页集合位移全正向（pages_loss=0）、五对照 SET-IDENTICAL 满足。
F2P 全落 css-fonts 靶面（3 页）＝刀 N 二期界内。

## 遗留与挂账

- escape 词法（`\\s imple`/`\\1f4a9` 转义 ident valid 组）经放行臂保
  形通过——`parse_font_family` 转义序列词法挂账（FIXME 注记在
  validate_decl_value font 臂），挂上后收窄 escape 放行。
- style-attr 通道（`setAttribute('style', …)` → `_zwStyleAttrDeclFilter`）
  未挂 font 臂（WPT 靶页不经此通道；g44 gradient 过滤臂先例可扩，
  挂账后续刀）。
- font 简写串化面（gCS `font` 一期已挂）不受本刀影响（五对照 +
  fonts 页零 P2F 实证）。
