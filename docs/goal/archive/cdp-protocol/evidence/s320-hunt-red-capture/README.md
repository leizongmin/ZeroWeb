# S320 — hunt 遗留网 RED 捕获取证与排除（2026-09-14）

## 来源

S316 会话（提交 736f16525，09:08:58）的延伸负载 hunt 活动，从未记入任何轮次，
S320 轮（2026-09-14 09:45 发现）首次记档。全部产物原位于 `/tmp/`（易失），
本目录为保全副本。

## 时间线（文件 mtime + 提交时间戳重建）

| 时刻 | 事件 |
|------|------|
| 08:46–08:48 | S316 平静态全帧追踪 zw-trace-run1..8（已记入 S316，未归档此处） |
| 08:50:20 | `zw-loop2.sh` 启动（3 busy-loop 载荷 + ZW_IPC_TRACE + ZW_IPC_VALIDATE，10 轮） |
| 09:01 | **RED 触发**（GREEN≠33）→ `zw-trace-RED.log` 落盘（212,946 B），loop2 按设计 kill 载荷后退出 |
| 09:01:51 / 09:02:49 | 并行渲染流 R4331 代码/文档提交（23a20a84f / 74e5bc8ab） |
| 09:02 | `zw-loop3.sh`（无 busy-loop + trace + 每轮 timeout 90s，6 轮） |
| 09:03 | loop3 期间 `/tmp/zw-trace-hunt.log` 再写（212,946 B，与 RED log 不同 pid、108/1732 行差异） |
| 09:04 | `zw-loop4.sh`（无 trace、仅 VALIDATE（内存走查近零开销），4 轮） |
| 09:05 | `/tmp/zw-hunt-stderr.log` 最后写入（**0 字节**） |
| ~09:05–09:40 | 会话收尾，loop2 的 3 个 busy-loop 子壳未获清理而孤儿化（PPID=1） |
| 09:08:58 | S316 提交（其「hunt 负载循环全收尾零遗留」记账对 loop2 不准确） |
| 09:11 / 09:20 / 09:34 | S317 门 + make test 两腿**在孤儿 3 busy-loop 载荷下绿跑** |
| 09:40–09:43 | S317/S318/S319 提交（三轮「零遗留进程」记账均不准确） |
| 09:45 | S320 发现孤儿（3 × /bin/bash zw-loop2.sh，各 ~99% CPU，56 分钟） |
| 09:51 | S320 kill 三个孤儿 busy-loop，pgrep 清零 |

## RED 判定分析（双证据排除损坏家族）

RED 触发条件是 `GREEN≠33`（**非** stderr corruption 标记）：

1. **stderr 0 字节**——`zw-trace-RED-stderr.log` 与 `zw-hunt-stderr.log` 均为
   0 字节：无 `Deserialization error`、无 `ipc reader terminated`、无
   `malformed frame buffer`、无 ZW_IPC_VALIDATE 告警（S315 损坏形态的
   标志性证据链在本捕获中完全缺席）。
2. **trace multiset 干净**——`zw-trace-RED.log` 解析 866 W + 866 R，
   按 hex multiset 对照 written-not-read = 0、read-not-written = 0、
   不可解析行 = 0。写集 = 读集，零 chimera 零丢失。

结论：**#0 损坏家族未被捕获**；RED 属 S78 家族 GREEN≠33 负载触发形态
（traced + 3 busy-loop 载荷组合下捕获步失败/计步偏差，具体失败步不可恢复——
`out/steps-report.json` 已被 S317 门等后续运行覆盖）。「≥2 并发 hunt 实例
共写共享 out/ 与 /tmp trace 文件」可解释 GREEN 判定不可靠，但文件时间线
同样可由串行实例解释（loop2 09:01 退出 → loop3 09:02 起），并发竞争无法
证实亦无法排除，不作唯一归因。

## 后续 hunt 结果（未触发）

- loop3（traced，无载荷）：6 轮 0 RED。
- loop4（无 trace 仅校验器）：4 轮 0 RED。
- `zw-trace-RED.log` 未被覆盖（mtime 维持 09:01）→ loop2 之后零 RED。

## 对 S78/#0 记账的影响

- **#0（renderer 写侧组装点流损坏）维持开放**：本轮负载窗 RED 为 GREEN≠33
  形态，双证据排除损坏，无组装现场产出。
- **S78 家族（负载触发 GREEN≠33）新增一例观测**：traced + 3 busy-loop
  组合触发、平静态（S316 0/18+）与无载荷 traced（loop3 0/6）、
  无 trace 校验器（loop4 0/4）均不触发。
- **S317 门（09:11）与 make test 两腿（09:20/09:34）在孤儿载荷下绿跑**——
  门 + 全量测试对 3 busy-loop 载荷稳健，间接收窄 S78 家族触发条件
  （hunt 式 traced browser 会话载荷 > 常规门载荷）。
- **方法论教训（后续 hunt 遵守）**：GREEN≠33 判定读共享
  `tests/playwright-matrix/out/steps-report.json`，且各实例共写同一
  `/tmp/zw-trace-hunt.log`（每轮 `rm -f`）——hunt 必须**串行单实例**执行；
  需要并发/带载时，每实例独立 `out/` 与 trace 路径，否则 RED 判定自身不可靠。
- **机器卫生修正**：S316「hunt 负载循环全收尾零遗留」、S317/S318/S319
  「零遗留进程」记账与事实不符（孤儿 08:50:20 起在跑，至 09:51 清理），
  以本轮为准确记录；此前三轮检查模式（zero-browser/renderer/compositor
  进程名匹配）无法覆盖 bash hunt 脚本遗留，后续卫生检查加入
  `pgrep -af 'zw-loop|zw-hunt'` 模式。

## 文件清单

- `zw-loop2.sh` — RED 所在 hunt（3 busy-loop + trace + validate，10 轮）
- `zw-loop3.sh` — 后续迭代（无载荷 + trace + timeout 90s，6 轮）
- `zw-loop4.sh` — 后续迭代（无 trace 仅校验器，4 轮）
- `zw-trace-RED.log` — RED 时点双侧逐帧 hex 追踪（866 W + 866 R，multiset 干净）
- `zw-trace-hunt.log` — loop3 期间再写的追踪（不同 pid，108/1732 行差异，同样干净）
- stderr 捕获（`zw-trace-RED-stderr.log` / `zw-hunt-stderr.log`）均为 0 字节，
  即其证据内容，不再存空文件。

## 保真注记

- **路径净化**：三个脚本首部 `cd` 的仓根绝对路径已替换为
  `${ZW_REPO_ROOT:-.}` 占位（其余内容逐字节保留）；原始为
  `<repo-root>/tests/playwright-matrix` 形式。
- **zw-loop2.sh 的 bash -n 状态**：现档（含 /tmp 原件）在 line 45 报引号
  失配 EOF，但该脚本确曾在 08:50:20 以 `/bin/bash /tmp/zw-loop2.sh` 运行
  并产出全部在案输出——最一致的解释是 on-disk 内容曾在运行中被编辑
  （bash 对脚本增量读取，已启动实例不受后续编辑影响），这也与后续以
  新文件（loop3/loop4）而非原地编辑迭代的形式吻合。归档保留现状，
  不代为修复。
