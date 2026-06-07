# ZeroWeb 深度代码审查报告

> **摘要**
>
> **审查范围**：ZeroWeb 项目全部 18 个 crate 的核心源码（251k 行 Rust），聚焦实现缺陷、性能问题、安全漏洞、可靠性隐患。
>
> **关键发现**：共发现 **148 个问题**（**高 21 / 中 58 / 低 69**），覆盖安全、渲染、布局、样式、解析、DOM 全栈。
>
> **最高优先级**：CSP 前缀匹配绕过（S-001）— 攻击者可注册 `allowed-domain.evil.com` 绕过 CSP 保护加载恶意脚本；CSS inline style 特异性过低（ST-001）— ID 选择器可覆盖 inline style，违反 CSS 规范。
>
> **验证状态**：未经验证（建议在修复前执行 Phase 5 二次验证）

## 审查上下文

| 字段 | 内容 |
|------|------|
| **审查对象** | ZeroWeb 全项目（18 crate，251k 行 Rust） |
| **审查维度** | 实现缺陷、性能问题、安全漏洞、可靠性、API 契约 |
| **代码版本** | main 分支，commit f5eb85b |
| **审查方法** | 9 个并行审查 agent（engine×3 + security + net + render + layout + dom + style + css-parser） |

---

## 问题清单

### 🔴 高优先级（Critical）— 21 个

#### 安全漏洞（5）

| ID | 位置 | 问题 |
|----|------|------|
| S-001 | `security/csp.rs:171-179` | CSP `starts_with` 前缀匹配绕过 — `cdn.example.com.evil.com` 可绕过 CSP |
| S-002 | `net/cookie.rs:408-430` | Cookie domain=None 匹配所有 host（supercookie） |
| S-003 | `net/cookie.rs:221-330` | Cookie Domain 属性未验证与请求 host 关系 |
| S-004 | `net/client.rs:69-164` | 无 SSRF 防护，请求可访问内部网络 |
| S-005 | `net/client.rs:80-88` | 跨域重定向时敏感 header 泄露 |

#### 渲染/布局缺陷（9）

| ID | 位置 | 问题 |
|----|------|------|
| E-001 | `engine/hit_test.rs:29-31` | hit_test 坐标变换缺少 content_x/content_y 偏移，有 padding 的元素子节点点击不准 |
| E-002 | `engine/paint/painter/mod.rs:72-156` | `paint_node_in_rect` 缺少 opacity/transform/filter/border-image/counter 等 12 个渲染步骤 |
| E-003 | `engine/pipeline.rs:146-154` | `render_html_animated` 每帧重建 DOM 导致 NodeId 不匹配，transition 检测为死代码 |
| R-003 | `render-foundation/surface.rs:51` | `FrameBuffer::new` 中 `width*height*4` u32 整数溢出风险 |
| L-002 | `layout-engine/engine.rs:214-218` | `compute_incremental` 跳过 `adjust_float_positions` 和 `remeasure_text_with_float_exclusions` |
| L-003 | `layout-engine/table.rs:167-176` | Table row-group 内行的 child_index 是 row-group 局部索引，非 table 全局索引 |
| L-013 | `layout-engine/converter/mod.rs:74` | `align_content` 从 `justify_content` 取值（属性错误），cross-axis 对齐全部不正确 |
| D-002 | `dom/document.rs:260-304` | `remove_child` 不删除 SlotMap 节点，无限内存增长 |
| D-003 | `dom/document.rs:655-679` | `set_text_content` 泄露孤儿节点和 id_map 条目 |

#### DOM 完整性（2）

| ID | 位置 | 问题 |
|----|------|------|
| D-013 | `dom/document.rs:367-415` | `replace_child` 缺少 cycle/root-insertion 检查 |
| D-011 | `dom/range.rs:207-230` | `Range::collect_top_level_nodes` 跨容器时遗漏中间节点 |

