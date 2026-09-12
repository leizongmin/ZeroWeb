# M1 切片 1 — IO/RO WPT 用例导入与通过率基线（2026-09-11）

## 导入面

- **上游 pin**: WPT `315976933870b34d6ea30e3f6643403edae678ba`（全仓统一 rev）
- **fetch 脚本**: `tests/wpt-runner/scripts/fetch-observers-subset.sh`（110 IO + 40 RO top-level .html 全量 + resources/ 全目录；idlharness.window.js 等 .window.js 包装形态不拉取）
- **runner 入口**: `make testharness-intersection-observer` / `make testharness-resize-observer`（test-guard 包裹）
- **运行面筛减规则**（`observers_case_skipped`，fetch 全量 + 运行筛减——名字规则会漏判，如 document-scrolling-element-root.html 内容才见 iframe）:
  - `*-ref.html` / `*-notref.html`（reftest 参照页）
  - source 含 `<iframe`（runner 无 iframe 文档/几何管道）
  - `intersection-observer/v2/`（IO v2 trackVisibility 范围外）
  - `*/resources/`（helper，inline 消费）
- **runner 环境注意**：testharness runner 不调 `__zw_observers_tick`（renderer 的 post-render 跟踪面在此环境不存在），observe 初次通知可工作、动态跟踪（scroll/resize 后再通知）天然不可达——下表 Timeout/部分 Fail 与此直接相关，正是 M1 切片 3（语义修齐）的标尺。

## 基线通过率

| 域 | 运行用例 | subtests | Pass | Fail | Timeout | Pass 率 |
|---|---|---|---|---|---|---|
| intersection-observer | 81 | 217 | 67 | 148 | 2 | 30.9% |
| resize-observer | 33 | 56 | 14 | 36 | 6 | 25.0% |
| **合计** | 114 | 273 | 81 | | | 29.7% |

机器可读明细：[2026-09-11-m1-observers-wpt-baseline.json](2026-09-11-m1-observers-wpt-baseline.json)

## intersection-observer — 失败聚类（top 消息簇）

- **65×** `assert_equals: entries.length expected N but got N`
  - 例：intersection-observer/bounding-box.html, intersection-observer/containing-block.html, intersection-observer/edge-inclusive-intersection.html, intersection-observer/initial-observation-with-threshold.html
- **12×** `assert_approx_equals: entries[N].boundingClientRect.top expected N +/- N but got N`
  - 例：intersection-observer/containing-block.html, intersection-observer/edge-inclusive-intersection.html, intersection-observer/multiple-thresholds.html, intersection-observer/root-margin-root-element.html
- **10×** `assert_approx_equals: intersectionRatio expected N.N +/- N.N but got N`
  - 例：intersection-observer/scroll-and-root-margin.html, intersection-observer/scroll-margin-4-val.html, intersection-observer/scroll-margin-clip-path.html, intersection-observer/scroll-margin-horizontal.html
- **9×** `assert_approx_equals: entries[N].boundingClientRect.left expected N +/- N but got N`
  - 例：intersection-observer/inline-client-rect.html, intersection-observer/root-vertical-rl.html, intersection-observer/svg-clipped-rect-target.html, intersection-observer/svg-group-target.html
- **7×** `assert_approx_equals: entries[N].rootBounds.right expected N +/- N but got N`
  - 例：intersection-observer/clip-path.html, intersection-observer/display-none.html, intersection-observer/fixed-position-child-scroll.html, intersection-observer/fixed-position-scroll.html
- **6×** `assert_throws_dom: function "function() {`
  - 例：intersection-observer/observer-exceptions.html
- **4×** `assert_equals: expected N but got N`
  - 例：intersection-observer/empty-root-margin.html, intersection-observer/isIntersecting-threshold.html, intersection-observer/target-is-root.html
- **3×** `assert_approx_equals: entries[N].intersectionRect.left expected N +/- N but got N`
  - 例：intersection-observer/bounding-box.html, intersection-observer/clip-path-animation.html, intersection-observer/root-margin.html
- **3×** `assert_array_equals: value is undefined, expected array`
  - 例：intersection-observer/observer-attributes.html
