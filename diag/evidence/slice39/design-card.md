# slice39 设计卡：page_scripts 两处 200ms 有界等待的时限语义钉 + 多 tab 并发隔离

- 日期：2026-10-07
- 分支：`slice39-notify-bounded-wait-multitab`（base `29e7c967116f8975264e96f206654a9be6c6c191` = origin/main 合并树 tip）
- 池化来源：slice34 汇总残余申报第 6 项（顺延两轮）

## 1. 对象与调用链

两处产品代码（`apps/renderer/src/page_scripts.rs`）：

1. `drain_pending_dom_mutations`（:950）：`take_pending_async_callbacks()`（消费
   `async_callbacks_ready` 旗标）→ `execute_script_direct_bounded("", 200ms)`（js_worker.rs:391，
   普通 FIFO 通道 + `recv_timeout`）→ `apply_recorded_mutations`（无条件执行，不依赖有界等待结果）。
   调用链：runtime.rs 主循环 tick（:673）、automation.rs（:506）。
2. `notify_shim_apply_generation`（:1146）：`execute_script_direct_bounded("__zw_apply_generation_bump …", 200ms)`，
   在 `apply_recorded_mutations` 的 webview 在场臂（:1103）与 HTML 回写臂（:1130）apply 成功后调用。

关键机制事实（读码结论）：

- `execute_script_direct_bounded` 超时**只放弃等待，不撤回命令**——Execute 命令已入普通
  FIFO 通道，worker 空闲后照常消费（js_worker.rs:1194 Execute 臂对 dead reply `let _ = reply.send`）。
- timer 回调体在 `ResolveAsyncCallback` 命令臂内同步执行（`resolve_async_callback` →
  `__zwResolveCallback` → 取 `__zw_pending[id]` 直接调用），mutation 于该臂记入队列，臂末
  `async_callbacks_ready.store(true)`（js_worker.rs:1408-1413）——积压回调被 worker 继续处理时
  旗标逐个重新置位。
- `apply_recorded_mutations` 对队列 `drain(..)`（page_scripts.rs:1044），apply 与有界等待成败无关；
  超时轮同轮即应用「已记录」mutation，尚未执行的滞留回调 mutation 留队列，等旗标重置后的
  下一轮 checkpoint。
- bump 脚本滞留 FIFO 的三个出路：(a) worker 空闲后补执行（迟到必达）；(b) 期间导航 →
  `ResetDocumentState` 按 seq 丢弃滞留 bump（survives_document_reset，js_worker.rs:825-834），
  但 reset 重建整个 shim context——补偿状态等价新鲜，丢弃无损；(c) worker 关闭 = tab 关闭，
  状态随实例消亡。无「永久失步且无重试路径」形态。

## 2. 根因假设（残余风险 = 测试盲区，非产品缺陷）

「超时放弃本轮」存在两种误读回退面，当前零测试钉：

- 误读 A：超时 → 放弃**整轮 drain**（跳过 `apply_recorded_mutations`）→ 已记录 mutation
  延迟交付，且「超时轮返回值恒 false」改变宿主节奏。
- 误读 B：超时 → 丢弃/撤回滞留命令（如改成 try_recv 丢弃或超时即 return 不入队）→
  bump 丢失 = shim 补偿状态与 host 永久失步（R381 桶 stamp 恒 stale，融合视图重复并入
  —— vue_mount 双份 `<p>` 同族缺陷回归且**无自愈**）。

## 3. 语义结论：纯测试钉，无产品修复

两处等待的时限语义自洽：

- drain：超时放弃的是「本轮进入 worker 边界的时机」，不是本轮 apply，也不是滞留命令；
  mutation 不丢（同轮 apply 已记录部分 + 滞留部分经旗标重置下一轮补应用），滞留空脚本
  为 worker 侧 no-op（`__zw_begin_script` 仅重置 budget + 反射钩子）。
- notify：bump 超时 = 延迟交付（FIFO 补执行），非丢失；失步窗口有界且自愈（bump 落地即
  清 parse 补偿节点 + 推进 stamp），文档换代丢弃时 shim 全量重建等价覆盖。
- 产品改动备选项（改 `submit_script_priority` 让 bump 插队积压）被否：改变 FIFO 序无证据
  收益，且 bump 抢到积压回调之前反而让回调在「本代已换代」视图上执行，引入新的序语义
  风险——不在本切片范围。

## 4. 多 tab 并发隔离面盘点

apply/notify/代际路径的状态归属：