#### 样式系统（4）

| ID | 位置 | 问题 |
|----|------|------|
| ST-001 | `style-system/lib.rs:236-249` | Inline style 特异性 `(1,0,0)` 等同 ID 选择器，CSS 规范要求 inline 高于所有选择器 |
| ST-002 | `style-system/inheritance.rs:43-48` | `inherit` 关键字对非继承属性静默失败（如 `margin-top: inherit` 不生效） |
| ST-003 | `style-system/computed.rs:176-304` | `outline_width`/`text_indent`/`transform_origin` 等 em/rem 值未解析为 px |
| ST-006 | `style-system/shorthand/mod.rs:709-712` | `is_time_value()` 运算符优先级错误，`"abcms"` 被错误识别为有效时间 |

#### 解析器（1）

| ID | 位置 | 问题 |
|----|------|------|
| C-001 | `css-parser/tokenizer.rs:197-203` | `byte_offset()` 每次 O(n) 扫描使 tokenization 变为 O(n²) |

---

### 🟡 中优先级（Major）— 58 个

#### 安全（13）

| ID | 位置 | 问题 |
|----|------|------|
| S-006 | `security/csp.rs:186-202` | CSP 'self' 将 `ftp://`/`file://` 等非 HTTP 协议当作 self |
| S-007 | `security/csp.rs:353-356` | `frame-ancestors 'self'` 无条件允许所有嵌入 |
| S-008 | `security/mixed_content.rs:33-38` | Mixed content 不检测 `ws://` 协议 |
| S-009 | `security/csp.rs:244-258` | CSP nonce/hash 接受无引号形式 |
| S-010 | `security/sandbox.rs:120-126` | `allow-same-origin`+`allow-scripts` 可逃逸 sandbox |
| S-011 | `security/cors.rs:87-96` | CORS origin 格式化缺少括号，优先级隐患 |
| S-012 | `net/websocket.rs:106-119` | WebSocket 无 Origin header 控制 |
| S-013 | `net/cookie.rs:283-295` | `SameSite=None` 不要求 Secure |
| S-014 | `net/cookie.rs:423-427` | Cookie 路径 `starts_with` 无边界检查 |
| S-015 | `net/cookie.rs:240` | Cookie 值未过滤 CRLF 控制字符 |
| S-016 | `net/websocket.rs:124-143` | WebSocket 无消息大小限制 |
| S-017 | `net/websocket.rs:149-183` | `receive()` 实际阻塞调用但文档声称非阻塞 |
| S-018 | `net/url_parser.rs:35-47` | URL scheme 未验证，`javascript:` 可用于网络请求 |

#### 渲染/布局（16）

| ID | 位置 | 问题 |
|----|------|------|
| E-004 | `engine/pipeline.rs:324-355` | `incremental_render` 始终全量渲染，脏区域追踪无效 |
| E-005 | `engine/pipeline.rs:361-384` | `incremental_paint` 跳过 culling 和 batch_fills |
| E-006 | `engine/pipeline.rs:292-318` | `recompute_styles` 不更新 `cached_doc` |
| E-007 | `engine/hit_test.rs:29-31` | hit_test 无 z-order / stacking context |
| E-008 | `engine/hit_test.rs:19-31` | hit_test 无 overflow clipping |
| E-009 | `engine/paint/color.rs:23-48` | HSL 转 RGB 不处理负色相或 ≥360 |
| E-010 | `engine/paint/helpers.rs:390-391` | radial gradient 定位用 `length_to_f32` 不支持百分比 |
| E-011 | `engine/paint/painter/text.rs:539-541` | `paint_text` 遇到 `CurrentColor` 静默跳过 |
| E-012 | `engine/paint/painter/mod.rs:635-655` | CSS counter 无作用域 |
| E-013 | `engine/dirty.rs:33-37` | `mark_dirty` 在 `full_redraw=true` 时仍添加矩形 |
| P-002 | `render-foundation/gpu/renderer.rs:297-328` | GPU Atlas 重建未清除纹理残留 |
| L-001 | `layout-engine/engine.rs:526-537` | Float 高度未应用到容器 |
| L-004 | `layout-engine/table.rs:260-271` | Table colspan 宽度均匀分配不正确 |
| L-005 | `layout-engine/multicol.rs:52-66` | Multicol Vw/Vh 使用硬编码视口尺寸 |
| L-006 | `layout-engine/inline/mod.rs:1037-1043` | BiDi 强制 LTR paragraph level |
| L-010 | `layout-engine/inline/mod.rs:430-518` | `InlineBlock` 项目从不生成 |

