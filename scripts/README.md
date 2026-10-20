# 脚本入口

Rally 脚本按用途命名：

- `rally-goal-*.sh`：执行指定 goal。目标名称、范围和启动门控见脚本中的 `docs/goal/*.md`；目标归档后可连同失效引用清理。已有编号保留。
- `rally-cron.sh`：启动定时任务调度器，配置在 `docs/rally/jobs.yaml`。

```sh
# 预览 goal 执行命令
./scripts/rally-goal-rendering-compat.sh --dry-run

# 启动该 goal
./scripts/rally-goal-rendering-compat.sh

# 校验调度配置并显示触发时间
./scripts/rally-cron.sh --dry-run
```

改名后的 goal 脚本保留原有命令、agent 和参数透传行为。构建、测试与运行守卫见 [质量门禁](../docs/engineering/quality-gates.md)；Ladybird 后续目标见 [清单与启动说明](../docs/goal/ladybird-followups/README.md)。
