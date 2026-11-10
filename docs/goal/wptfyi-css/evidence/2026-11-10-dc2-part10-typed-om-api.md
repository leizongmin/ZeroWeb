# DC-2 首批：Typed OM API 入口面（part10）

**2026-11-10**。css-typed-om 语料簇全量同口径重跑：**Pass 25 → 9830**（净增
9,805），Fail 2,383，365 页（其中 65 页零失败）。DC-1 基线见
[baseline-and-classification](2026-11-10-dc1-baseline-and-classification.md)。

## 交付物

- `crates/engine/src/js_dom_shim/part10-typed-om.js`（新，~1,810 行，自包含
  IIFE 幂等）：CSSStyleValue 基类（parse/parseAll）+ CSSKeywordValue/
  CSSNumericValue（to/toSum/equals/算术四则）/CSSUnitValue/CSSMathValue 族
  （Sum/Product/Min/Max/Clamp/Negate/Invert/Abs/Sign/Hypot）/
  CSSUnparsedValue/CSSVariableReferenceValue/CSSTransformValue + 组件族/
  CSSImageValue/色类补挂（HWB/LCH/Lab/OKLCH/OKLab）/CSS 命名空间数值工厂/
  StylePropertyMap（inline+declared）/`Element.attributeStyleMap`/
  `Element.computedStyleMap`/`CSSStyleRule.styleMap`/`CSSNumericValue.parse`
  静态/225 属性语法表（语料 runPropertyTests 逐页提取）。
- `js_dom_bridge.rs`：part10 挂载块（8 行注释 + include_str）。
- `part06.js`：`__zwAttachRuleStyleMap` 钩子两处调用（规则对象 plain
  object 无原型挂点；钩子缺席无操作）。
- 零新 Rust 回调：computed 复用 part01 getComputedStyle 代理链，declared
  复用 part03 `_zwMStyleProxy`。

## 验证链

| 检查 | 结果 |
|---|---|
| 独立 node 冒烟 `.acceptance/wptfyi-css/part10-smoke.js` | 136 pass / 0 fail |
| 定向探针（numeric-objects/equals/set-shorthand/parse/transformvalue） | 全绿（numeric 138/0、equals 11/0、set-shorthand 8/0） |
| 全量 css-typed-om（test-guard 4G/8G/3600s + systemd scope） | Pass 9830 / Fail 2383 / 365 页 |
| `cargo test -p zero-engine --lib js_dom` | 1456 通过；7 失败为基线预存（stash 对照确认：fetch/websocket/img-settle 环境面，与本批无关）；r3046 expando 枚举回归已修（computedStyleMap 改非枚举 defineProperty） |
| `cargo fmt --all -- --check` | 本批文件无 diff（tests/wpt-runner/src/testharness.rs 预存差异非本批引入，不处理） |
| `cargo clippy -p zero-engine --all-targets -- -D warnings` | 干净 |
| transcript | /tmp/css-typed-om-part10-v2.log；JSON 汇总 `.acceptance/wptfyi-css/results-part10/`（本地，不入库） |

## 残余归口（Fail 2,383 分类）

1. **简写/多 token 语法长槽缺面**（最大簇）：border-radius/border-width/
   border-color/border-style/break 等 properties 页——set/get 需宿主简写展开
   与多 token 语法归一（part03 host style 面）；本批已补 margin 简写展开
   （set-shorthand 8/0）作为该簇首个案例。
2. **computed 颜色管线**：cssRGB/cssColorValue/cssHWB/cssLCH/cssOKLCH 等
   色类页——computed value 通路缺失（DC-2 次批 color-mix()/现代色函数同
   归口）。
3. **computed transform/几何管线**：cssRotate/cssSkew 残余——used-value/
   computed 结构化输出缺失。
4. **`.any.js` 通道**：idl_test 缺失 2 页脚本错（含 idlharness 依赖，通道
   批次处理）；channel 外余量同 DC-1 三分类。

## 语义要点（供后续批次复用）

- 序列化规范形：嵌套 Sum/Product 成员加括号、min/max/clamp 参数裸表达式、
  Sum 后位 Negate 融合 ' - '、Product 后位 Invert 融合 ' / '、首位
  Negate/Invert 括号保形（css-typed-om-1 § calc-serialization）。
- `toSum`：请求序贪心填充（同名直取 + 绝对族 factor 换算 + 相对按名直取），
  余量 TypeError，无参按单位名排序；换算浮点 12 位有效数字清尾。
- Clamp 构造器：参数量/类型 TypeError，兼容型（percent 并入对方单族）放行；
  解析器文法面经 `_zwOmCtorStrict` 旗标绕过（CSS 文法允许混合
  length-percentage）。
- `CSSNumericValue.parse`：裸 token → UnitValue；裸 min/max/clamp 直返；
  calc 包裹 Unit 原子 → Sum、math 节点保形；绝对族折叠；sign()/除零/复合
  单位 DOMException SyntaxError。
- 空值列表 variadic 数学构造器 DOMException SyntaxError（assert_throws_dom
  面）；属性名解析 ASCII case-insensitive + IDL camelCase dasherize 仅
  小写开头驼峰形。
