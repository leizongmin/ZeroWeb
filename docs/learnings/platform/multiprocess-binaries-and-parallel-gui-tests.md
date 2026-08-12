# 多进程二进制查找与并行 GUI 测试的子进程竞争

日期：2026-08-12
模块：zero-browser（compositor_client / process_backend / tests.rs）
关联修复：R3254

## 问题描述

4 个 GUI 测试（clicking_checkbox / typing_in_clicked_input / gpu_compositor_path /
form_fixture_physical_clicks）在本地 cargo test 下稳定挂起（wait_for_snapshot_after
60s 超时），但在 PR 验收（make browser 等）下通过。同时 browser 测试全量跑与
单测单独跑结果不一致。

## 根因分析

两个叠加的环境问题：

1. **二进制查找失败静默回退**：`spawn_transport`（compositor_client.rs）用
   `Command::new("zero-compositor")` 依赖 PATH，`resolve_renderer_binary`
   （process_backend.rs）依赖 current_exe 同目录 + PATH——cargo test 的 PATH
   不含 `target/debug`，测试二进制位于 `target/debug/deps/`。两个查找都失败 →
   静默回退单进程 worker → worker 路径没有 click 默认动作/焦点切换（PR 的表单
   交互只在多进程 renderer 实现）→ 表单交互测试全部超时。而且磁盘上的
   `target/debug/zero-renderer` 可能是旧产物（不在依赖树，cargo test 不会重编），
   即使找到也会因版本不匹配行为异常。

2. **并行测试的子进程竞争**：多进程可用后，并行测试同时 spawn 多个 renderer
   子进程（每个 ~582MB 二进制 + 字体加载）+ 共享进程内 compositor client
   （static CLIENT 全局单例）→ 快照轮询（500×10ms）在启动窗口内超时。
   单独跑每个测试都通过（15-33s），全量并行必挂。

## 解决方案

- 二进制查找统一为：`ZW_*_BIN`/`ZERO_RENDERER_PATH` → `CARGO_BIN_EXE_*`
  （cargo 注入，仅依赖 bin 可用）→ current_exe 同目录 → **上溯目录**
  （`target/debug/deps/` → `target/debug/`）→ PATH 兜底。
- 测试环境默认单进程 worker（`cfg!(test)` 分支）；断言真实多进程链路的测试
  显式 `enable_multiprocess_for_test()`，并用进程内全局 `Mutex<()>` 串行化
  （4 个多进程测试互斥，其余 277 个并行不受影响）。

## 如何避免

- 任何 `Command::new("xxx")` spawn 兄弟进程的代码，必须提供与
  `resolve_renderer_binary` 同级的查找链（env → CARGO_BIN_EXE → exe 目录上溯
  → PATH），并在找不到时显式 warn（不要静默回退不同行为路径）。
- 测试 spawn 子进程或共享进程内全局（compositor client）时，相关测试必须
  串行化（互斥锁），否则并行 CI 与单测结果不一致。
- 不在依赖树里的 workspace bin（zero-renderer/zero-compositor）不会被 cargo
  test 重编——验证前先 `cargo build --workspace`，或让测试通过 CARGO_BIN_EXE
  引用构建产物（需依赖声明）。
