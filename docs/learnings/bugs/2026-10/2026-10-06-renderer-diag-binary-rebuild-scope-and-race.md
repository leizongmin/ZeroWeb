---
date: 2026-10-06
modules: renderer, build
---

# 诊断轮换二进制：`-p zero-browser` 不重建 zero-renderer；build 未确认即跑 probe 用旧二进制

## 问题

E14 诊断给 renderer 侧（js_worker.rs、script-sandbox v8_runtime.rs）加打点后，`cargo build --release -p zero-browser` 再跑真站探针，**整轮日志打点全无**（r8 轮作废）；又一轮 build 命令带 `| grep -E "^error"` 管道后接 `node probe`，打点仍一度疑为零命中。

## 根因

1. **cargo `-p` 只保证指定包及其依赖**：`zero-browser` 对 `zero-renderer` 无依赖边（二者是兄弟 bin，共享 lib crate），`-p zero-browser` 不会重编/重链 `zero-renderer` 二进制——renderer 侧打点改完只 build browser 等于没改。
2. **build 与 probe 的时序竞争**：`cargo build ... | grep -E "^error" | head -3; node probe …` 中 `;` 顺序执行依赖 build 真的完成且产物落地；任何早退/失败（如编译错被 grep 过滤吞掉）probe 照跑，spawn 到的是磁盘上的旧二进制。另注意 browser 按 `resolve_renderer_binary()`（process_backend.rs）在 current_exe 同目录找 renderer，跑哪个 renderer 由产物目录决定，不由源码决定。

## 解决方案

- renderer 侧改动固定用 `cargo build --release -p zero-renderer`；两端都改则显式 `-p zero-renderer -p zero-browser`。
- 换二进制的诊断轮序列固定为：**build → `tail -1` 确认 `Finished` → 跑 probe → `ls -la --time-style=full-iso` 核对二进制 mtime 与 probe 启动时刻先后**，再采信日志。

## 如何避免

- 「打点零命中」先查二进制新鲜度（mtime 链）与 strings 验证，再怀疑代码路径；本例 r8 轮 30 分钟损失源于此。
- 诊断 build 命令避免把编译输出接进过滤管道后无条件续跑；失败要显式短路（`&&`）。
