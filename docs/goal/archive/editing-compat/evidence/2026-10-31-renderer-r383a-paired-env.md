# renderer R383a 对账：配对环境判定第二通道（host 换代标记）

日期：2026-10-31（R54xx 轮，void 簇收口后接控制面挂账「④ renderer R383a 对账」）
范围：`crates/engine/src/js_dom_shim/part05.js`（`_zwApplyPairedEnv` 第二通道）、`apps/renderer/src/page_scripts.rs`（`notify_shim_apply_generation` 通知脚本置标记 + 钉测两枚）
基线：6f35320f2（R5398 void 簇收口）

## 挂账与对账结论

控制面挂账（attr 门轮/域收缩轮）：R383a 配对环境闩以 `__zw_pending_attr_sels` 版本戳 cursor 分量为判据（webview apply 点 `publish_mut_applied_cursor` 配对发布），renderer 私有队列路径未发布游标——判定在该环境恒 false，需对账。

对账结论（本轮定谳）：

1. **cursor 通道覆盖 webview 形态**（含 runner——编辑语料跑无头 WebView 共享队列，`apply_pending_shared_mutations`/`apply_mutations_subset` apply 成功即发布，语料基线在 paired=true 下取得）。
2. **engine 纯测试环境 cursor 恒 0 → paired=false → 跨代际桶恒并**（t8r-4 臂②「apply 代际 bump 后 pending 子不消失」的既有保护语义）。
3. **renderer 私有队列路径（`apply_recorded_mutations` 两臂：webview 在场 `apply_dom_mutations_and_render` / HTML 回写）apply 后发换代通知但无游标发布**——`publish_mut_applied_cursor` 是 webview 线程 thread_local，发布不到 js_worker context；journal 边界 drain 清空使 cursor 数值对重放/attr 扫描本无意义（三消费点全被 `min(count)` 钳制，drain 后 count=0），唯一受影响面是 JS 侧 `_zwApplyPairedEnv` 与 R5356 种子过时判定。

## 修复（两笔）

① shim `_zwApplyPairedEnv` 加第二通道：`globalThis.__zwHostApplyBumpSeen === true` 置闩（闩语义与 cursor 通道同）。② renderer 换代通知脚本升级置标记（`notify_shim_apply_generation`，经 `execute_script_direct_bounded` 在 js_worker 线程执行——thread_local 正确性由执行线程保证）。标记语义 = 「apply 与换代成对」事实本身：通知只在 apply 成功分支发（两臂同）。engine 测试手写的旧形态 bump 模拟串不含置位 → 恒并语义不受扰；webview 侧通知脚本未动（cursor 通道已覆盖，零行为变化）。

## 行为面评估（如实记录）

- **触发组合推演**：R380 门三臂在 renderer 形态下，Arm1/Arm2 有同代门（换代后关）、Arm3 依赖 journal 未应用尾（drain 后新 turn attr 写可开）。attr 写不经 `_zwHCLiveInvalidate` 不重盖桶戳 → 「本 turn attr 写开 Arm3 × 旧代结构桶滞留」组合存在。
- **mirror 行为钉实证**：构造上述组合（appendChild → drain apply → 推新快照 → setAttribute → outerHTML），序列化恒恰 1 br 且 attr 落地；**stash A/B（撤 shim 第二通道）不红**——R382 条目级代际兜底（`_zwPaGen` 跨代际剔除已收编条目）在该纯页面形态已防双计。paired 通道的增量暴露面在 R5356 种子域等编辑域形态（js_worker 跑 contenteditable 页面的 CE 编辑域与 webview 同 shim），mirror 未构造出修复前红形态。
- 修复定性：**判定源对齐**（挂账实质完成）——renderer 生产环境 paired 语义与 webview 形态一致（true），不再是「配对环境被误判为非配对」；非「修复已证双计」。行为钉保留为该组合的回归位（主钉 = 标记置位钉）。

## 验证

### 新钉（zero-renderer，test-guard 包裹）

- `apply_generation_notice_sets_host_paired_marker`：apply 前 `__zwHostApplyBumpSeen` undefined → drain apply（HTML 回写臂）→ 标记 true。
- `cross_gen_serialization_skips_bucket_after_apply`：Arm3 门开 × 跨代际桶 × 序列化读 → 恰 1 br（探针全链：`attrSels=1:0|#box | bumpSeen=true | gen=1 | sel=#box | outer=…<br></div>`）。

### 回归面

- renderer lib 全量：**355P/5F**——5 红与基线（stash 两处改动复跑）逐名 identical（fetch 3 + form_interaction_fixture_complete_sequence + javascript_disabled_skips_listeners_not_default_actions，既有非绿面）；+2P = 两枚新钉。
- engine lib（DISPLAY=:20）：**3,623P/10F**——红集 10 名逐名 identical（fetch/Request 族 8 + iframe SW + s24 flip，零新增；对上轮账 3,617P +6P 无红移）。
- 语料门禁抽查（新 shim runner 二进制，零位移）：void 双语料 **72P/0F + 48P/0F**、inline-host **880P/0F**、join paragraphs 案 **3456P/0F**。runner 走 cursor 通道（paired 闩本来 true），第二通道在 runner 环境不触发（`__zwHostApplyBumpSeen` undefined——webview 通知脚本未升级），账面零位移与预期一致。

## 范围外

- webview 侧通知脚本未升级（cursor 通道已覆盖生产语义；统一形态留待有真实行为差异时再做）。
- renderer mirror 环境未构造 R5356 种子域的修复前红形态（CE 编辑域在 js_worker 的真实暴露面待编辑语料在 renderer 形态的后续轮次验证）。
