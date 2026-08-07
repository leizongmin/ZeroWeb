如无特别要求，默认使用中文来编写文档和注释，执行日志和反馈给用户的信息（包括警告、报错等）则应该使用英文。
Rally 本来就是跨轮次、跨 session 的长期执行循环。遇到需要多轮推进的架构任务时，不要把“需要多会话/长期推进”当成需要用户决策的阻塞；应更新对应的状态文档/控制面（例如 goal 的 master.md），并在最终一行输出 `CONTINUE: <下一步>` 把明确下一步传给后续轮次。
有阶段性进展时应该及时在当前的分支提交代码并推送到远端，也要及时拉取远端的更新并rebase。
单个代码文件一般不要超过2000行，如果超过了应该考虑合理拆分成多个文件。
跑测试或 WPT reftest 时必须用 `make test` / `make reftest`（release 构建 + scripts/test-guard.rs 包裹），禁止裸跑 `cargo test` 或 `cargo run --bin zero-wpt-runner -- reftest`：内存型 bug（如无限循环 realloc）只会被杀掉测试进程树，不会触发系统 OOM 连累整个 tmux session / rally 流程。阈值/兜底见 docs/rally/oom-guard.md。
涉及渲染/布局变更时建议额外跑 `make product-smoke`（DC-13 welcome.html vs chromium Oracle 回归门禁，diff>20% 退出 2）：`make test` + scoped reftest 不覆盖产品 fixture，曾致 R428 min-size:auto 的 welcome +7.65pp 回归藏了 14 轮未被发现（R541）。阈值可调：`make product-smoke MAX_DIFF=22`。
- legacy HTML 产品 smoke（DC-13 Tier 1，HTML 3.2/4 + CSS1/2）：`make product-smoke-legacy`（51 fixture vs chrome-127 oracle，trend-only exit 0）。diff% 为 font-wall 趋势数据；**struct-check FAIL 是「待查清单」诊断入口，不阻 CI**——run-all.sh 现打印 issue 详情（sibling overlap / collapsed / text concatenation）。历史 known struct FAIL = 37-form-controls（Phase A 阻塞，**R2156 slice 1 + R2162 slice 2 default-on 后已 struct PASS 3.85%**，非再 FAIL；R2163 实测 legacy 51/51 struct PASS）。涉及 UA 样式 / 表单 / legacy 元素变更时跑，防结构性退化藏匿（曾抓到 R1651 center / R1653 caption / R1657 noframes / R1669 area+frame+keygen / R1675 datalist+source+track 等真 bug）。

7. 取得重大进展或遇到卡点（如需用户决策、长时间阻塞、无法继续推进）时，应及时通过飞书 CLI 以应用机器人身份通知本人，消息需说明具体的进展或卡点信息。此通知仅为告知，不要因此阻塞或改变后续工作流程。命令：`SELF=$(lark-cli auth list | python3 -c "import sys,json;print(json.load(sys.stdin)[0]['userOpenId'])") && lark-cli im +messages-send --user-id "$SELF" --text "<具体内容>" --as bot`。

8. **并行开发（双独立 clone + 同一 main）**：两条 rally 流各跑一个独立 clone（勿同仓多 worktree）；push 前必 `git pull --rebase`（non-fast-forward 常态，自主 rebase、禁强推），commit 小而频繁。

9. 并行时**工作面不重叠**：各流只改自己的 crate/文档域（P1a：engine/dom/script-sandbox/net + zero-web/*；渲染：css-parser/style-system/layout-engine/render-foundation + rendering-compat/*）；共享面（engine、Cargo.lock、imported-tests.txt、wpt-data）冲突 = 碰头信号，暂停一边记入 master.md，不硬解。

10. 并行时**归因纪律**：main 是两流组合态，单树全绿 ≠ main 全绿；红灯先 `git log`/bisect 归因到流再修；跨流计数（如渲染文档的测试总数）异步漂移，更新前先 pull 核对；各流只写自己的 goal 控制面。

11. 并行不改变**用户决策门禁**：深结构 / 改 Mission / 许可证 / 破坏性操作仍须用户点名或拍板。

