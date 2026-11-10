# DC-2 再批：css-color corpus 声明面归一（part11-computed-color declared 覆盖段）

**2026-11-10**。css-color 簇 63 页全量重跑：**Pass 9,318 → 10,530（净增 +1,212）**，
Fail 1,692 → 480。终树产物 `.acceptance/wptfyi-css/results-cc6/css-color.json`
（守卫包裹，`make testharness-css-color`）。

## 账面与逐页不降核对

- 基线（次批后半，1cc2f4e0f）：9,318P / 1,692F（results-cc4）。
- cc5 首跑：10,527P / 483F；6 页净增、55 页持平、2 页 contrast-color 回归
  （-3）——嵌套 contrast-color 参与 contrast-color origin 被覆盖层误拒。
- contrast 透传修复后定向重跑核验恢复（results-cc5b：valid 16P、computed
  13P 与 cc4 基线一致）；cc6 全量终跑为提交账面：**10,530P/480F**，
  逐页不降对比 cc5 为 **2 增（contrast valid 14→16、computed 12→13）/
  61 平/0 回归**，其余页逐位一致。
- 主要增面：**valid-color-mix-function 250 → 677**（+427）、
  **valid-relative-color 583 → 1,145**（+562）、invalid-color-mix-function
  0 → 129、invalid-relative-color 1 → 73、invalid-contrast-color-function
  0 → 9、computed-color-mix-function 930 → 943。

## 改动面（单文件 `js_dom_shim/part11-computed-color.js`，+527/-17）

declared 规范化覆盖段（`__zwColorNormalizeDecl` 别名链尾挂覆盖）：

| 块 | 内容 |
|---|---|
| 相对色声明面 | rgba/hsla 别名折返；origin 规范化（keyword 小写保形、srgb 族 legacy 数字折叠——通道 none→0 消解 `'rgb(none none none)'→'rgb(0, 0, 0)'`、alpha none→0、`'hsl(120deg 40% 40% / none)'→'rgba(61, 143, 61, 0)'`；lab 族 clamp/hue mod/alpha 数字化自形；color() xyz 别名展开；嵌套 mix/relative 递归规范化）；通道表 rest 槽 calc 规范形（数×通道、除常数折 6 位 `0.333333 * a`、减常数折负前置 `-20 + l`、加减中乘式项括号化）、color() 通道 `xyz`→`xyz-d65` 别名；rest alpha 前导点补 0 语料行为表（rgb 恒补、hwb/lab 仅 origin 保原样时补、hsl/lch/oklab 恒保——Chrome 按 fn 实现差异，无规范统一规则）；整值有效性以 computed resolve 为 oracle |
| color-mix 声明面 | method 子句规范化（插值空间缺省 oklab 与 hue 缺省 shorter 时整体省略子句、`in xyz`→`in xyz-d65`、非缺省 hue 方法保形）；n≥1 色参支持（单参 `red 100%`→`red`、多参部分未指定补均分剩余 `red 50%, green, blue`→25%/25%、全未指定全省）；权重序列补全后等于均匀缺省才省略（`70% red, 50% blue` 全保留、`red 75%, blue 75%` 双保留锚）；calc 权重保形逐侧输出；无 in 子句浅面走基座 declared 序列化（hsl→rgb 折、`color(srgb .1 .2 .3)` 保原 token 数字）；域外权重/method 子句非法即拒 |
| contrast-color | 恰一色参否则拒；嵌套 contrast-color 与 origin 形未建模面保原串透传（cc4 回归锚） |
| 浅面委托 | `_zwCCShallowColor` 基座 `_zwCCPrevNormalizeDecl` 引用绑定（mix 无 in 侧） |

## 关键语义决定（语料实证，decl-smoke 1,811/1,811 全绿）

- **origin/side 声明面折叠 ≠ computed 口径**：srgb 族含 none 的 origin 走
  legacy 数字形（none→0），与 computed 面保 none 形（part03 raw 影子分流）
  是两个口径——`rgb(from rgb(20% none 60%) r g b)` → origin 段
  `rgb(51, 0, 153)`（声明面）而非 `color(srgb 0.2 none 0.6)`（computed 面）。
- **权重省略=缺省重建一致**：仅当输出省略权重后按均匀缺省能重建原权重序
  列才省略（双 50%、单参 100%、多参全未指定）；其余相等权重保留双写
  （`red 75%, blue 75%`、`red 0%, blue 0%` corpus 锚，修正前批「双相等即省」
  过宽规则）。
- **mix 双口径**：带 in 子句（解析面）完整规范化（侧数字化），无 in 子句
  （浅面）逐侧基座 declared 序列化（`color(srgb .1 .2 .3)` 保 `.1`）——
  同一色形两种期望（`in xyz` 锚数字化 vs 无 in 锚保形）。
- **rest alpha 前导点**：`.25`→`0.25` 补 0 保原位数字（不重排尾零 `.20`→
  `0.20`）；按 fn 语料行为表驱动（rgb/hwb/lab/hsl/lch/oklab 交叉无统一规律，
  注释标注「Chrome 实现差异」）。
- **css-color-6 未建模面透传**：contrast-color 嵌套/origin 形保原串（cc4
  曾以基座透传通过，覆盖层不得收紧）。

## 验证

- 冒烟 `.acceptance/wptfyi-css/decl-smoke.js`（真执行语料页 + stub 收集器，
  1,811 对）：fail 435 → 0；color-layers 页（css-color-6）本批明确跳过不
  入冒烟口径（透传保持，master.md 前批决定）。
- 全量 `make testharness-css-color`（systemd scope + test-guard）：cc5
  10,527P/483F；cc5b 定向 contrast 核验恢复；cc6 终树账面 10,530P/480F。
- `cargo fmt --check` 零 diff；守卫 `cargo clippy -p zero-engine
  --all-targets` exit 0；`cargo test -p zero-engine --lib` 全量（既有 7 失败
  与上批证据同名单）。

## 残余

- color 簇 480F：color-layers 语法面（93 invalid + 10 valid）、light-dark、
  杂项 computed（alpha-invalid、system-color 族等）——后续批次。
