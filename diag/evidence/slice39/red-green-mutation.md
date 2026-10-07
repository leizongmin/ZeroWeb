# slice39 mutation 证据：三钉检出力实证（纯钉，无产品修复）

- 日期：2026-10-07
- 基准文件：`apps/renderer/src/page_scripts.rs`（含新模块声明后）
- 基准 sha256：`266015164dfa0c0c1fcefd80904e5b93befcac697bc3fd934c9692adc3eb8c9b`
- 方法：手工行为变异（编译通过的语义回退形态）→ 钉红 log → 还原 → sha256 自证一致 → 复绿 log。
  每轮还原后基准文件 sha256 恒等上方值（三轮共用同一基准）。

## M1：drain 超时误读 A（「超时 ⇒ 跳过本轮 apply」）

- 变异：`drain_pending_dom_mutations` 有界等待改捕获结果，`is_err()` 即 `return false`
  （超时放弃整轮，不 apply 已记录 mutation）。
- diff：`mutation-M1-timeout-skip-apply.diff`
- RED：`red-M1.log` — `drain_timeout_applies_recorded_and_recovers_backlog_s39` FAILED，
  断言「超时轮已记录的 mutation 仍须 apply（drain 返回 true）」（bounded_wait_tests.rs:99）。
- GREEN（还原后）：`green-M1.log` — 1 passed / 0 failed。

## M2a：HTML 回写臂 notify 调用点删行

- 变异：删 `apply_recorded_mutations` HTML 回写臂 `notify_shim_apply_generation(ctx);` 调用。
- diff：`mutation-M2a-writeback-notify-removed.diff`
- RED：`red-M2a.log` — 仅 `apply_generation_bump_lands_after_bounded_wait_timeout_s39` FAILED
  （gen 恒不前进，5s 轮询超时；同批 test 1/3 按预期存活——分钉不串）。
- GREEN（还原后）：sha256 复验一致；`green-M2.log` — 3 passed / 0 failed。

## M2b：webview 在场臂 notify 调用点删行

- 变异：删 `apply_recorded_mutations` webview 在场臂（path A）`notify_shim_apply_generation(ctx);` 调用。
- diff：`mutation-M2b-webview-notify-removed.diff`
- RED：`red-M2b.log` — 仅 `apply_generation_state_isolated_across_tab_instances_s39` FAILED
  （tab A 代际不推进断言红；test 1/2 按预期存活）。
- GREEN（还原后）：sha256 复验一致；`green-M2.log` 同上（3 passed / 0 failed）。

## 结论

- 三钉各守一个独立回退面（drain 超时轮 apply / 回写臂 notify 调用点 / 在场臂 notify 调用点 +
  跨实例代际隔离），删任一守卫面恒红，无重叠盲区。
- 语义结论（设计卡 §3）维持：两处 200ms 有界等待超时语义自洽（放弃等待、不放弃 apply、
  滞留命令 FIFO 迟到必达），纯测试钉交付，产品代码零行为变更。
