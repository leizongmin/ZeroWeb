# ZeroWeb 运行时控制平面

**最后更新**: 2026-06-02
**执行状态**: 16/16 crate 已实现，6037 个测试全绿，16/16 crate 有 criterion 基准测试（77 个基准），V8 JS 引擎已集成，M11 浏览器应用 + DOM Bridge polyfill（事件/Fetch/console/timer/Web Storage/MutationObserver/IntersectionObserver/ResizeObserver + insertBefore/replaceChild/cloneNode/style/classList/innerHTML/outerHTML/textContent/innerText/navigation properties）+ 地址栏自动补全 + 标签页拖拽排序 + 右键上下文菜单已集成

> **说明**
> 本文记录的是实验性项目的当前实现进度。测试全绿、CI 通过或里程碑推进，并不等于项目已经适合日常使用、商用或其他生产用途；相关风险仍需自行评估。

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace + 16 crate（全部有实质实现） |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ `cargo test --workspace` 6037 个测试全绿 |
| Clippy | ✅ 零警告（全 workspace） |
| 基准测试 | ✅ 16/16 crate 有 criterion 基准（77 个基准） |
| CI | ✅ GitHub Actions（ubuntu/macos/windows）|

### 已实现 crate（16 个）

| Crate | 测试 | 基准 | 说明 |
|-------|------|------|------|
| dom | 503 | ✅ | DOM 树、html5ever 集成、查询 API、序列化、属性、MutationObserver、Range API、遍历/比较方法、Shadow DOM、slot、id_map 自动清理、**模块级单元测试**、**Range select_node/text_content/clone**、**normalize()**、**import_node()**、**slot 分配解析**、**get_elements_by_tag_name_ns**、**has_attribute/remove_attribute/split_text/class_list_replace/contains**、**TreeWalker 深度优先遍历**、**get_elements_by_class_name/set_id/create_comment/insert_before/inner_text**、**NodeIterator 遍历**、**clone_node fragment/replace_child invalid/wildcard tag/nested text_content/insert_before invalid ref**、**HTML 解析器测试（实体/void 元素/错误恢复/Unicode/大文档）**、**MutationObserver 回调/记录验证**、**Event 传播/stopPropagation/stopImmediatePropagation/非冒泡事件**、**节点比较/文档工厂/DOMTokenList 边界/Range 空/序列化 DOCTYPE/TreeWalker 混合/Event 断连节点** |
| css-parser | 738 | ✅ | Tokenizer、Parser、选择器、值解析、@规则、:has()、@container、scroll-snap、calc() 嵌套、媒体查询 range syntax、Token 源位置追踪、min()/max()/clamp() 数学函数、**float/clear**、**vertical_align/list_style/viewport calc**、**parse_cursor(26 关键字)/parse_opacity**、**grid-area 解析**、**hwb color/3D transform/嵌套 var**、**148 种 CSS 命名颜色**、**fit-content() 函数**、**conic-gradient at 位置修复**、**min-content/max-content 关键字**、**word-break 属性**、**writing-mode 属性**、**text-decoration-line/text-transform/letter-spacing/word-spacing**、**3D transform 函数**、**媒体查询 only/逗号 OR/prefers-color-scheme/prefers-reduced-motion/pointer/resolution**、**text-overflow/text-indent/table-layout/caption-side/border-collapse/resize**、**counter-reset/counter-increment/content/quotes**、**page-break/box-decoration-break/image-rendering/isolation**、**overflow-wrap/text-align-last/font-variant-numeric**、**direction/unicode-bidi/tab-size**、**column-count/column-width/object-fit/filter**、**border-image-source/slice/width/repeat/outset** |
| style-system | 964 | ✅ | 级联、继承、计算值、DOM 集成、选择器匹配、简写展开、Grid、@media 评估、Transform、Transitions、Animations、逻辑属性、var() 解析集成、revert 关键字、grid-template-areas、calc/min/max/clamp 管线集成、aspect-ratio、**float/clear**、**grid-area/grid-column/grid-row 简写**、**cursor/opacity 管线集成**、**specificity 竞争/!important/继承/shorthand 展开/var 回退/media 无视口**、**word-break + writing-mode + text-decoration-line + text-transform + letter-spacing**、**3D transform + transform-origin + perspective + perspective-origin + transform-style + backface-visibility**、**grid place-items/place-content/place-self/grid-template 简写**、**counter/content/quotes**、**list-style/page-break/box-decoration-break/image-rendering/isolation**、**overflow-wrap/text-align-last/font-variant-numeric/pointer-events**、**direction/unicode-bidi/tab-size**、**background/font/text-decoration 简写展开**、**column-count/column-width/object-fit/filter**、**border-image-source/slice/width/repeat/outset 管线集成**、**text-shadow/box-shadow 管线集成**、**outline 简写展开**、**list-style-image/column-gap 管线集成**、**known_properties 补全（transform/grid-template-*/grid-auto-flow/row-gap）**、**justify-items/justify-self/align-content/empty-cells/border-spacing 管线集成**、**gap 简写展开** |
| layout-engine | 477 | ✅ | taffy 集成（Block/Flex/Grid/Position）、Grid 轨道解析、Grid 项放置、auto-fill/minmax()、grid-template-areas、零尺寸容器、深层嵌套、aspect-ratio 布局、box-sizing:border-box 测试、**z_index/is_sticky 字段**、**fixed 视口坐标调整**、**text-align center/right/justify**、**vertical_align Sub/Super/TextTop/TextBottom**、**converter 全变体覆盖**、**混合字号/零容器/空白文本**、**overflow/z_index/content_clamp/深层嵌套**、**负 margin/嵌套 flex/absolute-in-relative/overflow hidden/grid auto/零高度块**、**grid 3x3 区域/auto-fill minmax/命名区域解析/百分比 gap**、**grid dense/span/min-max 约束**、**负 margin 合并/grid 行跨行/混合 CJK-Latin/absolute-in-relative/flex 不增长**、**grid 全跨/flex gap/大 padding/absolute 拉伸/inline-block 百分比**、**CJK 字符检测/字符串宽度估算/converter 私有函数/overflow 转换/fixed 视口调整/absolute_position 边界** |
| engine | 503 | ✅ | 渲染管线、paint（文本/glyph、overflow clip、border-radius）、dirty tracking、compositing（z-index 排序）、CSS transform、增量渲染、**DOM Bridge（polyfill: 事件系统 + Fetch API + console + setTimeout/setInterval + insertBefore/replaceChild/cloneNode + CSSStyleDeclaration + DOMTokenList + innerHTML/outerHTML + textContent/innerText + 导航属性）**、**opacity/text-decoration/text-transform 渲染集成**、**命令解析边界测试**、**DomBridge 句柄映射边界**、**DomResult 构造/相等性**、**paint 辅助函数边界/compositing 默认值/dirty 坐标验证/pipeline 状态/dom_bridge 边界** |
| render-foundation | 300 | ✅ | GPU/CPU 渲染、字体栈、image cache + GC、clipping/scissor、颜色 RGBA clamping、image cache eviction、surface resize、**文本整形器（TextShaper + 换行）**、**多次 resize/RGBA clamp/零 max_entries**、**空字符串/单字整形/opacity 零**、**damage tracker 单矩形/重叠合并/颜色钳位/resize 保留/max_entries 零**、**rect 交集/并集/颜色 alpha 混合/面积**、**20 非重叠 rect/Color lerp 透明/缓存 GC 优先级/帧缓冲四角/圆角矩形包围盒** |
| host-runtime | 192 | ✅ | winit 窗口、事件循环、mouse/cursor/IME 事件、**resize 事件**、**鼠标坐标**、**IME composition**、**键盘修饰键**、**修饰键组合/按键重复/鼠标按钮/零尺寸 resize**、**mouse 坐标/keyboard key_code/resize/touch/IME composition**、**多触点/按钮坐标/按键码**、**连续 resize/全修饰键/中键/IME 空/键盘释放**、**HostError debug/TouchPhase 比较/scroll delta 转换/Destroyed 事件忽略/MouseButton 相等性** |
| net | 286 | ✅ | HTTP client、URL、导航历史、Cookie、send 集成测试、cookie 过期/SameSite、**URL userinfo/port/query 边角场景**、**SameSite 全矩阵**、**重定向深度边界**、**非默认端口 origin**、**第三方 cookie/会话 cookie/前进超出**、**URL fragment/空路径/导航历史检查/cookie httpOnly/响应状态文本**、**WebSocket 桩（状态机+消息队列）**、**URL hash/查询参数/请求链/状态文本**、**IPv6 host/SameSite Strict/go_back initial/304 status/URL encoded chars**、**blob/file URL/Cookie path 匹配/导航边界/查询参数边界** |
| security | 304 | ✅ | 同源策略、CORS（preflight）、CSP（nonce/hash/navigation/document）、mixed content blocking、sandbox、COOP/COEP、**CSP scheme-source**、**report-only**、**CORS 简单请求/preflight 生成**、**sandbox 导航/弹窗**、**origin null/invalid/port**、**CSP img-src/nonce/default-src、CORS max-age/wildcard**、**CSP 同源脚本/内联样式/data URI/简单请求 GET**、**report-only/preflight 自定义方法/mixed content/不同端口/sandbox allow-scripts**、**CSP default-src/frame-src/CORS 凭证/混合内容/sandbox popups**、**CORS custom header/CSP data URI/cross-protocol origin/sandbox popups/mixed content ws**、**CSP upgrade-insecure-requests/strict-dynamic/CORS 多方法预检/同源默认端口/sandbox dangerous combo/mixed content blob/COOP popups 矩阵** |
| protocol | 174 | ✅ | IPC 消息、bincode 序列化、**mock channel 契约**、**确定性编码**、**对抗性反序列化**、**大消息/unicode/排序**、**空载荷/unicode 载荷/顺序保持/确定性编码/大载荷 10KB**、**FIFO 循环/Session 存储类型/零 ID/二进制 body/错误 Display**、**NavigateParams referrer/KeyboardEvent 修饰键/MouseEventType 字节区分/ScrollEvent 负值/GoBack vs GoForward**、**method 大小写/referrer 自引用/Ok vs Error 字节/status codes/non-ASCII headers/StorageOp value/交错 send-recv/Send+Sync/空 headers/空 key/负坐标** |
| storage | 259 | ✅ | localStorage、sessionStorage、IndexedDB（IdbKeyRange/IdbIndex/IdbCursor/IdbTransaction）、Cache API、**事务缓冲/回滚**、**NaN/Infinity key 排序**、**唯一索引冲突**、**Cache API CRUD**、**cursor advance/continue/索引迭代**、**事务 commit/abort**、**key/used_size/cache delete+has**、**clear+set/delete range**、**delete_object_store/update_existing/clear/空 store cursor/cache has**、**update/会话隔离/count/cursor 越界/keys/事务提交/remove 不存在**、**cursor reverse/cache put URLs/localStorage key order/multiEntry index/sessionStorage clear**、**IDB 事务空 store/KeyRange 多类型/cursor advance(0)/Cache 覆写/空字符串值/唯一索引/multiEntry 空数组** |
| canvas | 362 | ✅ | Canvas 2D API、路径、变换、drawImage、shadow 属性、**Path2D 高级方法**、**lineDash**、**roundRect 圆角扁平化**、**alpha 混合**、**像素边界溢出**、**clip+drawImage**、**ellipse/arcTo/conic_gradient**、**line_join/line_cap stroke 渲染**、**is_point_in_stroke**、**composite operation 像素级验证**、**image_smoothing_enabled**、**resize/clear/stroke_zero/negative_translate/restore_nosave/globalAlpha_clamp**、**gradient 多 stop/radial gradient/fillRule/lineDash/measure_text/shadow 属性**、**createImageData/getTransform/transform() 乘法/miterLimit/textDirection**、**同心圆渐变/路径跨 resize/退化变换/脏矩形越界/零长度渐变**、**嵌套 save/restore/clear_rect/translate+scale/line_width 边界/fillText/stroke_rect 零/putImageData roundtrip** |
| webview | 251 | ✅ | WebView 嵌入 API、Builder、event callbacks、load_url fetch、execute_script、**CSS 缓存持久化**、**状态机**、**配置**、**多次导航/注入 CSS/自定义视口**、**事件系统集成测试**、**Fetch API 端到端测试**、**console/timer API 端到端测试（log/warn/error/time/setTimeout/setInterval）**、**DOM API 端到端测试**、**生命周期状态/resize+render/事件回调/CSS 累积注入/脚本错误/回调移除/outerHTML/documentFragment** |
| wasm-sandbox | 158 | ✅ | WASM 运行时（wasmi）、host function imports、fuel/execution limiting、**host 错误传播**、**参数类型校验**、**offset 溢出**、**memory grow/多参数 host/递归限制**、**memory 读写/多函数/fuel 消耗/global 读取/无效模块错误**、**多实例隔离/table 导出/global 读取/fuel 追踪/错误处理**、**fuel 禁用 get_fuel/u64::MAX fuel/内存边界读写/i64 Display/config chaining/has_memory 误匹配/空字符串函数名/多实例独立/内存 roundtrip/start 函数 trap** |
| script-sandbox | 77 | ✅ | **V8 引擎集成（rusty_v8）**、Isolate/Context 管理、脚本编译执行、JSON 输出、错误处理（编译/运行时/超时）、**状态隔离、execute_json 边界测试、ES6+ 特性（Map/Set/Symbol/Proxy/async/await/rest/for-of/静态方法）、77 个单元测试全绿** |
| browser-shell | 209 | ✅ | **浏览器应用数据模型**：Tab/TabManager（多标签页管理、导航历史、**拖拽排序 move_tab**）、Bookmarks（书签/文件夹增删改查）、History（页面访问记录、搜索、清除）、BrowserShell（顶层协调器）、**Autocomplete（地址栏自动补全，历史+书签搜索、分数排序、书签优先）**、**ContextMenu（右键上下文菜单，5 种场景默认菜单项）**、**Tab 拖拽边界/导航历史边界/Bookmarks 过滤/History clear+search/Download 移除/Autocomplete 空查询+大小写/BrowserShell 导航清空前进/Settings 搜索/ContextMenu 子菜单查找** |