- **3×** `assert_throws_js: function "function() {`
  - 例：intersection-observer/observer-exceptions.html
- **2×** `assert_equals: expected (string) "Npx Npx Npx Npx" but got (undefined) undefined`
  - 例：intersection-observer/observer-attributes.html
- **2×** `assert_false: isIntersecting expected false got true`
  - 例：intersection-observer/scroll-margin-no-intersect.html, intersection-observer/scroll-margin-zero.html
- **1×** `promise_test: Unhandled rejection with value: "<long>"`
  - 例：intersection-observer/animating.html
- **1×** `testharness completion callback was not called (test=function, hook=function, scripts=[N, N], state={"loaded":`
  - 例：intersection-observer/explicit-root-different-document.html
- **1×** `assert_approx_equals: entries[N].intersectionRect.bottom expected N +/- N but got N`
  - 例：intersection-observer/initial-observation-with-threshold.html

## resize-observer — 失败聚类（top 消息簇）

- **14×** `assert_unreached: Timed out waiting for notification. (Nms) Reached unreachable code`
  - 例：resize-observer/eventloop.html, resize-observer/notify.html, resize-observer/svg-with-css-box-001.html, resize-observer/svg.html
- **11×** `assert_equals: target width expected N but got N`
  - 例：resize-observer/observe-001.html, resize-observer/observe-009.html, resize-observer/observe-010.html, resize-observer/observe-011.html
- **5×** `assert_unreached: Missing ResizeObserver notification Reached unreachable code`
  - 例：resize-observer/fragments.html
- **4×** `testharness completion callback was not called (test=function, hook=function, scripts=[N, N, N], state={"loade`
  - 例：resize-observer/eventloop.html, resize-observer/notify.html, resize-observer/observe-006.html, resize-observer/svg.html
- **1×** `testharness completion callback was not called (test=function, hook=function, scripts=[N, N], state={"loaded":`
  - 例：resize-observer/change-layout-in-error.html
- **1×** `page script threw (declared tests: N): Script error: module compile: Compile error: Module ./create-pattern-da`
  - 例：resize-observer/devicepixel2.html
- **1×** `assert_throws_js: function "_=> {`
  - 例：resize-observer/observe-003.html
- **1×** `assert_equals: expected "success" but got "fail"`
  - 例：resize-observer/observe-007.html
- **1×** `testharness completed without reporting registered tests (state={"loaded":true,"pending":N,"phase":N,"tests":N`
  - 例：resize-observer/observe-020.html
- **1×** `assert_equals: expected N but got N`
  - 例：resize-observer/scrollbars-2.html
- **1×** `assert_equals: ResizeObserver should subtract scrollbar sizes from content-box rect expected N but got N`
  - 例：resize-observer/scrollbars.html
- **1×** `assert_equals: content-rect top should be scaled by zoom expected N but got N`
  - 例：resize-observer/zoom.html

## Timeout 用例

- intersection-observer/explicit-root-different-document.html
- intersection-observer/timestamp.html
- resize-observer/change-layout-in-error.html
- resize-observer/eventloop.html
- resize-observer/notify.html
- resize-observer/observe-006.html
- resize-observer/observe-020.html
- resize-observer/svg.html

## 结论与下一步

- 基线成立：IO 30.9% / RO 25.0%（合计 29.7%）。零源码改动，验收标尺已建立。
- **IO 最大失败簇** = 初次通知后的动态跟踪面（65× `entries.length expected N but got 1`——style/scroll mutation 后无二次通知；runner 不调 `__zw_observers_tick` 且无渲染循环，二次通知天然不可达）。次簇 = boundingClientRect/rootBounds 几何真值偏差 + IO 构造器异常校验全缺（threshold 范围/rootMargin 语法不 throw）。
- **RO 主失败面** = 动态跟踪（6/33 Timeout：notify/eventloop/change-layout-in-error 等 delivery-timing 面依赖 resize 后再通知）。
- M1 切片 3（IO/RO 语义轻量修复队列）从本表失败聚类取序；首切候选 = runner 侧 observer tick 接线（对齐 renderer `tick_observers` 语义，解锁动态跟踪面），随后 IO 构造器异常校验（零几何依赖，纯语义）。
