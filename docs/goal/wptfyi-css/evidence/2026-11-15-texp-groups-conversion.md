# `target` expando 刀逐组转化率实测——337 页 harness 面全量

2026-11-15。承接 [刀落地](2026-11-15-target-expando-shim-knife.md) §3：按
[解锁 evidence](2026-11-15-target-expando-interpolation-unlock.md) §3 的 337 页
分布逐组实测 B1 转化率，并按失败签名分桶定位后续刀。

**结论一句话**：337/337 页全部实测，335 页解锁（2 页待 WAAPI 门刀）、
setup 崩溃面清零；解锁面 B1 **6,691P / chrome 分母 55,861（12.0%）**，
相对刀前（全 0 声明）净增 **+6,691P**。残差三大桶：CSS.supports 覆盖门
**25,135（49.5%）**、WAAPI 支撑门 **11,020（21.7%）**、插值语义真差
**14,502（28.6%）**——前两桶都是支撑面刀，语义桶占比不足三成。

## 1. 方法

- **页面全集**：`grep -rl interpolation-testcommon` wpt-data/css 全 337 页
  （42 顶层目录；zz- 探针页剔除）。B0 侧全部为 declared 0 崩溃
  （[刀落地](2026-11-15-target-expando-shim-knife.md) §2 跨组抽验同证），故
  B1 单边跑即可：**转化率 = Pass / chrome summary `c[1]` 分母**。
- **跑法**：`run-css-testharness-corpus.sh` JSON 通道按 invocation 覆写
  `{JSON_DIR}/{corpus}.json` 且 **transcript 每次 invocation 开头截断**（共
  用 transcript 的两次 invocation 前次块丢失，css-animations 首跑即踩）→
  按 pass 分目录互不覆盖：`results-texp-b1-p1`（ONLY_SUBSTR=interpolation）、
  `-p2`（composition）、`-p3`（残余页单页子串 / 小语料全量）、`-p4`（单页
  补测），masking 21 页在刀当轮 `-b1` 根目录（全量 61 页）。分析合并四
  pass 与根目录，逐页对 chrome
  `tests/wpt-runner/wptfyi-cache/summary-chrome.json`（key `/css/...`，
  `c:[pass,total]`）。分析器与链脚本为本机产物（`.acceptance/wptfyi-css/
  texp-b1-groups-analysis.py`、`run-texp-b1-groups.sh`），不入库。
- **二进制**：`target/release/zero-wpt-runner` mtime 09:17:36（含刀
  8ee02114e，[provenance](2026-11-15-target-expando-shim-knife.md) §2），
  全链同二进制零重建；主链 systemd unit `zw-texp-b1-groups`
  （MemoryMax=10G、RuntimeMaxSec=10800），单 invocation 经 test-guard
  `--per-proc-mem 4 --total-mem 8 --time-limit 3600`。主链 65 invocation +
  补测链 6（漏 p1 的 6 corpus：gaps/anchor-position/animations/borders/
  color-adjust/content）+ 单页补测 1。
- **解锁判定**：B1 声明子测数 ≥ 0.8 × chrome 分母（crash 页声明数为 0）；
  声明数偏差 >20% 的页单列核查。

## 2. 逐组转化率（42 组全量）

| 组 | 页 | 解锁 | B1 P | chrome P | chrome T | 转化率 |
|---|---|---|---|---|---|---|
| css-masking | 21 | 21 | 489 | 4028 | 5818 | 8.4% |
| css-backgrounds | 30 | 30 | 787 | 4930 | 5126 | 15.4% |
| css-transforms | 32 | 32 | 269 | 4800 | 4800 | 5.6% |
| motion | 22 | 22 | 0 | 4562 | 4586 | 0.0% |
| css-values | 18 | 18 | 518 | 4051 | 4051 | 12.8% |
| css-grid | 16 | 16 | 1200 | 3460 | 3544 | 33.9% |
| css-sizing | 16 | 16 | 1031 | 3483 | 3483 | 29.6% |
| css-fonts | 16 | 16 | 313 | 3268 | 3268 | 9.6% |
| css-gaps | 21 | 21 | 0 | 2392 | 2392 | 0.0% |
| css-shapes | 7 | 7 | 0 | 2224 | 2224 | 0.0% |
| filter-effects | 12 | 12 | 134 | 2048 | 2076 | 6.5% |
| css-text | 12 | 11 | 126 | 1250 | 1292 | 9.8% |
| css-multicol | 6 | 6 | 114 | 1272 | 1272 | 9.0% |
| CSS2 | 7 | 7 | 297 | 1122 | 1122 | 26.5% |
| css-ui | 11 | 11 | 90 | 1038 | 1038 | 8.7% |
| css-position | 10 | 10 | 115 | 971 | 971 | 11.8% |
| css-flexbox | 6 | 6 | 32 | 786 | 786 | 4.1% |
| css-anchor-position | 4 | 4 | 402 | 700 | 700 | 57.4% |
| css-transitions | 5 | 5 | 76 | 660 | 700 | 10.9% |
| css-lists | 2 | 2 | 74 | 420 | 696 | 10.6% |
| css-align | 6 | 6 | 54 | 686 | 686 | 7.9% |
| css-box | 10 | 10 | 0 | 648 | 648 | 0.0% |
| css-borders | 3 | 3 | 57 | 626 | 626 | 9.1% |
| css-text-decor | 3 | 2 | 83 | 608 | 608 | 13.7% |
| css-break | 3 | 3 | 0 | 438 | 438 | 0.0% |
| css-images | 5 | 5 | 5 | 372 | 372 | 1.3% |
| fill-stroke | 2 | 2 | 0 | 192 | 364 | 0.0% |
| css-animations | 3 | 3 | 203 | 354 | 354 | 57.3% |
| css-color | 3 | 3 | 15 | 332 | 332 | 4.5% |
| css-viewport | 1 | 1 | 0 | 202 | 202 | 0.0% |
| css-inline | 5 | 5 | 0 | 198 | 198 | 0.0% |
| css-size-adjust | 1 | 1 | 0 | 196 | 196 | 0.0% |
| css-logical | 6 | 6 | 22 | 180 | 180 | 12.2% |
| css-contain | 1 | 1 | 71 | 130 | 130 | 54.6% |
| css-overflow | 1 | 1 | 52 | 126 | 126 | 41.3% |
| css-tables | 1 | 1 | 6 | 120 | 120 | 5.0% |
| css-color-adjust | 2 | 2 | 25 | 84 | 84 | 29.8% |
| css-display | 2 | 2 | 30 | 67 | 67 | 44.8% |
| css-color-hdr | 1 | 1 | 0 | 64 | 64 | 0.0% |
| css-content | 2 | 2 | 1 | 43 | 43 | 2.3% |
| compositing | 1 | 1 | 0 | 42 | 42 | 0.0% |
| css-variables | 1 | 1 | 0 | 36 | 36 | 0.0% |
| **TOTAL** | **337** | **335** | **6691** | 53209 | **55861** | **12.0%** |

