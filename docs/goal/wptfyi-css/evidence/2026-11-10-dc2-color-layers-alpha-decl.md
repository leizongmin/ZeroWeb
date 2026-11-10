# DC-2 四批：css-color 语法面收批——color-layers/alpha()/relative 通道表/color-mix hue 空间门

**2026-11-10**。css-color 簇 63 页全量重跑：**Pass 10,530 → 10,756（净增 +226）**，
Fail 480 → 254。终树产物 `.acceptance/wptfyi-css/results-cc8/css-color.json`
（守卫包裹，`make testharness-css-color`）。

## 账面与逐页不降核对

- 基线（再批，c658ca70c）：10,530P / 480F（results-cc6）。
- cc7 首跑：10,736P / 274F；5 页净增、1 页回归——computed relative-color
  1100→1080（-20），通道表槽校验过严误拒 corpus valid 形。
- 校验放宽修正后 cc8 全量终跑为提交账面：**10,756P/254F**，逐页不降对比
  cc6 为 **5 增 / 58 平 / 0 回归**。
- 主要增面：**invalid-color-layers-function 0 → 93**、
  **invalid-relative-color 73 → 161**（+88）、
  **invalid-color-mix-function 129 → 141**（+12，hue 方法空间门）、
  **alpha-color-parsing-invalid 0 → 23**、
  **valid-color-layers-function 160 → 170**（+10，normal 前缀省略）。

## 改动面（单文件 `js_dom_shim/part11-computed-color.js`，+205/-3）

| 块 | 内容 |
|---|---|
| relative 通道表槽校验 | `_zwCCDeclRelSlotOk`：槽 token 三态——`none` 恒合法、数值形按单位归类（百分比禁入 hue 槽、角度仅 hue 槽）、关键字白名单（rgb r/g/b、hsl h/s/l、hwb h/w/b、lab/oklab l/a/b、lch/oklch l/c/h）；calc 形按是否含通道关键字分流——含关键字项与带维数项加减混型即拒（`calc(h + 1deg)`/`calc(r + 1%)` invalid 锚），乘除维数标量合法（`calc(r * 100%)`、`calc((h / 360) * 360deg)` corpus valid 锚）；纯字面 calc 按结果类型查槽 |
| 通道表整表校验 | `_zwCCDeclRelTableOk`：恰好 3 通道槽 + 可选 `/ alpha` 槽；color() 表首 token 为空间名（rgb 族→r/g/b、xyz 族→x/y/z）；关键字在全部槽位自由排列且含 `alpha`（`b alpha r / g`、`h alpha l / s`、`x x x / x` valid 锚）；rgba/hsla 别名折返提前到校验前；校验失败整值拒（null） |
| color-mix hue 空间门 | hue 方法（shorter/longer/predecessor/successor hue）仅限 hue 承载空间 hsl/hwb/lch/oklch；其余插值空间带 hue 方法整值非法（corpus invalid 12 面） |
| alpha() 声明面 | `_zwCCDeclAlpha`（css-color-5 §relative-alpha）：形态恰为 `alpha(from <color> / <alpha>)`——缺 from/缺 origin/缺 alpha 值/额外 token（`0.5 0.5`）/origin 非色（`42`/`none`）均拒；嵌套 alpha 递归校验；alpha 槽深度 0 恰一 token；calc 内 ident 白名单（alpha + 树函数 calc/min/max/clamp/sibling-index/sibling-count，通道关键字 `calc(r * 0.5)` 拒）；裸 `sibling-index()`/`sibling-count()` 与合法形委托基座归一（alpha-color-parsing-valid 42 锚零漂移） |
| color-layers 声明面 | `_zwCCDeclLayers`（css-color-6 #color-layers）：可选单一 blend-mode（16 关键字白名单，plus-lighter/plus-darker 在列）+ n≥1 逗号分隔色层（逐层可解析校验、无空段）；序列化 `normal,` 前缀省略（缺省重建一致）、其余 blend 与色层逐字保形（valid 160 逐字锚）；`color-layers(normal)` 零色层拒 |
| computed 入口 | color-layers 进 computed 路径递归校验，合法返 declared 归一形 `{ s }`、非法 null |

## 关键语义决定（语料实证，decl-smoke 2,440/2,440 全绿）

- **校验从严收紧的边界**：cc7 回归（computed relative-color -20）定性为
  `_zwCCDeclRelSlotOk` 对 calc 混型项的切分误判——修正后 valid 锚全绿。
  声明面校验只拒 corpus 有 invalid 锚的形态，无锚形态保基座口径不拦
  （`_zwCCDeclRelTableOk` 非 channel-table fn 返 true 保上层）。
- **color-layers computed 口径**：本批以 declared 归一形作为 computed 值
  （色层不可解析为单一色，语料期望即层列表形）；`normal` 前缀缺省省略
  与 corpus 替换锚一致。
- **alpha() 不重排**：合法形不产自定义规范串，委托基座 `_zwCCPrevNormalizeDecl`
  （origin 规范化 + calc 折形），避免与 alpha-color-parsing-valid 42 锚漂移。

## 验证

- 冒烟 `.acceptance/wptfyi-css/decl-smoke.js`（真执行语料页 + stub 收集器，
  扩容后 2,440 对，含 color-layers 两页）：fail 0、skip(var) 14。
- 全量 `make testharness-css-color`（systemd scope + test-guard）：cc7
  10,736P（1 回归）；修正后 cc8 终树账面 **10,756P/254F**，零回归。
- 守卫 `cargo clippy -p zero-engine --all-targets` exit 0；
  `cargo test -p zero-engine --lib` 3828 passed / 7 failed——失败名单与
  上批证据同名单（网络/img/iframe 侧既有失败，与本批色彩 shim 无关）。

## 残余（254F 分桶）

- computed relative-color 69、computed-rgb 21、alpha-color-computed 5、
  computed lab/mix/contrast/color-function 小额（合计 ~15）——computed
  面校验/序列化残余。
- light-dark 32（light-dark-image 31 + currentcolor 1）、system-color
  55（consistency 27 + compute 14 + support 12 + with-system-color 2）
  ——css-color-5/6 特性面未建模。
- opacity-computed 20、relative-color-out-of-gamut 22、animation/杂项
  ~15。
