# M3 缺陷1修复——gBCR 反映 pending 样式写（worker 侧有效视图布局）

**日期**：2026-11-04
**前置**：[M1+M2 首轮](../2026-11-04-m1m2-dom-ops-protocol-first-run/README.md)（DC-1 语义 FAIL 定位本缺陷）
**结论**：缺陷1已修复——语义断言、引擎单测、探针、全量基准四层证据齐备；`geom.write_read.thrash` ZeroWeb 侧 ✗FAIL→✓PASS。配对回归（相邻面 108+15 测试）全绿。

## 根因（与 [2026-10-02 gBCR 劫持学习](../../../../learnings/bugs/2026-10/2026-10-02-textcontent-mutation-gbcr-identity-hijack.md) 不同层）

- `getComputedStyle` 在 js_worker 侧以**代际缓存有效视图**求值（`computed_style_cache.rs`：键 `(html, drain_gen, style_version)` + css/adopted 分量，幂等 replay pending mutations）——样式写立即可见。
- `getBoundingClientRect` 走 [`RectBridge`](../../../../../crates/engine/src/rect_bridge.rs) 读 **renderer 渲染后 snapshot**（`fill_layout_rect_snapshot`，仅 render 后刷新）。JS 同任务内 `style.X=` 后 gBCR 读到渲染前旧 rect；renderer 主循环被 automation evaluate 阻塞，同步 flush 会构成三方互等死锁（准则 #24 同形），worker 侧现算是唯一不锁死路线。
- 规范依据：CSSOM View `getBoundingClientRect()` 须先 run the update style and layout steps（https://drafts.csswg.org/cssom-view/#ref-for-getboundingclientrect）。

## 修复（`make_effective_dom_rect_handler`，crates/engine/src/rect_bridge.rs）

- mutation 队列存在**未应用尾**（`queue.len() > applied cursor`；webview 发布游标，renderer 直连恒 0 → 队列非空即 pending）→ worker 线程以有效视图现算布局取 rect：复用 `with_cached_document_styles_lazy`（与 gCS 同一代际缓存，同代际共享一次 parse+cascade）→ `LayoutEngine::compute`（renderer 管线同款布局相位）→ `HitTestCache::fill_layout_rect_snapshot`（生产同款填表语义，inline reported rect 消费一致）。布局结果单槽 TLS 缓存（`RECT_LAYOUT_CACHE`，键含视口），同代际第二次查询起 O(1)。
- 队列已 drain（renderer apply + 重渲染后）→ 回落 snapshot 快路径，与旧行为逐字节一致（零回归面）。
- kill-switch `ZW_RECT_PENDING_LAYOUT=0` 整面回落旧 handler（工厂构造期读一次，准则 #9）。
- **已知边界**（pending 路径限定）：worker 布局不带 renderer 字体度量注入（`FontMetricMap` 随 font-load 异步入管线）与图片内在尺寸，pending 窗口内文本/图片几何可能与其后 snapshot 不同源，apply+重渲染后自愈。FIXME(M3 后续) 记入源码。
- **接线范围**：renderer js_worker（CDP automation/bench 路径，`SetViewportHint` 命令臂同步视口 cell）；browser tab_js_worker 沿旧 handler——TabJsWorker 无视口提示通道，FIXME 记入源码，本缺陷在 tab 路径仍在。

## 配对回归

| 面 | 结果 |
|---|---|
| 引擎单测（本次新增，与修复同提交） | `rect_bridge` 15/15 ✓——含 `test_effective_rect_handler_reflects_pending_style_write`（pending 33px ✓ / drain 后回落 snapshot 标记值 ✓）与 `test_effective_rect_handler_layout_follows_viewport`（1280→800 视口重算，50% 宽差=240 ✓） |
| 引擎相邻面 | `js_dom_bridge` 1421 通过/5 失败＝HEAD 既有红（stylesheets/adopted 族，stash 往返复核同 5 例，与本改动无关）；`computed_style_cache` 2/2 ✓；`hit_test` 45/45 ✓ |
| renderer 相邻面 | `js_worker` 108/108 ✓——两例 handle-identity gBCR 测试补 drain+`bump_mut_drain_gen` 收口（生产 `apply_recorded_mutations` 为 drain+consume-once+必 bump 不变式，测试模拟此前缺收口） |
| 行为探针 | P1 `{"r0":24,"stored":"33px","r1":33,"csWidth":"33px","offsetWidth":33,"r0Again":24}`——修复前 r1/offsetWidth 恒 24（见 v1 evidence），写后读已反映写入 |
| 汇编核验（准则 #28） | `ZW_RECT_PENDING_LAYOUT` 字符串在 `target/release/zero-renderer` 命中 1 次（执行体），`zero-browser` 0 次（链接器 GC，正常） |

