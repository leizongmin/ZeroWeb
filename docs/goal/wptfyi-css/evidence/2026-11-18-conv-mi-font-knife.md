# 收敛轮：mask 六长手补全刀 MI + font 刀 N 一期（2026-11-18 00:3x–0x）

## 背景

前轮 resume-swarm 多 fork 在树遗留三认领刀（`CLAIM-knife-mask-interp.md` /
`CLAIM-knife-n-fonts-parsing.txt` / `CLAIM-knife-supports-columns.txt`，
2026-11-17 23:1x–23:5x），rate-limit 窗中断后未验证未提交。本轮（c4fb58fd）
按 resume-swarm 收敛纪律接手：编译修复 → 死靶定谳 → A/B 对称消去 → 收口。

## 刀 MI：mask 六长手补全（css-masking-1 §5.3–5.6）

前 fork 实现在树，本轮补编译与验证：

- css-parser：`MaskGeometryBoxValue`/`MaskClipValue`/`MaskCompositeValue` 三枚举
  + `parse_mask_geometry_box`/`parse_mask_clip_layers`/`parse_mask_origin_layers`/
  `parse_mask_composite_layers` 解析器（含 SVG 几何盒 fill/stroke/view-box，
  css-masking-1 §5.4）+ mask-position/size/repeat 复用 background 同构 list parser。
- style-system：`supports.rs` 六长手 @supports oracle 六臂；`inherit_property`
  显式 inherit 拷父六臂（css-cascade-4 §inherit，Inherited: no 属性显式
  inherit 强制拷父）；`apply_advanced` 六长手写值通道（position/size/repeat
  background_* 镜像 + clip/origin/composite masking 枚举）。
- engine：`computed_style.rs` gCS 六串化臂（position/size/repeat 走 background_*
  串化镜像，clip/origin/composite masking 专用串化）；`animation.rs` 宽松臂
  mask-position 逐槽 lerp（mix_mask_position）+ image/mode/repeat/clip/origin/
  composite/mask 离散臂（t≥0.5 翻转）+ 严格臂一行委托（刀 K1 契约同型）。

本轮修复（前 fork 遗留编译破）：

1. `shorthand/mod.rs` E0425：`parse_font_family` 未限定路径 →
   `crate::property::parse::parse_font_family`（本文件全路径惯例）。
2. `computed_style.rs` 9×E0425/E0433：`FontVariantCapsValue` 未导入 →
   `use zero_style_system::{… FontVariantCapsValue …}`（types.rs pub re-export）。

## 刀 N：css-fonts 解析/串化一期

前 fork 实现在树，本轮修复语义与验证：

- `shorthand/mod.rs expand_font`：font-stretch 位（`<font-stretch-css3>` 九关键字，
  % 形简写位拒收）+ 非 normal 值逐位至多一次（`||` 组合器）+ `normal` 多次占位
  略过 + family 位文法校验委托 `parse_font_family` 单源。
- 本轮修正 family 门：前 fork 写成 `.len() != 1`（单 family 名）——css-fonts-4
  §font-prop 简写 family 位是 `<family-name>#` 逗号列表（`font: 14px Arial,
  sans-serif` 合法），改 `.is_empty()`（解析空 = 非法 token 才拒收）。
- `apply.rs`：`font` 简写 supports oracle 臂（css-conditional-3 §supported，
  expand_font 严格校验单源；apply_advanced 无 font 直接臂故无拦截冲突）。
- `computed_style.rs` gCS `font` 简写串化：font-variant 位（非 normal 保留，
  font_variant_caps_str 全值域）+ font-stretch 关键字 canonical 反查表
  （css-fonts-4 §font-width-prop 同表单源，表外 % 保 % 形）+ family 位多 ident
  序列免引号（is_css_ident 全 Token 判别，需引号名走 quote_font_family）。
- 钉测更新 2 例（error_cases.rs）：`test_font_size_only` /
  `test_font_size_line_spacing_with_spaces` 旧形编码缺 family 半展开（13 decl
  含空 family）——css-fonts-4 语法 family 必填、chrome 拒收整条，改断言整条
  拒收 + 补带 family 空格分隔形正例。

