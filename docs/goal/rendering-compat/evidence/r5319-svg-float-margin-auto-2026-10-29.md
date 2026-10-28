# R5319 — svg float margin-auto R4091 臂侧放行（float-replaced-width-007/008 整盒消失收口）

- **日期**: 2026-10-29
- **驱动案**: `css/CSS2/floats-clear/float-replaced-width-007.xht` / `-008.xht` ✗ → **✓0.00%**
- **corpus**: 821F → **819F**（16592 案，FIXED=2，REGRESSED=0；trend 行 15773P/819F/95.1% @ `bc78e1ac7`）
- **域**: CSS2 §10.3.2（float-replaced-width：float 替换元素 auto margin used = 0）× R4091 svg default-size margin-auto 门

## 定性（探针链，临时诊断 ZW_R5319_DIAG 已撤）

007/008 共同形态：floated `<svg:svg>`（XHTML svg 命名空间，仅 height attr，无 viewBox，
无 width 来源）+ `margin-left/right: auto`，ref 期望 used = (min(bbox_w,300), 显式高) =
200×50（007）/200×100（008，CSS height:100px 覆盖 attr 50）。实渲染整盒消失
（仅 div 边框）。四探针隔离：

| 探针 | 形态 | svg 终态 LayoutBox |
|---|---|---|
| q1 | 无 float 无 margin | 200×50 ✓ |
| q2 | `float:left`（margin 初始 0px） | 200×50 ✓（float 本身无恙） |
| q3 | float + width/height attr 双定 | 100×50 ✓（attr 定宽路径） |
| q4 | float + `margin-left/right:auto`（≙007） | **0×50（塌 0 消失）** |

**根因**：`svg_default_used_size` 的 R4091 例外臂（宽无来源 + 高 abs 来源 → used =
(min(bbox_w,300), 显式高) 定 definite taffy 宽）带 margin-auto 门
`no_auto_margin`——R4091 定案时该门是给 **inline/abspos** 侧 dodging ZW IFC 对 atomic
inline 的 auto-margin 中心化伪影（definite 双尺寸 + margin auto → x 偏 44px 实测）。
float 形态被门一并截走：交旧路径（abs 早退 → 无定宽）→ taffy 侧 float 臂 R5308 以
`Size::MAX_CONTENT` 测 inherent 宽，svg 无定宽测量 0 → **整盒 0 宽消失**。float 不走
IFC 中心化（converter `convert_length_to_lpa(margin, is_float=true)` 已按 §10.3.2 把
float 的 auto margin 解析 used 0；float 定位归 taffy R5308 + ZW
`adjust_float_positions`，无 auto-margin 中心化语义），中心化 dodge 对 float 无必要。

## 实现（kill-switch `ZW_SVG_FLOAT_MARGIN_AUTO`，default-on，`=0` 回退）

`svg_default_used_size` margin-auto 门加 float 侧放行：

```rust
let is_floated = !matches!(style.float, zero_css_parser::values::FloatValue::None)
    && std::env::var("ZW_SVG_FLOAT_MARGIN_AUTO").as_deref() != Ok("0");
let no_auto_margin = is_floated
    || (!matches!(style.margin_left, LengthValue::Auto) && !matches!(style.margin_right, LengthValue::Auto));
```

放行后 float svg 落 R4091 臂同 q2 路径（definite taffy 宽 200 + 显式高），taffy R5308
float 臂与 ZW float 定位消费同一定宽。R4091 臂入口 `!is_abspos` 未动——abspos 永不入
本臂（§9.7 float 不适用于 abspos，原 7 案 abspos margin-auto 族行为不变）。inline
svg margin-auto（is_floated=false）同旧路径，R4091 原判例不变。

## 回归测试

`svg_default_size::tests::r5319_float_margin_auto_svg_keeps_definite_width`（既有 tests
模块内，沿用 `setup` helper）：float svg + margin 双 auto + height attr，bbox (200,100)
→ `(Some(200), 50)`。**mutant**：`ZW_SVG_FLOAT_MARGIN_AUTO=0`（进程级）→ `dw=None`
（abs 早退）→ 测试 FAIL——双向可判别（实测）。

## 门禁

- fmt 0；clippy `zero-layout-engine --all-targets -- -D warnings` 0。
- **layout-engine lib 2225P/0F**（含本轮新增 1 案；2 ignored 既有；全量跑）。
- 本地 723 套件 722/723，唯一红 = `float-shrink-inline-children-zw-001`（R5311 既有记账）。
- corpus 全量：**819F**，FIXED=2（007/008 双 ✓0.00%）REGRESSED=0；
  content-none-select-1 本轮在绿侧（R5317 负载脆弱族，R5318 曾 +1 伪红，基线口径
  821 clean − 2 = 819 自洽）。
- product-smoke 全腿 PASS（welcome 15.68% ≤ 20% 预算；struct-check 多 viewport 零
  issue；form-input-perf PASS）；product-smoke-legacy 42 fixtures struct 0 失败。
- **bench-gate 判定不适用**：改动位于树构建期 `svg_default_used_size` 谓词（每 svg
  元素一次；非 floated svg 短路在 float 枚举比较，env 读取仅 floated 形态触发），
  不在 bench 测量的布局热循环（block/flex_layout measure 路径）。R5318 的 bench-gate
  必要性源于 `box_content_max_width_inner` 热路径，本轮无对应面。

## 挂账

- **收口**：float-replaced-width 域 00x 族全绿（1-9 ✓0.00%）。
- **维持**：bench-gate 静窗复测（跨 R5317/R5318 两轮，本轮新增「改动不在热循环」判定
  先例）、非 img inline-block content/border 口径分裂、content-none-select-1 负载脆弱族、
  c44-003 0.47% length-va 残差、va 族余 122/baseline-003/intrinsic-size-009/R5314/R5315。
- **遗留观察（不挂账，记录）**：无 float 的 svg 宽度口径——q1/q2 用 R4090 bbox 钳制得
  200 恰与 007 ref 期望一致；若未来 ratio 传输域（007 assert 文本的 used height ×
  intrinsic ratio 路径）入账，需另案核对 `svg_has_ratio_source` 对 rect 内容比的缺失
 （本轮 007/008 ref 期望 = bbox 宽，R4091 路径已达标，无缺口）。
- **下一步**：①821→819F 账本续扫择轻（避开 va 域与 OOF 锚域）②css-multicol 103F /
  css-transforms 67F / css-flexbox 52F 域内挑轻 ③CSS2/css1 邻案（c548-ln-ht-000、
  c5525-fltmrgn-000、floats-031）。
