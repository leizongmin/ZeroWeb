# 进程级 Paint 开关不应在 fragment 热路径读取

- 日期：2026-08-17
- 相关模块：`zero-engine` text paint 与 variable-font metrics

## 问题描述

文本绘制会为每个 fragment 判断 fallback、author fallback、shaped layout、adjusted generic advance 和 variable-font 策略。旧实现每次都调用 `std::env::var`，medium 页面 profile 中 `getenv` 占全帧约7.7%，其中多个 paint 调用栈可直接归因于这些进程级开关。

## 根因分析

这些环境变量只在进程启动前用于选择兼容策略，运行中没有动态更新需求。fragment 热路径却把它们当作实时数据源，反复进入 libc 并复制、分配和释放字符串。调用次数随文本 fragment 数增长，而策略值始终不变。

## 解决方案

用共享 helper 和各开关独立的 `OnceLock<bool>` 按进程读取一次。`ZW_PAINT_ENV_SNAPSHOT=0` 让所有新增快照恢复旧 live lookup，便于同一 release 二进制 A/B 和紧急回滚。原本要求 live lookup 的 `ZW_SHAPED_TEXT` 不纳入快照，避免借性能修改扩大行为语义。

通用规则：先按配置的真实生命周期选择快照边界。进程级策略用进程快照，事务级策略用事务快照；总回滚开关必须恢复被优化调用点的原读取语义。

## 验证

闭包计数测试证明快照路径连续读取两次只执行一次 reader，回滚路径执行两次且不写缓存。两组可比 medium A/B 的 paint p95 从 `195.80→175.51ms`、`198.72→177.14ms`，p50 从 `145.46→122.77ms`、`149.38→128.05ms`。profile 中 `getenv` self 从7.87%降至6.09%，fallback、size-adjust 和 variations 的 getenv 调用栈退出。reftest `687/687`，welcome `16.61%`，产品、完整测试与性能绝对门通过。