### 跨 crate 集成测试

| 测试模块 | 测试数 | 覆盖场景 |
|----------|--------|----------|
| DOM Bridge Polyfill (V8) | 53 | createElement/textNode/setAttribute/appendChild/insertBefore/cloneNode/textContent/replaceChild/DocumentFragment/getElementById/CSSStyleDeclaration/DOMTokenList/navigation properties/innerHTML/outerHTML/Fetch/Headers/Response/Storage/MutationObserver/CustomEvent/IntersectionObserver/ResizeObserver/setTimeout/setInterval |
| CSS + Style System | 3 | 样式计算、级联优先级、继承 |
| Render Pipeline | 4 | 完整管线、CSS 集成、耗时分解、复杂页面 |
| Net + Security | 3 | 同源判断、CORS 策略、安全上下文 |
| Storage + Protocol | 3 | localStorage CRUD+IPC、session 隔离、origin 隔离 |
| Protocol + Navigation | 1 | 导航历史 + IPC 序列化 |
| Canvas + Render | 5 | Canvas 绘图图元、路径、变换、save/restore、WebView 集成 |
| WASM Sandbox | 5 | 编译、调用、导出查询、内存读写、错误恢复 |
| WebView Full Pipeline | 4 | 完整生命周期、复杂页面、重复加载、脚本占位 |
| CSS Transform Pipeline | 1 | CSS 解析→样式系统→计算值 |
| Media Query Integration | 1 | 媒体查询解析 + 上下文评估 |
| Canvas Gradient | 1 | 渐变色采样 + 颜色插值验证 |
| Grid Layout Pipeline | 1 | 网格模板→样式→布局引擎 |
| Counter Cascade | 1 | counter-reset/increment 级联集成 |
| Overflow Wrap Pipeline | 1 | overflow-wrap 解析→样式→继承验证 |
| Text Align Last Pipeline | 1 | text-align-last 解析→样式验证 |
| Direction Pipeline | 1 | direction:rtl 解析→样式→继承验证 |
| Tab Size Pipeline | 1 | tab-size 解析→样式→继承验证 |
| Storage Protocol IPC | 1 | StorageManager 操作→IPC 序列化→反序列化验证 |
| Border Image Source Pipeline | 1 | border-image-source 解析→样式→计算值 |
| Border Image Slice Pipeline | 1 | border-image-slice 解析→样式→fill 验证 |
| Border Image Repeat Pipeline | 1 | border-image-repeat 解析→样式→水平/垂直验证 |
| Border Image Width Pipeline | 1 | border-image-width 解析→样式→Number/Length 验证 |
| Border Image Outset Pipeline | 1 | border-image-outset 解析→样式→Length/Number 验证 |
| Text Shadow Pipeline | 1 | text-shadow 解析→样式→偏移/颜色验证 |
| Box Shadow Pipeline | 1 | box-shadow 解析→样式→偏移/模糊验证 |
| Box Shadow Inset Pipeline | 1 | box-shadow:inset 解析→样式→inset 标志验证 |
| Text Shadow Inheritance | 1 | text-shadow 继承验证（父→子） |
| Outline Width Pipeline | 1 | outline-width 解析→样式验证 |
| List Style Image Pipeline | 1 | list-style-image url 解析→样式→计算值 |
| Column Gap Pipeline | 1 | column-gap 解析→样式验证 |
| Text Shadow Inheritance Pipeline | 1 | text-shadow 继承（父→子）验证 |
| Box Shadow No Inheritance | 1 | box-shadow 不继承验证 |
| Outline Shorthand Pipeline | 1 | outline 简写→展开→样式验证 |
| Gap Shorthand Pipeline | 1 | gap 简写→row-gap + column-gap 展开 |
| Empty Cells Pipeline | 1 | empty-cells:hide 解析→样式验证 |
| Border Spacing Pipeline | 1 | border-spacing:5px 10px 解析→样式验证 |
| Empty Cells Inheritance | 1 | empty-cells 继承验证 |
| Border Spacing Inheritance | 1 | border-spacing 继承验证 |
| Box Shadow Render Pipeline | 1 | box-shadow 通过完整管线生成 ShadowPrimitive |
| Box Shadow + Background Pipeline | 1 | background-color + box-shadow 组合渲染 |
| Box Shadow Negative Offset | 1 | box-shadow 负偏移渲染验证 |
| Box Shadow None Pipeline | 1 | 无 box-shadow 时 shadows 为空 |
| Text Shadow Render Pipeline | 1 | text-shadow 通过完整管线生成阴影 glyph |
| Text Shadow + Color Pipeline | 1 | text-shadow 特定颜色渲染验证 |
| Text Shadow None Pipeline | 1 | 无 text-shadow 时无额外 glyph |
| Background Image URL Pipeline | 1 | background-image url() 生成 ImagePrimitive |
| Background Image None Pipeline | 1 | background-image none 时 images 为空 |
| Background Image + Color Pipeline | 1 | background-color + background-image 组合渲染 |
| Box Shadow Not Inherited | 1 | box-shadow 不继承验证 |
| Text Shadow Inherited Pipeline | 1 | text-shadow 继承验证（父→子） |
| Box Shadow + Outline Pipeline | 1 | box-shadow + outline 组合渲染 |
| All Three New Properties | 1 | box-shadow + background-image + text-shadow 全组合 |
| Box Shadow Spread Only | 1 | box-shadow 仅 spread-radius 渲染验证 |

---

## 最近完成的改进

### -56. 全 crate 边界条件测试覆盖率提升第三轮（本轮，6037 测试）

