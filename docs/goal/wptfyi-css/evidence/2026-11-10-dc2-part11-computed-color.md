# DC-2 次批后半：computed 真色彩数学管线（part11-computed-color 接线）

**2026-11-10**。css-color 簇 63 页全量重跑：**Pass 7,229 → 9,318（净增 +2,089）**，
Fail 1,692。终树产物 `.acceptance/wptfyi-css/results-cc4/css-color.json`
（守卫包裹，`make testharness-css-color`，rebase 后树复跑与 rebase 前 cc3
逐位一致）。

## 账面与逐页不降核对

- 基线（次批，51ddf47ae）：7,229P。
- 终树：9,318P / 1,692F；**9 页净增、54 页持平、0 回归**。
- 主要增面：computed-relative-color **0 → 1,100**（前轮 -14 currentColor 豁免页
  恢复至基线 583P 持平后整页 computed 面翻绿）、computed-color-mix-function
  **0 → 930**、alpha-color-computed 0 → 24、color-mix/relative out-of-gamut
  两页 0 → 23、valid-rgb 回归修复 54 → 70（基线 70 持平）。
- 中间轮 cc1/cc2 各 8/2 页回归全部定位修复后再入账（weakmap 键类型、
  NeedsPlain 关键字误抢答——见修复记录）。

## 改动面

| 文件 | 内容 |
|---|---|
| `js_dom_shim/part11-computed-color.js`（新增接线，1,871 行） | computed 数学管线：色空间表（srgb/hsl/hwb/lab/lch/oklab/oklch/xyz×2/display-p3×2/a98/rec2020/prophoto 全精度规范矩阵 + rec2020 传递函数 β=0.08145 拐点/exponent 1/0.45 + prophoto 1.8 γ/16·512 线性段）；`__zwComputedColor` 覆盖别名（`_zwCCPrevComputed` 回落链）；relative color 解析（origin 关键字 r/g/b/h/s/l/alpha/x/y/z + calc/percent 域换算 + color() 目标 x/y/z 序数别名任意空间）；color-mix 坐标混合；none 传播策略（同语法形传播、color() 目标消解 0）；序列化（8 位有效/6 位 hue、e-notation 展平、rgb 目标不 clamp 保 color(srgb) 现代形） |
| `js_dom_bridge.rs` | include part11-computed-color（后挂覆盖 part11-color 的 computed 别名，declared 归一仍走前者） |
| `js_dom_shim/part03.js` | inline style raw 影子（`_zwRawByHandle` WeakMap/`_zwRawBySel` Map 双键）：声明写面归一变形且原值含 none 时存原值；`__zwRawDecl` 读取钩子；键限对象（WeakMap CanBeHeldWeakly——wpt-runner 嵌入 handle 为宿主标量 id，`set` 非对象键 TypeError 曾致 valid-rgb 16 写面翻红） |
| `js_dom_shim/part01c.js` | `_zwPlainDecl` raw 影子优先读取（declared 归一形与 computed 保 none 形分流——valid corpus 期望 `rgb(128, 0, 0)`、computed corpus 期望 `color(srgb 0.50196078 none none)`） |
| `js_dom_shim/part11-color.js` | 数值精度修正：declared lab 族/color() 分量 8 位有效（round-trip 等值）、e-notation 字面解析、rgb 恒消解注释对齐 |

## 关键语义决定（语料实证）

- **none 策略**（css-color-5 §none）：同语法形目标通道传播 `none`；
  color() 目标一律消解为 0（css-color-4 无 none 通道概念，Chrome 实证）。
- **color() 字面 origin 同空间直取**：`kind 'own'` 无 fn 字段，空间相等即判别；
  rgb 往返投影会引入 e-4 级噪声与负号，语料以精确 0 断言。
- **legacy rgb 相对目标不 clamp**：越界序列化保 `color(srgb ...)` 现代形
  （Chrome 行为；resolve 级 clamp 曾毁嵌套无损往返）。
- **alpha 关键字**：legacy rgb 组件槽按 number 语义 ÷255，alpha 槽原样。
- **NeedsPlain 关键字排除**：css-wide 关键字（initial/inherit/unset/revert/
  revert-layer）不抢答——plain 爬升臂不建模关键字，`initial` 误爬升会取父值
  （inheritance.html 回归实证）；宿主 cascade 对关键字本就正确产出。

## 验证链

- 语料对账器 `.acceptance/wptfyi-css/cc-corpus-check.js`（页真值比较器：
  getNumbers 长度相等 + 逐个 |Δ|≤ε + 非数字文本一致，默认 ε=0.01）：
  **1,128/1,148 过，fail=5** 全为嵌套无损往返容差组（tol=0.0001，Δ≤0.0036；
  默认 ε=0.01 全过——转换链系统精度，停追不入账）。
- 全装配 `node --check`（25 片按 bridge include 序拼接，146k 行）。
- js_dom 全库（终树，守卫包裹）：**1,457 通过 / 7 失败**；7 失败与上批证据
  逐位一致（fetch filtered matrix、websocket×2、readable stream、iframe SW
  dispatch、img/svg settle），既有非本批引入；t46/t48×2/t49/t49fix 钉测全绿。
- `cargo fmt --all -- --check` 零 diff；`cargo clippy -p zero-engine
  --all-targets -- -D warnings` 零告警（守卫包裹）。

## 残余桶（1,692F 归类）

| 桶 | 量 | 定性 |
|---|---|---|
| 声明面归一缺失 | ~991 | valid-relative-color 564、valid-color-mix 427——现代形声明序列化保原串，valid corpus 期望归一形（次批后半只建模 computed 面） |
| invalid 真校验缺失 | 403 | invalid-relative 160、invalid-mix 141、invalid-layers 93（透传形需真解析后才可拒） |
| computed 杂项 | ~298 | computed-relative 69、alpha-invalid 23、computed-rgb 21、opacity 20、relative-out-of-gamut 20、computed-mix 18、system-color 族 ~53、light-dark-image 31 等 |

## 下一步

1. **DC-2 再批**：未 fetch 目录语料扩面（~8.1k）；`.any.js` 变体通道（588）。
2. 声明面归一（~991）与 invalid 真校验（403）。
3. **DC-3**：全量对账（验收线 182,392）。