#### 性能（10）

| ID | 位置 | 问题 |
|----|------|------|
| P-001 | `render-foundation/cpu.rs:83-87` | CPU fill_rect 逐像素 set_pixel（200万次调用/全屏） |
| P-003 | `render-foundation/image_cache.rs:166-182` | ImageCache::gc O(n²) 淘汰 |
| P-004 | `render-foundation/gpu/renderer.rs:586-612` | 每帧分配 GPU Uniform/Vertex buffer |
| P-005 | `engine/pipeline.rs:191-196` | 每次 render 深克隆完整 LayoutTree |
| P-006 | `engine/pipeline.rs:147` | 每帧克隆整个 styles HashMap（又因 E-003 无效） |
| P-007 | `engine/dirty.rs:66-111` | merge_overlapping O(n³)（含 Vec::remove O(n)） |
| P-008 | `engine/paint/painter/mod.rs:293-630` | paint_node 约 40 次 HashMap::get 查找 style |
| P-009 | `engine/paint/color.rs:51-53` | `named_color_to_render` 每次分配 String |
| P-010 | `engine/paint/helpers.rs:214-228` | `clip_glyphs` 用 font_size 作近似边界 |
| P-011 | `css-parser/tokenizer.rs:172-179` | Tokenizer 同时存 String 和 Vec<char>，内存翻倍 |

#### 网络可靠性（5）

| ID | 位置 | 问题 |
|----|------|------|
| E-014 | `net/http_cache.rs:102-131` | 缓存 key 未 URL 归一化 |
| E-015 | `net/http_cache.rs:146-197` | 不遵守 Vary header |
| S-019 | `net/client.rs:45-63` | 缺少 connect_timeout |
| S-020 | `net/url_parser.rs:24-27` | URL 凭据静默转发 |
| S-021 | `net/request.rs:76-79` | Header 注入未在构建时验证 |

#### 样式/解析（7）

| ID | 位置 | 问题 |
|----|------|------|
| ST-004 | `style-system/inheritance.rs:66-78` | `revert` 实现为 `unset` 语义，不回退到上一级 origin |
| ST-005 | `style-system/lib.rs:380-389` | `!important` inline style 与 author `!important` 无优先级区分 |
| ST-007 | `style-system/computed.rs` | `line-height: 1.5` 单位倍数未解析为 px |
| ST-008 | `style-system/property/apply_advanced.rs:1190-1234` | text-shadow/box-shadow em/rem 值静默转为 0 |
| ST-009 | `style-system/computed.rs:310-324` | `calc(50%-10px)` 百分比无 containing-block 尺寸 |
| ST-010 | `style-system/property/registry.rs:306-341` | `list-style-type`/`list-style-position` 未标记为 inherited |
| C-002 | `css-parser/tokenizer.rs:810-839` | `\|`、`^`、`$` 发射为 Ident 而非 Delim |

#### DOM（7）