系统化扫描 16 个 crate 的测试密度，按 test/code ratio 排序，对密度最低的 crate 集中添加边界条件测试。8 个 agent 并行工作：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| layout-engine | **converter 私有函数**（find_top_level_comma/tokenize_track_list/parse_single_auto_track/parse_min_track/parse_max_track/parse_minmax_as_non_repeated）、**inline 模块**（is_cjk_character Unicode 范围/estimate_string_width 空串/ASCII/CJK/混合/零字号）、**types 模块**（absolute_position 零/带 border/带 parent/负 parent/fixed）、**engine 模块**（convert_overflow_to_clip 全变体/adjust_fixed_to_viewport 嵌套/根级/非 fixed/深层/重置） | +52 |
| engine | **paint 辅助函数**（named_color 大小写/simple_hash 边界/length_to_f32 Px 变体/lowercase transform/nested opacity/text decoration 零宽/gradient 角度/单色 stop/box-shadow 负偏移）、**compositing**（new 默认值/absolute+z-index/Debug+Clone）、**dirty**（padding+border/坐标验证/大偏移/包含矩形/循环重置/零高框）、**pipeline**（viewport accessors/不同视口/dirty_tracker_mut 持久化/重复渲染/cached layout/初始状态）、**dom_bridge**（u64::MAX/escaped quotes/Unicode/clear+re-register/u64::MAX unregister/Default Eq） | +37 |
| dom | **节点比较**（同节点 DocumentPosition 为 0）、**文档工厂**（hyphenated element name）、**DOMTokenList**（重复 class/remove 不存在/空 class 操作）、**Range**（空元素 select_node_contents/嵌套 clone_contents 独立性）、**序列化**（DOCTYPE PUBLIC+SYSTEM/SYSTEM only）、**TreeWalker**（混合节点类型/child+sibling 导航）、**Event**（断连节点 dispatch/不可取消 prevent_default） | +13 |
| wasm-sandbox | **fuel 禁用 get_fuel**、**u64::MAX fuel**、**内存边界读写**、**i64 Display**、**config chaining**、**has_memory 函数导出名**、**空字符串函数名**、**多实例独立**、**内存 roundtrip**、**start 函数 trap** | +11 |
| protocol | **method 大小写保持**、**referrer 自引用**、**Ok vs Error 字节区分**、**ProcessRole Debug**、**HTTP status codes 100/204/304/500/503**、**non-ASCII headers**、**StorageOp value with non-Set**、**交错 send-recv**、**Send+Sync compile check**、**空 headers vec**、**空 key**、**负坐标** | +12 |
| storage | **IDB 空事务 store 列表**、**KeyRange only 多类型**、**cursor advance(0)**、**Cache put 覆写**、**Cache delete 不存在**、**CacheStorage has 精确匹配**、**localStorage 空字符串值**、**localStorage get 不存在**、**唯一索引 不同/相同**、**multiEntry 空数组** | +10 |
| net | **blob: URL**、**file: URL**、**超长 URL**、**空查询 ?**、**key-only 查询参数**、**cookie path 子路径匹配**、**cookie expires 精确 now**、**navigation replace at beginning**、**navigation 重复 URL**、**navigation max entries 边界** | +10 |
| security | **CSP upgrade-insecure-requests**、**CSP strict-dynamic + nonce**、**CSP 导航指令不回退 default-src**、**CORS 多方法预检部分拒绝**、**同源默认端口归一化**、**sandbox allow-same-origin+allow-scripts 危险组合**、**sandbox 导航标志组合**、**mixed content blob URI**、**COOP same-origin-allow-popups 全矩阵**、**mixed content + CSP data URI 组合** | +9 |
| webview | **load_url→complete_load 生命周期**、**fail_load 重置**、**resize+render**、**set_title 事件**、**inject_css 累积**、**execute_script 编译错误**、**execute_script 运行时错误**、**remove_callback 越界**、**DOM outerHTML**、**DOM createDocumentFragment** | +10 |
| browser-shell | **Tab 拖拽首到末/无效索引**、**Tab 导航历史边界**、**Bookmarks 按 URL 过滤**、**History clear+search**、**DownloadManager 移除不存在**、**Autocomplete 空查询/大小写不敏感**、**BrowserShell 导航清空前进**、**Settings 搜索 URL**、**ContextMenu 子菜单查找** | +11 |
| canvas | **嵌套 save/restore pixel verify**、**clear_rect 区域**、**translate+scale 组合**、**line_width 边界值**、**fillText no panic**、**stroke_rect 零尺寸**、**putImageData+getImageData roundtrip** | +12 |
| style-system | **cascade 空/单一/多属性**、**position ordering**、**specificity ordering**、**inherit 无 parent 非继承属性**、**空级联全初始值**、**initial on 继承属性** | +7 |

Total: 5863 → 6037 (+174 tests)

### -55. browser-shell + script-sandbox 基准测试补全（前轮，5848 测试）

补全最后 2 个 crate 的 criterion 基准测试，实现 16/16 crate 全覆盖：

| 模块 | 新增内容 | 基准数 |
|------|----------|--------|
| browser-shell | **标签页创建吞吐量**、**书签批量添加 1k**、**历史记录搜索 1k**、**自动补全建议 500 条**、**下载管理器 100 并发** | 5 |
| script-sandbox | **简单表达式执行**、**字符串操作**、**循环计算 1k**、**JSON 序列化**、**沙箱创建开销**、**自定义配置创建** | 6 |

Total: 5796 → 5848 tests, 14/16 → 16/16 crates with benchmarks (77 total benchmarks)

### -54. 测试文件拆分 + script-sandbox 边界测试 + browser-shell 集成测试（前轮，5768 测试）

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **文件拆分**：tests.rs (5637行) → tests/mod.rs + tests_1..4（每个 <1930 行） | — |
| dom | **文件拆分**：tests.rs (7841行) → tests/mod.rs + tests_1..5（每个 <1900 行） | — |
| script-sandbox | **状态隔离**：execution 间变量不泄漏、独立 sandbox 隔离；**execute_json 边界**：嵌套对象/空对象/空数组/null/boolean/undefined/语法错误/空脚本/特殊字符；**大脚本**：10k循环/大字符串拼接；**ES6+**：Map/Set/Symbol/Proxy/async-await/默认参数/rest参数/for-of/Object静态方法/Array静态方法/JSON内置/Math方法 | 25 |
| integration | **browser-shell + protocol + storage 集成**：导航IPC序列化/多标签IPC排序/前进后退IPC/书签+Storage持久化/历史+Storage持久化/下载+FetchResponse IPC/自动补全+书签/设置+StorageOp IPC/缩放+Reload IPC/页面加载+LoadComplete IPC | 9 |

Total: 5733 → 5796 (+63 tests)

### -53. 集成测试文件拆分 + 新增 106 个测试（本轮，5729 测试）

重构集成测试文件结构，新增 HTML 解析器/MutationObserver/Event/DomBridge 边界测试、V8 polyfill 全 API 行为测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| integration | **文件拆分**：6424 行 lib.rs → 17 个模块文件（每个 <2000 行） | — |
| integration/dom_bridge_polyfill | **V8 polyfill 全 API 行为测试**：DOM 操作、CSSStyleDeclaration、DOMTokenList、导航属性、innerHTML/outerHTML、Fetch/Headers/Response、localStorage/sessionStorage、MutationObserver、CustomEvent、IntersectionObserver、ResizeObserver、setTimeout/setInterval | 53 |
| dom/parser | **HTML 解析器测试**：空文档/纯文本/完整文档/属性/嵌套/void 元素/错误恢复/未闭合标签/HTML 实体/注释/script+style/DOCTYPE/Unicode/大文档 | 22 |
| dom/mutation | **MutationObserver 测试**：ChildList/Attributes/CharacterData 记录、回调调用、空记录、MutationType 相等性、clone 一致性 | 7 |
| dom/event | **Event 系统测试**：stopPropagation 阻止冒泡、多监听器、stopImmediatePropagation、非冒泡事件、EventPhase、preventDefault 不可取消、init_for_dispatch 重置、EventListenerHandle | 10 |
| engine/dom_bridge | **命令解析边界**：空白容错/单引号/空字符串/未知命令；**DomBridge 句柄边界**：重复注册/批量注册/unregister+resolve/clear；**DomResult 测试** | 14 |

Total: 5623 → 5733 (+110 tests)

### -52. DOM Bridge polyfill 增强 + browser-shell 边界测试（本轮，5568 测试）

新增 DOM Bridge polyfill 核心扩展和 browser-shell 边界测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| engine/dom_bridge | **新 DomCommand 变体**：InsertBefore、ReplaceChild、CloneNode、GetStyle、SetStyle、SetInnerHtml、GetParentNode | 9 |
| engine/dom_bridge | **polyfill 新 API**：insertBefore、replaceChild、cloneNode、CSSStyleDeclaration（getProperty/setProperty/cssText）、DOMTokenList/classList（add/remove/toggle/replace）、innerHTML getter+setter、outerHTML、textContent/innerText、导航属性（firstChild/lastChild/nextSibling/previousSibling）、createDocumentFragment | 10 |
| browser-shell | **DownloadManager 边界**：暂停已完成/恢复非暂停/取消已完成/失败后完成/移除活跃/多下载活跃计数/进度转换/清除保留活跃 | 9 |
| browser-shell | **FindState 边界**：零匹配环绕/关闭重置/环绕上一条/中间范围上一条 | 6 |
| browser-shell | **Zoom 边界**：最大钳位/最小钳位/重置/直接设置 | 4 |
| browser-shell | **BrowserShell 集成**：页面加载记录历史/标题更新/错误停止加载/多次导航/后退后导航清空前进 | 5 |
| browser-shell | **Bookmarks 文件夹**：级联删除/多文件夹隔离 | 3 |
| browser-shell | **Settings 验证**：默认值/搜索 URL 编码/自定义主页 | 3 |
| 全 crate | **测试警告修复**：移除 10 个 crate 中不必要的 mut/unused/unsafe 警告 | — |

Total: 5520 → 5568 (+48 tests)

### -51. DOM Bridge 事件系统 + WebView 事件集成测试 + 右键上下文菜单（本轮，5436 测试）

新增 DOM Bridge 事件命令、WebView 端到端事件集成测试、browser-shell 右键上下文菜单：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| engine/dom_bridge | **AddEventListener/RemoveEventListener/DispatchEvent** DomCommand 变体；polyfill 新增完整事件系统（addEventListener、removeEventListener、dispatchEvent、CustomEvent） | 8 |
| webview | **V8 + DOM polyfill 端到端事件测试**：addEventListener/removeEventListener/dispatchEvent 可用性、CustomEvent 构造、事件监听器触发、事件对象传递、preventDefault、capture 顺序、built-in 节点、attribute 往返 | 11 |
| browser-shell/context_menu | **ContextMenu 数据模型**：MenuItem（action/separator/sub_menu）、ContextType（Page/Link/Image/Selection/Editable）、5 种场景默认菜单项、子菜单递归查找 | 15 |

Total: 5396 → 5436 (+40 tests)

### -50. DOM Bridge + browser-shell 下载/设置/缩放/查找模型（前轮，5396 测试）

新增 DOM Bridge 模块和 browser-shell 四个数据模型：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| engine/dom_bridge | **DomCommand** 枚举（16 种 DOM 操作命令）、**DomResult** 枚举（6 种结果类型）、**DomBridge**（JS 句柄↔DOM NodeId 映射）、**命令解析器**（解析 JS DOM API 调用字符串）、**polyfill 生成器**（注入 document/Element API 桩代码到 JS 环境） | 37 |
| browser-shell/download | **DownloadManager**：开始/暂停/恢复/取消/完成下载，进度追踪，清除已完成 | 12 |
| browser-shell/settings | **BrowserSettings**：4 种搜索引擎、主页、JS/Cookie/隐私设置、缩放、下载目录 | 5 |
| browser-shell/browser | **FindState**（页面查找状态）、**zoom 操作**（放大/缩小/重置/钳位） | 10 |

Total: 5337 → 5396 (+59 tests)

### -49. browser-shell 数据模型 + zero-browser 应用入口（前轮，5337 测试）

