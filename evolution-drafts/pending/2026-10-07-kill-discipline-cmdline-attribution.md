# 自进化草案：kill 纪律升级——按 cmdline 路径归属过滤，禁止按进程名批量 kill

- 日期：2026-10-07
- 来源事件：t8d 收尾清理时 `pgrep -x zero-browser | xargs kill` 误杀兄弟会话（zeroweb-1e）的浏览器进程 PID 936894（/home/lei/work/ZeroWeb 领地），违反「兄弟仓绝不触碰」约束。
- 现有规则的缺口：现有纪律为「用 kill 指定 PID、验证进程用 `pgrep -x`」——但 `pgrep -x` 只按进程名匹配，多仓同名二进制（ZeroWeb 与 ZeroWeb-2 都产出 zero-browser）下 `pgrep -x name | xargs kill` 等价于按名批量杀，杀到谁的进程全凭运气。

## 提议目标文件与内容

**MEMORY.md**（或 AGENTS.md 安全约束节，由用户定）追加一条：

> **kill 纪律（cmdline 归属过滤）**：多仓同名二进制并存（zero-browser 等），杀进程前必须先 `pgrep -af <name>` 核对完整 cmdline，**只 kill cmdline 含本仓路径（/home/lei/work/ZeroWeb-2）或本任务特征（端口/profile 路径）的 PID**；禁止 `pgrep -x <name> | xargs kill`、`pkill -f <name>` 等按名批量杀。清理脚本同样适用（xargs 前必须过 grep -v 兄弟仓路径或白名单过滤）。

## 质量闸门自评

- 具体：可执行的一条命令模式替换。✓
- 可执行：给出核对→过滤→kill 的确定性序列。✓
- 长期有效：与 workspace 无关的进程操作纪律。✓
- 非重复：是现有「kill 指定 PID」规则在多仓场景的补强，覆盖 `pgrep -x| xargs` 这个已实际肇事的形态。✓
- 低风险：不含密钥/隐私。✓
- 证据：本 run.md 事故记录（真实误杀一次）。✓
