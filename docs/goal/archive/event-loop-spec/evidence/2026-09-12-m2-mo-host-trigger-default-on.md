# M2 收口 — MO host trigger default-on（②b，2026-09-12）

**决策项**：goal 待用户决策清单「MO host trigger default-on」——前置 TODO「全量 make test
ON 臂补做」（M2 MO-S1 记账遗留）本轮完成，绿 → 按分期建议翻默认。

## ON 臂证据（`ZW_MO_HOST_TRIGGER=1` 全量 make test，2026-09-12）

- 结果：**16,659P / 1F**——唯一失败 `test_mo_host_trigger_default_off_no_notify`，
  断言的正是「默认 OFF」前提本身（构造默认 WebView 依赖 env 初值），env 翻 ON 后按设计
  必失败，非回归。
- 既有 corpus A/B（M2 MO-S1 记账）：testharness IO/RO corpus 双臂零 delta + MO 主
  corpus（dom/nodes MutationObserver 12 上游文件）138 subtests 逐条零 delta。

## 翻转内容

`crates/webview/src/webview.rs`：

- `WebView::new`：`ZW_MO_HOST_TRIGGER` 语义从「`=1` 开启（默认 OFF）」翻转为
  「`=0` 关闭（默认 ON）」。
- 三处注释同步（字段 doc / setter doc / `drain_native_mutations_to_mo` doc）——
  DC-3 门禁状态从「待 default-on」记为「已翻（本 evidence 为据）」。

`crates/webview/src/tests/mo_host_trigger.rs`：

- `test_mo_host_trigger_default_off_no_notify` → `test_mo_host_trigger_opt_out_no_notify`：
  OFF 路径经 `set_mo_host_trigger(false)` 直设（仓内既定模式，避免进程级 env 竞态——
  并行测试共用进程 env，构造期读值不可测）。
- 新增 `test_mo_host_trigger_default_on_notify`：无 env 干预下 native 写 → MO 收
  record（钉住新默认，防意外回退）。

## 质量门禁

- `cargo test -p zero-webview mo_host_trigger` 4P/0F（default_on_notify +
  opt_out_no_notify + 既有 setter-true/fragment 用例在新默认下全过）
- 全量 `make test`（新默认）见本文件提交轮 commit message 汇总
- clippy `-D warnings` 零警告；fmt 无 diff

## 结论

- host 侧 mutation 通知自本提交起为默认行为；`ZW_MO_HOST_TRIGGER=0` / 
  `set_mo_host_trigger(false)` 为显式逃生口。
- M2 剩余（均缓行记档）：MO-S3 批派发粒度合并（无 driving 用例不冒进）；MO-S4 +
  escape-hatch 收敛归 zero-web 流（前置三条件见 master.md 决策记账）。
