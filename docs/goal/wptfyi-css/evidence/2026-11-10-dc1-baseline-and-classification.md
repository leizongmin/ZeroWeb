# DC-1 基线与试跑：Grid / Typed OM / Color / Values 分片重跑与缺口三分类（2026-11-10）

## 结论

1. **复现性成立**：本地 runner（HEAD `dd95461d9`，release）四目录重跑数字与
   [coverage-2026-11-09](../../../compat/wptfyi/coverage-2026-11-09.md) 快照一致
   （typed-om 25 Pass、color 1,293 Pass 逐位相同；values 753 vs 快照 739，
   +14 为快照后 runner 微调，非口径漂移）。
2. **缺口三分类成立**（详见下表）：css 全目录缺口 166,503 项中，
   **运行面断言失败 ~139.5k**（Chrome 在同一运行面通过 95%——纯引擎差距）、
   **未接语料 ~19k**（8 个目录从未 fetch：css-transitions/css-animations/
   cssom-view/css-view-transitions/geometry/css-properties-values-api/
   css-highlight-api/css-page 等）、**通道外形态**（`.any.js` 变体/`.xht`/
   `.https.`/iframe/病态案名级排除）余 ~5-8k。
3. **DC-1 优先问题定性**（Typed OM 348 文件仅 25 Pass）：CSS Typed OM API
   整体未实现——`CSSKeywordValue`/`CSSUnitValue`/`CSSStyleValue`/
   `computedStyleMap`/`CSS.px` 等入口全 undefined（与
   [归档 goal M3 批次 2](../../archive/css-testharness-compat/evidence/2026-10-30-m3-batch2-typed-om-fonts-shapes.md)
   定性一致，当前 runner 未变化）。ZW 每案只产出 ~2 子测试（页脚本首个断言
   即 ReferenceError 中断已注册测试回报），Chrome 同面 12,815 子测试——API
   实现后 ZW 子测试账面将放大约 18 倍。
4. **DC-2 优先序**（按单点杠杆）：① Typed OM API 入口面（~12.8k）；
   ② color-mix()/现代色函数 computed value 通路（color 目录单簇 7,199）；
   ③ grid 布局断言面（13.2k，聚类待本 evidence 后续补）；④ trig/attr()/if()
   序列化长尾（values ~4.8k 宽簇）。

## 运行面重跑（每案独立进程分片，test-guard 4G/8G/3600s，JSON 通道）

| 目录 | 案 | 子测试 | Pass | Fail | TO | 与 11-09 快照 |
|---|---:|---:|---:|---:|---:|---|
| css-typed-om | 348 | 689 | 25 | 664 | 0 | **25 = 25 一致** |
| css-color | 62 | 11,009 | 1,293 | 9,716 | 0 | **1,293 = 1,293 一致** |
| css-values | 256 | 5,593 | 753 | 4,833 | 7 | 753 vs 739（+14 快照后 runner 微调） |
| css-grid | 653 | 7,716 | 1,251 | 6,389 | 76 | 1,251 vs 1,239（+12 快照后 runner 微调） |

- 产物：`.acceptance/wptfyi-css/results/{css-color,css-typed-om,css-values,css-grid*}.json`
  （runner `[[case,[{name,status,message}]]]` 形状，直接投喂 `wptfyi_gap.py`
  merge_zw_results）。运行单元 `zwcss-*`（systemd user service，日志
  `.acceptance/wptfyi-css/logs/`）。
- **分片并行坑（已修复本地下跑法）**：corpus 分片脚本 transcript 默认
  `/tmp/css-<corpus>-corpus.log` 不含 shard 后缀——两分片并行共享同一
  transcript 交错覆写，收尾合并 JSON 块损坏（grid-1 首跑实证）。并行分片须
  显式 `CSS_CORPUS_TRANSCRIPT=/tmp/css-grid-shard<k>-corpus.log` 区分；
  脚本头注释「各写各的 transcript」仅指 JSON 通道，文本通道需调用方自分区。

## 缺口三分类（wpt.fyi revision `792c7512e3`，与 11-09 快照同口径）

分母 = 三家 aligned stable summary 并集路径子测试（chrome `c[1]`）；本地语料
逐路径对账 + runner skip 谓词（`css_testharness_case_skipped` 镜像）分类。
脚本与逐路径缺失清单：`.acceptance/wptfyi-css/{dc1_inventory.py,dc1-missing-plain-html.json}`。

