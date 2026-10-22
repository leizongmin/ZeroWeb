# M3-b 拆面：多次导航缓存释放——只读静态审计（DC-4 保留子项）

**运行身份**：审计基线 HEAD `60e467be5`（M4 本流收口后）。**只读审计切片：零代码变更，未触碰 engine/layout-engine 生产与测试文件**（crates/engine 为并行流活跃工作面，run-rules §9——本切片只读源码，不做修改）。
**范围**：DC-4 判据"缓存不保留旧文档"（多次导航/释放缓存）的静态可达面。半透明帧复用、命中/像素（compositor/paint 面）不在本切片，维持跨流协调。

## 导航入口的缓存换代行为（`render_html`，pipeline/mod.rs:1694-1721）

每次全量渲染（导航 = 新 HTML 全量 parse）尾部对文档派生缓存的处置：

| 缓存 | 键 | 导航行为 | 位置 | 结论 |
|---|---|---|---|---|
| `cached_styles` | 旧文档 NodeId | **整体替换**（`= styles`，非 merge） | pipeline/mod.rs:1707 | 旧键值对全部丢弃 |
| `cached_stylesheets` | — | 替换 | :1710 | ✔ |
| `cached_layout` | 旧文档盒树 | 替换 | :1721 | ✔ |
| `cached_doc` | 文档本体 | 替换（slotmap 换代） | :1696 | ✔ |
| `persistent_handle_nodes` | 旧文档 handle→NodeId | **显式 clear**（R100 注记：换代全失效） | :1698 | ✔ |
| `cached_css_text` | CSS 解析缓存 | 置 `None`（新文档 `<style>`/meta 可能不同） | :1712 | ✔ |
| `form_control_values`/`compositions` | NodeId | clear | :1694 | ✔ |

## 下游引擎的文档键状态

| 状态 | 导航行为 | 位置 | 结论 |
|---|---|---|---|
| layout taffy `cached_state` | 每次 compute **整体重建**（新 `taffy_tree` 由 `build_layout_tree_with_r109` 从 (doc, styles) 构建，`cached_state = Some(新树)`） | layout-engine/engine.rs:1482；compute_with_img_intrinsic :435 起 | 旧文档树整体丢弃；且生产只写不读（撤用增量臂，[m1-layout] 边界） |
| StyleSystem 实例内状态 | 无文档键持久缓存；viewport 每 render 重设；registered_properties/font_feature_values 每 compute 重算 | style-system lib.rs:486-487（增量臂同源） | ✔ |
| JS 桥 computed-style 槽缓存 | 换代纪律已有：webview 文档换代就地 clear 的例外由 `register_dom_callbacks` 每次注册调 `clear_generation_cache` 覆盖 | js_dom_bridge/computed_style_cache.rs:38-39/:522 | ✔（bridge 面自身已处理，本切片只确认存在） |
| 动画 `keyframes_registry` | 同名 `HashMap::insert` **覆盖**（animation.rs:817）；旧文档独有名成为死数据——`start_animation` 按新文档 `animation-name` 查表，永不命中 | animation.rs:817/:840 | 无行为泄漏；死数据量以旧样式表为界 |

## 观察项（方向安全 / 协调候选，非缺陷断言）

- `dirty_tracker`：普通 `render_html` 不清空；旧文档 NodeId→rect 残留只可能**放大**下一次 `incremental_render` 的脏区（过失效方向，不会漏失效），且消费点即清空自愈（pipeline/mod.rs:1795-1824，:1824 clear）。
- `animation_clock` 活跃动画状态：静态渲染路径 clone-后-丢弃（状态纪律注记 pipeline/mod.rs:1574-1590），不污染主时钟；webview 泵路径（真实时间轴）的换代生命周期归 animation/webview 面——并行流活跃域（M2-S1b 等），列跨流协调。
- `canvas_registry`：宿主持有的 `Arc<Mutex<CanvasRegistry>>`，条目随文档切换的释放属 bridge/host 面（:38-39 换代纪律覆盖 gCS 槽，canvas 条目本切片未逐项核），列跨流协调。
- `image_sizes`/`image_natural_sizes` 等：URL 键资源缓存，跨导航共享是合法资源复用，非文档缓存。`scroll_offsets`：用户状态，非文档派生缓存。

## 结论与余项

**DC-4"缓存不保留旧文档"子项：静态审计通过**——导航路径上全部文档派生缓存或整体替换、或显式清空、或换代清零，未发现旧文档状态进入新文档渲染的保留路径。

余项（控制面维持）：多次导航的**运行时**对照（navigate A→B→A′ 输出比较）与半透明帧复用、命中/像素在 engine pipeline/paint 面——product-smoke 为单页运行，静态审计不能替代运行时观察；待 engine 面解封后补。

[m1-layout]: ../2026-10-23-m1-layout-dualpath-snapshot/ANALYSIS.md
