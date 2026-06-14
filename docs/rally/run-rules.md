如无特别要求，默认使用中文来编写文档和注释，执行日志和反馈给用户的信息（包括警告、报错等）则应该使用英文。
有阶段性进展时应该及时在当前的分支提交代码并推送到远端，也要及时拉取远端的更新并rebase。
单个代码文件一般不要超过2000行，如果超过了应该考虑合理拆分成多个文件。
跑测试或 WPT reftest 时必须用 `make test` / `make reftest`（由 scripts/test-guard.rs 包裹），禁止裸跑 `cargo test` 或 `cargo run --bin zero-wpt-runner -- reftest`：内存型 bug（如无限循环 realloc）只会被杀掉测试进程树，不会触发系统 OOM 连累整个 tmux session / rally 流程。阈值/兜底见 docs/rally/oom-guard.md。


