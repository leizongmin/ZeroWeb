# 文本收集、空白、断行与双向文字

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 `collect_inline_items`、`break_items_into_lines`、扁平化 walk 与文本 owner。Taffy 不处理普通 IFC；此域把 DOM、样式与 atomic inline 收集成条目再断行。绘制空样式重建路径的恢复契约见 [终化与复用](finalization-and-reuse.md)。

## Gotchas
- `has_direct_text` 只排除 CSS 可折叠白空间，nbsp-only 内容仍触发 IFC；用 Rust `trim()` 或 `is_whitespace()` 会使 nbsp-only 行盒消失。 (`crates/layout-engine/src/inline_content.rs`)
- white-space 必须在折叠文本之前按 run 求值；伪元素注入文本优先读取文本节点自身 style，普通文本回退父 style，之后再用容器值兜底，否则丢失的换行无法恢复。 (`crates/layout-engine/src/inline/collect_items.rs`)
- PreWrap 的 trailing 空白可悬挂而 BreakSpaces 不悬挂；两者都 preserve 且可软换行，不能只存一个“保留空白”布尔值。 (`crates/layout-engine/src/inline/collect_items.rs`)
- 空 inline 仍生成零宽 fragment 并推进水平 padding/正 margin，便于后处理同步背景边框几何；它的负 margin advance 与非空 run 特意分开门控。 (`crates/layout-engine/src/inline/break_lines.rs`)
- `unicode-bidi:plaintext` 先按逻辑文本断行，再按段落方向逐行重排和 start 对齐；整段先视觉重排会把 Latin 词搬到 RTL 段首、改变软换行。 (`crates/layout-engine/src/inline/break_lines.rs`, `crates/layout-engine/src/inline/tests/plaintext.rs`)
- display:none 与 OOF 剪枝必须覆盖后代 walk，不仅直接子；layout 用 styles，paint 空 styles 用注入的节点集，无任何信号时不会凭空剪枝。 (`crates/layout-engine/src/inline/mod.rs`, `crates/layout-engine/src/inline/tests/r5102_oof_walk_prune_tests.rs`)
- 空 styles 的 UA 隐藏兜底不包含 rt/rtc/option/optgroup，这些有 ruby/select 专门语义；ruby 自身作为 paint 容器时另走扁平 base run，防注音变成普通正文。 (`crates/layout-engine/src/inline/collect_items.rs`)
- 非 preserve、非 pre-line 文本默认由 collector 消解实际生成的 U+000B 段断行标记：两侧落入当前 CJK/假名范围则移除，否则为空格；同一折叠序列在换行前已有空格时不生成标记，当前 `中 \n文` 会保留为 `中 文`，不能把两侧 CJK 概括为统一移除换行及空格。 (`crates/layout-engine/src/inline/collect_items.rs`, `crates/layout-engine/src/inline/inline_types.rs`, `git:6f3f60763690998f640614f20f800311370230c6`)
- 段断行跨文本节点取相邻非空白字符，直接相邻层遇块级、原子 inline、ruby 或上溯 IFC 容器边界会停止；嵌套子树的 None 未区分边界与无文本，调用层可能继续越过该子树取字，深度32限制也只作用于子树递归，不能据此保证整个扫描严格止于边界。 (`crates/layout-engine/src/inline/collect_items.rs`, `crates/layout-engine/src/inline/inline_types.rs`)
- 当前 U+000B 同时作为内部标记与原始 VT 字符，ruby 祖先豁免会在消解前返回；“哨兵不落 run”的设计注释不能证明全部分支已满足，控制字符和 ruby 消费者需单独复核。 (`crates/layout-engine/src/inline/collect_items.rs`, `crates/layout-engine/src/inline/inline_types.rs`)
- 跨 run 的内容词前断点若来自上一 run 的尾部可折叠空格，折行由空格所属 run 的 no_wrap 决定；空白词和 run 内部断点仍按自身 no_wrap，不能统一套后继内容 run 的 white-space。 (`crates/layout-engine/src/inline/break_lines.rs`, `git:24328107d9a12d44867d469bb8fd78dd44369965`)

## Architecture
- R109 `fragment_node_ids` 限定当前匿名片段收集范围；run-in 的 inline 内容先放在常规子内容之前，而 run-in 内 block 子保守跳过。 (`crates/layout-engine/src/inline/collect_items.rs`)
- 行盒以 NaN y 标识尚未定位；float 推压过的行在 push 时记录实际 y，末尾只为未定位行累计补位，不能无条件重算全部 y。 (`crates/layout-engine/src/inline/break_lines.rs`)
- `vertical=true` 在断行入口直接转列算法，fragment 坐标仍是物理 x/y；水平 break_lines 的对齐与推进不能直接推断竖排分支。 (`crates/layout-engine/src/inline/break_lines.rs`, `crates/layout-engine/src/inline/mod.rs`)

## Patterns
- 下载字体 ascent override 只从已经收集到的 text run 查询一次，不为每个 IFC 扫描全 DOM；run style 同样从节点或父节点解析。 (`crates/layout-engine/src/inline/collect_items.rs`)
- plaintext 测试同时约束软行文本顺序、段落方向 start 对齐和显式 left 不被覆盖，保存 override 集的测试还覆盖 paint 空样式恢复。 (`crates/layout-engine/src/inline/tests/plaintext.rs`)
- ZW_SEGMENT_BREAK_CJK 在 OnceLock 中按进程快照，值为0关闭；同进程后改环境不会切换 collector 行为，配对实验不能只在两个调用之间 set_var。 (`crates/layout-engine/src/inline/collect_items.rs`, `crates/layout-engine/src/inline/mod.rs`)
