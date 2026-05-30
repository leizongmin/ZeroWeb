# ZeroBrowser 运行时控制平面

**最后更新**: 2026-05-31
**执行状态**: 14/16 crate 已实现，1140 个测试全绿，14 个 crate 有基准测试

> **说明**
> 本文记录的是实验性项目的当前实现进度。测试全绿、CI 通过或里程碑推进，并不等于项目已经适合日常使用、商用或其他生产用途；相关风险仍需自行评估。

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace + 16 crate（14 个有实质实现） |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ `cargo test --workspace` 1140 个测试全绿 |
| Clippy | ✅ 零警告（全 workspace） |
| 基准测试 | ✅ 14/16 crate 有 criterion 基准 |
| CI | ✅ GitHub Actions（ubuntu/macos/windows）|

### 已实现 crate（14 个）

| Crate | 测试 | 基准 | 说明 |
|-------|------|------|------|
| dom | 93 | ✅ | DOM 树、html5ever 集成、查询 API、序列化、属性、MutationObserver |
| css-parser | 162 | ✅ | Tokenizer、Parser、选择器、值解析、百分比/auto、媒体查询、Transform、**@keyframes** |
| style-system | 239 | ✅ | 级联、继承、计算值、DOM 集成、选择器匹配、简写展开、Grid、@media 评估、Transform、Transitions、Animations、逻辑属性、**Grid 项放置** |
| layout-engine | 79 | ✅ | taffy 集成（Block/Flex/Grid/Position）、Grid 轨道解析、**Grid 项放置**、几何验证 |
| engine-core | 52 | ✅ | 渲染管线、paint、dirty tracking、compositing |
| render-foundation | 53 | ✅ | GPU/CPU 渲染、字体栈、图片缓存 |
| host-runtime | 18 | ✅ | winit 窗口、事件循环、事件类型 |
| net | 74 | ✅ | HTTP client、URL、导航历史、Cookie |
| security | 56 | ✅ | 同源策略、CORS、CSP |
| protocol | 57 | ✅ | IPC 消息、bincode 序列化 |
| storage | 47 | ✅ | localStorage、sessionStorage、IndexedDB |
| canvas | 81 | ✅ | Canvas 2D API、路径、变换 |
| webview-api | 43 | ✅ | WebView 嵌入 API、Builder |
| wasm-sandbox | 22 | ✅ | WASM 运行时（wasmi 纯 Rust 解释器） |

### 跨 crate 集成测试

| 测试模块 | 测试数 | 覆盖场景 |
|----------|--------|----------|
| DOM + CSS Parser | 3 | HTML→DOM 树、CSS 规则、元素属性 |
| CSS + Style System | 3 | 样式计算、级联优先级、继承 |
| Render Pipeline | 4 | 完整管线、CSS 集成、耗时分解、复杂页面 |
| Net + Security | 3 | 同源判断、CORS 策略、安全上下文 |
| Storage + Protocol | 3 | localStorage CRUD+IPC、session 隔离、origin 隔离 |
| Protocol + Navigation | 1 | 导航历史 + IPC 序列化 |
| Canvas + Render | 5 | Canvas 绘图图元、路径、变换、save/restore、WebView 集成 |
| WASM Sandbox | 5 | 编译、调用、导出查询、内存读写、错误恢复 |
| WebView Full Pipeline | 4 | 完整生命周期、复杂页面、重复加载、脚本占位 |

### 占位 crate（2 个）

| Crate | 说明 |
|-------|------|
| script-sandbox | JS 引擎（V8/QuickJS feature gate）— 需要二进制依赖 |
| browser-shell | 浏览器 UI — 需要 UI 框架选型 |

---

## 最近完成的改进

### 1. CSS 简写属性展开（style-system）

实现了完整的 CSS 简写属性展开模块（`shorthand.rs`），在级联之前将简写属性自动展开为长属性：

| 简写属性 | 展开为 |
|----------|--------|
| `margin` | `margin-top/right/bottom/left` |
| `padding` | `padding-top/right/bottom/left` |
| `border-width/style/color` | 4 边对应属性 |
| `border` | 12 个长属性（4 边 × width/style/color） |
| `border-top/right/bottom/left` | 3 个长属性 |
| `overflow` | `overflow-x` + `overflow-y` |
| `border-radius` | 4 个角半径 |
| `flex` | `flex-grow` + `flex-shrink` + `flex-basis` |
| `inset` | `top` + `right` + `bottom` + `left` |
| `transition` | `transition-property/duration/timing-function/delay` |
| `animation` | 8 个长属性（name/duration/timing/delay/iteration-count/direction/fill-mode/play-state） |
| `margin-block/inline` | `margin-block-start/end` 或 `margin-inline-start/end` |
| `padding-block/inline` | `padding-block-start/end` 或 `padding-inline-start/end` |
| `inset-block/inline` | `inset-block-start/end` 或 `inset-inline-start/end` |