| 状态 | 归属 | 串扰风险 |
|---|---|---|
| `_zwApplyGenCounter`/shim 补偿状态/`__zw_pending` | per worker sandbox（per tab worker 线程） | 无（语言级隔离） |
| `mutations` 队列、`handle_selector_map`、`focus_changes` | per worker Arc | 无 |
| webview `cached_html`/`last_render`/handle 双表 | per WebView 实例 | 无 |
| `DRAIN_RECORD`（gcs drain 记录，computed_style_cache.rs:76） | **进程级全局单槽** | 有界：消费侧 `old_html`/`new_html` 双等值守卫（:131）——串扰需两 tab 字节级全等文档对；不衔接走全量 parse 兜底，正确性不依赖同步成功（:33 设计声明）。tab B 的 `clear_generation_cache`（注册时清全局槽）至多让 tab A 丢一次优化 |
| `MUT_DRAIN_GEN`/`DOM_VIEW_GEN`/`MUTATION_VERSION`/`REG_EPOCH` | 进程级全局计数器 | 仅作缓存键成分，跨 tab bump = 缓存 miss（性能），不产生错数据 |

结论：正确性关键面均 per-instance；全局面全部按「错配即兜底」设计。钉渲染层最近似面：
双 (worker, webview, html buffer) 实例，A apply/notify 后断言 B 的代际/队列/HTML/render 零变化，
B 再 apply 独立推进且不扰动 A。

## 5. 测试计划（apps/renderer/src/page_scripts/bounded_wait_tests.rs，新 cfg(test) 模块）

超时制造：helper 线程经 `worker.executor()`（ScriptFn = `Arc<dyn Fn + Send + Sync>`）提交
~1.2s JS 忙臂（`Date.now()` 忙循环；V8 watchdog 30s 不触发；TAB_JS_CHANNEL_TIMEOUT=35s
不受扰），`execution_count_for_test()` 轮询确认臂被领取；200ms 有界等待在臂内必超时——
不测「恰好 200ms」，测「超时后语义可恢复」，总测试墙钟 ~1.3s/例。

1. `drain_timeout_applies_recorded_and_recovers_backlog_s39`：C1 快 timer 先执行（mutation
   入队 + 旗标置位）→ 忙臂占住 → drain 超时轮：断言**返回 true 且 C1 mutation 已落宿主
   HTML**（钉误读 A）→ 长臂结束后优先通道注册 C2 快 timer（旗标由积压回调处理重新置位）
   → 轮询 drain 至再次 true：断言 C2 mutation 补应用（钉「滞留 mutation 不丢」）。
2. `apply_generation_bump_lands_after_bounded_wait_timeout_s39`：记录 mutation → 忙臂占住 →
   回写臂 apply 成功（bump 有界等待超时）→ 长臂结束后轮询 `_zwApplyGeneration()` 至推进：
   断言超时后 bump 滞留补执行（钉误读 B：滞留命令丢弃形态在此恒红）。
3. `apply_generation_state_isolated_across_tab_instances_s39`：双实例（worker 185/186 +
   webview + 独立文档），A apply（webview 在场臂）→ A HTML 更新 + A 代际推进；断言 B 代际
   不动、B 队列空、B HTML 不变、B render 不变；B 再 apply → B 独立推进、A 不受扰动。

## 6. RED→GREEN / mutation 证据计划

纯钉无产品修复，但钉必须证真抓得住回退面（对齐 slice34/35 mutation 文化）：

- mutation M1（钉误读 A）：在 `drain_pending_dom_mutations` 有界等待后插入
  `if result.is_err() { return false; }`（超时跳过本轮 apply）→ 测试 1 必红 → 还原复绿。
- mutation M2（钉误读 B / 调用点回退）：删除 HTML 回写臂 `notify_shim_apply_generation(ctx);`
  调用行（:1130）→ 测试 2、3 必红（gen 恒不推进）→ 还原复绿。
- 证据落 `diag/evidence/slice39/`：mutation.diff、red/green log、sha256 自证
  （**仅文本类证据；位图截图一律不进 Git**——评审截图留本地 .acceptance/ 或 PR 原生附件）。

## 7. 验收判据

- 三测试进常驻回归（make test 腿），非 flaky（无 «恰好 200ms» 断言；全部语义可恢复性断言）。
- 门禁全绿：cargo fmt 零 diff、clippy `-D warnings`（v8 腿；环境允许补 quickjs 腿）、
  `make test` ≥ 20,146P + 新 3 腿 / 0F、`make reftest` 704/704、WPT named-access 43P/1F/2T 同位。
- 无产品行为变更（diff 仅测试 + 模块声明 + 证据）。