实现 browser-shell crate（M11 浏览器应用数据模型层）和 zero-browser 应用入口：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| browser-shell | **Tab/TabManager**：标签页管理（创建/关闭/切换）、导航历史（前进/后退）、加载状态；**Bookmarks**：书签/文件夹增删改查；**History**：页面访问记录、不区分大小写搜索、清除；**BrowserShell**：顶层协调器 | 93 |
| zero-browser | **BrowserApp**：连接 BrowserShell + WebView + HostRuntime 的完整浏览器应用，GPU 渲染工具栏（标签栏/地址栏/导航按钮），键盘快捷键（L=地址栏/T=新标签/W=关闭/R=刷新/←→=前后退/Home=首页） | — |

browser-shell 修复：`Tab::go_forward()` 在空历史时 usize 下溢 panic。

Total: 5246 → 5337 (+91 tests)

### -48. V8 引擎集成 + script-sandbox 实现（前轮，5246 测试）

将 rusty_v8 预编译版本集成到 script-sandbox crate，实现完整的 JavaScript 执行能力：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| script-sandbox | **V8Sandbox 结构体**：Isolate/Context 生命周期管理、脚本编译执行、JSON 输出、错误处理（编译/运行时/超时） | 52 |
| webview | **execute_script 使用 V8 沙箱**：JavaScript 脚本现在可以真正执行，返回结果字符串 | 测试更新 |
| integration | **WebView 脚本执行集成测试**：验证端到端脚本执行 | 1 更新 |

script-sandbox crate 实现的功能：
- `V8Sandbox::new()` — 创建 V8 Isolate（首次调用全局初始化 V8 平台）
- `V8Sandbox::with_config(config)` — 自定义堆限制和超时
- `V8Sandbox::execute(code)` — 编译并执行 JS，返回 `ScriptResult`
- `V8Sandbox::execute_json(code)` — 执行 JS 并返回 JSON 字符串
- `V8Sandbox::v8_version()` — 获取 V8 引擎版本号
- 支持的 JS 特性：ES6+（箭头函数、类、解构、展开运算符、模板字符串）、JSON、Math、数组方法等

Total: 5194 → 5246 (+52 tests)

### -43. CSS transition/animation/custom properties + 交互属性管线测试 + 43 测试（本轮，5111 测试）

新增 CSS transition/animation/custom properties 交互属性管线集成测试、engine paint 边界测试、4 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| integration | **transition 长属性/animation 简写/custom property var()/cursor/pointer-events/white-space/letter-spacing/user-select** 管线；**cursor/letter-spacing/white-space 继承**；**visibility:hidden 渲染** | 13 |
| engine/paint | **outline-style:none/border-style:hidden/multiple box-shadow/opacity=0/text-transform capitalize/border-radius/outline-offset/line-through/no-node-id** | 10 |
| protocol | **NavigateParams referrer/KeyboardEvent 全修饰键/MouseEventType 字节区分/ScrollEvent 负值/GoBack vs GoForward** | 5 |
| wasm-sandbox | **partial overwrite/WasmValue Display/跨类型相等/LinkerConfig 追加/SandboxConfig 默认** | 5 |
| webview | **config 默认值/last_render 状态/resize+render/is_loading/callback 移除** | 5 |
| host-runtime | **HostError debug/TouchPhase 比较/scroll delta 转换/Destroyed 事件忽略/MouseButton 相等性** | 5 |
Total: 5068 → 5111 (+43 tests)

### -44. CSS 表格/文本/布局属性管线 + 4 crate 边界测试 + 23 个测试（本轮，5134 测试）

新增 CSS 表格/文本/布局属性管线集成测试、4 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| integration | **table-layout/caption-side/border-collapse/resize/word-break/writing-mode/isolation** 管线；**isolation 非继承** | 8 |
| dom | **append_child 排序/set_attribute 覆写/元素大小写保留/remove_child 中间兄弟/create_text_node 空字符串** | 5 |
| layout-engine | **display:none 排除/块级占满父容器/flex 换行/inline-block 尺寸/absolute 定位** | 5 |
| css-parser | **空媒体查询/:not 嵌套伪类/自定义属性/@supports/多动画名称** | 5 |
Total: 5111 → 5134 (+23 tests)

### -45. style-system/security/storage/net 边界测试 + 23 个测试（本轮，5157 测试）

新增 style-system 属性管线测试、安全模块边界测试、存储边界测试、网络边界测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| style-system | **background-color transparent/color currentColor/display inline-block+flex/position fixed/overflow-x/y/z-index auto/font-weight** | 8 |
| security | **CSP report-only/origin scheme/CORS simple request/mixed content HTTP/sandbox allow-scripts** | 5 |
| storage | **localStorage bulk+clear/session origin isolation/special characters/IDB abort+insertion order** | 5 |
| net | **URL userinfo/cookie SameSite+Secure/navigation forward limit/response status/query param** | 5 |
Total: 5134 → 5157 (+23 tests)

### -46. CSS flexbox/font/overflow 管线集成 + canvas/engine 边界测试 + 19 个测试（本轮，5176 测试）

新增 CSS flexbox/font/overflow 属性管线集成测试、canvas 和 engine 边界测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| integration | **flex-direction/justify-content/align-items/flex-wrap** 管线；**font-family/font-weight/line-height** 管线；**var() 回退**；**overflow 简写** | 9 |
| canvas | **嵌套 save/restore line_width/多 arc 路径/零尺寸 resize/空路径 fill+stroke/global_alpha 边界** | 5 |
| engine/paint | **border solid 四边/负坐标/超大尺寸/RGBA 钳位/多子节点布局** | 5 |
Total: 5157 → 5176 (+19 tests)

### -47. CSS Grid/position/box-model 集成 + render/layout 边界测试 + 18 个测试（本轮，5194 测试）

新增 CSS Grid/position/box-model 属性管线集成测试、render-foundation 和 layout-engine 边界测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| integration | **grid-template-columns/rows、grid-auto-flow、display:grid** 管线；**position:absolute + top/left**；**margin/padding 简写**；**width/height** | 8 |
| render-foundation | **Color lerp 边界/Rect 无交集扩展/ImageCache 零 max_entries/DamageTracker 同区域合并/Size 零面积** | 5 |
| layout-engine | **LayoutBox 默认值/LayoutResult 视口/LayoutBox 子节点/OverflowClip 可见vs隐藏/内容区域计算** | 5 |
Total: 5176 → 5194 (+18 tests)

### -42. CSS 渐变渲染管线集成 + 44 测试（上轮，5038 测试）

将 CSS 渐变（linear-gradient / radial-gradient / conic-gradient）从"已解析未使用"推进到"已渲染集成"：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **BackgroundImageValue::Gradient** 变体；**parse_background_image** 调用 parse_gradient 回退 | 8 |
| style-system | **BackgroundImageComputedValue::Gradient** 变体；**expand_background** 识别渐变函数 | 6 |
| engine/paint | **gradient_to_primitive** 将 GradientValue 转为 GradientPrimitive；**linear_direction_to_kind** 方向→坐标；**convert_color_stops** 色标转换 | 15 |
| integration | linear/radial/conic/repeating 渐变管线、简写展开、不继承验证 | 10 |
| render-foundation | GradientPrimitive bounding box、多渐变合并、色标顺序 | 5 |

支持范围：linear-gradient（所有方向 + 角度）、radial-gradient（所有尺寸 + 自定义位置）、conic-gradient（仅存储暂不渲染）。

### -41. CSS box-shadow/text-shadow/background-image 渲染集成 + 59 测试（上轮，4994 测试）

将 box-shadow、text-shadow、background-image 从"已解析存储"推进到"已渲染集成"：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| engine/paint | **paint_box_shadow()** — 生成 ShadowPrimitive；**paint_background_image()** — 生成 ImagePrimitive；**paint_text() text-shadow** — 阴影 glyph 在主 glyph 之前绘制 | 19 |
| integration | box-shadow/text-shadow/background-image 完整渲染管线测试、继承性验证、组合渲染验证 | 15 |
| css-parser | box-shadow/text-shadow/background-image 解析边界测试（空字符串、颜色位置、引号 URL、大小写） | 10 |
| style-system | box-shadow/text-shadow/background-image 计算值边界测试（默认值、RGBA 解析、负值、继承） | 8 |
| render-foundation | ShadowPrimitive/ImagePrimitive 包围盒边界测试（大模糊、负偏移、多阴影合并） | 7 |

### -40. CSS border-image 简写 + counter-set + 5 集成测试 + 8 crate 边界测试 + 28 个测试（上轮，4935 测试）

新增 CSS border-image 简写展开、counter-set 属性、5 个跨 crate 集成测试、8 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| shorthand | **border-image 简写展开**：source/slice/width/outset/repeat 解析 | 5 |
| css-parser | **counter-set**：none/name integer；边界解析测试 | 6 |
| style-system | **counter-set 管线**（非继承）；border-image 简写测试 | 10 |
| integration | **border-image 简写/counter-set/empty-cells/border-spacing 继承/justify-items** | 5 |
| dom | root-only doc/text 特殊字符/re-append/空属性值/空 tag search | 5 |
| layout-engine | grid named span 2/flex align-self stretch/margin auto center/inline-block in flex/nested grid | 5 |
| engine | border-image paint/empty-cells paint/zero-opacity composite/border-spacing render/counter-set pipeline | 5 |
| canvas | drawImage zero/identity transform/quadraticCurveTo/compositeOperation roundtrip | 5 |
| security | CSP multi script-src/CORS PUT/same host diff port/sandbox forms+scripts/report-only no-block | 5 |
| net | URL query only/cookie domain/301 redirect/URL hash fragment/navigation replace | 5 |
Total: 4907 → 4935 (+28 tests)

### -39. CSS justify-items/justify-self/align-content + empty-cells/border-spacing + gap 简写 + 5 集成测试 + 8 crate 边界测试 + 61 个测试（前轮，4907 测试）

新增 5 个 CSS 属性、gap 简写展开、5 个跨 crate 集成测试、8 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **empty-cells**：show/hide；**border-spacing**：1-2 length 值 | 10 |
| style-system | **justify-items/justify-self**（非继承）；**align-content**（非继承）；**empty-cells**（继承）；**border-spacing**（继承）；**gap 简写展开** | 25 |
| integration | **gap 简写/empty-cells/border-spacing/empty-cells 继承/border-spacing 继承** | 5 |
| dom | 无效标签/无属性/detached fragment/shallow clone 属性/子节点计数 | 5 |
| security | 空 CSP/CORS 通配符+凭证/data: URI 混合/空 sandbox/同源 | 5 |
| storage | 自增 key/缓存空匹配/localStorage 覆写/空索引 cursor/session 越界 | 5 |
| net | 仅 scheme URL/cookie max-age=0/forward 超出/双斜杠路径/空 body | 5 |
| protocol | 空载荷/空 recv/删除空 key/fetch 空 url/navigation roundtrip | 5 |
| layout-engine | flex 负 order/grid auto-fill 空/margin auto 居中/absolute 全边/inline max-width | 5 |
Total: 4846 → 4907 (+61 tests)

