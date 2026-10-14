# 动画、过渡、命中与 DOM 几何

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 CSS 动画首帧、过渡事件重复、鼠标穿透以及 getBoundingClientRect 与命中矩形分歧。AnimationClock 和 TransitionClock 产生插值与待派事件，宿主负责推进和派发；HitTestCache 从布局及样式提取可跨线程、跨进程的纯数据，RectBridge 与 ElementFromPointBridge 提供同步查询。

### User Task Navigation
- 动画求值与首帧接线：`crates/engine/src/animation.rs`、`crates/engine/src/transition.rs`、`crates/engine/src/pipeline/mod.rs`。
- 命中、矩形快照和脚本桥：`crates/engine/src/hit_test.rs`、`crates/engine/src/element_from_point.rs`、`crates/engine/src/rect_bridge.rs`；身份绑定见 [脚本身份桥](../../scripting/dom-binding/identity/index.md)。
- 上述模块含插值、事件 drain、快照往返、嵌套坐标和 inline 上报矩形断言；本轮只读这些断言，没有运行测试或 Chrome 对比。

## Gotchas
- implicit 0%/100% 边界在动画启动时以元素 computed 基值填充动画私有 keyframe 克隆，仅覆盖 individual translate/rotate/scale；不能修改共享 registry，也不能据此宣称任意属性的 implicit 边界已齐全。 (`crates/engine/src/animation.rs`)
- 多段动画先选关键帧段，再将段内归一化进度交给 timing；段起帧自己的 timing 优先于全局 timing，先扭曲全局进度会改变选段。 (`crates/engine/src/animation.rs`)
- 颜色在预乘 alpha 空间插值后反预乘，透明端点不能按未预乘 RGB 直接混合，否则透明颜色污染中间帧。 (`crates/engine/src/animation.rs`)
- 属性插值存在实现边界：普通 transform 不在专用插值臂，未知属性在进度大于 0.5 时离散切换；translate 分量经简化数值解析输出 px，不等于完整百分比或三维插值。 (`crates/engine/src/animation.rs`)
- 过渡 duration、delay、timing 直接按 property 索引取值，缺项回落 0/0/Ease，duration 不大于零直接跳过；不能把该实现描述为完整 CSS 列表循环匹配。 (`crates/engine/src/transition.rs`)
- 命中以布局深度优先，同深度后访问者覆盖，elementsAtPoint 按深度排序去重；这只是布局树近似，不能据 API 注释中的“绘制序”推断完整 stacking-context/z-index 命中。 (`crates/engine/src/hit_test.rs`, `crates/engine/src/element_from_point.rs`)
- 祖先不包含点时仍递归，以保留 overflow:visible 子盒命中；overflow:hidden 的裁剪没有完整建模，被裁子盒仍可能命中。 (`crates/engine/src/hit_test.rs`)
- visibility:hidden/collapse 与 pointer-events:none 只取消自身候选资格，不剪掉后代；显式恢复 visible/auto 的后代仍须搜索，隐藏祖先的布局矩形仍保留。 (`crates/engine/src/hit_test.rs`)

## Architecture
- transitionrun 在创建时入队，transitionstart 在 delay 后首次活跃帧入队，首次帧即完成也先 start 后 end；三个 drain 使用 take，各记录只应由宿主消费一次。 (`crates/engine/src/transition.rs`)
- display:none 取消只为未完成过渡生成 cancel，elapsed 按 now-start-delay 钳在零与 duration 间；已完成项移除而不再派 cancel。 (`crates/engine/src/transition.rs`)
- inline_reported_rect 独立承载上报 y/height，命中和 gBCR 消费此值，递归子坐标仍以原布局行盒为锚；把上报值覆盖快照 y 会给嵌套子盒重复偏移。 (`crates/engine/src/hit_test.rs`, `crates/engine/src/rect_bridge.rs`)
- NodeId 快照键使用 SlotMap KeyData 的完整 u64 编码，不能只取槽索引；匿名盒无节点身份，不进入 rect 查询表，填表前清旧键防止上一帧残留。 (`crates/engine/src/hit_test.rs`, `crates/engine/src/rect_bridge.rs`)
- elementFromPoint 桥在锁内只克隆最新 HitTestCache 的 Arc，遍历在锁外；无缓存、NaN 或无命中返回空结果，不能以空结果判断 DOM 本身不存在。 (`crates/engine/src/element_from_point.rs`)

## Patterns
- rect handler 未注入或身份未命中返回 None，脚本回落零 rect；handle 身份先经持久 handle→selector 表，再按同 HTML 解析出的 NodeId 查布局快照，漏接表写入会让动态元素稳定返零。 (`crates/engine/src/rect_bridge.rs`)
- HTML rect 查询缓存是线程局部且以完整 HTML 字符串为键；它依赖相同 fresh parse 插入顺序产生相同 NodeId，不能让查询文档和渲染快照各用不同版本。 (`crates/engine/src/rect_bridge.rs`)