| ID | 位置 | 问题 |
|----|------|------|
| D-001 | `dom/node.rs:15-24` | `NodeId::is_valid()` 恒返回 true |
| D-004 | `dom/document.rs:260-304` | `remove_child` 不清理事件监听器 |
| D-006 | `dom/document.rs:509+` | 遍历方法 excessive `.children.clone()` |
| D-007 | `dom/document.rs:492-505` | `next/previous_sibling` O(n)，TreeWalker O(n²) |
| D-009 | `dom/range.rs:85-96` | Range offset 未验证 |
| D-010 | `dom/range.rs:243-267` | `compare_boundary_points` 大部分情况返回 0 |
| D-016 | `dom/document.rs:418-445` | `clone_node` + `append_child` 可破坏 id_map 唯一性 |

---

### 🟢 低优先级（Minor）— 69 个

<details>
<summary>展开查看全部低优先级问题</summary>

#### 安全（9）

| ID | 位置 | 问题 |
|----|------|------|
| S-022 | `security/hsts.rs:170-187` | HSTS scheme 检查大小写敏感 |
| S-023 | `security/context.rs:220-222` | HSTS preload 时间戳依赖溢出算术 |
| S-024 | `security/csp.rs:207-217` | CSP `*.example.com` 不匹配 `example.com` 本身 |
| S-025 | `security/site_isolation.rs:52-66` | eTLD+1 对公共后缀不正确 |
| S-026 | `security/permission.rs:93-95` | Permission store key 格式不一致 |
| S-027 | `net/cookie.rs:299-309` | Max-Age 溢出 |
| S-028 | `net/client.rs:45-63` | TLS 配置不可定制 |
| S-029 | `net/websocket.rs:188-196` | WebSocket close 不等待服务器 close frame |
| S-030 | `net/http_cache.rs:375-397` | HTTP 日期解析近似不准确 |

#### 渲染（8）

| ID | 位置 | 问题 |
|----|------|------|
| E-016 | `engine/pipeline.rs` | 无 viewport resize 方法 |
| E-017 | `engine/pipeline.rs:170` | AnimationClock::cleanup_finished 从未调用 |
| E-018 | `engine/pipeline.rs:338` | dirty_area() 返回 f32::MAX 哨兵值 |
| E-019 | `engine/paint/painter/text.rs:715-721` | paint_text 二次遍历字符 |
| E-020 | `engine/paint/painter/text.rs:755-756` | text-overflow 不考虑 word_spacing |
| E-021 | `engine/paint/painter/text.rs:993-1007` | object-fit 缺少零除保护 |
| E-022 | `engine/paint/painter/border.rs:480-488` | border 角落重叠 |
| E-023 | `engine/paint/painter/text.rs:370-392` | ol start/reversed/value 不支持 |

#### Render Foundation（4）

| ID | 位置 | 问题 |
|----|------|------|
| R-006 | `render-foundation/surface.rs:66-83` | get/set_pixel 无边界检查 |
| R-007 | `render-foundation/image_cache.rs:140-145` | get 递增引用计数语义不直观 |
| R-008 | `render-foundation/geometry.rs:147-159` | DamageTracker::add_damage O(n) |
| R-009 | `engine/paint/helpers.rs:264-289` | 零 alpha 不可见图元仍被提交 |

#### 布局（6）

| ID | 位置 | 问题 |
|----|------|------|
| L-007 | `layout-engine/engine.rs:440-461` | Float 堆叠忽略正常流位置 |
| L-008 | `layout-engine/inline/mod.rs:443,482` | 过度 trim 丢失空白 |
| L-009 | `layout-engine/converter/mod.rs:211+` | Calc 静默映射为 0 |
| L-011 | `layout-engine/converter/mod.rs:757-868` | "." null token 存入 grid area map |
| L-012 | `layout-engine/engine.rs:186-211` | 增量布局不更新 taffy style |
| L-014 | `layout-engine/multicol.rs:119-135` | column-count max/min 逻辑反转 |

#### DOM（9）