### -38. CSS list-style-image + column-gap + known_properties 修复 + 5 集成测试 + 8 crate 边界测试 + 61 个测试（前轮，4846 测试）

新增 CSS list-style-image 属性（继承）、column-gap 属性（非继承）、修复 known_properties 缺失 6 个条目、5 个跨 crate 集成测试、8 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **list-style-image**：none/url；边界解析测试 | 8 |
| style-system | **list-style-image 管线**（继承）；**column-gap 管线**（非继承）；**known_properties 补全 6 条目** | 17 |
| integration | **list-style-image/column-gap/text-shadow 继承/box-shadow 不继承/outline 简写** | 5 |
| dom | create_comment 空/class_name 无匹配/attribute 覆写/remove_child invalid/fragment 空 | 5 |
| canvas | restore 无 save/fillText 空/strokeRect 零/radial 负半径/clip 空 | 5 |
| render-foundation | ImageCache 覆写/DamageTracker clear/Color max RGBA/FrameBuffer 非法坐标/TextShaper 空白 | 5 |
| host-runtime | keyboard 未知/mouse 负坐标/touch ended/scale changed/IME commit 空 | 5 |
| wasm-sandbox | 无导出/参数类型错/无 global/fuel 0 调用/memory 同引用 | 5 |
| webview | 大尺寸/注入空/minimal html/title 无回调/CSS 累积 | 5 |
Total: 4785 → 4846 (+61 tests)

### -37. CSS text-shadow/box-shadow 管线 + outline 简写 + 5 集成测试 + 8 crate 边界测试 + 60 个测试（前轮，4785 测试）

新增 CSS text-shadow/box-shadow 样式管线集成、outline 简写展开、5 个跨 crate 集成测试、8 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| style-system | **text-shadow 管线集成**（继承属性）；**box-shadow 管线集成**（非继承）；**outline 简写展开** | 20 |
| integration | **text-shadow/box-shadow/inset/inheritance/outline-width 管线集成** | 5 |
| css-parser | mixed slice values/two-value position/contain strict/multiple filters/text-shadow color | 5 |
| dom | clone_node fragment/replace_child invalid/wildcard tag/nested text_content/insert_before invalid ref | 5 |
| security | CORS custom header/CSP data URI/cross-protocol origin/sandbox popups/mixed content ws | 5 |
| storage | cursor reverse/cache put URLs/localStorage key order/multiEntry index/sessionStorage clear | 5 |
| net | IPv6 host/SameSite Strict/go_back initial/304 status/URL encoded chars | 5 |
| engine | box-shadow paint/text-shadow paint/negative z-index/outline render/dirty merge adjacent | 5 |
| layout-engine | grid full span/flex gap/large padding/absolute stretch/inline-block percent | 5 |
Total: 4725 → 4785 (+60 tests)

### -36. CSS border-image 属性 + 5 集成测试 + 8 crate 边界测试 + 106 个测试（前轮，4725 测试）

新增 5 个 CSS border-image 属性、5 个跨 crate 集成测试、8 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **border-image-source**：none/url；**border-image-slice**：number/percent/fill/1-4 值；**border-image-width**：auto/number/length/percent；**border-image-repeat**：stretch/repeat/round/space/1-2 值；**border-image-outset**：number/length/1-4 值 | 28 |
| style-system | **5 属性管线集成**（全部非继承）+ PropertyValue 枚举 + apply/initial | 38 |
| integration | **border-image-source/slice/repeat/width/outset 管线集成** | 5 |
| wasm-sandbox | 多实例隔离/table 导出/global 读取/fuel 追踪/错误处理 | 5 |
| host-runtime | 连续 resize/全修饰键/中键/IME 空/键盘释放 | 5 |
| protocol | FIFO 循环/Session 存储类型/零 ID/二进制 body/错误 Display | 5 |
| layout-engine | 负 margin 合并/grid 行跨行/混合 CJK-Latin/absolute-in-relative/flex 不增长 | 5 |
| render-foundation | 20 非重叠 rect/Color lerp 透明/缓存 GC 优先级/帧缓冲四角/圆角矩形包围盒 | 5 |
| engine | 深层嵌套可见性/同 z-index/@media 管线/多次增量 dirty/border-image paint | 5 |
| canvas | 同心圆渐变/路径跨 resize/退化变换/脏矩形越界/零长度渐变 | 5 |
| webview | load 前注入/title 事件/零视口/失败恢复/连续 load | 5 |
Total: 4619 → 4725 (+106 tests)

### -35. CSS background-clip/origin + 5 集成测试 + 6 crate 边界测试 + 56 个测试（前轮，4619 测试）

新增 2 个 CSS 背景属性、5 个跨 crate 集成测试、6 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **background-clip**：border-box/padding-box/content-box/text；**background-origin**：padding-box/border-box/content-box | 6 |
| style-system | **2 属性管线集成**（全部非继承） | 15 |
| integration | **background-size/background-attachment/background-clip/background-origin/accent-color 管线集成** | 5 |
| security | CSP/CORS/sandbox 边界 | 5 |
| storage | IndexedDB/Cache API 边界 | 5 |
| net | URL/请求/Cookie 边界 | 5 |
| host-runtime | 窗口/事件边界 | 5 |
| render-foundation | 渲染/图像缓存/字体边界 | 5 |
| protocol | IPC 序列化边界 | 5 |
Total: 4563 → 4619 (+56 tests)

### -34. CSS background-size/attachment + 6 crate 边界测试 + 54 个测试（前轮，4563 测试）

新增 2 个 CSS 背景属性、6 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **background-size**：auto/cover/contain/length/percent；**background-attachment**：scroll/fixed/local | 8 |
| style-system | **2 属性管线集成**（全部非继承） | 16 |
| dom | 属性/遍历/序列化边界 | 5 |
| css-parser | 选择器/媒体查询边界 | 5 |
| style-system | 级联/简写/继承边界 | 5 |
| engine | 渲染/合成/dirty 边界 | 5 |
| layout-engine | 布局/转换器边界 | 5 |
| wasm-sandbox | 内存/函数/模块边界 | 5 |
Total: 4509 → 4563 (+54 tests)

### -33. CSS background-image/position/repeat + 5 集成测试 + 6 crate 边界测试 + 77 个测试（前轮，4509 测试）

新增 3 个 CSS 背景属性、5 个跨 crate 集成测试、6 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **background-image**：none/url；**background-position**：center/left/right/top/bottom/length/percent/two-value；**background-repeat**：repeat/repeat-x/repeat-y/no-repeat/space/round | 20 |
| style-system | **3 属性管线集成**（全部非继承） | 23 |
| integration | **text-wrap/hyphens/line-clamp/background-image/background-repeat 管线集成** | 5 |
| security | CSP/CORS/sandbox 边界 | 5 |
| storage | IndexedDB/Cache API 边界 | 5 |
| net | URL/请求/Cookie 边界 | 5 |
| canvas | 路径/变换/像素操作边界 | 5 |
| render-foundation | 渲染/图像缓存/字体边界 | 5 |
| webview | 状态/渲染/导航边界 | 5 |
Total: 4432 → 4509 (+77 tests)

### -32. CSS text-wrap/hyphens/line-clamp + 6 crate 边界测试 + 71 个测试（前轮，4432 测试）

新增 3 个 CSS 文本属性、6 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **text-wrap**：wrap/nowrap/balance/pretty/stable；**hyphens**：none/manual/auto；**line-clamp**：none/count | 20 |
| style-system | **3 属性管线集成**（text-wrap/hyphens 继承，line-clamp 不继承） | 21 |
| canvas | 路径/变换/像素操作边界 | 5 |
| wasm-sandbox | 内存/函数/模块边界 | 5 |
| dom | 属性/遍历/序列化边界 | 5 |
| engine | 渲染/合成/dirty 边界 | 5 |
| layout-engine | 布局/转换器边界 | 5 |
| host-runtime | 窗口/事件边界 | 5 |
Total: 4361 → 4432 (+71 tests)

### -31. 6 集成测试 + columns 简写 + 6 crate 边界测试 + 36 个测试（前轮，4361 测试）

新增 6 个跨 crate 集成测试（filter/mix-blend-mode/scrollbar-width/contain 多值/appearance/columns 简写管线）、columns 简写 apply_property_value 支持、6 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| integration | **filter/mix-blend-mode/scrollbar-width/contain 多值/appearance/columns 简写**管线 | 6 |
| style-system | **columns 简写 apply_property_value** + 边界测试 | 5 |
| security | CSP/CORS/sandbox 边界 | 5 |
| storage | IndexedDB/Cache API 边界 | 5 |
| net | URL/请求/Cookie 边界 | 5 |
| css-parser | 解析边界 | 5 |
| protocol | IPC 序列化边界 | 5 |
Total: 4325 → 4361 (+36 tests)

### -30. CSS mix-blend-mode/scrollbar-width/scrollbar-gutter + 6 crate 边界测试 + 78 个测试（前轮，4325 测试）

新增 3 个 CSS 属性、6 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **mix-blend-mode**：17 种混合模式；**scrollbar-width**：auto/thin/none；**scrollbar-gutter**：auto/stable/stable-both-edges | 15 |
| style-system | **3 属性管线集成**（全部非继承） | 33 |
| dom | 属性/遍历/序列化边界 | 5 |
| engine | 渲染/合成/dirty 边界 | 5 |
| layout-engine | 布局/转换器边界 | 5 |
| host-runtime | 窗口/事件边界 | 5 |
| render-foundation | 渲染/图像缓存/字体边界 | 5 |
| webview | 状态/渲染/导航边界 | 5 |
Total: 4247 → 4325 (+78 tests)

### -29. CSS appearance/accent-color/caret-color + 6 crate 边界测试 + 71 个测试（前轮，4247 测试）

新增 3 个 CSS 属性、6 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **appearance**：15 值枚举；**accent-color/caret-color**：auto + ColorValue | 15 |
| style-system | **appearance/accent-color/caret-color 管线**（accent-color/caret-color 继承） | 24 |
| security | CSP/CORS/sandbox 边界 | 5 |
| storage | IndexedDB/Cache API 边界 | 5 |
| net | URL/请求/Cookie 边界 | 5 |
| canvas | 路径/变换/像素操作边界 | 5 |
| wasm-sandbox | 内存/函数/模块边界 | 5 |
| protocol | IPC 序列化边界 | 5 |
Total: 4176 → 4247 (+71 tests)

### -28. CSS contain + column-rule-color + 5 集成测试 + 5 crate 边界测试 + 43 个测试（前轮，4176 测试）

