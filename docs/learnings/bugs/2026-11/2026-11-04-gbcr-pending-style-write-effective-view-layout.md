---
date: 2026-11-04
modules: engine
---

# gBCR 不反映同任务样式写——读路径数据源分叉与 worker 侧有效视图布局

## 问题描述

页面脚本同一 JS 任务内 `el.style.width = '33px'` 后立刻读 `el.getBoundingClientRect().width` 得写前旧值 24，而同元素 `getComputedStyle().width` 正确返回 33px——两个几何/样式读 API 对同一次写入可见性分叉。dom-ops 基准（tests/benchmarks/dom-ops）`geom.write_read.thrash` 语义断言在 ZeroWeb 侧 FAIL、Chrome 侧 PASS，写读交替循环全程读旧 rect。截图/绘制正确，纯查询错。

## 根因分析

两个 API 的**数据源不同**，不是同一条管线的新旧两帧：

- `getComputedStyle` 在 js_worker 线程以代际缓存**有效视图**求值（`computed_style_cache.rs`：键 `(html, drain_gen, style_version)` + css/adopted 分量，幂等 replay mutation 队列 pending 尾）——写入即时可见。
- `getBoundingClientRect` 经 RectBridge 读 **renderer 渲染后 snapshot**（`fill_layout_rect_snapshot` 只在 render 后刷新）。JS 写产生 mutation 记录，renderer 异步 drain + 重渲染后才落到 snapshot；同任务内的读命中旧表。

排查时「computed style 对、rect 错」极易误判为样式系统 bug；先对表两 API 的数据源（谁在什么线程、什么时候刷新）直接暴露分叉。同页面 2026-10-02 的 gBCR 劫持（shim trap 优先级）是 JS 层同症状不同层——gBCR 族第三次踩坑，症状相同、根因各在 shim 优先级/布局 pass/快照时序三层。

修复路线的约束：renderer 主循环在 automation evaluate 期间被阻塞等待 js_worker，gBCR 若同步等 renderer flush 构成三方互等死锁（准则 #24 同形）；跨线程快照刷新不可行，worker 侧现算是唯一不锁死路线。

## 解决方案

`make_effective_dom_rect_handler`（crates/engine/src/rect_bridge.rs）：mutation 队列存在**未应用尾**（`queue.len() > applied cursor`；renderer 直连宿主游标恒 0 → 队列非空即 pending）时，worker 线程现算布局——复用 gCS 的 `with_cached_document_styles_lazy` 代际缓存（同代际共享 parse+cascade）+ `LayoutEngine::compute`（管线同款布局相位）+ `HitTestCache::fill_layout_rect_snapshot`（生产同款填表语义），布局结果单槽 TLS 缓存（键含视口）；队列 drain 后回落 snapshot 快路径，与旧行为逐字节一致（零回归面）。kill-switch `ZW_RECT_PENDING_LAYOUT=0`。

配套要点：

- pending 判据依赖生产不变式「apply = drain(consume-once) + bump_mut_drain_gen」（page_scripts `apply_recorded_mutations`）。模拟 apply 的测试必须同步收口两步，否则 pending 路径把已应用 mutations 重放（renderer 侧两例 handle-identity 测试因此翻红——测试补 drain+ bump 后全绿）。
- 新读路径接代际缓存而非另起缓存：同代际 gCS/gBCR 共享一次 parse+cascade；css/adopted 键分量由 `generation_cache_matches` 单点维护（本例将其改 pub(crate) 复用）。
- 已知边界记入源码 FIXME：worker 布局无 renderer 字体度量注入/图片内在尺寸，pending 窗口内文本几何可能与其后快照不同源；接线范围（renderer 通道修、tab 通道待视口管道）记 FIXME。

## 如何避免

- 同语义读 API（gBCR/getClientRects/offsetWidth/getComputedStyle）新增或排查时先对表数据源与刷新时机；分叉是最便宜的报警器。
- 快照型 DOM 架构加「读必须反映 pending 写」语义时：判据用队列未应用尾，实现复用既有有效视图代际缓存，drain 后必须回落权威快照；禁止跨线程同步 flush（等待方互相持有即死锁）。
- 测试模拟生产事务（apply）时对照生产实现核对全部副作用（drain、代际 bump、map merge），缺一项就会在新消费者引入后暴露为假红。
