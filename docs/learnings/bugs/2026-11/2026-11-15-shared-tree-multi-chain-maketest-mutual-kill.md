---
date: 2026-11-15
modules: make test, target-disk-guard, rally 无人值守执行流程
---

# 共享树多条 make test 链互踩尽灭——disk-guard rm 并发竞争与指纹撞写

## 问题描述

2026-11-15 jsanim 刀门禁窗口：同一 clone（ZeroWeb-2）内多个并行 session 各自发
`make test`，16:40–16:55 四条链全部死亡，无一产出可作账结果。死亡形态三种：

1. **artifact-127**：workspace 腿 `test-guard --compile-first` 编译清单成功，但逐
   artifact 启动时报「无法启动测试 artifact: No such file or directory (os error
   2)」——编译完成与 exec 之间产物文件消失。
2. **指纹撞写**：第三方 `cargo test -p <crate> <单钉>` 复跑时报「could not
   parse/generate dep info … failed to write .fingerprint/... No such file or
   directory」——fingerprint 目录在写入窗口被移除。
3. **disk-guard rm 竞争**：disk-guard 全量清空 `target/` 时报「rm: cannot remove
   'target/debug/deps': Directory not empty」→ make 在 `Makefile:53
   target-disk-guard` 即 Error 1，测试未启动。

## 根因分析

`target/` 是全树共享的单一构建目录，cargo 内部锁只串行化单次构建内的目标级竞争，
不保护跨链的三类窗口：

- make test 链 A 的 `cargo test --no-run` 刚列完 artifact 清单，链 B 的 disk-guard
  开始分级/全量清理 `target/debug/deps`——清单里的可执行文件在 exec 前被删
  （artifact-127）。
- 链 A 正在写 `.fingerprint/<hash>/dep-lib-*`，链 B 的清空删掉父目录——ENOENT。
- 两条链同时触发 disk-guard（阈值各查各的），两者并发 `rm -rf` 同一目录树，
  互相删对方的遍历目标——「Directory not empty」是 rm 遍历中被并发写入方重新
  填充该目录的竞态标志。

深层原因：无人值守 session 各自「查活跃单元→发射」间隔内，对方链可能刚启动
（本例：16:57:44 与 16:58:20 两次发射相隔 36 秒），基于进程清单的避让存在固有
窗口竞态；且 make test 冷编译 + disk-guard 清理的组合把窗口放大到分钟级。

## 解决方案

1. **单链收敛契约**：共享树同一时刻只允许一条 make test 链。多 session 间经
   显式消息协商（发射方广播 unit 名 + 日志路径 + 「我方只监视不发射」承诺），
   其他 session 停掉自己后发的链、只挂 watcher 看 `MAKE-EXIT=`。
2. **先见者收口**：MAKE-EXIT 落盘后先见者负责把结论写入控制面（纯 docs 小提交、
   显式 pathspec），避免多 session 重复写账。
3. **守卫形态**：单钉归因复跑在链窗口内一律延后；非跑不可时用 test-guard 轻形态
   且接受被指纹撞写杀死后不重试（重试即第三条链）。
4. disk-guard 自身的并发安全（flock 或单飞语义）归 build-support 流另修；本条
   只约束调用方纪律。

## 验证

单链收敛后 jsanim6（16:57:44 起）全量跑通至 workspace 腿 Error 101（在册代码红），
fmt/clippy 绿、167 条 test result 全部产出——同一树此前四链互踩零产出。
