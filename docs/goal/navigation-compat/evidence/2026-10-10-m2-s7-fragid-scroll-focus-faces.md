# M2-S7 — fragid 滚动/聚焦三面 + getElementById 特殊字符 id（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案迭代 + 临时探针页
（用后即删）+ 全量 per-subtest 精确 diff vs S6 终态）
**前序**: [evidence/2026-10-10-m2-s6-dispatched-click-activation.md](evidence/2026-10-10-m2-s6-dispatched-click-activation.md)

## 定位链（临时探针页实证）

`scroll-frag-percent-encoded`（hash-setter → `location.hash='has%20two%20spaces'` →
断言 `scrollingElement.scrollTop === 100` 得 0）三根因叠加，探针页逐值定位：

1. **getElementById 特殊字符 id 整体抛错**：主文档 `getElementById` 经
   `querySelector('[id="' + id + '"]')` 属性选择器实现且无捕获——id 含 CSS 选择器
   语法外字符（`%`/`[` 等）时选择器不可解析 → SyntaxError 外溢（`_scrollToAnchorForHash`
   的 raw/decoded 双查找全灭）。spec getElementById = 精确 id 匹配不经选择器解析。
   修：选择器面 try/catch + `[id]` 枚举精确比对回落（R125 pending 门同面适用）。
   探针另证：引号内空格合法（decoded id 'has two spaces' 选择器形态可解析）。
2. **documentElement.scrollTop 不镜像窗口滚动**：getter 读 `_scrollOffsets` 独立槽
   恒 0——spec scrollingElement 语义（standards 模式 documentElement.scrollTop ===
   window.scrollY）。修：getter/setter 双侧 HTML 元素门镜像 `_winScroll`/scrollTo
   （round-trip 自洽保持）。
3. **`_scrollToAnchorForHash` 目标位双计**：`_targetTop = rect.top + 滚动前位` 按视口
   相对坐标假设——探针实证 native gBCR 为**文档绝对**坐标（scrollTo 后 rect 不漂移；
   R3060 scrollIntoView 的 `scrollTo(0, gBCR.y)` 同约定）→ +`_preTop` 双计
   （探针「y2=200」根因）。修：`_targetTop = rect.top` 直接绝对。

## 第四面：fragment 滚动聚焦（同 chokepoint 顺收）

spec scrolling-to-a-fragment focusing steps——指示元素可聚焦（内在可聚焦标签或
tabindex 属性）→ `focus()`（focus 事件派发）；否则 viewport 焦点步（现焦点
`blur()` → activeElement 回落 body）。WPT focus-changes-after-scroll-to-fragment
「tabindex 目标聚焦 / 非可聚焦目标落 viewport」。`_scrollToAnchorForHash` 单一
chokepoint（hash-setter/navigate/traverse 共经）。

## 数字（按 evidence txt 原始 Pass 行）

| corpus 域 | S6 后 | 本轮 | Δ |
|---|---|---|---|
| 全量 | 400 P / 476 = 84.0% | **406 P / 476 = 85.3%** | +6 |
| navigation-api | 237 / 255 | 237 / 255（92.9%） | 0（六案均 html/browsers 域） |

六翻全 Fail→Pass（scroll-to-fragid 族：003 / percent-encoded / anchor-name /
id-top / top / focus-changes）。**零回归**：全量 diff 恰六行全翻绿——focus-reset
族 / scroll-restoration 族（fragment-focus 与 traverse 滚动门同 chokepoint 的
关联面）恒绿复验。

**家族余项重定性**：scroll-position-vertical-lr/rl + inline-nearest 3F =
竖排/水平书写模式滚动边（scrollLeft 几何——真渲染域挂账维持）；
scroll-behavior reload 族 4F = scroll anchoring 断言面（buffer.remove() 位移
——渲染域挂账维持，manual-scroll 面已顺带核验非本域可达）。

## 挂账核验（本轮顺带）

- DC-4 `make reftest` 补跑（S6 态）：708 比较 **0 不一致**（550 真通过-可信 +
  111 可疑 + 47 近似）——S4X~S6 四轮 shim/testharness 变更零 reftest 回归，
  门禁账龄清零。

## 质量门禁

- `make test`：全绿（数字见本轮终态行；part02/04/05/06.js 变更轮）。
- fmt/clippy 不适用（纯 .js shim 变更，无 .rs 改动）。