### 四焦点目录分解（chrome 侧分母）

| 目录 | in-face 子测试 | chrome Pass | ZW Pass | in-face 缺口 | 缺语料（文件/子测试） | 通道外 |
|---|---:|---:|---:|---:|---|---|
| css-grid | 14,463 | 13,297 (92%) | 1,239 | 13,224 | 252/1,381 | reftest 形态 1,591 页（分母 0） |
| css-typed-om | 12,815 | 12,092 (94%) | 25 | 12,790 | 5/42 | any-variant 44 路径/588 |
| css-color | 11,448 | 11,156 (97%) | 1,293 | 10,155 | 13/1,304 | reftest 形态 256 页（分母 0）、.xht 51 |
| css-values | 9,921 | 9,421 (95%) | 753 | 9,168 | 23/137 | iframe 11/146、病态案 1/203、.xht 4/5 |

### 全目录机会地图（in-face 缺口降序，Chrome Pass% = 可回收性指标）

> in-face = 本地语料在 runner 运行面内的 chrome 分母子测试；缺语料 = chrome
> 清单有、本地语料缺（含从未 fetch 的目录）。

| 目录 | in-face | ZW Pass | 缺口 | chrome% | 缺语料子测试 |
|---|---:|---:|---:|---:|---:|
| css-grid | 14,463 | 1,239 | 13,224 | 92% | 1,405 |
| css-typed-om | 12,815 | 25 | 12,790 | 94% | 42 |
| css-color | 11,448 | 1,293 | 10,155 | 97% | 1,304 |
| css-values | 9,921 | 753 | 9,168 | 95% | 137 |
| css-masking | 6,387 | 121 | 6,266 | 72% | 11 |
| css-anchor-position | 13,640 | 7,455 | 6,185 | 100% | 21 |
| css-fonts | 8,435 | 2,258 | 6,177 | 96% | 246 |
| css-backgrounds | 6,198 | 271 | 5,927 | 95% | 38 |
| css-shapes | 7,514 | 1,691 | 5,823 | 100% | 13 |
| css-transforms | 5,511 | 114 | 5,397 | 100% | 2 |
| css-sizing | 5,707 | 677 | 5,030 | 99% | 457 |
| css/motion | 4,955 | 105 | 4,850 | 100% | 4 |
| css-align | 5,360 | 627 | 4,733 | 92% | 75 |
| css-flexbox | 4,745 | 880 | 3,865 | 99% | 68 |
| css-gaps | 4,148 | 345 | 3,803 | 100% | 9 |
| css-text | 4,515 | 1,104 | 3,411 | 89% | 308 |
| css-images | 3,587 | 781 | 2,806 | 99% | 14 |
| filter-effects | 2,452 | 156 | 2,296 | 99% | 0 |
| selectors | 4,396 | 2,142 | 2,254 | 92% | 192 |
| CSS2 | 2,503 | 814 | 1,689 | 100% | 63 |
| cssom | 3,520 | 1,946 | 1,574 | 98% | 416 |
| 其余 39 目录合计 | 16,987 | 5,321 | 11,666 | — | 5,199 |
| **合计（58 接线目录）** | **169,488** | **29,958** | **139,530** | **95%** | **10,519** |
| **未 fetch 目录合计（25 个）** | — | — | — | — | **~8,100** |

- 未 fetch 目录大头：css-view-transitions ~1,437 / css-animations ~1,429 /
  cssom-view ~2,635 / css-transitions ~3,439 / geometry ~843 /
  css-properties-values-api ~1,393 / css-highlight-api ~176 / css-page ~159 /
  scroll-anchoring ~98 / scrollbars ~83 / overscroll-behavior ~109 /
  navigation ~82 / parser-api ~71（逐路径实跑对账后入账）。
- **css 缺口总账核对**：in-face 缺口 139,530 + 未 fetch ~8,100 + 缺文件
  ~10,500 + 通道外 ~5-8k ≈ 166,503（11-09 快照 css 缺口）✓（部分目录缺文件
  子测试已含在未 fetch 桶内，账目按实跑对账时归并）。

## 失败聚类（message 归一化：数字→N、串→'S'）

