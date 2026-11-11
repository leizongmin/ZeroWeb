# DC-2 组 E：RCS+var 入口放行——var-origin 相对色 13F 翻正

日期：2026-11-11　批次：DC-2 color none 后挂账桶·var-origin 批　基线：results-nonefinal
（css-color 10,887P/124F，var 相关 13F 在 parsing/color-computed-relative-color.html）
+ probe-vrprobe 定向（wpt-data gitignore 面，证据走 transcript 与 results-vrprobe）

## 1. 钉面与归因（13F 两形态）

| 组 | 案数 | 形态 | 实测 actual |
|---|---|---|---|
| 形桶 | 7F | 值已语义正确，序列化 legacy vs 期望现代形 `color(srgb)`/`lch()`/`lab()` | `rgba(0,0,255,0.8)`/`rgb(110,110,110)`/`rgb(152,152,152)` 等 |
| calc 桶 | 6F | 通道槽 `calc(h + 180)`/`calc(l / 2)`/亮度混合式/`calc(alpha * 0.8)`（含 sibling-index 组合 1 案）→ 黑底 | `rgb(0, 0, 0)` |

死点链（源码追踪 + 探针实证）：raw 含 `var(` 被 `__zwComputedColor` 入口守卫
短路给基座 part11-color → 基座 var 形返 null（form raw）→ 落宿主链。宿主侧
var 级联替换后经 Rust `parse_relative_color`（css-parser values/color.rs）计算：
值面正确（灰 110/152 = origin 语义变换结果）但**序列化恒 legacy RGBA**（Rust
侧无现代形 gCS 输出）；calc 通道 `parse_rcs_channel` 拒 → 整声明 invalid →
initial 黑。JS 管线（`_zwCCVarLookup` inline+作者快照臂、chanexpr calc 通道、
`_zwCCSerialRelative` 现代形序列化）能力完备但入口被守卫阻断——var 作者臂
上批交付后从未被 RCS 案触达。

## 2. 修复（part11-computed-color.js 入口一处）

`__zwComputedColor` 的 var 分支先试 RCS 自解：`_zwCCRcsFormRe`（10 色函数
`from` 语法）命中 raw，或纯 var 引用且 `_zwCCVarLookup` 替换值含 RCS 形时，
走 `_zwCCResolveText`（var 臂文本替换递归 + 嵌套 relative origin）→
`_zwCCSerialResolve` 现代形输出；自解失败/非 RCS 形保基座→宿主零漂移。

零翻红论证：宿主对 RCS 形输出恒 legacy（或 calc 拒黑），WPT RCS 期望恒现代形
（css-color-5 computed serialization）——接管域内不存在「宿主已通过」案，
pass→fail 面为空；非 RCS var 形（legacy/现代色值的 var 引用）不接管，通路不变。
嵌套链（`--mygray: lch(from var(--mycolor) l 0 h)`、`lch(from var(--mygray) l 30 h)`）
由 var 臂递归文本替换覆盖（作者快照返声明原文，内部 var 再查——host 自定义属性
computed 值已替换，两级结果一致）。

## 3. 探针（results-vrprobe，zz-probe-varrel 跑后即删）

- A 组基线（无 var）2 案：relative-out 现代形 + chanexpr calc 既有能力对照 ✓。
- B 组语料 13 案逐字：**13/13 翻正**（形桶 7 + calc 桶 6，含 sibling-index
  组合案——`_zwCCResolveText` 入口 `_zwCCSubSibling` 先替换再 var 臂）。
- C 组零漂移控制：`var(--plain)`（非 RCS 纯 var，基座/宿主通路）✓。
- D 组入口直调：`__zwComputedColor('lch(from var(--color) l 0 h)')` 返
  `lch(46.27771 0 134.38387)`——8 位精度输出经语料 fuzzy 通道（epsilon 0.02）
  正确通过（探针 D1 断言用精确串比对属探针自身写法问题，非代码缺陷；语料 B4
  同值 P 实证）。
- provenance：build 日志 `Compiling zero-engine` ✓（include_str! 陈旧二进制坑，
  touch js_dom_bridge.rs 强制）。

## 4. 终验（results-vrfinal，三语料）

- **css-color 10,887 → 10,900（净 +13）逐页零回归**（逐页 JSON diff：翻正恰
  13 案全在 color-computed-relative-color 页；vrfinal 原始 10,916P/112F 含
  zz-probe 页 16P/1F，语料口径剔除后 10,900P/111F；JSON 口径 F 122→109 同向）。
- **css-values 2,213P 持平**（逐页零回归）。
- **css-typed-om 9,872P 持平**（逐页零回归；无 RCS+var 钉面）。
- make test（systemd zw-vvtest 唯一日志 zvvf-make-test.log）：**3848P/7F**，
  7F = main 既有欠账组逐位一致（img settle ×2 / fetch / iframe SW /
  readable stream / websocket ×2，HEAD 判定见 master.md 碰撞记录），零新增。

## 5. 挂账（css-color 残余 111F）

- **pow/math 通道 1F**（`oklch(from green pow(l, 1) c h)`）：根因已定位——
  `_zwCCSplitModern` 按空白切通道 token，`pow(l, 1)` 内部逗号+空格被切碎
  （4 token ≠ 3 拒），需括号平衡感知切分（part11-color `_zwCTopSplitComma`
  同型）；var 批不带。
- **嵌套 relative 往返精度 5F**（`color(from oklab(from color(xyz ...)))`，
  xyz-d50/d65 各半）：中间空间转换第 5 位小数损耗，期望精确回值；需同空间
  原生坐标直取链（mix 面有先例）。
- 系统色 27F（consistency 页）、out-of-gamut 20F（gamut mapping 建模）、
  opacity-computed 20F、rgb 5F / lab 4F / contrast 6F / mix 3F、
  color-valid-relative 2F 等。
