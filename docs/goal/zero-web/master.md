# ZeroWeb 运行时控制平面

**最后更新**: 2026-05-31
**执行状态**: 14/16 crate 已实现，2977 个测试全绿，14 个 crate 有基准测试

> **说明**
> 本文记录的是实验性项目的当前实现进度。测试全绿、CI 通过或里程碑推进，并不等于项目已经适合日常使用、商用或其他生产用途；相关风险仍需自行评估。

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace + 16 crate（14 个有实质实现） |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ `cargo test --workspace` 2977 个测试全绿 |
| Clippy | ✅ 零警告（全 workspace） |
| 基准测试 | ✅ 14/16 crate 有 criterion 基准 |
| CI | ✅ GitHub Actions（ubuntu/macos/windows）|

### 已实现 crate（14 个）

| Crate | 测试 | 基准 | 说明 |
|-------|------|------|------|
| dom | 317 | ✅ | DOM 树、html5ever 集成、查询 API、序列化、属性、MutationObserver、Range API、遍历/比较方法、Shadow DOM、slot、id_map 自动清理、**模块级单元测试** |
| css-parser | 359 | ✅ | Tokenizer、Parser、选择器、值解析、@规则、:has()、@container、scroll-snap、calc() 嵌套、媒体查询 range syntax、Token 源位置追踪、min()/max()/clamp() 数学函数、**float/clear** |
| style-system | 478 | ✅ | 级联、继承、计算值、DOM 集成、选择器匹配、简写展开、Grid、@media 评估、Transform、Transitions、Animations、逻辑属性、var() 解析集成、revert 关键字、grid-template-areas、calc/min/max/clamp 管线集成、aspect-ratio、**float/clear** |
| layout-engine | 233 | ✅ | taffy 集成（Block/Flex/Grid/Position）、Grid 轨道解析、Grid 项放置、auto-fill/minmax()、grid-template-areas、零尺寸容器、深层嵌套、aspect-ratio 布局、box-sizing:border-box 测试、**z_index/is_sticky 字段**、**fixed 视口坐标调整**、**text-align center/right/justify**、**vertical_align** |
| engine | 183 | ✅ | 渲染管线、paint（文本/glyph、overflow clip、border-radius）、dirty tracking、compositing（z-index 排序）、CSS transform、增量渲染、内联文本渲染、**inline paint 增强** |
| render-foundation | 232 | ✅ | GPU/CPU 渲染、字体栈、image cache + GC、clipping/scissor、颜色 RGBA clamping、image cache eviction、surface resize、**文本整形器（TextShaper + 换行）** |
| host-runtime | 135 | ✅ | winit 窗口、事件循环、mouse/cursor/IME 事件、**resize 事件**、**鼠标坐标**、**IME composition**、**键盘修饰键** |
| net | 176 | ✅ | HTTP client、URL、导航历史、Cookie、send 集成测试、cookie 过期/SameSite、**URL userinfo/port/query 边角场景**、**SameSite 全矩阵**、**重定向深度边界** |
| security | 155 | ✅ | 同源策略、CORS（preflight）、CSP（nonce/hash/navigation/document）、mixed content blocking、sandbox、COOP/COEP、**CSP scheme-source**、**report-only** |
| protocol | 87 | ✅ | IPC 消息、bincode 序列化、**mock channel 契约**、**确定性编码**、**对抗性反序列化** |
| storage | 158 | ✅ | localStorage、sessionStorage、IndexedDB（IdbKeyRange/IdbIndex/IdbCursor/IdbTransaction）、Cache API、**事务缓冲/回滚**、**NaN/Infinity key 排序**、**唯一索引冲突**、**Cache API CRUD** |
| canvas | 179 | ✅ | Canvas 2D API、路径、变换、drawImage、shadow 属性、**Path2D 高级方法**、**lineDash**、**roundRect 圆角扁平化**、**alpha 混合**、**像素边界溢出** |
| webview | 107 | ✅ | WebView 嵌入 API、Builder、event callbacks、load_url fetch、execute_script、**CSS 缓存持久化**、**状态机**、**配置** |
| wasm-sandbox | 83 | ✅ | WASM 运行时（wasmi）、host function imports、fuel/execution limiting、**host 错误传播**、**参数类型校验**、**offset 溢出** |

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

### -5. 6 crate 边界条件测试覆盖率提升（本轮，2977 测试）