### 2. CSS Transitions（css-parser → style-system）

- `TimingFunctionValue` 枚举：ease、linear、ease-in/out、cubic-bezier、steps
- `parse_time()` 支持 s/ms 时间值
- `ComputedStyle` 新增：`transition_property`、`transition_duration`、`transition_timing_function`、`transition_delay`
- `transition` 简写展开，正确处理 cubic-bezier() 和 steps() 内部逗号
- 21 个新测试

### 3. CSS Animations + @keyframes（css-parser → style-system）

- `KeyframesRule`、`KeyframeBlock`、`KeyframeSelector` AST 类型
- `@keyframes` 专用解析器：from/to/百分比选择器、逗号分隔、声明块
- `AnimationDirectionValue`、`AnimationFillModeValue`、`AnimationPlayStateValue` 枚举
- `ComputedStyle` 新增 8 个动画字段
- `animation` 简写展开（8 个子属性）
- matcher 中正确跳过 `Rule::Keyframes`
- 23 个新测试

### 4. CSS 逻辑属性（style-system）

- 12 个逻辑长属性：`margin-block-start/end`、`margin-inline-start/end`、`padding-block-start/end`、`padding-inline-start/end`、`inset-block-start/end`、`inset-inline-start/end`
- 水平书写模式映射：block→top/bottom，inline→left/right
- 6 个轴简写：`margin-block`、`margin-inline`、`padding-block`、`padding-inline`、`inset-block`、`inset-inline`
- 21 个新测试

### 5. CSS @media 媒体查询（css-parser → style-system）

- 媒体类型：`screen`、`print`、`all`
- 媒体特性：`width/min-width/max-width`、`height/min-height/max-height`、`orientation`
- `not` 取反和多条件 `and` 组合
- 无视口信息时 `@media` 规则不应用

### 6. CSS Transform 属性（css-parser → style-system）

- 变换函数：translate/translateX/translateY、rotate、scale/scaleX/scaleY、skew
- 多函数链式组合
- `ComputedStyle` 新增 `transform` 字段

---

## Tier 1 CSS 覆盖状态

| Tier 1 类别 | 状态 |
|-------------|------|
| 选择器全量 | ✅ ~95% |
| 盒模型 | ✅ 100% |
| Block/Inline/Flexbox 布局 | ✅ 已实现 |
| Grid 布局 | ⚠️ ~50%（display + auto-flow + **项放置** + grid-area；缺 repeat()、auto-rows/cols） |
| 颜色 | ✅ ~95% |
| 字体 | ✅ 100% |
| 定位 | ✅ 100% |
| Overflow | ✅ 100% |
| Transforms | ✅ ~70%（核心 2D 函数） |
| **Transitions** | ✅ 已实现 |
| 自定义属性 | ✅ ~90% |
| 媒体查询 | ✅ ~70% |
| **逻辑属性** | ✅ 已实现 |
| **Animations/@keyframes** | ✅ 已实现 |

---

## 里程碑完成情况

| 里程碑 | 状态 |
|--------|------|
| M1 项目骨架 + 渲染基础设施 | ✅ |
| M2 HTML 解析 + DOM 树 | ✅ |
| M3 CSS 解析器 + 样式系统 | ✅ |
| M4 布局引擎 | ✅ |
| M5 渲染管线集成 | ✅ |
| M6 JavaScript 集成 (V8) | ⏸ 需要 rusty_v8 |
| M7 网络栈 + 导航模型 | ✅ |
| M8 多进程架构 (IPC) | ✅ (protocol crate) |
| M9 Canvas + Storage | ✅ |
| M10 WebView API | ✅ (webview-api + integration tests) |

---

## 下一步优先级

1. **渲染管线改进**（高优先级）— 文本渲染（paint 阶段 glyph 生成）、border-radius 绘制、overflow clip
2. **Grid 布局增强**（高优先级）— grid-column/row 放置、grid-area、repeat()/minmax() 解析
3. **内联布局**（高优先级）— 行内格式化上下文、文本换行、white-space 处理
4. **更多 DOM API**（中优先级）— Shadow DOM、Range、Selection 等
5. **安全增强**（中优先级）— 沙箱、混合内容阻止、COOP/COEP

---

## 归档记录

- **M1** ✅ → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
- **M2** ✅ → [archive/m2-dom.md](archive/m2-dom.md)
- **M3** ✅ → [archive/m3-css-parser-style-system.md](archive/m3-css-parser-style-system.md)
- **M4** ✅ → [archive/m4-layout-engine.md](archive/m4-layout-engine.md)
- **M5** ✅ → [archive/m5-rendering-pipeline.md](archive/m5-rendering-pipeline.md)
- **M7** ✅ → [archive/m7-network-security.md](archive/m7-network-security.md)
