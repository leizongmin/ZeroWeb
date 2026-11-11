# DC-2 组F anchor 族声明面文法批（anchorbase2 10,182 → anchorf2 10,455 净 +273）

日期：2026-11-11。前序：[@position-try CSSOM 批](2026-11-11-dc2-position-try-cssom-face.md)。
master.md 下一步声明的组F 声明面文法桶（~142F 七页零布局依赖）全数兑现，布局
无关的 position-try-order 行为页与 computed 页连带翻正。

## 批内容（提交 431d229e5 + 跟进刀）

1. **`dom_bindings/anchor_props.rs`（新增，grid G1/position_area 单实现双通道
   同型）**：anchor 族七属性（position-anchor/position-visibility/
   position-try-order/position-try-fallbacks/position-try 简写/anchor-name/
   anchor-scope）文法校验 + canonical 序列化（`canonicalize_decl`），anchor()/
   anchor-size() 函数属性域 + 参数文法拒面（`anchor_funcs_valid`，只拒不重写），
   computed 恒等折叠 + 初始值（`computed_value`），`position-try` 简写拆分/合成
   （`split_position_try`/`join_position_try`）。序列化矩阵以语料为 oracle：
   try-fallbacks ident 夹心拒（`flip-inline --bar flip-block` invalid，语料
   parsing/position-try-fallbacks-parsing.html 钉面）/ `none` 整体单独 /
   position-area 段委托 `position_area::canonicalize_decl` 复用（`is_area_kw`
   布尔包装不外泄私有 Axis）/ try-tactic 含 flip-x/flip-y 别名；anchor-size 语料
   特面——隐式空参 `anchor-size()`、单参 length 形 `anchor-size(10px)`、`||` 两侧
   序（`anchor(top --foo)` flip 形）、dashed 名后槽仅 size 关键字（`--foo 10em`
   拒）、unitless zero fallback（`anchor(--foo left, 0)`）、属性域
   size+inset+margin 三族全集（top 拒 anchor-size 为**误**——语料 valid 循环
   insetProperties 全集含 top；域外 font-size 拒）。
2. **shim 接线四处**：part06 `_zwNormalizeDeclValue` 族臂 + 通用 anchor()/
   anchor-size() 拒面臂（var() 保形先行）；part03 position-try 简写拆分写臂
   （双长手落库、`position-try` 键不占声明位、空值清双长手）+ 简写合成读臂 +
   removeProperty 清双长手臂；part01c `__zwPlainComputed` 族 computed 臂
   （as-specified 恒等 + css-wide 计算期语义 + position-try 双长手合成）；
   js_dom_bridge `decl_supported` 族臂 + 函数拒面臂（CSS.supports 前置门）。
   宿主回调四枚：`__zw_anchor_normalize_decl`/`__zw_anchor_computed`/
   `__zw_anchor_try_split`/`__zw_anchor_func_check`。
3. **part01 宿主回声覆盖两臂（跟进刀，首跑后补）**：anchor 族七属性在 gCS
   getPropertyValue 与直取形两路径 plain-first（position-area 先例同构——host 对
   registry 外未设值回 'initial'/'inherit' 字面量、设值回 ''，均失真；
   test_computed_value 走 `getComputedStyle(target)[prop]` 直取形）。首跑
   position-visibility-computed 等五 computed 页 0 翻正的根因即此缺臂。
4. **`anchor-size(` 子串门修正（跟进刀）**：`anchor-size(` 不含 `anchor(` 子串
   （anchor 与 `(` 间有 `-`）——首跑 anchor-size-parse-invalid 仅 2 翻（含字面
   `anchor(` 的嵌套案）根因；part06 写面臂与 js_dom_bridge CSS.supports 臂双形
   并查修正（Rust 扫描器本体无此病，短路前置 `contains` 检查同病同修）。

## 验证（两段跑测 + 单测 + 门禁）

- **anchorf（基线刀）**：anchor 语料 **10,182P → 10,353P 净 +171**，12 页
  improved 零回归（anchor-parse-invalid 25F→0、position-try-parsing 27F→0、
  position-try-order-basic 27F→0、position-try-order-position-area 27F→0、
  position-try-fallbacks-parsing 24F→0、position-visibility-parsing 18F→0、
  anchor-size-parse-invalid 22F→2P/20F、anchor-scope-parsing 5F→0、
  position-try-order-parsing 3F→0、position-anchor-basics 12F→7F、
  anchor-name-basics 9F→5F、position-try-order-inset-modified-containing-block
  4F→0）；css-color 10,905P 零漂移。
- **anchorf2（computed/子串跟进刀）**：anchor **10,353P → 10,455P 净 +102**，
  9 页 improved 零回归（visibility-computed 19F→0、fallbacks-computed 24F→0、
  anchor-size-parse-invalid 20F→0、position-anchor-basics 7F→0、
  anchor-name-basics 5F→0、anchor-scope-computed 10F→0、try-computed 9F→0、
  order-computed 7F→0、anchor-ident-function 13F→12F）；css-color 10,905P 零漂移。
- **批合计**：anchor 语料 10,182P → 10,455P **净 +273 逐页零回归**，18 页清零。
- **单测**：`anchor_props` 8 组（文法矩阵/函数拒面/保形放行/computed 恒等），
  单测矩阵与语料逐案对齐（含 anchor-size 隐式形/flip 序/zero fallback/
  夹心拒）。
- make test 全量门禁：见 master.md 本批段（zwafgate 日志）。
- 证据产物：`.acceptance/wptfyi-css/results-anchorf{,2}/`（JSON + transcript），
  比对器 `cmp-anchorf.py`，日志 `logs/anchorf{,2}-*.log`、
  `logs/zwafgate-make-test.log`。

## 挂账

- anchor-ident-function 12F：`ident()` **求值**面（`ident("--a" calc(...))` →
  `--a42` 串接，需 calc/sign 求值管线）——parse 接受面已通（写不拒），本批面外。
- position-area 布局行为桶（scrolling/insets/wm-dir/align-justify 等）+ 组F
  布局桶 ~2,470F：需 anchor 定位布局实现，架构面独立批。