| ID | 位置 | 问题 |
|----|------|------|
| D-005 | `dom/document.rs:292` | retain() 单元素删除非最优 |
| D-008 | `dom/document.rs:1349-1370` | is_ancestor O(d) per insertion |
| D-012 | `dom/document.rs:1219-1223` | 事件调度 target phase 逻辑正确但混乱 |
| D-014 | `dom/query.rs:107-182` | 查询只支持单个属性选择器 |
| D-015 | `dom/query.rs:152-153` | 属性值含 `]` 解析错误 |
| D-017 | `dom/focus.rs:94` | focus scan_node 不必要 clone |
| D-018 | `dom/parser.rs:76-127` | DomBuilder::into_document 内存翻倍 |
| D-019 | `dom/document.rs:1806-1839` | NodeIterator done flag 边界 |
| D-020 | `dom/document.rs:1185-1188` | remove_event_listener 移除所有而非特定 |

#### 样式（5）

| ID | 位置 | 问题 |
|----|------|------|
| ST-011 | `style-system/property/registry.rs:27` | width/height initial 值为 0px 而非 auto |
| ST-012 | `style-system/lib.rs:365-394` | inline style 不处理引号内分号 |
| ST-013 | `style-system/shorthand/mod.rs:1063-1066` | `list-style: none` 不重置 list-style-image |
| ST-014 | `style-system/shorthand/mod.rs:494-594` | looks_like_color 遗漏 78 个命名颜色 |
| ST-015 | `style-system/property/inherit.rs:145-146` | apply_initial_value 每次创建完整 default struct |

#### 解析（10）

| ID | 位置 | 问题 |
|----|------|------|
| C-003 | `css-parser/tokenizer.rs:647-659` | `#` 不后跟 ident 时发 Error 而非 Delim |
| C-004 | `css-parser/selector.rs:22-61` | 选择器特异性计算正确但 AST 结构混乱 |
| C-005 | `css-parser/tokenizer.rs:252-269` | `consume_newline` 将 `\t` 当作换行 |
| C-006 | `css-parser/values/parse_transform.rs:947-969` | `split_gradient_args` depth 可变为负数 |
| C-007 | `css-parser/parser.rs:479-508` | nth 表达式解析脆弱 |
| C-008 | `css-parser/parser.rs:708-726` | declaration block 可能消费 RBrace |
| C-009 | `css-parser/values/color.rs:128-150` | rgb() 不支持空格分隔语法 |
| C-010 | `css-parser/values/color.rs:174-195` | HSL 不归一化色相和饱和度范围 |
| C-011 | `css-parser/parser.rs:155-251` | selector 双重 skip_whitespace 冗余 |
| C-012 | `css-parser/values/parse_basic.rs:5-57` | parse_length 不处理 calc()/min()/max() |

#### 网络（5）

| ID | 位置 | 问题 |
|----|------|------|
| E-024 | `net/request.rs:151-156` | Set-Cookie header() 只返回第一个 |
| R-010 | `net/navigation.rs:47-53` | Navigation O(n) eviction |
| R-011 | `net/http_cache.rs:233-244` | conditional_headers 返回过期条目 |
| R-012 | `net/http_cache.rs:327-339` | HTTP 缓存 LRU Vec O(n) |
| R-013 | `css-parser/supports_condition.rs:133-152` | `contains_top_level_keyword` O(n²) |

#### 其他（13）

