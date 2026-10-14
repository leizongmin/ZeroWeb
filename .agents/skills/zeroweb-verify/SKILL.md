---
name: zeroweb-verify
description: ZeroWeb 功能验证入口与地图。按明确范围复用存储生产 API 测试、桌面演示流、WebView、WebDriver、CDP、Chrome parity 和 Android 验收，记录准备、动作、结果、副作用及清理；不默认全图运行。
---

# ZeroWeb 功能验证

本入口覆盖桌面 GUI、Rust API、WebDriver HTTP、CDP 和 Android 操作。按用户任务读
[功能地图](features/index.md)，只跑请求范围。地图存在不代表全图通过，也不自动授权
业务修复、发送通知、发布、提交或推送。不要启动网站优化长任务。

## Launch / Prepare

所有命令在 Git 项目根执行。先读根 `AGENTS.md` 与
[既有命令和门禁](../../../docs/engineering/quality-gates.md)。
源码发现必须过滤 Git ignore，包含已跟踪但现在被忽略的源码：

```sh
git rev-parse --show-toplevel
git status --short
git ls-files -c -o --exclude-standard --deduplicate |
  git check-ignore --no-index --stdin -v -n |
  rg '^::' | cut -f2
```

默认低成本入口不启动浏览器。要求 macOS/Linux、Node.js 20+、Rust/Cargo、`ps` 和
已缓存的 Cargo.lock 依赖；`--offline --locked` 不安装依赖、不联网。唯一新 helper
[run-storage.mjs](scripts/run-storage.mjs) 复用 `scripts/test-guard.rs` 和既有生产存储测试。
没有守卫二进制时，在本次目录编译单文件守卫，自举受 4/8 GiB、60 秒监控；
Cargo 编译和测试分别受既有守卫 4/8 GiB、600 秒保护，不使用无保护的
`--compile-first` 编译阶段。本轮不运行全仓 fmt、clippy、测试或浏览器构建。

```sh
node .agents/skills/zeroweb-verify/scripts/run-storage.mjs "$(date +%Y%m%d-%H%M%S)-storage"
```

参数为新的运行 ID，仅字母、数字、下划线或短横线，最多 80 字符；已有目录会拒绝。
输出在本资产 `.verify-task/<ID>/`；临时路径、Cargo target 均隔离在该目录。
只有执行测试并核对数据恢复断言后，才输出 `PASS`。离线依赖缺失记 `BLOCKED`，
按 [平台准备说明](../../../docs/development/linux-macos.md) 补齐后用新 ID 重试。
不把依赖准备或 `--list` 当功能通过。

GUI/IPC 路径另需当前源码对应的 browser、renderer、compositor、image-decoder。
共享构建和执行体核验遵循 [工程方法论](../zeroweb-guidelines/SKILL.md) 的既有契约，
本入口不新增构建目录约定。需要构建时按既有门禁使用内存与墙钟守卫，
不能为“初始化验证资产”自动触发全仓构建。GUI 需窗口/GPU/字体，Android 需设备。

## Doctor

首次动作、新 session、失败或异常后必须只读检查并重查：

```sh
git rev-parse HEAD
git status --short
node --version
rustc --version
cargo --version
ps -ax -o pid=,ppid=,rss=
df -h .
git check-ignore --no-index -v .agents/skills/zeroweb-verify/.verify-task/probe
```

存储 helper 自动核对项目根、相关工作区差异、依赖、ignore 和工具。它不使用用户
profile，不需要鉴权/seed；生产测试自建固定 origin、schema 和数据，重开读取真实磁盘。
任何检查失败均停止后续动作，不清空用户数据。

浏览器路径先核对二进制及版本、renderer/合成器状态、fixture HTTP 响应和端口归属。
WebDriver 的 `/status`、CDP 的 `/json/version` 只证明接口健康，不证明功能完成。
UI/会话异常时只重启本次实例、重建本次 session，然后重新 Doctor；不得杀用户实例。
端口被占用时选独立端口，不抢占已有服务。记录 locale、viewport、DPR、主题、字体、
GPU 和 JS feature；平台不支持的脚本标 `BLOCKED`，不能用 headless 冒充窗口。

## Drive

[存储流程](features/storage.md) 提供本轮实跑的生产 API 路径。
其他功能从地图跳转到已有 smoke、WebView 测试、CDP、WebDriver 或 Android 清单。
真实点击使用宿主输入/WebDriver/CDP input，不用 `element.click()` 冒充指针动作。
Chrome 完整对比必须重新读取
[parity 入口及证据契约](../zeroweb-browser-chrome-parity/SKILL.md)，由其生产采集器驱动，
不另造协议或比较框架。静态路径、单测、WebDriver CPU 截图不能证明生产 GPU 一致。

## Evidence

运行前必须确认本目录 `.gitignore` 的 `.verify-task/` 生效。保留每次运行的 HEAD、
相关未提交差异、日期环境、输入/前置、命令/动作、断言结果、持久副作用和清理记录。
helper 生成 `result.json`、`compile.log`、`test-list.log`、`test.log`、`guard-build.log`，
记录真实测试 executable 的 SHA-256 后才清理。测试代码决定判据，不能仅按 exit 0 判 PASS。

`PASS` 仅限实跑范围；`FAIL` 为实际结果不符；缺工具/权限/前置为 `BLOCKED`；
其余为 `NOT_RUN`。旧证据保留原版本，不刷新未测入口。机器报告仅记录运行事实；
功能文档说明这些断言对应哪些用户结果和不能覆盖的边界。

## Cleanup

helper 独占新建运行目录，只删除其中 `tmp/`、`target/`、`test-guard`，不删除日志和
`result.json`。正常测试的 `TestDirectory::drop` 先清掉自建数据库；helper 检查空目录，
再清理构建资源。异常也执行本次目录清理并保存错误。
每次清理后实际读取 JSON 和全部日志，记录哈希，证据丢失不能报告完整闭环。

其他路径记录本次 PID、端口、profile、下载文件及设备应用归属，结束本次服务/session，
只清理本次数据。不要执行原 smoke 中会覆盖旧目录的流程，始终使用新输出目录。
不要把恢复默认设置当恢复用户原设置；无法隔离 profile 时停止持久数据操作。

## Feature Map

[索引](features/index.md) 按用户任务组织，不按 crate 代替功能。维护时对照每个功能文档、
实际用户入口、相关 Git 变更和测试断言，检查遗漏、重复、死链接及新增入口。
知识说明从 [zeroweb-knowledge](../zeroweb-knowledge/SKILL.md) 按领域导航，
工程经验另见 [learnings 索引](../../../docs/learnings/INDEX.md)；不复制或改写知识条目。

本次初始化及交接检查见 [初始化记录](initialization.md)。入口和 helper 自测不等于
宿主已自动发现，也不等于未参与编写的执行者独立验收。