构建/测试均走守卫：release 构建 systemd-run scope（MemoryMax 24G / 2400s）；测试 test-guard（time-limit 600–900s、per-proc 4G、total 8G）套 systemd-run scope。

## 基准重测（官方采集 v2，`reports/report.json`）

同协议 v1、同环境（Chrome 156.0.8078.79；ZeroWeb release 2026-11-04 16:25 重建）、正反序配对。运行时工作树含本修复未提交改动（`git_dirty: true`，修复与本证据同一提交）。

| 行 | v1 首轮 | 本轮 | 变化 |
|---|---|---|---|
| geom.write_read.thrash | INCONCLUSIVE zw:FAIL 36.3x | **OK PASS 9885.5x**（zw 12604µs/样本 vs ch 1.27µs） | 语义修复；比值爆炸为真实热点数据——ZeroWeb 强制布局=全文档重建 vs Chrome 增量布局 |
| style.write.color | 182.8x ANOMALY | 119.4x ANOMALY | 量级不变 |
| geom.read.same / rotate | 49.8x / 47.1x ANOMALY | 49.9x / 63.1x ANOMALY | 量级不变 |
| mut.replace.pool | INCONCLUSIVE zw:FAIL 246.7x | INCONCLUSIVE zw:FAIL 228.8x | 缺陷2（timer 回调丢 data-\*）未动，仍 FAIL——M3 下一步 |
| mut.move.identity | INCONCLUSIVE timer 89x | OK ANOMALY 83.0x | 语义检查本就 PASS，本轮未触发分辨率阈 |
| cv.read.imageData | 30.2x ANOMALY | 31.1x ANOMALY | 量级不变 |
| mut.build.append | INCONCLUSIVE 顺序效应 | INCONCLUSIVE 顺序效应（2747.8x） | 未动 |
| mut.remove.children / cv.state.fillStyle / cv.draw.rect | INCONCLUSIVE timer | INCONCLUSIVE timer | 未动（ZeroWeb performance.now ~100µs 粒度平台事实，v1 已记录） |

其余 PASS 行（style.read.same 3.9x notable、style.read.rotate 3.9x notable、cv.state.saveRestore 6.1x notable）与 v1 量级一致。

## 定向性能门禁（准则 #23，`ZERO_WEB_BENCH_CRATES=engine`）

`bench-report.sh + perf-gate.sh` 同构建连跑 3 次（守卫内）：`page/medium/first_paint_wall_ms` 首轮 365.44 FAIL（预算 362.35，超 ~1%），复跑两轮 323.96 / 326.72 PASS——归因环境方差（基线 280.31，预算公式 `×1.15+40` 即为吸收 ±27~31% 共享 runner 方差校准），非本改动回归。机械证据：测量走 `zero-wpt-runner perf` → `render_with_layout_inner` 进程内 engine 直连管线（无 renderer 进程、无 js_worker、无 RectBridge 装配），且 `ZW_RECT_PENDING_LAYOUT` 符号在 `target/release/zero-wpt-runner` 0 命中（链接器 GC，装配只在 renderer/tab worker）。其余 18 指标三轮全 PASS。

## 清理

- 探针 `probe-semantics-and-scaling.mjs` 补 `fileURLToPath` import（v1 SL-008 改造时漏import，本轮运行发现即修）。
- `tests/benchmarks/dom-ops/results/m3-defect1/` 为 gitignore 本地产物，官方三件套已拷贝入本目录 `reports/`。