## columns 刀死靶定谳（撤码）

CLAIM-knife-columns-oracle v2 形1（"chrome 不实现 column-height、columns 简写
串化不含 height"）**不成立**：本机 google-chrome headless 探针
（/tmp/cols-probe/probe.html）实测 gCS('columns') height 非 auto 恒输出 slash
段（"10px / 100px"/"auto / 100px"/"2 / 100px"/"10px 2 / 100px"四形逐项 =
R2755 钉测 Chromium 150 oracle 复证），columnHeight 长手支持。前 fork 推断把
columns-interpolation.html 页面硬编码 expect（column-height 落地前作）当成
chrome 现行为——该页无 height 期望形对 chrome 自身红，非 ZW 可回收面。

本轮曾试砍 `columns_shorthand_to_css` height 段并补 apply.rs columns oracle 臂，
按本定谳全部回退；apply.rs 臂另暴露 supports oracle 臂不可用于有直接 apply 通道
的简写（apply_advanced:1841 columns 直接臂被拦截致 r3749/
zero-is-width 两钉测红；font 臂无此冲突因 apply_advanced 无 font 臂）。
@supports matcher 侧 columns_supported（supports.rs:113）既有不动。
残值：形2 allow-discrete / 形3 WAAPI 端点回落仍在册挂账。

## A/B 对称消去

- **b0 污染定谳**：本树 `target/release/zero-wpt-runner`（mtime 11-17 23:24）
  masking 实测 P4068/F2287 = 被 mask WIP 污染（收账态 Fail=2336），不得作 b0；
  改用 `bin-mm-b1`（md5 `c2424b73`，mask 简写刀收账 bin；HEAD 相对仅 docs 提交
  码面同）。b0 新鲜腿 masking P4012/F2337 vs 收账 4011/2336 ±1 子测 flake。
- b1 = `bin-conv-b1`（md5 `156fd30b`，收敛轮全码）。
- 六腿 corpus：css-masking / css-fonts（靶面）+ css-gaps / css-multicol /
  css-typed-om / css-shapes（对照 SET-IDENTICAL）。
- v1 十二腿结果（`/tmp/conv-ab-svc-003955.log`，00:39–00:56，rc 全 0）：
  masking F2P=56 / P2F=0 / NEW-Pass=6 / GONE=0（4012→4074，+62），翻转全落
  mask-repeat/mask-size/mask-position 三靶页＝刀 MI 界内；fonts 3914→3914
  零位移（刀 N 实装子面无回收）；gaps/multicol/shapes SET-IDENTICAL；
  typed-om **P2F=1**（见下节，零容忍归因闭环后由 v2 复跑清偿）。

## typed-om P2F=1 归因闭环（v1 后置归因，01:0x）

- 面：`the-stylepropertymap/inline/append.tentative.html :: append with a
  CSSVariableReferenceValue property throws TypeError`（值实为
  `title.computedStyleMap().get('mask-position')` 返回值）。
- 单页双 bin 探针（`results-probe-append-{mm-b1,conv-b1}/`）：b0 14P/0F、
  b1 13P/1F＝确定性回归非 flake。
- chrome oracle（/tmp/typedom-probe/probe{,3}.html）：computedStyleMap
  `.get('mask-position')` 返**基类 CSSStyleValue**（str "0% 0%"，无 typed
  分型）；writable attributeStyleMap 跨属性 append/set 该基类值恒 TypeError，
  同属性 set 放行（probe2 曾误用 computedStyleMap ReadOnly 面全 TypeError，
  probe3 修正为 attributeStyleMap 定谳）。
- 插桩探针（/tmp/typedom-probe/results-internals/，b1 bin 四点）：ctor
  CSSStyleValue / instanceof ✓ / kind opaque ✓ / **`_zwOmAssociatedProp`
  undefined**＝`_zwOmValidateSet` 关联属性错配门（shim 1609 行既有）空转。
