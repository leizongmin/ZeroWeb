# 初始化记录

以下为初始化时的历史记录；正式资产随后已在
`d0f281d509de3bf4f9dcd7e8dd01dac3464a663b` 提交，原运行版本、回执和7/7结果不刷新。
后续局部维护范围见 [功能地图](features/index.md)，不作为新的产品实跑证据。

2026-10-14，重命名后的 `zeroweb-verify` 入口已由未参与编写的执行者独立实跑：
从入口和地图选择 F07 存储生产 API，7/7 通过，准备、观察和清理闭环完成。
证据已独立复读并核对哈希，无本次资源或进程残留。
仅这 7 项记 PASS；GUI、Chrome parity、页面 JS/IPC、WebStorage/cookies、OPFS、
Service Worker、媒体、Android 和其他地图功能均未在本轮执行，不记 PASS。

## 重命名后的独立验证

新运行 ID `20261014-115126-storage-independent`，
时间 2026-10-14 03:51:26.764–03:51:59.884 UTC，
HEAD `e523b1c73b96360a6451948f0b748b7b4657750f`；
macOS 26.6.2 arm64、Rust/Cargo 1.98.1、Node 24.15.0。
存储相关源码及 Cargo/lock 运行前后无差异，其他并行工作区改动保留，不归为 HEAD。

- [独立交接](.verify-task/20261014-115126-storage-independent/handoff.md)：
  从入口获得操作答案的顺序、Doctor、实际命令、断言范围、证据复读和名称检查。
- [本轮机器证据](.verify-task/20261014-115126-storage-independent/result.json)、
  [本轮实际日志](.verify-task/20261014-115126-storage-independent/test.log)：
  7 passed / 0 failed / 0 ignored / 710 filtered out，测试耗时 0.15 秒。
- 清理后 tmp/、target/、test-guard 均不存在，4 份日志全部可读且 SHA-256 与机器记录一致，
  JSON 可解析，无归属进程；未清理用户服务、数据或既有构建产物。
- Git ignore 过滤后检查 4307 个文件，旧名称引用为 0；旧名目录不存在，
  `zeroweb-verify` 与 `zeroweb-knowledge` 的目录和 frontmatter name 相符。
  ignored 历史证据不在名称扫描范围，知识仅检查名称，不作功能验收。
  不声称宿主自动发现成功，不创建固定名 current-project 入口。

独立执行前未读取作者 helper 实现、初始化结论或历史 run；运行后才读取本记录以增补事实，
并复核生产测试断言。本轮正式资产仅增补本记录，证据只写入上述新运行目录；
未改业务、AGENTS、知识或历史 run，未提交或推送。

## 原初始化自测

以下保留原作者自测记录；其运行版本、证据和未测边界不因独立复验而刷新。

## 已完成

1. 已完整读取 lei-project-verify 的入口、format/create、仓库原 AGENTS，
   并在主 agent 更新后重读当前 AGENTS 与 `docs/engineering/quality-gates.md`。
2. 已核对 README、实际用户入口、既有测试与验收流程，源码清单通过 Git ignore 过滤，
   包括 tracked-but-ignored；没有扩展扫描 ignored 源码/构建目录。
3. 已建立入口、功能索引和 8 份分组操作文档。复用 M1~M4、WebView、WebDriver、
   CDP 矩阵、Chrome parity 及 Android 清单，不搬移或改写旧资产。
4. 已实跑 F07 的 7 个生产持久化测试。helper 的独立 target 只是本次资源隔离选择，
   共享构建按 zeroweb-guidelines 原契约，不新增 GUI/IPC 构建目录限制。

## 本次证据

运行 ID `20261014-storage-init`，2026-10-14 03:30:28~03:31:29 UTC。
HEAD `e523b1c73b96360a6451948f0b748b7b4657750f`；
Cargo/lock、storage/runtime-config 源码运行前后无差异。
开始记录中 AGENTS 暂时处于并行删除/重建阶段；不影响所测生产源码，
该未提交规则变化不能归为 HEAD 内容。
macOS 26.6.2 build 25G83、arm64，Rust/Cargo 1.98.1、Node 24.15.0。

- [result.json](.verify-task/20261014-storage-init/result.json)：版本、命令、测试名、身份哈希、清理。
- [test.log](.verify-task/20261014-storage-init/test.log)：7 passed / 0 failed / 0 ignored，
  710 filtered out，测试动作耗时 0.17 秒。
- [compile.log](.verify-task/20261014-storage-init/compile.log)：当前源码离线/锁定编译和 artifact JSON。
- [test-list.log](.verify-task/20261014-storage-init/test-list.log)：所选模块恰有 7 项测试。
- [guard-build.log](.verify-task/20261014-storage-init/guard-build.log)：守卫成功编译，无输出属正常。

测试确实调用 `StorageManager`/持久化实现，在磁盘落盘后重建 owner，读取恢复状态。
覆盖 schema/记录/索引/生成键与 origin 隔离，缓存名称/状态/头/body 恢复及删除，
写失败保留旧 live state、损坏文件拒绝、备份恢复/orphan 清理。
没有 mock；未经过浏览器页面 JS、renderer IPC、GUI、真实网络或设备输出。
详见 [F07 判据](features/storage.md)，其余功能保持 NOT_RUN。

## 清理

正常测试 Drop 清理自建数据库，helper 实际观察 `tmp/` 空目录。
本次 tmp/target/守卫已删除，result.json 记录 remainingOwnedResources 为空。
JSON 和全部日志清理后已重读并保存 SHA-256，证据目录按局部 `.gitignore` 忽略；
正式入口、helper 和地图不在此忽略范围内。
本轮没有常驻服务或浏览器实例；不清理用户既有服务、profile 或构建产物。

## 可交接性与缺口

交接为作者自测，不声称独立执行者验收或宿主自动发现成功。后续执行者从
[入口](SKILL.md) 按任务选择 [地图](features/index.md)，使用新运行 ID；
存储 helper 可重建守卫、离线编译、核对 7 项、驱动并清理，不依赖本次已删除二进制。
本机之外需 Node/Rust、ps/kill 与离线 Cargo 依赖；Windows helper 尚不支持。
GUI 脚本的 Linux display/平台 profile 差异、生产进程新鲜度、GPU/字体必须另行 Doctor；
没有本轮环境证据时不能把这些路径写为 PASS。
`zeroweb-knowledge` 入口尚不存在时不添加链接，不用目录存在宣称可用。

静态交接检查已完成：11 份 Markdown 的 80 个本地链接、F01–F12 地图、
helper 语法、`git diff --check`、局部 ignore 与 4 份日志哈希复核均通过。
无本次运行进程残留。[自检记录](.verify-task/20261014-storage-init/handoff-check.md)
保存方法与边界。证据只留在当前工作区，新 clone 不会携带 ignored 日志，
需重跑或另行交接原证据目录；不能从文档文字复制 PASS。
