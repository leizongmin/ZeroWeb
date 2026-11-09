# M2 切片 375（s375）——系统色 dark 面板 Canvas 背景族（default-color 案2 翻绿；dialog 域真案 19F→18F）

日期：2026-11-10（承 s374 挂账：default-color 案2 系统色 color-scheme 分支）
域：html/semantics/interactive-elements/the-dialog-element（default-color 案2）+ css-parser 系统色面板
性质：**default-color 案2 翻绿（-1，2/2 全绿）；dark 面板机制已存在，本刀补背景族缺键——最小施工**

## 勘根

**案面**（default-color.html）：案2 断言 `#light{color-scheme:light}` 与 `#dark{color-scheme:dark}` 两 dialog 的 computed color/backgroundColor 不等（UA 注入 `color: CanvasText; background-color: Canvas`，s373 注入面）。败态 solo：案1 绿、案2 红，消息 `Dialog background should react to color-scheme got disallowed value "rgb(255, 255, 255)"`——**color 断言已过，只缺背景族**。

**机制盘点**（比账本预期完整，非从零施工）：
1. `parse_color_with_scheme(value, dark)`（css-parser color.rs:22）——dark=true 面板**已存在但仅覆 CanvasText 族五键**（canvastext/captiontext/infotext/menutext/windowtext→白，先例驱动 meta color-scheme 语料）；**Canvas 族不在面板**→dark dialog 的 Canvas 恒白与 light 相等→not_equals 红。
2. 元素级 scheme 传导链已完备：`style.color_scheme_dark`（computed_style.rs:132）由 `compute_inherited_style_with_quirks` 按 `parse_color_scheme_dark(value, prefers_dark)` 预解析（color-scheme 先于其他属性计算，apply.rs 注释在案）→ apply 色解析唯一消费点 `apply.rs:86 parse_color_with_scheme(value, dark)`——UA decl 的 "Canvas" 字符串在 apply 期按元素 used scheme 解析。
3. 波及面：Canvas 系统色用点仅 dialog UA 注入（lib.rs:2085，本流 s373）与根色 CanvasText；render-foundation 无 Canvas 字符串消费（值固化 computed style）。meta color-scheme 语料（document-metadata/the-meta-element/color-scheme/，10 案）断言面只用 CanvasText 差分（compute-root-color-scheme.js `get_system_color("CanvasText")`）——零波及。

## 施工

crates/css-parser/src/values/color.rs dark 臂补背景族两键：`canvas | window` → `rgb(32, 33, 36, 255)`（Chrome dark=#202124 先例，与 CanvasText dark 白配套）；scrollbar 等 dark 面板异值键不入臂（无钉面不扩刀，light 静态值兜底）。light 面板零改动（parse_color 恒 dark=false 既有调用方零回归）。同步扩写面板注释（仅覆文本族→覆文本族+背景族，driving 补本刀案2）。新钉测 `test_system_color_dark_panel_canvas_family`（canvas/window dark 值+light 静态不变+scrollbar 不在臂四断言）。

## 验证

- **solo default-color**：2/2 全绿（修复前断言2 红态证据 /tmp/s374/solo-default-color-baseline.log，本地）。
- **新钉测**：css-parser lib 单测 PASS。
- **meta color-scheme 语料**：24P/0F 零波及（solo FILTER=the-meta-element/color-scheme/）。
- **interactive-elements 全域 A/B**（同 s374 口径）：真案 **19F→18F**（-1=default-color 案2，零新增红零改名）。首跑全域现 `modal-dialog-in-visibility-hidden` 偶红——同二进制 solo 3/3 绿+复跑全域回绿=时序 flake（s292 坑册先例），不入回归；以复跑 sweep（s375b，464P/22 Fail 行=18 真案 F+3 探针单列+closed-details 出域 1）为干净口径。
- **lib 基线**：css-parser 3051P/0F（含新钉测）；style-system 2504P/0F；engine 3826P/7F——7 红逐名同构 s374 基线集（img settle/fetch/worker/websocket），零新增红。
- **fmt/clippy**：css-parser PASS（clippy --all-targets -D warnings 净）。
- **product-smoke**：6/6 struct-check PASS（系统色 dark 面板对默认 light 页零影响；Canvas 仅 dark 元素触发）。
- runner 通道零改动（css-parser 单点，shim 未动）。

## 挂账（移交下一切片）

1. 既有在册承继：modal-dialog-ancestor-is-inert（前置=R-s31 UA dialog 定位簇，shim retarget 半边已就位）、select-dialog-mode-focus 2F（focus 行为面）、select picker open 态注入通道（39 涉案文件）、UA popover/dialog 定位全规则簇随 R-s31 渲染流、overlay CSS 属性 + top-layer 强制定位（dialog-overlay 2F / top-layer-position / abspos-dialog-layout 6F / multiple-centered-dialogs / non-modal-dialog-layout / modal-dialog-scroll-height 渲染流域）、corner-cases（modeless showModal 转模态）、dialog-closedby-bounds-clicking 2F（bounds 点击几何分类，流内候选）、dialog-form-submission 亚像素坐标、inert hit-testing 穿透下层近似面（无红案钉面，dialog-inert 已绿——不扩刀）、fresh parse 通道 modal 注入缺口、基建面 T=3 + probe8/9/10 转正决策、s374 挂账全部承继。
2. 探针产物无（本刀零探针）；验证日志 /tmp/s374/（本地）。
