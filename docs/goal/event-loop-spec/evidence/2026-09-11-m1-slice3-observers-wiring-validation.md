# M1 切片 3a+3b — IO/RO 动态跟踪接线 + 语义修齐第一批（2026-09-11）

## 切片 3c 追加（同日第二批，in-shim 两小修 + 两实验回退）

**落地**：
1. **target-is-root**（part01.js `_compute`）：root==target → spec skip-to-step-11 臂——
   intersection root 为 Element 且 target 非其后代（自身非自身后代）→ targetRect/
   intersectionRect 留零、isIntersecting false、ratio 0，仍派初通知。IO +1。
2. **documentElement.clientWidth/Height viewport 化**（part04.js element get trap）：
   html 根元素 client 尺寸 = viewport 尺寸（CSSOM View spec；shim 布局 rect 根盒高是
   内容高，直读使 `documentElement.clientHeight` 失去 viewport 语义）。IO +6（含
   empty-root-margin 全簇 + scrollTo 族两案首 rAF 翻绿）。offsetHeight 保持内容高。

**实验后回退**（记录避免复走）：
3. observe-初通知「就绪门」（handle-identity 元素经 `__zw_selector_for_handle` 反查，
   反查空 = 未 apply → 跳过派发）+ rect 读 handle→selector 回落：**已回退**。诊断
   （zw-ro-probe）证实 apply 会给 createElement 元素派 `div` 这类**裸 tag 稳定
   selector**——与文档中既有同 tag 元素歧义，gBCR 按 selector 反查命中错误元素/miss。
   RO observe-001..020（`target width expected N got 0`）族的真实根因在 **engine apply
   路径稳定 selector 唯一性**（tag selector 需 nth 化或唯一化），跨 js-dom/WC 流协调
   项，非本流可闭合；且就绪门改变了初通知时序使 observe-007/017 两案回退（早派发
   零 rect 反而满足其断言时序）。回退后全量无回归。

**切片 3c 后基线**：IO 94/217 = 43.3%、RO 19/56 = 33.9%、合计 **113/273 = 41.4%**
（立项基线 29.7%，累计 +11.7pp）。

## 改动面

1. **runner 侧 observer tick 接线**（`tests/wpt-runner/src/testharness.rs` probe 循环）：
   每轮 probe 调 `__zw_observers_tick()`（对齐 renderer `tick_observers` 语义）——解锁
   testharness runner 环境的 IO/RO 动态跟踪面（此前 observe 初通知后 scroll/样式变化
   永不再通知）。shim 侧空表早返，无 observer 时零开销。
2. **IO threshold 越界判定 spec 化**（`crates/engine/src/js_dom_shim/part01.js`）：
   - `_thresholdIndex`：不相交 → 0；相交 → 首个 > ratio 的 threshold 索引（无则
     thresholds.length）——替换旧 `(prevRatio>=th)!==(ratio>=th)` 双侧比较（默认
     threshold [0] 下 off→on 两侧均 >=0 → 永不判越阈 → 65× `entries.length` 失败簇根因）。
   - `_crossed`：通知条件改为 spec OR 臂直译——thresholdIndex 或 isIntersecting 任一
     与上次派发不同即排队 entry。多 threshold 下部分相交（ratio < threshold[0] 但
     相交）index 与不相交同为 0，仅靠 isIntersecting 臂派发（multiple-thresholds 断言面）。
     内部态 `_lastIndex` → `_lastState {index, intersecting}`（结构对齐 RO `_lastSize`）。
3. **IO 构造器校验 + 属性 getter**（spec 构造器步骤直译）：
   - `_io_parseThresholds`：非 number 元素 → TypeError（IDL union 分支不匹配）；
     <0 或 >1 → RangeError（替换旧 clamp 静默收窄）；升序排序；空 → [0]；无去重。
   - `_io_parseRootMargin`："parse a margin" 直译——1-4 个 px/% 组件 CSS margin 简写
     展开；>4 组件 / unitless "1" / em 等相对单位 / calc() / !important /
     SyntaxError DOMException（替换旧静默按 0）。绝对长度单位换算（cm/in 等）未实现
     （导入面未覆盖，接受限制）。
   - `root` / `thresholds` / `rootMargin` readonly getter（此前全 undefined）——
     rootMargin 返规范化 4 组件串（"10% 20px" → "10% 20px 10% 20px"）。
   - `observe()`：nodeType !== 1 → TypeError（observe("foo") 断言面）。第一版按
     `__zwHandle`/`__zwSelector` 身份判据拒收了 shadow DOM / detached document 元素
     （shadow-content 3 subtest 回归），改 nodeType===1（全元素 proxy 实现）后回归清零。