通过并行扫描 test-to-code 比率最低的 6 个 crate，识别测试缺口并批量添加边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| style-system | **级联优先级边界**：specificity 竞争、!important 覆盖、inherit/initial/unset/revert 关键字、非继承属性、ComputedStyle 默认值 | 74 |
| dom | **节点操作边界**：insert_before 错误路径、replace_child、clone_node_deep、has_child_nodes、Document 工厂方法、属性覆盖/删除、get_elements_by_tag_name 通配符 | 57 |
| render-foundation | **geometry 负坐标**：负坐标 Rect contains/intersection、负值 Size area、DamageTracker NaN/重复 rect；**surface 1x1/零尺寸**；**image_cache gc 后 insert、max_entries=1**；**primitive clips-only bounding_box** | 27 |
| layout-engine | **types 负 margin/z_index**：outer_area 负值、深层嵌套 position、sticky flag；**tree display:none 全跳过**；**engine absolute-in-fixed、flex 窄容器 wrap**；**converter grid-auto-flow dense、percentage 尺寸** | 22 |
| engine | **paint HSL 极端值**：色相 120/240、饱和度/亮度 0/100；**border-style hidden**：不产生填充；**dirty 负坐标/链式合并**；**composite opacity=0、z_index 极值** | 21 |
| wasm-sandbox | **fuel 边界**：set_fuel(0) 即耗尽、fresh instance get_fuel；**memory round-trip**；**extreme i32**：MIN+MAX 溢出语义；**config/error display** | 7 |

### -4. 多 crate 测试覆盖率提升（前轮，2769 测试）

| 模块 | 实现内容 | 新增测试 |
|------|----------|----------|
| security | **mixed content data/blob/javascript URI**、**CORS wildcard + headers**、**same-origin 显式默认端口** | 13 |
| security | **CSP scheme-source 匹配**、**frame-src 限制**、**report-only 模式** | 3 |
| canvas | **变换组合非交换性**、**set_transform 替换验证**、**putImageData 边界** | 8 |
| canvas | **gradient 多 stop 排序**、**重复 offset**、**越界 offset** | 3 |
| dom | **shadow root closed 模式**、**compare_document_position 深度分支** | 4 |
| net | **URL fragment+query**、**IPv4 host**、**相对路径解析**、**组合边界** | 7 |
| storage | **Cache API CRUD**、**覆盖/keys**、**localStorage clear**、**session 隔离** | 5 |

### -3. z-index/is_sticky + 字体整形器 + 行内格式化增强（前轮，2661 测试）

| 模块 | 实现内容 | 新增测试 |
|------|----------|----------|
| layout-engine | **z_index/is_sticky 字段**：LayoutBox 新增 z_index 和 is_sticky 字段，engine 提取 z_index | 2 |
| layout-engine | **fixed 视口坐标调整**：adjust_fixed_to_viewport 递归修正 fixed 元素坐标 | 0 |
| layout-engine | **text-align center/right/justify**：InlineFormattingContext 支持居中、右对齐、两端对齐 | 30+ |
| layout-engine | **vertical_align**：TextRun 新增 vertical_align 字段，DOM 布局集成 | 多处 |
| render-foundation | **文本整形器（TextShaper）**：单行/多行整形，空格处换行，显式换行符支持 | 15 |
| css-parser | **float/clear 属性**：FloatValue/ClearValue 枚举，不区分大小写解析 | 4 |
| style-system | **float/clear 管线集成**：ComputedStyle、apply_property_value、PropertyRegistry | 4 |
| engine | **inline paint 增强**：paint 集成 InlineFormattingContext | 8 |

### -2. CSS 数学函数 + aspect-ratio + DOM 模块测试（前轮，2591 测试）

| 模块 | 实现内容 | 新增测试 |
|------|----------|----------|
| css-parser | **CSS min()/max()/clamp() 数学函数**：解析、求值、嵌套支持，LengthValue::Calc 变体 | 14 |
| style-system | **calc/min/max/clamp 管线集成**：parse_length_or_math 辅助函数，所有长度属性自动支持数学函数 | 6 |
| style-system | **aspect-ratio 属性**：ComputedStyle.aspect_ratio 字段，支持 auto/数字/w:h 斜杠语法 | 4 |
| layout-engine | **aspect-ratio 布局**：converter 传递 aspect_ratio 到 taffy | 2 |
| layout-engine | **box-sizing:border-box 布局验证**：确认 border-box/content-box 布局正确性 | 2 |
| style-system | **@supports selector() 条件验证**：连续组合器检测，无效选择器拒绝 | 4 |
| dom (serializer) | **ProcessingInstruction/Doctype PUBLIC+SYSTEM/Fragment/void 元素/转义 测试** | 8 |
| dom (document) | **PI 内容、set_text_content on Comment/Fragment、多 class 查找、quirks_mode 等** | 10 |
| dom (event) | **事件重用、Debug 格式、捕获阶段 prevent_default、深层嵌套传播** | 5 |

### -1. 关键功能修复 + 测试覆盖率提升（前轮，2359 测试）