新增 CSS contain 属性（含多值位标记）、column-rule-color、5 个跨 crate 集成测试、5 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **contain**：none/strict/content/size/layout/style/paint + 多值位标记组合 | 5 |
| style-system | **contain + column-rule-color 管线集成** | 11 |
| integration | **break-inside/column-count/object-fit/direction 继承链/contain 管线集成** | 5 |
| dom | 属性/遍历/序列化边界 | 5 |
| css-parser | 选择器/媒体查询边界 | 5 |
| style-system | 级联/简写/继承边界 | 5 |
| layout-engine | Grid/Flex/转换器边界 | 5 |
| engine | 渲染/合成/dirty 边界 | 5 |
Total: 4133 → 4176 (+43 tests)

### -27. CSS break-inside/before/after + column-rule + columns/column-rule 简写 + 5 crate 边界测试 + 43 个测试（前轮，4133 测试）

新增 5 个 CSS 属性、2 个简写展开、5 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **break-inside**：auto/avoid/avoid-page/avoid-column；**break-before/after**：auto/avoid/column/page 等；**column-rule-width**：medium/thin/thick/length；**column-rule-style**：10 种边框样式 | 12 |
| style-system | **5 个属性管线集成** + **columns/column-rule 简写展开** | 18 |
| net | URL/请求/Cookie 边界 | 5 |
| protocol | IPC 序列化/大消息边界 | 5 |
| canvas | 路径/变换/像素操作边界 | 5 |
| host-runtime | 窗口/事件边界 | 5 |
| wasm-sandbox | 内存/函数/模块边界 | 5 |
Total: 4090 → 4133 (+43 tests)

### -26. CSS column-count/column-width/object-fit/filter + 6 crate 边界测试 + 31 个测试（前轮，4090 测试）

新增 4 个 CSS 属性、6 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **column-count**：auto/number；**column-width**：auto/length；**object-fit**：fill/contain/cover/none/scale-down；**filter**：none/blur/brightness/contrast 等 11 种函数 | 6 |
| style-system | **column-count/column-width/object-fit/filter 管线集成** | 17 |
| dom | 属性操作/节点遍历边界 | 6 |
| engine | 渲染/合成边界 | 5 |
| layout-engine | 布局模式/网格边界 | 5 |
| security | CSP/CORS/sandbox 边界 | 5 |
| storage | IndexedDB/Cache API 边界 | 5 |
Total: 4059 → 4090 (+31 tests)

### -25. CSS direction/unicode-bidi/tab-size + background/font/text-decoration 简写 + 集成测试 + 56 个测试（前轮，4059 测试）

新增 3 个 CSS 属性、3 个简写展开、6 个跨 crate 集成测试、3 个 crate 边界测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **direction**：ltr/rtl；**unicode-bidi**：6 值枚举；**tab-size**：Number/Length | 13 |
| style-system | **direction/unicode-bidi/tab-size 管线** + **background/font/text-decoration 简写展开** | 25 |
| integration | **overflow-wrap/text-align-last/direction/tab-size 管线集成** + **storage-protocol IPC roundtrip** | 6 |
| host-runtime | 事件/IME/窗口边界 | 5 |
| render-foundation | 渲染/图像缓存/字体边界 | 5 |
| webview | 状态转换/渲染/导航边界 | 5 |
Total: 4003 → 4059 (+56 tests)

### -24. CSS overflow-wrap/text-align-last/font-variant-numeric + 8 crate 边界测试 + 115 个测试（前轮，4003 测试）

修复上一轮遗留的编译错误和缺失属性处理，新增 3 个 CSS 属性，8 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **overflow-wrap**：normal/break-word/anywhere；**text-align-last**：auto/start/end/left/right/center/justify；**font-variant-numeric**：normal/ordinal/slashed-zero 等 9 值 | 15 |
| style-system | **overflow-wrap/text-align-last/font-variant-numeric 管线集成** + **pointer-events inherit_property 补全** + **overscroll-behavior/touch-action/pointer-events/user-select/will-change apply_initial_value 补全** | 13 |
| dom | 属性覆盖/文本节点/子节点操作边界 | 9 |
| canvas | 路径/变换/像素操作边界 | 25 |
| security | CSP/CORS/sandbox 边界 | 10 |
| storage | IndexedDB/Cache API 边界 | 5 |
| net | URL/请求/cookie 边界 | 5 |
| engine | 渲染管线/合成边界 | 5 |
| layout-engine | Grid/Flex 布局边界 | 5 |
| protocol | IPC 序列化/大消息边界 | 7 |
| wasm-sandbox | 内存/函数调用边界 | 3 |
| webview | 状态转换/渲染边界 | 4 |
| 修复 | 移除重复测试名（dom/canvas/wasm-sandbox）、f64→f32 类型修复（webview）、apply_initial_value 补全 5 属性 | — |
Total: 3888 → 4003 (+115 tests)

### -23. CSS list-style/page-break/box-decoration-break/image-rendering/isolation + 集成测试 + 39 个测试（前轮，3888 测试）

新增 CSS list-style 简写、page-break、box-decoration-break、image-rendering、isolation 属性，跨 crate 集成测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **page-break-before/after/inside**、**box-decoration-break**、**image-rendering**、**isolation** 属性解析 | 8 |
| style-system | **5 个新属性管线** + **list-style 简写展开** | 22 |
| integration | **CSS transform pipeline**、**media query + style**、**canvas gradient**、**grid layout pipeline**、**counter cascade** | 9 |
Total: 3849 → 3888 (+39 tests)

### -22. CSS counter/content/quotes + 10 crate 边界测试 + 80 个测试（前轮，3849 测试）

新增 CSS counter-reset/counter-increment、content、quotes 属性，10 个 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **counter-reset/counter-increment**：CounterActionValue + 解析；**content**：ContentValue 枚举；**quotes**：QuotesValue 枚举 | 23 |
| style-system | **counter-reset/counter-increment/content/quotes** 管线集成 | 14 |
| dom | 属性覆盖/文本节点/子节点操作/文档工厂方法 | 7 |
| security | CSP default-src/frame-src/CORS 凭证/混合内容/sandbox popups | 7 |
| render-foundation | rect 交集/并集/alpha 混合/面积/damage tracker | 6 |
| storage | update/会话隔离/count/cursor 越界/keys/事务提交 | 6 |
| layout-engine | grid dense/span/min-max 约束 | 5 |
| engine | visibility hidden/复合子层/dirty flag/复杂页面 | 5 |
| net | URL hash/查询参数/请求链/状态文本 | 5 |
| host-runtime | 多触点/按钮坐标/按键码 | 3 |
Total: 3769 → 3849 (+80 tests)

### -21. CSS 属性扩展 + grid 简写 + CanvasStyle + 75 个测试（前轮，3769 测试）

新增 CSS 属性（text-overflow/text-indent/table-layout/caption-side/border-collapse/resize）、grid 简写展开、CanvasStyle 枚举支持渐变填充，以及多 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **text-overflow/text-indent/table-layout/caption-side/border-collapse/resize** 属性解析 | 22 |
| style-system | **6 个新属性管线** + **grid place-items/place-content/place-self/grid-template 简写** | 21 |
| canvas | **CanvasStyle 枚举**：Color/LinearGradient/RadialGradient/ConicGradient/Pattern，渐变色采样 | 15 |
| layout-engine | **grid-template 简写 + grid-template-areas 矩形验证 + named grid lines** | 5 |
| dom | 属性操作边界条件 | 2 |
| 其他 | css-parser wpt-runner/script-sandbox 增量 | 10 |

### -20. CSS 3D transforms + transform-origin + perspective + media query 增强 + Canvas API + 83 个测试（前轮，3694 测试）

新增 CSS 3D 变换函数、transform-origin/perspective 属性、媒体查询增强（only/逗号 OR/prefers-*/pointer/resolution）、Canvas API 方法，以及多 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **3D transform 函数**：rotateX/Y/Z、translate3d、scale3d、rotate3d、perspective()、matrix()；**媒体查询增强**：only 关键字、逗号分隔 OR 查询、prefers-color-scheme、prefers-reduced-motion、pointer、resolution 特性 | 39 |
| style-system | **transform-origin/perspective/perspective-origin/transform-style/backface-visibility** 属性管线集成 | 7 |
| canvas | **createImageData**：空白图像数据；**getTransform**：当前变换矩阵；**transform()**：乘法变换；**miterLimit**：斜接限制；**textDirection**：文本方向 | 25 |
| layout-engine | **grid 3x3 区域/auto-fill minmax/命名区域解析/百分比 gap/负 z-index** | 7 |
| engine | **perspective 偏移/transform-origin/负坐标/深层 z-index** | 5 |

### -19. CSS text-shadow/box-shadow + DOM Range 增强 + Path2D addPath + 36 个测试（前轮，3611 测试）

新增 CSS text-shadow 和 box-shadow 解析、DOM Range cloneRange/compareBoundaryPoints、
Canvas Path2D addPath 方法，以及多 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **text-shadow**：TextShadowValue 结构体 + 解析；**box-shadow**：BoxShadowValue 结构体 + 解析（含 inset） | 7 |
| dom | **Range cloneRange**：克隆范围；**Range compareBoundaryPoints**：边界点比较 | 5 |
| canvas | **Path2D addPath**：合并路径 + closePath/isPointInPath 验证 | 4 |
| layout-engine | **display:none 级联/百分比高度/flex 居中/grid 显式列/border-box** | 5 |
| style-system | **级联源序/border 简写/var 链式引用/inherit 显式/revert** | 5 |
| net + storage | **URL 查询参数/cookie secure/websocket 生命周期/URL 相对路径** + **IDB add 拒绝/put 覆盖/key 迭代/length/keys** | 10 |

### -18. CSS text-decoration/text-transform/letter-spacing + DOM NodeIterator + 39 个测试（前轮，3575 测试）

新增 CSS 文本装饰属性、文本变换属性、字距/词距属性，DOM NodeIterator，以及多 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **text-decoration-line**：5 种装饰线；**text-transform**：4 种变换；**letter-spacing/word-spacing**：parse_spacing 支持normal+长度 | 10 |
| style-system | **text-decoration-line 管线**（不继承）；**text-transform 管线**（继承）；**letter-spacing 管线**（继承） | 9 |
| dom | **NodeIterator**：next_node/previous_node 深度优先遍历 | 5 |
| canvas | **putImageData+getImageData/createConicGradient/font/textAlign/textBaseline** | 5 |
| engine | **outline 无宽度/outline offset/visibility hidden/opacity zero/border none** | 5 |
| layout-engine | **inline-block+text/sticky/wrap-reverse/grid gap/absolute top+left** | 5 |

### -17. WebSocket 桩 + CSS writing-mode + 跨 crate 集成测试（前轮，3536 测试）