4. **RO `observe()`**：同款 nodeType TypeError（observe-003 `ro.observe({})` 断言面）。

## 验证结果（WPT 上游用例，pin 3159769338）

| 域 | 基线（M1 切片 1） | 3a 后 | 3a+3b 后 | 3c 后 | 基线→现在 |
|---|---|---|---|---|---|
| intersection-observer | 67/217 = 30.9% | 69/218 = 31.8% | 87/217 = 40.1% | **94/217 = 43.3%** | **+27 subtests** |
| resize-observer | 14/56 = 25.0% | 17/56 = 30.4% | 19/56 = 33.9% | **19/56 = 33.9%** | **+5 subtests** |
| 合计 | 81/273 = 29.7% | — | 106/273 = 38.8% | **113/273 = 41.4%** | **+11.7pp** |

（IO 现含本地探针 zw-probe 1 subtest——tick 接线金丝雀，非上游用例、不计账本；
corpus-only 口径 86/216 = 39.8%。）

逐 subtest 翻绿清单（Fail → Pass，对照切片 3a 运行 log）：
- observer-attributes 8（root/thresholds/rootMargin getter 面全簇）
- observer-exceptions 9（threshold RangeError/TypeError + rootMargin SyntaxError×6
  + observe("foo") TypeError 全簇）
- resize-observer observe-003 + iframe notifications（observe TypeError 面）

## 质量门禁

- `make test`：19,134 passed / 0 failed（含 js_dom_bridge part03 R3062 IO cross-threshold
  tick、part09 R2966 rootMargin、integration dom_bridge_polyfill 全绿——通知条件改为
  OR 臂后自写测试语义不变）
- 定向复跑（observe TypeError 落地后）：`test_intersection_observer_root_margin_r2966` /
  `test_io_cross_threshold_host_tick_r3062` / `test_ro_size_change_host_tick_r3063` 全过
- 切片 3c 后全量复跑：zero-engine + zero-integration-tests 3,455 passed / 0 failed
- `cargo clippy --workspace --all-targets -- -D warnings`：0 warning
- `cargo fmt --all -- --check`：无 diff

## 残留与后续（按失败聚类取序）

1. **几何真值簇**（boundingClientRect/rootBounds/intersectionRect absolute 值偏差，
   现 IO 最大簇 62× `entries.length` 多数源于此）：scroll offset
   （`document.scrollingElement.scrollTop` 不改 layout rect）、CSS transform / zoom /
   clip-path 不参与 IO/RO 几何、inline 元素盒语义（observe-017）——属 layout-engine /
   渲染流域，需碰头协调。
2. **runner viewport 校准**：runner 为 1280×800，上游 WPT 校准 800×600——rootBounds /
   innerWidth 断言系统性偏差。改 runner viewport 牵动全部 testharness 套件绝对几何
   期望 → **待用户决策**。
3. **engine apply 路径稳定 selector 唯一性**（跨流协调项）：createElement 元素被派
   `div` 类裸 tag selector，与既有同 tag 元素歧义 → handle→selector→gBCR 反查链断裂
   （RO observe-001..020 族 + IO handle 族根因；zw-ro-probe 诊断证实）。属 js-dom /
   web-components 流的 apply 域。
4. **zero-area target intersectionRatio=1**（spec §2.2.11-12）+ 边缘相接
   isIntersecting（edge-inclusive）——需配 display:none/未渲染判定与 fixed 定位几何
   （zero-area-element-hidden 用 `position:fixed; top:-1000px`），与几何真值簇同源。
5. **detached document 不产初通知**（target-in-detached-document `Adopt target.` 链）
   ——需 ownerDocument≠主文档判定 + adopt 后重算，与 takeRecords 排队模型一并做。
6. **RO takeRecords 真排队模型**：shim 现恒返 []（spec：返回并清空未派发队列；
   initial-observation-with-threshold 等 runTestCycle 用例的消费面）。
7. **scroll-margin / rootMargin scroll-margin 臂**（scroll-margin-* 2×
   isIntersecting 误报真）。