### css-typed-om（664 F，单簇 = API 缺面）

`CSSKeywordValue is not defined` 243 + undefined 属性链读 101 +
`CSS[unit] is not a function` 44 + `CSSUnitValue` 41 + `computedStyleMap` 34 +
`CSSStyleValue` 21 + throws 类型不符 21 + `CSSMathSum` 15 + `CSSUnparsedValue`
15 + `CSSMatrixComponent` 11 + `CSSVariableReferenceValue` 8 + `CSS.px` 6 +
`CSSNumericValue` 6 + 长尾。**引擎工作面 = Typed OM API 入口面**：CSS 命名
空间工厂（px/number/percent/unit 索引/keyword 等）+ CSSStyleValue 类族 +
StylePropertyMap（get/set/append/remove/has/size）+ `Element.computedStyleMap()`。
现有基建：`globalThis.CSS` 已有 escape/supports（js_dom_shim part06）；
part05 有 color 内部用的局部 `CSS.percent/deg` 雏形；computed style 读取面
（getComputedStyle）在 js_dom_bridge 已通。

### css-color（9,716 F）

**`color doesn't seem to be supported in the computed style` 单簇 7,199**
（74%）——`getComputedStyle().color` 等对 `color-mix()`/相对色/oklch 等现代
色函数返回空（computed-value 解析/序列化通路未实现该形态）。其余：
assert_equals 序列化错位 826 + canonical serialization 558 + 浅簇长尾
（system-color/light-dark/color-scheme 小簇）。

### css-values（4,833 F / 宽浅簇群）

assert_equals 期望/实得错位 1,251（三角函数/attr()/if()/calc-size 等多函数
面的无效值拒绝与有效值接受）+ trig 序列化（`rotate(acos(1))` →
`rotate(calc(0deg))` canonical）425 + `assert_approx_equals`（ch/lh 单位
实测）194 + canonical nesting 139 + computed serialization 131 +
`attr()` 全类型 94 + 长尾小簇。

### css-grid（6,389 F / 76 TO / 双分片合并）

- **computed track-list 序列化**：`gridTemplateColumns value not in array` 334
  + `grid-template-columns/rows doesn't seem to be supported` 213 +
  assert_equals 值错位 229+49——resolved value 形态（`1fr`/`[a] 10px`/track
  列表 canonical）与 computed 支持探测。
- **布局几何断言（checkLayout `data-offset-x/expected-width` 面）136+135+128+
  52+…≈2,700+**：真实 grid 布局引擎的 item 放置/尺寸断言——**归口布局流
  （rendering-compat / 布局账），按 goal 协作规则记归属，不在本 goal 的
  CSSOM 断言面内修**。
- grid-lanes（tentative 新特性）64 + animation keyframes 229 小簇。
- **TO 76 案**：执行截断桶（case 级 30s 内部超时 + 90s 外部墙钟）；后续按
  案名清单甄别真截断 vs 页脚本死循环。

## DC-1 验收核对

| 项 | 结果 |
| --- | --- |
| wpt.fyi 缓存刷新至 `792c7512e3` | ✓（与 11-09 快照同 revision，三家 aligned） |
| 本地四目录重跑 vs 快照复现 | typed-om/color 逐位一致；values +14（runner 微调，非口径漂移） |
| 缺口三分类账 | 见上表；总账与快照缺口 166,503 对得上 |
| runner/守卫 | test-guard 4G/8G/3600s；release runner 重建于 HEAD `dd95461d9` |
| Typed OM 348 文件 25 Pass 定性 | ✓ API 未实现（同归档 goal 定性，当前未变化） |

## 遗留与下一步

- DC-2 首批：Typed OM API 入口面（新 shim part + computedStyleMap 回调桥）。
- 未 fetch 目录（transitions/animations/om-view/view-transitions/geometry/
  properties-values-api/highlight-api 等 ~8.1k 子测试）走语料扩面批次：
  fetch 脚本 DIRS 追加 + runner 子命令 + 分片接线，模板零改动复用。
- `.any.js` 变体通道（typed-om 588 子测试 + 其他目录同形态）：net-api 先例
  （`run_any_js_window_case` / `any_js_window_wrapper` 基建在
  testharness.rs 已就绪），css 语料通道接入为后续批次。