新增 WebSocket 基础桩实现、CSS writing-mode 属性、跨 crate 集成测试，以及多 crate 边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| net | **WebSocket 桩**：WebSocketState 状态机、send/receive 消息队列、connect/close 生命周期 | 5 |
| css-parser | **writing-mode 属性**：WritingModeValue 枚举、parse_writing_mode 函数 | 4 |
| style-system | **writing-mode 管线集成**：ComputedStyle 字段、apply/initial、不继承 | 4 |
| integration | **跨 crate 测试**：CSS→render 颜色、URL→导航、DOM→style、storage 隔离、WASM host、canvas+webview | 6 |
| dom | **get_elements_by_class_name/set_id/create_comment/insert_before/inner_text** | 5 |
| security | **report-only/preflight 自定义方法/mixed content/不同端口/sandbox allow-scripts** | 5 |

### -16. CSS word-break + DOM TreeWalker + canvas/webview/engine 边界测试（前轮，3507 测试）

新增 CSS word-break 属性、DOM TreeWalker 深度优先遍历，以及 6 个 crate 的 29 个边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **word-break 属性**：WordBreakValue 枚举、parse_word_break 函数 | 4 |
| style-system | **word-break 管线集成**：ComputedStyle 字段、apply/inherit/initial 完整集成 | 4 |
| dom | **TreeWalker**：深度优先遍历 DOM 子树，支持 next_node/first_child/next_sibling | 5 |
| canvas | **gradient 多 stop/radial/fillRule/lineDash/measure_text/shadow** | 6 |
| webview | **默认配置/data URI/多次导航/CSS 注入/状态转换** | 5 |
| engine | **管线 basic/dirty tracking/z-index/border-radius/命名颜色** | 5 |

### -15. CSS min-content/max-content 关键字（前轮，3478 测试）

新增 CSS `min-content` 和 `max-content` 尺寸关键字解析：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **min-content/max-content 关键字**：LengthValue::MinContent/MaxContent 变体、大小写不敏感解析 | 1 |
| layout-engine | converter 全 4 个函数支持 MinContent/MaxContent → Auto 映射 | 0 |
| style-system | computed resolve_length 支持 MinContent/MaxContent → 0.0 | 0 |

### -14. 6 crate 边界条件测试覆盖率提升第二轮（前轮，3477 测试）

在 6 个 crate 添加 33 个边界条件测试，覆盖级联、布局、渲染、事件、IPC、WASM 等：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| style-system | **级联优先级边界**：ID vs class specificity、!important 覆盖、颜色继承、display 默认值、margin 简写展开、var() 回退、@media 无视口不应用 | 7 |
| layout-engine | **布局模式边界**：负 margin、嵌套 flex、absolute-in-relative、overflow hidden、grid auto-placement、零高度块 | 6 |
| render-foundation | **渲染管线边界**：damage tracker 单矩形/重叠合并、颜色 RGBA 钳位、surface resize 保留内容、image_cache 零 max_entries | 5 |
| host-runtime | **事件系统边界**：mouse 坐标、keyboard key_code、resize 尺寸、touch 单点、IME composition | 5 |
| protocol | **IPC 边界**：空载荷、Unicode 载荷、顺序保持、确定性编码、10KB 大载荷 | 5 |
| wasm-sandbox | **WASM 边界**：memory 读写、多导出函数、fuel 消耗、global 读取、无效模块错误处理 | 5 |

### -13. CSS 148 命名颜色 + fit-content() + conic-gradient 修复 + 35 个新测试（前轮，3444 测试）

扩展 CSS 命名颜色至完整 148 种标准颜色、新增 fit-content() CSS 函数解析、
修复 conic-gradient(at position) 解析 bug，以及 6 个 crate 的 33 个边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **148 种 CSS 标准命名颜色**：从 17 种扩展至完整标准集（含 transparent/currentColor、grey/gray 别名）；**fit-content() 函数**：LengthValue::FitContent 变体、解析 px/em/%/0 参数；**conic-gradient(at position) 修复**：支持无 from 前缀的 at 位置、位置在逗号处截断 | 2 |
| dom | **has_attribute/remove_attribute**：属性存在检查和删除；**split_text**：文本节点分割；**class_list replace**：类名替换；**contains**：节点包含检查 | 6 |
| canvas | **resize/clear_entire/stroke_zero_width**：Canvas 生命周期边界；**negative_translate/restore_without_save/globalAlpha_clamp**：变换和状态边界 | 6 |
| engine | **paint 空文档/composite 单盒/recompute_style**：管线边界；**named_color_crimson/hsla_pure_red**：颜色转换验证 | 5 |
| net | **URL fragment/空路径/导航历史检查/cookie httpOnly/响应状态文本**：网络边界 | 5 |
| security | **CSP 同源脚本/内联样式/data URI/简单请求 GET/sandbox same-origin**：安全策略边界 | 5 |
| storage | **delete_object_store/update_existing/clear/空 store cursor/cache has**：存储边界 | 6 |

同时更新 layout-engine converter 和 style-system computed 中的 LengthValue 穷尽匹配，
以及 engine paint 中的 ColorValue::Named 兼容处理。

### -12. br 元素 + Path2D closePath/isPointInPath + scroll-snap 集成 + 字符宽度优化 + 43 个新测试（前轮，3409 测试）

实现 <br> 元素行内换行、Path2D 闭合路径和点击检测、scroll-snap 管线集成、
逐字符宽度估算替代固定 0.6 系数，以及 12 个 crate 的 43 个边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| layout-engine | **br 元素**：InlineItem::Br 强制换行，与文本和 inline-block 协同；**逐字符宽度估算**：CJK 全宽、ASCII 0.55x、空格 0.25x、标点 0.4x、数字 0.5x | 9 |
| canvas | **Path2D closePath()**：子路径跟踪 + 闭合线段；**is_point_in_path()**：射线法点在多边形内检测 | 5 |
| style-system | **scroll-snap 管线集成**：scroll-snap-type/align/stop 计算值验证 | 5 |
| dom | **序列化边界**：void 元素自闭合、深层嵌套、script/style 内容保留、无值属性 | 4 |
| css-parser | **选择器边界**：media range syntax、:has() 组合器、:not() 多选择器、:is()/:where() | 4 |
| security | **CSP form-action**、mixed content 升级、CORS header 验证 | 3 |
| net | **请求方法链式**、响应状态码分类 | 2 |
| storage | **对象仓库重命名**、多条目索引 | 2 |
| webview | **Builder 默认值**、data URI、CSS 注入渲染 | 3 |
| wasm-sandbox | **内存初始页**、函数参数传递 | 2 |
| host-runtime | **resize 状态保持** | 1 |
| render-foundation | **damage tracker 全量重绘**、填充颜色钳位 | 2 |

### -11. CSS hwb color + inline-block 布局 + MutationObserver 集成 + container queries + 36 个新测试（前轮，3366 测试）

