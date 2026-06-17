如无特别要求，默认使用中文来编写文档和注释，执行日志和反馈给用户的信息（包括警告、报错等）则应该使用英文。
有阶段性进展时应该及时在当前的分支提交代码并推送到远端，也要及时拉取远端的更新并rebase。
单个代码文件一般不要超过2000行，如果超过了应该考虑合理拆分成多个文件。
跑测试或 WPT reftest 时必须用 `make test` / `make reftest`（由 scripts/test-guard.rs 包裹），禁止裸跑 `cargo test` 或 `cargo run --bin zero-wpt-runner -- reftest`：内存型 bug（如无限循环 realloc）只会被杀掉测试进程树，不会触发系统 OOM 连累整个 tmux session / rally 流程。阈值/兜底见 docs/rally/oom-guard.md。
取得重大进展或遇到卡点（如需用户决策、长时间阻塞、无法继续推进）时，应及时通过飞书 CLI 以应用机器人身份通知本人，消息需说明具体的进展或卡点信息。此通知仅为告知，不要因此阻塞或改变后续工作流程。命令：`SELF=$(lark-cli auth list | python3 -c "import sys,json;print(json.load(sys.stdin)[0]['userOpenId'])") && lark-cli im +messages-send --user-id "$SELF" --text "<具体内容>" --as bot`。


