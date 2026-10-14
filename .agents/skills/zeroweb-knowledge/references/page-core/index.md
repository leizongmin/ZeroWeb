# 页面内核：文档、样式、布局与绘制

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

页面管线为 DOM 文档 → CSS 规则/计算样式 → 布局树与最终行内格式化结果 → Painter 图元。覆盖 HTML 树身份、CSS 错误恢复与层叠、Taffy 装配及后处理、资源抽取、页面效果和命中几何。JS 原生绑定归 [脚本领域](../scripting/index.md)，实际字体与光栅化归 [图形领域](../graphics/index.md)，网络/权限归 [网络存储安全](../web-data/index.md)。

### User Task Navigation
| 用户问题 | 当前源码入口 | 深入知识 |
| --- | --- | --- |
| 节点移动、表单状态、Shadow 查询异常 | `crates/dom/src/document/mod.rs` | document |
| CSS 声明解析成功却不生效 | `crates/css-parser/src/parser/mod.rs`、`crates/style-system/src/lib.rs` | styles |
| 尺寸、位置、换行或布局回填异常 | `crates/layout-engine/src/engine.rs`、`crates/layout-engine/src/inline_finalization.rs` | layout |
| 首帧缺资源、DOM 换代或静态/动画入口分歧 | `crates/engine/src/pipeline/mod.rs` | pipeline |
| 内容盖字、SVG/过滤丢像素、点击或 rect 偏移 | `crates/engine/src/paint/painter/mod.rs`、`crates/engine/src/hit_test.rs` | paint |

验证入口复用 `crates/engine/src/tests/pipeline.rs`、相关 crate 常驻测试与 `tests/wpt-runner/src/reftest.rs`；[项目验证入口](../../../zeroweb-verify/SKILL.md) 是当前未提交的导航资产，不作为 HEAD 产品行为证据。本次不运行验证流程。

## Gotchas
- DOM NodeId 只能在所属 Document 的节点表内证明存活；fresh HTML 重建后旧 handle 映射失效，跨阶段缓存不能仅因 ID 数值相同继续使用。 (`crates/dom/src/node.rs`, `crates/dom/src/document/mod.rs`, `crates/engine/src/pipeline/mod.rs`)
- 解析 CSS 值不证明 used size 或规范行为完整：style converter、Taffy 装配和后处理都能改变结果，纯 Taffy 测试正确不足以证明生产布局树正确。 (`crates/layout-engine/src/converter/mod.rs`, `crates/layout-engine/src/engine.rs`, `crates/layout-engine/tests/r5010_taffy_relative_flow_position.rs`)
- 静态、动画、预算与增量接口不是等价管线；阶段预算只在阶段间检查 deadline，不能抢占单个解析/样式/布局阶段。 (`crates/engine/src/pipeline/mod.rs`, `crates/engine/src/pipeline_budget.rs`)

## Architecture
- LayoutBox 子坐标相对父内容区，后处理、Painter 与命中须累积父 border/padding 及相应滚动；把它当页面绝对坐标会在嵌套盒产生偏移。 (`crates/layout-engine/src/tree.rs`, `crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/hit_test.rs`)
- 最终 IFC 以布局侧保存的 fragments/metrics 向绘制传递，复用受 fail-closed 条件约束；改变字号、行高、字体或宽度必须核对 stored 与重排路径，而非强制所有盒复用。 (`crates/layout-engine/src/final_ifc_reuse.rs`, `crates/layout-engine/src/inline_metric_storage.rs`, `crates/engine/src/paint/painter/text.rs`)
- 资源抽取只输出 URL/描述符，宿主负责获取与策略；解码图片或过滤结果通过资源 key 与 ImageCache 接入图元，不能把占位图元非空当作像素已到齐。 (`crates/engine/src/pipeline/extract.rs`, `crates/engine/src/pipeline/mod.rs`, `crates/webview/src/webview.rs`)

## Child Knowledge Nodes
- [文档模型与 HTML 语义](document/index.md) — 树身份、解析、mutation/Shadow、表单和焦点状态。
- [CSS 解析与样式计算](styles/index.md) — 语法恢复、选择器、级联、计算值与变量。
- [格式化上下文与布局算法](layout/index.md) — 盒树、Flex/Grid/Taffy、行内、表格、分列与失效预算。
- [HTML 到绘制结果的管线与资源抽取](pipeline.md) — 各渲染入口差异、资源发现、缓存失效与宿主输入。
- [页面绘制、合成与命中几何](paint/index.md) — 绘制顺序、SVG 过滤、动画、指针命中和 DOM 矩形。