读法：grid/sizing（calc-size 面未接即 29-34%）与 anchor-position/
animations/contain/display/overflow（41-57%）转化率最高——支撑面到位的
组即刻兑现；motion/shapes/gaps/box/break/inline/viewport 等 0% 组全部卡
supports 门（§3 首桶），非语义不能。

## 3. 失败签名分桶（全量 50,731 fail）

| 桶 | 数量 | 占比 | 性质与后续刀 |
|---|---|---|---|
| CSS.supports 覆盖门 | 25,135 | 49.5% | `CSS.supports(prop, val)` 假：supports 覆盖刀族 |
| 插值语义真差 | 14,502 | 28.6% | `assert_equals expected/got`：插值引擎长尾 |
| WAAPI 支撑门 | 11,020 | 21.7% | `'animate' in Element.prototype` 假：WAAPI 门刀 |
| 杂项 | 74 | 0.1% | matrix 序列化 / null 面 |

**supports 门属性分解**（top）：offset-path 2280 / clip-path 1631 /
shape-outside 1577（`shape()` 函数族 ≈ 5.5k）、height 1137 + width 787
（calc-size 族 ≈ 1.9k——**与 peer stash@{0} calc-size 链正交复用**）、
backdrop-filter 943、mask-border-width/slice/source/outset ≈ 1.7k、
offset-rotate 534、offset-anchor/position ≈ 608、font 294、columns 282、
column-rule/row-rule（gaps 装饰属性）≈ 1.4k、counter-reset 210。

### WAAPI 支撑门根因（zz-mprobe-waapi 实证）

门在 `interpolation-testcommon.js:179`：
`webAnimationsInterpolation.isSupported = 'animate' in Element.prototype`。
探针（同二进制）：`typeof el.animate === 'function'` 且真跑通
（`animate(...)` → paused / currentTime=500 → computed opacity=1），
但 `'animate' in Element.prototype` 与 `'animate' in el` 均 **false**——
R2965 `animate`/`getAnimations` 只在实例 get trap 面，缺
`Element.prototype` 定义与 has 白名单臂。harness 对 unsupported type 跳过
注册，故 2 页分母差恰为 25% WAAPI 面（§4）。

**WAAPI 门刀规格（下轮第一刀）**：① `globalThis.Element.prototype` 挂
`animate`/`getAnimations`（this→sel/handle 解析后委托既有 `_makeAnimation`
part03.js 入口，签名不变）；② part05 has 白名单补
`animate`/`getAnimations` 两名（R317/R152 同款先例形态）。预期解锁
11,020 门子测转真跑。

## 4. 覆盖核对

- 337/337 页实测（missing 0）；**still declared=0：0**（崩溃面清零）。
- 分母偏差 >20% 仅 2 页，比值同为 0.75（declared=252/336、126/168）：
  WAAPI face 占该两页 chrome 子测 25%，unsupported 时 harness 不注册——
  WAAPI 门刀后应归零，非异常。
- 探针页 zz-mprobe-expando / zz-mprobe-waapi 留 wpt-data（gitignored）；
  结果 JSON 与分析器均在 `.acceptance/wptfyi-css/`（results-texp-b1{,-p1,
  -p2,-p3,-p4}），不入库。

## 5. 后续（按桶排刀）

1. **WAAPI 门刀**（规格 §3，最小两面改）→ 解锁 21.7% 门面。
2. **supports 覆盖刀族**（每族独立 A/B）：`shape()` 基本形函数（≈5.5k）、
   calc-size（peer WIP 复用，≈1.9k）、mask-border 长手族（≈1.7k）、
   backdrop-filter（943）、offset-\* 值形（≈3.3k）、gaps 装饰属性（≈1.4k）。
   其中**全局关键字切片 11,096**（from/to 端点 inherit/initial/unset/revert
   系）最小刀先行：cascade 实证已全通（inline/sheet `height:inherit`→100px、
   `initial`→auto、`unset`→auto），仅 supports 判定（js_dom_bridge.rs
   `decl_supported` 通用路径头部加全局关键字→known-property 臂）与
   @keyframes 声明 apply（keyframe-inherit-anim 探针证 from 帧被丢）两处
   拒收；余 14,039 为属性未实现真缺口（offset-path 1965 / shape-outside
   1319 / clip-path 1140 深刀族）。
3. **插值语义长尾**（14.5k）：按属性族逐个啃，WAAPI/supports 刀落地后
   该桶占比自然上移成主矛盾。
