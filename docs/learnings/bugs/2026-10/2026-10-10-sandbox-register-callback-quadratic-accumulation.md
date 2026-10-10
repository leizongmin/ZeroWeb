---
date: 2026-10-10
modules: script-sandbox,webview,engine
---

# 沙箱宿主回调重复注册只增不减 → 整页脚本二次方退化

## 问题

`html_compat::deterministic_short_action_replay_across_hosts`（webview 宿主 `dispatch_user_action` 路径，20 轮 × 4 动作）挂死：120/150/240/600s 全部超时，主线程 R 态、utime 持续增长。三个反直觉特征误导了初期排查方向：

1. **挂死点漂移**——两轮复现分别停在第 32 / 第 49 个动作，不同的动作类型（Activate vs InsertText），像状态依赖的死循环；
2. 上一动作内**同型脚本刚成功**（beforeinput native dispatch 绿，textInput native dispatch 挂）；
3. 挂死脚本走的 parent 链、事件派发路径与成功时完全相同。

## 根因

不是死循环，是**二次方退化**：单脚本执行成本随累计脚本数线性增长，整页 O(n²)。「挂死点」只是累计成本越过 timeout 的漂移截面。

累积源在 `script-sandbox`：

- 两生产引擎 `register_callback` 对同名重复注册**只 push 不去重**：V8 侧 `HOST_CALLBACKS`（thread-local `Vec<HostCallback>`）与 `self.callbacks`（`Vec<(String, usize)>`）双表同增；QuickJS 侧 `callbacks` 同型。
- `execute` 每次执行**按注册史全量重装**全局函数：V8 对每条 `(name, idx)` 建 `FunctionTemplate` + `global.set`；QuickJS `install_callbacks` 同型。
- 调用方 webview `execute_dom_script` **每次脚本执行调 `register_dom_callbacks` 两遍**（首注册 fresh mutations 队列 + 尾注册回共享队列），每遍数十个 `__zw_*` 回调 → 每脚本 +2K 条目，execute 重装 O(S)，整页 O(S²)。实测 445 脚本后单脚本 ~1s（正常 µs 级）。

缺陷成立需要三个条件同时满足：接口允许重复注册、注册表 append-only、执行入口全量重装。三者分开看都「无害」，组合起来是隐性 O(n²)。stub 实现（fetch_bridge/ws_bridge 的测试沙箱）恰好都写成了替换语义，进一步掩盖了生产引擎的问题。

## 解决方案

双引擎 `register_callback` 同名重注册改**替换语义**（最后注册者胜）：

- V8：`self.callbacks` 查同名条目，命中则复用其自有槽位（`HOST_CALLBACKS` 槽位为 `Option<HostCallback>`，原位替换就地 drop 旧闭包）；未命中才 append。`V8Sandbox::drop` 同步改 **own-slot 清空**（只把本 sandbox 注册过的槽位置 None），不再整表 `clear_host_callbacks()`——该函数保留为公开 API，仅限确认整线程无存活 sandbox 的拆卸场景。
- QuickJS：`iter_mut().find` 原位替换 `slot.1 = cb`（`callbacks` 为 per-sandbox `Vec`，无 thread-local 共享，无 own-slot 面）。

**语义影响（如实记录，首版修复声明被审查证伪后更正）**：单 sandbox 场景替换语义与修前可见行为一致——修前 `execute` 全量重装时后安装条目本就覆盖前者（最后注册者胜），仅消除表累积。**同线程多 sandbox 场景语义有变**：旧 `Drop` 整表清空会杀死幸存 sandbox 的宿主回调（invoke 静默退回 `""`），且幸存者陈旧槽位索引在表重填后会越界 panic（盲写复用槽位）或撞写他人活槽位（跨实例回调被静默替换）；own-slot 清空 + 槽位所有权（`Vec<Option<HostCallback>>`、洞不复用、表长上界 = 线程历史不同名字注册总数）后幸存者回调保持可用——这一行为变化正是审查发现 D1/D2 的修复本身，不是回归。

钉测：双引擎各一「50 次重注册 + 1 异名」（表长断言 + latest-wins + 异名不受影响，红态回退 append-only 双红）；V8 侧另加三钉覆盖 D1/D2（他 sandbox drop 后重注册不 panic、他实例重注册不撞写本实例槽位、空 sandbox drop 后幸存者回调仍可 invoke），红态为整表清空 + 盲写槽位的旧实现。

## 如何避免

- **重复注册接口必须定义替换语义**：凡「注册/挂载/订阅」类 API，同名重复调用的行为要么显式替换、要么显式报错，不能静默 append。append-only 表 + 全量重放入口的组合，写代码时无任何单点看起来慢，只有累计曲线暴露。
- **诊断「挂死」先区分死循环与二次方退化**：给逐次操作打**带时间戳**的日志看耗时曲线——死循环是定点突变（某次永不返回），二次方退化是单调渐增（每操作都变慢）。本次三处插桩（轮次计数 → 脚本级 begin/done → elapsed 时间戳）逐层收窄，时间戳曲线一步定性。
- **退化定位看「哪一类操作变慢」**：全部操作等比变慢 → 共享层累积（本例：execute 回调重装段，先于任何脚本本体）；只有特定类型变慢 → 该路径状态累积。uniform slowdown 直接把嫌疑从 shim/渲染管线拉到沙箱执行入口。
- **结构性既有缺陷会伪装成「近期回归」**：挂死漂移点 + 近期 merge 窗口重叠，极易误归因新提交（本案初判 S4O/t8n 嫌疑）。base 对照只能证明「base 也挂」，不能证明「窗口引入」——同一缺陷在更早窗口可能因测试序/时序未触达越界点。
