# M2-S4Z — traverse 锚滚按目标 entry scrollRestoration 门（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案迭代 + 全量
per-subtest 精确 diff vs S4Y 终态）
**前序**: [2026-10-10-m2-s4y-navigateerror-url-and-download-presence.md](2026-10-10-m2-s4y-navigateerror-url-and-download-presence.md)

## 切片内容

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| plain-history traverse（back/forward/go）路径的锚滚按**目标 entry** `scrollRestoration` 门：`manual` 不做锚滚（spec apply-the-history step scroll restoration mode——manual 恢复位不自动滚动，片段滚同属恢复滚动行为）。hash-setter 新导航不受门（case 基面「new navigations should scroll to fragment」）。与 S4D `_navTraverseRestoreScroll` 的 manual 门（Navigation API restore 侧）同谓词成对 | `part02.js`（traverse 单一 chokepoint） | WPT scroll-restoration-fragment-scrolling-samedoc「Manual scroll restoration should take precedent over scrolling to fragment」 |

## 为什么此前缺

S4R 落了 traverse 无匹配 hash 的 `_noTopFallback` 滚顶抑制，但**有**匹配锚时锚滚无
manual 门——case 场景（manual + back 到 `#fragment` entry，`<a name=fragment>` 在
top:800px）滚到 800，期望保持 0。

## 绿面核对（门不伤既有面）

- `scroll-restoration-navigation-samedoc`（S4R 回归门）：其 manual entry 无匹配锚元素、
  `_noTopFallback` 已抑制——门后两路径同果，恒绿复跑确认。
- `scroll-restoration-basic` ×3 / navigation-api scroll-behavior 族
  `after-transition-change-history-scroll-restoration-during-promise`：FILTER 复跑全绿。
- hash-setter / navigate('#frag') 提交锚滚两路径不受门（case 28 行基面断言滚 800 仍成立）。

## 数字（按 evidence txt 原始 Pass 行）

| corpus 域 | S4Y 后 | 本轮 | Δ |
|---|---|---|---|
| 全量 | 397 P / 476 = 83.4% | **398 P / 476 = 83.6%** | +1 |
| navigation-api | 237 / 255 | 237 / 255（92.9%） | 0（本案 html/browsers 域） |

**零回归**：全量 diff 恰一行（scroll-restoration-fragment-scrolling-samedoc
Fail→Pass），逐行比对。

## 质量门禁

- `make test`：全绿（数字见本轮终态行；part02.js 变更轮）。
- fmt/clippy 不适用（纯 .js shim 变更，无 .rs 改动）。
