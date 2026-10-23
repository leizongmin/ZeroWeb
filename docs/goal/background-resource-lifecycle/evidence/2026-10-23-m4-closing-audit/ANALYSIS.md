# M4/M3 收尾审计——交付物盘点与契约对账（2026-10-23）

审计基线：main @ 79f9887b8（pull --rebase 后最新）。方法：交付物存在性盘点 → 修复锚点与测试抽查（file:line）→ 契约 DC 验收文本比对 → 遗留项三分类。

## 1. 交付物盘点（全部在位）

| 类别 | 清单 | 核对 |
|---|---|---|
| 编排/门禁脚本 | `scripts/background-resource-{observe.sh,probe.mjs,sampler.py,fixture-server.py,gui-tab-switch.sh,gui-lifecycle.sh,gate.py,phase-stats.py}` | 8/8 存在 |
| fixture | `tests/fixtures/background-resource/`（bg01×2、bg02×4、bg03×3） | 9 文件 |
| 阈值基线 | `docs/perf/baselines/background-resource-linux-x86_64.json` | 7 场景，n=2–4 |
| 证据 | `evidence/*/ANALYSIS.md` | 13 份（M1×1、M2×3、DC-1×2、M3×4、M4×3） |
| 控制面 | goal 契约 v2.3 + master.md（本轮同步至对账结果） | 一致 |

## 2. 修复锚点与测试抽查（全部确认在位）

- DC-1：shim `_zwClampDelay`（part01.js ×5）、`_zwInternalTask`（part02.js ×16）；worker 面 `worker_timer_period_at`（script-sandbox/lib.rs:360）+ WORKER_BOOTSTRAP_JS 三文件；测试 `test_timer_nesting_clamp_zero_delay_chain_dc1` / `test_timer_nesting_clamp_interval_rearm_dc1` / `test_background_timer_clamp_hidden_page_dc1`（js_dom_bridge_tests/part01.rs:2475/2541/2631）。
- DC-2：`__zw_raf_interest`（part01.js ×3）；`SetVisibility`（protocol/src/message.rs:379）；`__ZW_HIDDEN`（part06.js ×5）；`page_hidden` 维度（renderer/paint_export.rs:363）；`refresh_compositor_animation_state`（renderer/runtime.rs:1314）；keep-alive 重臂（browser/process_backend.rs:548/2445）；测试 `compositor_state_clears_declarations_while_page_hidden` / `set_visibility_refreshes_compositor_animation_declarations_without_publish`（animation_export_tests.rs:618/651）、visibility_tests 两测、navigation_contract_tests 模块（process_backend.rs:2307）、fetch_scheduler 16 测。
- 媒体缺口三锚点本轮复核仍原样在位：`_zwMediaAutoplay`（part06.js:9959）、`_zwMediaTimeMarchesOn` 全仓仅定义+1 注释（零调用方）、audio 桥武装缺口（part03.js play 路径）。

## 3. 契约对账

| DC | 契约验收文本 | 判定 |
|---|---|---|
| DC-1 | 嵌套最小延迟符合规范；后台策略明确且恢复正确 | **DONE**——嵌套钳制（页面 ~237 iter/s / worker 同款）+ 隐藏页 arm 1s 下限 + 恢复自动回原延迟，shim/集成/多进程三层测试 + GUI 端到端（0.052→0.004→0.046） |
| DC-2 | 可跳过工作不持续重复呈现；音视频等合法工作不误停 | **DONE（附 INCONCLUSIVE 子项）**——前半全链 DONE（呈现链/rAF 泵/可见性/offscreen 门控/page_hidden，生产双腿差分）；video 面「不误停」通路成立；audio/autoplay 维度 **INCONCLUSIVE**：播放时钟三缺口（上游 media-playback 面），时钟走通前不可测——按契约「缺指标标 NOT_RUN/INCONCLUSIVE」记档，非本 goal 可修 |
| DC-3 | 已替换文档/回调/资源可释放；RSS 不无界增长 | **DONE**（headless + GUI 双面，M2 三步 + 40 周期腿） |
| DC-4 | 关闭/重开生命周期符合预期；前台恢复 | **DONE**（headless 20/20 + GUI close/reopen 双修复后收口） |
| DC-5 | 时长/预热/样本/阈值/环境先记录；缺指标标 NOT_RUN/INCONCLUSIVE | **DONE（v1）**——协议固定 + 阈值基线 7 场景 + 正 8/负 3 腿验证；媒体时钟下界挂基线 v2（依赖上游） |

## 4. 遗留项三分类

**已知边界（记录在案，不阻收敛）**：offscreen/hidden 期间 gCS 覆盖缺失、终结事件丢失、动画时间轴重启（CSS Animations 规范允许）；SW 面 timer 无延迟 FIFO 简化模型（钳制语义不适用，随 SW timer 语义升级另行处理）；promise 续体欠钳（FIXME）；intensive throttling 与 IDB 外内部 setTimeout 用户（超出 DC-1 验收最低面的增强项）；GUI/compositor 场景不入基线 v1（display 环境不一、协议未固化）。

**上游移交（INCONCLUSIVE，重开条件明确）**：媒体播放时钟三缺口（autoplay 置态绕桥 part06.js:9959 / `_zwMediaTimeMarchesOn` 死钩子零调用方 part06.js:16225 / audio load 时桥武装未命中）→ media-playback 面修复；修复落地后本 goal 回补 bg02-media 媒体时钟下界（基线 v2），届时 DC-2 INCONCLUSIVE 子项可复评。

**本 goal 可做且未做**：无。

## 5. 结论

本 goal 验收范围内五类场景（后台 timer、视口外动画、重复导航、停滞资源、页面关闭）全部有生产证据与回归面；资源门禁（阈值基线 + gate 脚本 + 正负验证）已落地可长期复测。建议 goal 收敛为 **DONE**（DC-2 媒体维度 INCONCLUSIVE 记档 + 上游移交）；重开条件唯一：媒体时钟缺口修复后回补基线 v2。