- 机制定谳：刀 MI 前 mask-position 无计算串→get 返 undefined→append 掷
  TypeError（测试绿）；刀 MI 补出计算串→get 返 opaque 基类实例→该实例未携
  来源属性→append 跨属性放行（测试红）。刀 MI 暴露 typed-om 潜伏分歧，
  非刀 MI 自身缺陷。
- 修复（sibling fork 先手，本 session 采彼）：`part10-typed-om.js
  _zwOmParseWithText` 补 opaque 实例 `_zwOmAssociatedProp` 盖戳（+7 行），
  chrome 同形。v2 复跑（bin-conv-b1v2，verify-conv-ab-v2.sh +
  /tmp/conv-ab2-010306.log，b0 沿用 v1 六腿）判据：masking F2P 56 保持/
  P2F=0、typed-om SET-IDENTICAL 9947 恢复、余四 SET-IDENTICAL。终账见
  master.md 收口段与 `.acceptance/wptfyi-css/NOTES-conv-round-arbitration.txt`。

## v2 终账（01:11:09 reconcile，verify 角色轮盯账回填）

六腿全 rc=0（01:03:06–01:11:09，`/tmp/conv-ab2-010306.log`），reconcile
（results-conv-b0 vs results-conv-b1v2，逐子测键）：

| corpus | b0 | b1v2 | 判读 |
|---|---|---|---|
| css-masking | 4012 | 4074 | **F2P=56 / P2F=0** / NEW P6 F0 / GONE 0，页 62=62＝v1 主靶账逐位保持（shim 修零扰动） |
| css-fonts | 3914 | 3914 | SET-IDENTICAL（刀 N 中性） |
| css-gaps | 3838 | 3838 | SET-IDENTICAL |
| css-multicol | 943 | 943 | SET-IDENTICAL |
| css-typed-om | 9947 | 9947 | **SET-IDENTICAL 恢复**（v1 P2F=1 清偿，NEW/GONE 全零） |
| css-shapes | 5125 | 5125 | SET-IDENTICAL |

**TOTAL 27779 → 27841 净 +62**；P2F=0 零容忍满足、page-only 零容忍满足
（NEW-PAGE/GONE-PAGE 全零）、四对照 SET-IDENTICAL 满足。bin-conv-b1v2
md5 `4ff1057dcf5b1127b21cb8e00a96824d`（盖戳修复后增量构建）；SCRIPT-EXIT
rc=0。masking NEW 6P 与 v1 同集（inheritance.sub 12 + parsing 三页族）。

### 双流收敛与 provenance 补记（01:1x，主流盯账）

- 双流独立归因同点收敛：主流（上节"先手"落码方）census 该键 60+ 次历史
  运行全 Pass 仅 conv-b1 翻 Fail；sibling fork 单页双 bin 探针同谳（b0
  14P/0F、b1 13P/1F）；插桩四点定谳 `_zwOmAssociatedProp` undefined。
  门锚点核正：错配门 throw 在 shim **:1610**（上文 :1609/:1602 系两流
  各自行位快照，以 1610 为准）。
- b1v2 provenance：33.6s 增量构建（与 conv-b1 `156fd30b` 码面差＝仅 +7
  盖戳 fix）；peer 在飞文件（es_module/part01f/part115）mtime 00:19 未动
  ＝构建窗 provenance 干净。
- wpt-data 污染处置：sibling 插桩探针页 01:02 放入、01:06 移除，typed-om
  腿 01:10 起跑——页集 359/359 对称实证零污染。
- 仲裁：`.acceptance/wptfyi-css/NOTES-conv-round-arbitration.txt`（双流
  操作密度识别、先手修复采认记录）。

## 门禁

- fmt clean（registry.rs 一处 mask 简写刀遗留 fmt 漂移顺手折叠）。
- style-system lib tests **2560 passed / 0 failed**。
- clippy `-p zero-css-parser -p zero-style-system --all-targets -D warnings` rc=0。
- clippy `-p zero-engine` **挂起**：dep 链 zero-engine→script-runtime→
  zero-script-sandbox，peer es_module s419 在飞半成品（8 参函数
  clippy::too_many_arguments）域外拦截——peer 流落码后由其门禁消化，
  本轮 engine 面 clippy 欠账挂 master 待清。
