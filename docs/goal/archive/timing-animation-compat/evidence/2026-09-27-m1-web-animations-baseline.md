# M1 / DC-1 — web-animations window 子集通过率基线

**日期**: 2026-09-27
**套件**: `make testharness-web-animations`（等价 test-guard 包裹命令 + `--json`，
WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-27-m1-web-animations-baseline.json](2026-09-27-m1-web-animations-baseline.json)

## 结果

| 指标 | 值 |
|---|---|
| 导入面 | 92 案（animation-model 7 + animation-trigger 3 + interfaces/ 43 + timing-model/ 47，扫描子目录 14 个）|
| 执行 | 67 案（25 案内容级 skip：ref ×12、reftest-wait 渲染面 ×9、iframe/其它 ×4）|
| subtests | **52/963 = 5.4% Pass**（Fail × 901、Timeout × 10）|
| 全绿用例 | 2 |

低基线符合预期——WAAPI corpus 的断言面（Animation/KeyframeEffect 构造器、effect
解析、computed timing）正是 M3 要落地的缺口；现有 shim 只有 `el.animate()` 返回的
plain-object Animation（R2965 瞬间完成形态），无构造器全局、无 effect 属性。

## 失败聚类（M3 修齐方向）

| 簇 | 案数 | 形态 | 归属 |
|---|---|---|---|
| `KeyframeEffect is not defined` | ~370 | KeyframeEffect 构造器全局缺位（interfaces/KeyframeEffect 全族 + timing-model 大半）| M3 |
| `Cannot read properties of undefined` | ~256 | `anim.effect.getComputedTiming()` 等——Animation 无 effect 属性（现有 shim plain-object 形态）| M3 |
| `Animation is not defined` | ~64 | Animation 构造器全局缺位（interfaces/Animation constructor 族）| M3 |
| assert_throws_js 不抛 | ~30 | KeyframeEffect 关键帧参数校验（processing-a-keyframes-argument 全族）| M3 |
| Timeout ×10 | 10 | ready/pause 等异步 promise 语义等待 | M3 |

已实现的真面（基线绿）：`el.animate()` 存在性链、部分 Animation playState 瞬间完成
语义、Web Animations 与 CSS 动画事件桥的 feature-detect 面。

## headless 帧精度约束（如实标注，DC-3）

现有 rAF 为 runner execute 末派发（`__ZW_RAF_FRAME_DRIVEN` opt-in 帧驱动）——动画
时序断言（currentTime 推进、ready/finished promise 的相对时序）在 headless 无真实
vsync 时间轴下按「瞬间完成」近似。上游无 fuzzy 注解的时序用例不自行放宽容差：该族
Fail/Timeout 保持原样记入分母，修齐方向是语义面（构造器/effect/状态机）而非调容差。

## 导入面与排除

- 拉：web-animations/ 顶面（testcommon.js + resources/）+ animation-model/ +
  animation-trigger/ + interfaces/{Animatable,Animation,AnimationEffect,
  AnimationPlaybackEvent,Document,DocumentTimeline,KeyframeEffect,TimelineTrigger} +
  timing-model/{animation-effects,animations,time-transformations,timelines}
  （goals/10 脚本 DIRS 显式深面追加——lib.sh fetch_dir_html 递归一层不覆盖三级目录）
- 不扫描（已 fetch 记账）：`responsive/`（responsive reftest 面——动画渲染效果跨域
  记账 rendering-compat，fullscreen/rendering 先例）、`crashtests/`（崩溃面，
  web-components 先例）、`idlharness.window.js`（wrapper 形态）
- 运行面 skip ×25：`-ref.html` ×12（reftest 参照页）、`reftest-wait` 渲染等待面 ×9
  （animation-model/side-effects-* ×3 + timing-model/animations infinite-duration 等
  ×6——渲染效果跨域 rendering-compat）、iframe/Document-timeline 依赖 ×4
  （interfaces/Animation/constructor、interfaces/Document/timeline、
  timing-model/timelines/document-timelines 等）
- 本 corpus 无 `.any.js` 案面（全 html）

## 下一步（M3）

1. `Animation` / `KeyframeEffect` / `DocumentTimeline` 构造器全局 + `anim.effect`
   属性桥（三主簇 ~690 subtests）
2. KeyframeEffect 关键帧参数解析/校验（processing-a-keyframes-argument 族）
3. ready/finished promise 语义（Timeout 尾簇）——headless 瞬间完成约束如实标注