| ID | 位置 | 问题 |
|----|------|------|
| R-014 | `engine/dirty.rs:33-37` | dirty rect 无数量上限 |
| R-015 | `engine/dirty.rs:84-99` | NaN 矩形通过 is_empty |
| R-016 | `engine/dirty.rs:43-49` | mark_node_dirty 只覆盖 border box |
| R-017 | `engine/paint/helpers.rs:340-343` | BorderRadiusSpec is_zero 精确浮点比较 |
| R-018 | `engine/paint/painter/text.rs:437-450` | text 用 Unicode 码点作 glyph_id |
| R-019 | `engine/paint/painter/text.rs:824-828` | background-position 单值匹配不够精确 |
| R-020 | `dom/document.rs:1393-1449` | collect_by_tag 递归 clone |
| C-013 | `css-parser/tokenizer.rs:530-583` | URL 解析拒绝反斜杠转义 |
| C-014 | `css-parser/parser.rs:1021-1083` | @import URL 未验证 |
| C-015 | `css-parser/parser.rs:1252-1324` | container condition `<`/`>` 子串搜索脆弱 |
| C-016 | `css-parser/values/parse_transform.rs:1152-1199` | box_shadow 不验证多余 token |
| C-017 | `css-parser/values/parse_layout.rs+color.rs` | 两个文件定义重复类型 |
| C-018 | `css-parser/supports_condition.rs:133-152` | `contains_top_level_keyword` 逻辑+性能 |

</details>

---

## 统计总览

| 审查区域 | 高 | 中 | 低 | 合计 |
|----------|----|----|----|----|
| **security** | 5 | 8 | 9 | 22 |
| **net** | 0 | 5 | 5 | 10 |
| **engine (pipeline+paint+hit_test+dirty)** | 3 | 14 | 13 | 30 |
| **render-foundation** | 1 | 3 | 4 | 8 |
| **layout-engine** | 3 | 7 | 6 | 16 |
| **dom** | 3 | 7 | 9 | 19 |
| **style-system** | 4 | 7 | 5 | 16 |
| **css-parser** | 2 | 7 | 10 | 19 |
| **合计** | **21** | **58** | **61** | **140** |

> 注：部分问题跨多个维度，上表按主要维度归入对应区域。

---

## 修复建议优先级

### P0 — 立即修复（安全漏洞 + 数据损坏风险）

| 问题 | 建议动作 | 预估改动 |
|------|---------|----------|
| S-001 CSP 前缀匹配 | URL origin 精确匹配 | 20 行 |
| S-002 Cookie supercookie | 实现 host-only 标志 | 30 行 |
| S-003 Cookie domain 验证 | 验证 domain-host 关系 | 25 行 |
| S-004 SSRF 防护 | 内部 IP 黑名单 | 40 行 |
| S-005 跨域 header 泄露 | 重定向时剥离 | 20 行 |
| S-007 frame-ancestors | 传入 document origin | 15 行 |
| D-002 DOM 内存泄漏 | remove_child 时删除 SlotMap | 20 行 |
| D-013 replace_child cycle | 添加 cycle/root 检查 | 10 行 |

### P1 — 本迭代（渲染正确性）

| 问题 | 建议动作 | 预估改动 |
|------|---------|----------|
| E-001 hit_test 坐标 | 添加 content_x/y 偏移 | 5 行 |
| E-002 paint_node_in_rect | 共享渲染逻辑 | 100 行 |
| L-013 align_content 属性 | 修改一行取值 | 1 行 |
| L-002 增量布局缺步 | 添加两个后处理调用 | 5 行 |
| L-003 table row index | 修复索引映射 | 30 行 |
| ST-001 inline 特异性 | 引入 Origin::Inline | 30 行 |
| ST-002 inherit 非继承 | 扩展 inherit_property | 80 行 |
| R-003 FrameBuffer 溢出 | checked_mul | 5 行 |

### P2 — 后续跟进

性能优化（P-001~P-011）、剩余安全问题（S-006~S-030）、样式系统完善（ST-003~ST-015）、解析器修复（C-001~C-018）等 100+ 行改动。

---

## 审查方法说明

本次审查使用 9 个并行 AI 审查 agent，每个 agent 独立深入阅读源码文件，按以下维度系统性检查：实现缺陷、安全漏洞、性能问题、可靠性、API 契约。每个发现均包含文件路径、行号、问题描述和修复建议。

**建议下一步**：对 P0/P1 问题执行 Phase 5 二次验证（路径推演 + 反向举证），确认问题在生产环境中的实际可触发性。
