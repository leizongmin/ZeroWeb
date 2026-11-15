---
date: 2026-11-15
modules: site_optimizer
---

# 运行账本单副本灭失与变异面失控——v3 整树重建后的灾备与写纪律

## 问题描述

perf-mem run 的 v3 验收账本整树灭失（`.acceptance/site-optimizer/<run>/` 下的
workflow 快照链、checkpoint、回执一次性全部丢失），只能从 file-history、
transcript 与 bench 镜像三路考古，从 rev1 重建（v4）。事故复盘暴露四个结构
性风险，重建后陆续全部真实发作过：

1. 账本单副本无异地冗余——运行目录被误清即全灭；
2. `workflow.json` 只经 Bash/python 脚本变异、无任何文件历史——写坏无回滚点，
   只能靠快照链倒推；
3. checkpoint 文件被原地改写过——破坏了「版本化不可变引用」前提，已绑定
   该 sha 的校验记录全部失配；
4. bench 驱动与结果不入库（`bench/` 按约定不提交）——灭失后集成段证据无
   库内副本。

## 根因分析

账本的所有不变式（rev 链 append-only、CP sha 恒一、回执字节冻结）都建立在
文件不被变异的假设上，但文件系统层面没有任何机制强制它：单副本 + 可变 +
无历史，任何一个进程（包括清理脚本和工具自身的写 bug）都能无声破坏。灭失
不是单点故障，是这四个风险在同一次操作里叠加的结果。写入侧同理：heredoc
无引号 EOF 会剥除反引号（run.md 记录里命令片段被静默改形）、跨用途复用路径
变量会让 os.replace 落错目标——变异面没有护栏时，错误写入与丢失只隔一步。

## 解决方案

- 快照链 append-only 用**写前断言**强制：每个 rev 快照先
  `assert not exists` 再原子落盘（tmp + os.replace），写后回读核对关键
  字段与「其余任务零漂移」。
- checkpoint 一律版本化新文件（`checkpoint-t72-v4.json`、`-v5`），同名重生成
  仅限「未被任何 rev 引用的被拒草稿」，且先改名存证（`*.rejected-*`）。
- 时间戳现取现写（`%:z` 手拼偏移），`finished_at` 取产物 mtime 作实据；
  sha256/40-hex 一律现算，禁止手写。自查不变量：finished_at ≤ 快照 mtime、
  updated_at ≤ 墙钟。
- run.md 写入用**带引号的 heredoc**（`<<'EOF'`）防反引号剥除；账本更新
  脚本禁跨用途复用路径变量，os.replace 前断言目标、写后验证。
- 集成段证据双写：bench 结果落 `bench/results/<task>/` 的同时，把跨 run
  复用的最小证据集（名集存档、.exit 文件）拷贝进运行目录 evidence/，
  账本引用只指向运行目录内副本（校验器拒绝逃逸运行根的路径）。