通过工作流分析 14 个 crate 的高优先级功能缺口，并行修复并补充测试：

| 模块 | 修复内容 | 新增测试 |
|------|----------|----------|
| style-system | **var() 解析集成到样式计算管线**：级联值中的 var() 引用现在在继承/计算前被解析，包括嵌套自定义属性引用 | 4 |
| style-system | **@container 真实评估**：基于 ContainerContext 评估 min-width/max-width 等条件，无上下文时不应用 | 3 |
| canvas | **roundRect 圆角扁平化**：路径扁平化正确生成圆角弧线顶点（8段/角），而非退化为直角矩形 | 9 |
| storage | **IDB 事务缓冲/回滚**：事务操作缓冲到内存，commit 时应用，abort 时丢弃；tx_get 优先读缓冲区 | 10 |
| webview | **CSS 缓存持久化**：render() 不再丢弃 CSS，cached_css 字段在 load_html/inject_css 间保留 | 4 |
| dom | **节点生命周期测试**：移除后节点操作、重新挂载、错误路径 | 11 |

### 0. 全 crate 功能增强 + 测试覆盖率提升（前轮）

通过并行扫描 14 个 crate 识别出 158 个功能缺口和 133 个测试覆盖缺口，按优先级实现了 373 个新测试和对应功能：

| 模块 | 新增功能 | 新增测试 |
|------|----------|----------|
| engine | text/glyph 渲染、overflow clip、border-radius、z-index compositing、CSS transform、增量渲染 | ~18 |
| render-foundation | image cache + GC、GPU pixel readback、clipping/scissor | ~8 |
| security | CORS preflight、CSP nonce/hash/navigation/document、mixed content blocking、sandbox | ~20 |
| storage | IdbKeyRange、IdbIndex、IdbCursor、IdbTransaction、Cache API | ~25 |
| canvas | HSL/HSLA 颜色、gradient 解析 | ~20 |
| host-runtime | mouse/cursor/IME 事件、综合事件处理 | ~15 |
| net | HTTP send 集成、cookie 过期/SameSite enforcement | ~10 |
| wasm-sandbox | host function imports、fuel limiting | ~10 |
| webview | event callbacks、load_url fetch、execute_script | ~5 |
| css-parser | gradient 解析、bare 0 parsing、calc 改进 | ~40 |
| style-system | 所有属性初始值、grid e2e 测试、structural pseudo-class | ~40 |
| dom | 多 class selector 查询 | ~5 |

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
| 盒模型 | ✅ 100%（含 **box-sizing: border-box** 布局测试） |
| Block/Inline/Flexbox 布局 | ✅ 已实现（行内格式化上下文已实现） |
| Grid 布局 | ⚠️ ~65%（display + auto-flow + 项放置 + grid-area + repeat() + auto-rows/cols；缺 auto-fill 真实支持、命名区域） |
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
| **@supports** | ✅ 已实现（含 **selector() 条件验证**） |
| **@layer** | ✅ 已实现 |
| **@import** | ✅ 已实现 |
| **@container** | ✅ 已实现（解析 + 骨架评估） |
| **scroll-snap** | ✅ 已实现（scroll-snap-type/align/stop + scroll-margin/scroll-padding） |
| **CSS 数学函数** | ✅ 已实现（calc()/min()/max()/clamp() 解析、求值、样式管线集成） |
| **aspect-ratio** | ✅ 已实现（属性解析 + 布局引擎集成） |
| **float/clear** | ✅ 已实现（属性解析 + 样式管线集成） |

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
| M10 WebView API | ✅ (webview + integration tests) |

---

## 下一步优先级

1. **Shadow DOM**（高优先级）— Shadow root、slot、DOM 树封装
2. **Grid 布局增强**（高优先级）— auto-fill 真实支持、命名区域、minmax()
3. **内联布局集成**（高优先级）— 行内格式化上下文集成到 paint 管线、文本换行
4. **更多 Canvas API**（中优先级）— OffscreenCanvas、Path2D 完善、更多合成模式测试
5. **安全增强**（中优先级）— 沙箱、混合内容阻止、COOP/COEP

---

## 归档记录

- **M1** ✅ → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
- **M2** ✅ → [archive/m2-dom.md](archive/m2-dom.md)
- **M3** ✅ → [archive/m3-css-parser-style-system.md](archive/m3-css-parser-style-system.md)
- **M4** ✅ → [archive/m4-layout-engine.md](archive/m4-layout-engine.md)
- **M5** ✅ → [archive/m5-rendering-pipeline.md](archive/m5-rendering-pipeline.md)
- **M7** ✅ → [archive/m7-network-security.md](archive/m7-network-security.md)