实现 CSS hwb() 颜色解析、inline-block 行内布局、MutationObserver 完整集成测试、
container query 评估改进，以及跨 crate 集成测试和错误恢复测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| css-parser | **hwb() 颜色解析**：HWB→RGBA 转换、whiteness/blackness 钳位、alpha 支持；错误恢复：malformed selector/unclosed bracket/empty value | 8 |
| layout-engine | **inline-block 布局**：InlineItem 枚举、InlineBlockBox 结构体、原子性行内级盒子参与换行 | 4 |
| dom | **MutationObserver 集成**：child list 观察、属性变更、subtree 模式、disconnect、多 observer、take_records、clear_observers；错误恢复：malformed HTML、无效元素名 | 9 |
| style-system | **container query 评估改进**：min-width/max-width、range syntax、无 context 不应用 | 5 |
| integration | **跨 crate 集成测试**：Shadow DOM→layout、container query style、canvas ellipse render、grid-area named placement | 4 |
| protocol | **全字段类型消息**、**向后兼容性**测试 | 2 |
| render-foundation | **累积 dirty area**、**hex 颜色格式** (#RGB/#RRGGBB/#RRGGBBAA) | 2 |
| engine | **多层 composite 排序**、**@media 渲染** | 2 |

### -10. Shadow DOM 布局扁平化 + OffscreenCanvas + @layer 级联验证 + 44 个新测试（前轮，3330 测试）

实现 Shadow DOM slot 分配到布局树的扁平化连接、OffscreenCanvas API 桩、
@layer 级联排序验证，同时在 12 个 crate 添加 44 个边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| layout-engine | **Shadow DOM 布局扁平化**：build_subtree 中 shadow tree 遍历替代 light children、slot 元素替换为分配节点、fallback 内容、未分配隐藏 | 4 |
| canvas | **OffscreenCanvas 桩**：new(width,height)、get_context()、transfer_to_image_bitmap() | 4 |
| style-system | **@layer 级联验证**：层级排序、分层 vs 非分层、!important 覆盖、specificity 顺序、importance 顺序、origin 顺序 | 6 |
| dom | **compare_document_position 跨文档**、has_attributes、class_list toggle、fragment 子节点计数 | 6 |
| render-foundation | **混合 ASCII/CJK 整形**、显式换行符、颜色 lerp、image cache 双重 insert | 4 |
| engine | **head/body 渲染**、table 结构渲染 | 2 |
| net | **URL port+path**、query params、secure cookie、初始状态 | 4 |
| host-runtime | **touch 事件**、window 最小化/最大化 | 2 |
| security | **preflight 自定义方法**、多 CSP 指令组合、sandbox allow-forms、origin tuple 相等 | 4 |
| wasm-sandbox | **global export 读取**、table export 查询、同模块多实例 | 3 |
| storage | **compound key**、unique 约束、cache put 覆盖、remove 不存在键 | 5 |
| webview | **空 HTML 加载**、状态转换、多次 CSS 注入 | 3 |

### -9. Shadow DOM slot 解析 + canvas image_smoothing/stroke join-cap + 58 个新测试（前轮，3286 测试）

实现 Shadow DOM slot 分配解析、canvas image_smoothing_enabled 和 stroke join/cap 渲染集成，
同时在 8 个 crate 添加 58 个边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| dom | **Shadow DOM slot 分配**：resolve_slots() 和 get_assigned_nodes()，支持命名 slot、默认 slot、fallback 内容、多元素分配；**get_elements_by_tag_name_ns()** 命名空间感知查找，通配符支持 | 10 |
| canvas | **image_smoothing_enabled** 属性 + save/restore；**line_join/line_cap stroke 渲染集成**：Miter/Round/Bevel 接合、Butt/Round/Square 端点影响 stroke 顶点生成 | 9 |
| layout-engine | **tree building 边界**：comment 节点跳过、PI 跳过、20 层深层嵌套、混合 display:none 子元素、grid 容器+项 | 5 |
| engine | **pipeline/paint 边界**：inline style 渲染、script tag 不崩溃、渲染顺序稳定、重叠背景 z-order、部分边框绘制 | 5 |
| css-parser | **parser 边界**：hwb color、gradient 多类型 stop、3D transform 函数、3 层嵌套 var fallback、复杂 @supports 条件 | 5 |
| render-foundation | **shaper/geometry/primitive 边界**：空字符串整形、单字符、clear+remark、opacity 零图元 | 4 |

### -8. cursor/opacity 管线集成 + 7 crate 测试覆盖：45 个新测试（前轮，3228 测试）

完成 style-system cursor/opacity 属性管线集成，同时在 7 个 crate 添加 45 个边界条件测试：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| style-system | **cursor/opacity 管线集成**：parse_cursor/parse_opacity 接入 apply_property_value；cursor 继承、opacity 不继承；**transition/animation none 验证、box-sizing 效果、多重 transform** | 10 |
| canvas | **composite operation 像素级验证**：source-over、destination-over、copy、xor、source-atop 5 种合成模式实际像素混合行为测试 | 5 |
| protocol | **IPC 压力测试**：1MB+ 大消息、消息排序、空字段、unicode 载荷、确定性编码、并发消息、无导出查询 | 7 |
| host-runtime | **事件边界**：修饰键组合、按键重复标志、所有鼠标按钮、enter/leave 坐标、零尺寸 resize、IME 空组合 | 6 |
| net | **网络边界**：第三方 cookie（SameSite=None+Secure）、会话 cookie、前进超出可用、新导航清空前进历史、零超时 | 5 |
| security | **CSP/CORS/mixed-content**：img-src 限制、nonce 不匹配、default-src 回退、HTTPS 无需升级、worker 类型、max-age 零、methods 通配符 | 7 |
| wasm-sandbox | **WASM 边界**：memory grow+read、错误签名调用、无导出模块、fuel 消耗追踪、多参数 host、递归限制 | 6 |

### -7. 功能完善 + 测试覆盖：canvas line_join/line_cap、grid-area 简写、DOM import_node、49 个新测试（前轮，3183 测试）

并行实现 6 个功能改进和 16 个边界条件测试，覆盖 10 个 crate：

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| canvas | **LineJoin/LineCap 属性**：Miter/Round/Bevel、Butt/Round/Square，getter/setter + save/restore；**is_point_in_stroke()**：距离线段检测 | 7 |
| css-parser | **parse_grid_area()**：命名区域、斜杠分隔行号、auto 展开 | 5 |
| style-system | **grid-area/grid-column/grid-row 简写展开**：扩展到 grid_row_start/end、grid_column_start/end | 4 |
| dom | **import_node()**：浅层和深层克隆，导入节点无父节点 | 4 |
| layout-engine | **overflow Auto→Scroll/Clip→Clip 验证**、**z_index 输出验证**、**content area 超大 border 钳位**、**深层嵌套 fixed 位置** | 5 |
| engine | **overflow:hidden 部分交叉裁剪**、**hsla 零饱和度/亮度**、**paint_text 零宽度**、**嵌套 overflow:hidden** | 4 |
| render-foundation | **多次 resize**、**RGBA clamp 极端值**、**image_cache 零 max_entries** | 3 |
| webview | **多次导航**、**加载后注入 CSS**、**execute_script 占位**、**自定义视口 Builder** | 4 |

### -6. 全 crate 测试覆盖率提升：119 个新测试 + visibility:collapse 修复（前轮，3094 测试）

系统性分析 14 个 crate 的测试缺口，聚焦最低密度的 layout-engine 和 engine crate，
同时覆盖 dom、css-parser、security、storage、net、canvas 共 10 个 crate。
发现并修复 `visibility:collapse` 在 paint 中未按 CSS 规范隐藏元素的 bug。

| 模块 | 新增内容 | 新增测试 |
|------|----------|----------|
| layout-engine | **converter 全变体覆盖**：InlineFlex/InlineGrid display、Em/Rem/Vw/Vh/Vmin/Vmax/Ch 单位、Calc fallback、max-length infinity、WrapReverse、FlexBasis Content、align_content 9 变体、justify_content 6 变体、align_self 6 变体、float/clear 全变体、overflow Auto/Clip、tokenize 嵌套括号、malformed minmax | 24 |
| layout-engine | **inline 格式化边界**：vertical_align Sub/Super y 偏移、TextTop/TextBottom 与 Top/Bottom 等价、resolve_font_metrics Em/Rem 回退、非 Px font-size 回退、break_into_lines 重置状态、零 font_size/line_height | 9 |
| engine | **clip_fills/clip_glyphs 直接测试**：partial overlap、outside each side、start index、empty slice、exact match；**hsla hue≥300 fallback**；**CurrentColor**；**length_to_f32 non-Px**；**named_color 扩展** | 15 |
| engine | **composite 父子层路由**：promoted parent + non-promoted child、promoted parent + promoted child、single box bounding_box、grandchildren encompass；**dirty 150% merge boundary**、50 rect stress、negative size、100 non-overlapping、full_redraw merge；**pipeline malformed CSS**、50% threshold、recompute without render、mixed render ops | 14 |
| engine | **visibility 修复 + 测试**：visibility:collapse 按 CSS 规范隐藏元素（修复 bug）；child visible 覆盖 parent hidden；doc=Some+node_id=None fallback；paint_in_rect hidden skip | 4 |
| dom | **Range API**：select_node、select_node invalid (Document)、text_content partial、to_debug_string、set_start/set_end invalid offset、clone_contents partial text、insert_node mid text | 7 |
| css-parser | **5 个零覆盖 parse 函数**：parse_vertical_align (8 关键字+长度)、parse_list_style_type (15 关键字)、parse_list_style_position、parse_float、parse_clear；**eval_calc_with_context viewport** | 6 |
| security | **CORS 安全边界**：credentials+wildcard rejected、is_simple_request (GET/POST/PUT/自定义 header/content-type)、generate_preflight_response 字段验证；**sandbox 导航/弹窗**：effective_origin opaque/preserve、导航有/无 activation、弹窗允许/阻止 | 13 |
| storage | **IndexedDB cursor + transaction**：cursor advance/continue_to/iteration、key_cursor advance/continue、transaction commit/abort、put 覆盖/add 拒绝重复、count_with_range、index range query、cursor on index | 12 |
| net | **URL 边界**：非默认端口 origin、credentials in URL、fragment+query；**导航边界**：empty replace_current、max_entries boundary；**请求**：non-UTF8 body、header chaining | 7 |
| canvas | **clip+drawImage**：clip constrains draw_image、negative coordinates、zero dimensions、ImageData zero dims、out-of-bounds get_image_data、TextAlign/TextBaseline enum | 6 |

### -5. 6 crate 边界条件测试覆盖率提升（前轮，2977 测试）

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
| 颜色 | ✅ ~98%（含 **148 种标准命名颜色**、hwb/hsl/rgb/rgba/hex 全格式） |
| 字体 | ✅ 100% |
| 定位 | ✅ 100% |
| Overflow | ✅ 100% |
| Transforms | ✅ ~85%（2D + 3D 函数、transform-origin、perspective、transform-style、backface-visibility） |
| **Transitions** | ✅ 已实现 |
| 自定义属性 | ✅ ~90% |
| 媒体查询 | ✅ ~85%（only 关键字、逗号 OR、prefers-color-scheme、prefers-reduced-motion、pointer、resolution） |
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
| **direction/unicode-bidi** | ✅ 已实现（属性解析 + 样式管线集成） |
| **tab-size** | ✅ 已实现（属性解析 + 样式管线集成） |
| **overflow-wrap** | ✅ 已实现（属性解析 + 样式管线集成） |
| **text-align-last** | ✅ 已实现（属性解析 + 样式管线集成） |
| **font-variant-numeric** | ✅ 已实现（属性解析 + 样式管线集成） |
| **text-shadow** | ✅ 已实现（属性解析 + 样式管线集成，继承属性） |
| **box-shadow** | ✅ 已实现（属性解析 + 样式管线集成，含 inset） |
| **outline 简写** | ✅ 已实现（outline-width/style/color 简写展开） |
| **list-style-image** | ✅ 已实现（none/url，继承属性） |
| **column-gap** | ✅ 已实现（长度值，非继承） |
| **justify-items / justify-self** | ✅ 已实现（auto/normal/start/end/center/stretch/baseline，非继承） |
| **align-content** | ✅ 已实现（含 space-between/space-around/space-evenly，非继承） |
| **empty-cells** | ✅ 已实现（show/hide，继承属性） |
| **border-spacing** | ✅ 已实现（1-2 length 值，继承属性） |
| **gap 简写** | ✅ 已实现（gap → row-gap + column-gap） |

---

## 里程碑完成情况

| 里程碑 | 状态 |
|--------|------|
| M1 项目骨架 + 渲染基础设施 | ✅ |
| M2 HTML 解析 + DOM 树 | ✅ |
| M3 CSS 解析器 + 样式系统 | ✅ |
| M4 布局引擎 | ✅ |
| M5 渲染管线集成 | ✅ |
| M6 JavaScript 集成 (V8) | ✅ script-sandbox V8 引擎已集成，WebView.execute_script 可用 |
| M7 网络栈 + 导航模型 | ✅ |
| M8 多进程架构 (IPC) | ✅ (protocol crate) |
| M9 Canvas + Storage | ✅ |
| M10 WebView API | ✅ (webview + integration tests) |
| M11 浏览器应用 | 🔄 browser-shell 完成（135 测试）、zero-browser 应用入口完成、DOM Bridge 完成（40 测试）、WebView 事件集成（168 测试） |

---

## 下一步优先级

1. **浏览器应用 UI 完善（M11）**（高优先级）— 下载管理器、页面查找、缩放、右键菜单、设置页面
2. **DOM API 绑定**（高优先级）— JS → DOM bindings（document.getElementById、querySelector 等）
3. **事件系统集成**（高优先级）— JS 事件系统（addEventListener、事件冒泡/捕获）
4. **更多 Canvas API**（中优先级）— OffscreenCanvas、line_join/line_cap、更多合成模式测试
5. **Fetch API**（中优先级）— JS 中发起网络请求

---

## 归档记录

- **M1** ✅ → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
- **M2** ✅ → [archive/m2-dom.md](archive/m2-dom.md)
- **M3** ✅ → [archive/m3-css-parser-style-system.md](archive/m3-css-parser-style-system.md)
- **M4** ✅ → [archive/m4-layout-engine.md](archive/m4-layout-engine.md)
- **M5** ✅ → [archive/m5-rendering-pipeline.md](archive/m5-rendering-pipeline.md)
- **M7** ✅ → [archive/m7-network-security.md](archive/m7-network-security.md)
