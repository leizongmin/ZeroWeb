# M4：全链路串测与平台交付审计（DC-1..DC-4 串联 + DC-5 机制面）

**日期**：2026-10-22
**HEAD**：`9ce652d88b13dc9d1301947e39e995cddd04d6f8`（含本轮串测暴露缺陷的修复提交）
**对应**：DC-1（机制面安装/版本/卸载）、DC-2/DC-3/DC-4（已安装产物上的串联复验）、DC-5（所选平台审计与清理审计）
**产物与脚本**：[run-m4-full-chain.sh](run-m4-full-chain.sh) / 原始输出 [m4-chain-run.log](m4-chain-run.log)（**31 断言全 PASS**）

## 1. 串测设计与产物

单一脚本在"已安装产物"上串联完整交付链：安装 → 版本追溯 → 更新应用 → 崩溃报告 → 崩溃循环守卫与恢复 → 反馈导出 → 卸载 → 清理审计。每腿消费前一腿产物（更新消费安装根、崩溃报告消费更新后的运行态、导出消费崩溃报告、卸载后核验用户数据存活）。全程不注入 `ZERO_UPDATE_INSTALL_ROOT`——顺带端到端验证 `current_exe` 默认安装根推导；隔离仅靠 `XDG_CONFIG_HOME`/`ZERO_STORAGE_DIR`/`ZERO_CACHE_DIR` 三个环境变量。

两个 .deb 均从 HEAD `9ce652d88b13` 构建（`ZERO_BUILD_VERSION` 区分版本，commit 由打包脚本自动注入）：

| 包 | 用途 | SHA256 |
|---|---|---|
| `zero-browser_26.10.22_amd64.deb` | 初始安装 | `d3a9a82ed58f689dee2d8671ff41f9c3583c83541f960d349f591a5239e9ccc6` |
| `zero-browser_26.10.23_amd64.deb` | 更新包（本地源） | `fef43af37f4b2ab931b592c6df2b1608fb093094314ecb5aefbbaac32b5238b4` |

运行守卫：`systemd-run --user --scope -p MemoryMax=8G timeout 900`（串测）/ `-p MemoryMax=24G`（打包），工作目录 `/tmp/zeroweb-m4-chain/`（运行后清理）。

## 2. 串测发现并修复的产品缺陷（安装根推导）

首轮串测 leg3 FAIL 暴露真实缺陷：`--check-updates` 输出 `UPDATE_APPLIED 26.10.23 (previous none)` 但 `current` 链接不翻转、活跃版本仍 26.10.22。根因：`update_install_root()` 默认推导按 `<root>/current/usr/bin/zero-browser` 固定上溯三级，而 `/proc/self/exe` 恒解析符号链接到 `<root>/releases/<版本>/usr/bin/zero-browser`——`current` 分量消失后 nth(3) 得到 `<root>/releases`，更新被应用到 `<root>/releases/releases/`。S2 验收全程 `ZERO_UPDATE_INSTALL_ROOT` 注入，默认推导从未被端到端覆盖。

修复（commit `9ce652d88`）：从解析后 exe 路径向上定位最近 `releases` 祖先、取其父目录；无 `releases` 祖先（平铺前缀）返回 `None` 走 `UPDATE_REJECTED INSTALL_ROOT_UNKNOWN`，不再静默选错根。回归测试 `install_root_derives_from_nearest_releases_ancestor` / `install_root_is_none_without_releases_ancestor`。首轮 FAIL→修复→复跑全绿，是本串测的直接产出。

## 3. 链路走查结果（8 腿 31 断言全 PASS）

| 腿 | 场景 | 关键断言 | 结果 |
|---|---|---|---|
| leg1 | 初始安装（`dpkg-deb -x` 前缀隔离，`dpkg -i` 代理） | 四二进制 + desktop 齐备；`current → releases/26.10.22` 相对链接 | PASS ×4 |
| leg2 | 版本追溯（IF-1，安装内运行） | `--version` = `ZeroBrowser 26.10.22 (commit 9ce652d88b13)`，与宿主 git 逐字符一致 | PASS |
| leg3 | 更新应用（DC-2 串测面，无 env 覆盖） | `UPDATE_APPLIED 26.10.23 (previous 26.10.22)`；current/rollback 翻转；更新后 `--version` 26.10.23 且 commit 不变；无 staging 残留 | PASS ×5 |
| leg4 | 崩溃报告（DC-3 串测面，安装内 v2 GUI） | SIGKILL renderer → 报告落盘；报告 `version`=26.10.23、`git_commit`=9ce652d88b13；隐私 grep 零命中；`crash_count=1` 即时落盘 | PASS ×4 |
| leg5 | 崩溃循环守卫 + 恢复（DC-4 串测面） | 第二刀 `crash_count=2`；SIGINT `clean_exit=true`；重启自动恢复跳过崩溃 tab；观察窗 10s 计数清零；再次 SIGINT 干净退出可重复 | PASS ×5 |
| leg6 | 反馈导出（FR-5/IF-4 串测面） | `--list-crash-reports` 2 行格式正确；目录包 reports 逐字节一致；`environment.json` version/commit 绑定安装内 v2；摘要默认无 URL；整包隐私零命中；重复导出拒绝 | PASS ×4 |
| leg7 | 卸载完整性（DC-1 卸载机制面） | 删安装根后安装面消失；`session.json`（clean_exit=true、1 tab）、2 份崩溃报告、已导出包全部存活——卸载只删包提供面、不动用户数据 | PASS ×4 |
| leg8 | 清理审计（DC-5 机制面） | 无 staging 残留；无残留 browser/renderer 进程；真实 `~/.config/zeroweb` 零新增写入；仓库 tracked 树零污染 | PASS ×4 |

**版本/commit 贯穿线**：`--version`（leg2 v1 / leg3 v2）→ 崩溃报告 JSON 字段（leg4）→ `environment.json`（leg6）四处均为 `9ce652d88b13`，更新前后 commit 一致、版本 26.10.22→26.10.23 随包翻转——IF-1 追溯链在安装、更新、崩溃、导出四个环节闭环。

## 4. DC-5 审计：所选平台闭环证据与未支持平台

### 4.1 所选平台（Linux / .deb 本地源）证据链核对

| 判据 | 机制层证据 | 串测复验 |
|---|---|---|
| DC-1 安装/版本/卸载 | [M1 §3](../2026-10-22-m1-inventory-baseline/README.md)（隔离安装+CDP 版本查询）、[S1](../2026-10-22-m3-s1-version-traceability/README.md)（`--version`+commit 注入） | 本轮 leg1/2/7（安装→追溯→卸载保数据） |
| DC-2 更新/中断/回滚 | [S2](../2026-10-22-m3-s2-update-rollback/README.md)（六腿 22 断言） | 本轮 leg3（默认安装根推导下的更新串联） |
| DC-3 崩溃报告与导出同意 | [S3](../2026-10-22-m3-s3-crash-report/README.md)、[S4 leg B](../2026-10-22-m3-s4-feedback-loop-guard/README.md) | 本轮 leg4/6 |
| DC-4 会话恢复 | [S4 leg A](../2026-10-22-m3-s4-feedback-loop-guard/README.md) | 本轮 leg5 |

机制层闭环成立：单一脚本内八个环节全部走通且互相消费产物。**NOT_RUN 部分（非机制层）**：真机 `dpkg -i` 系统安装、`dpkg -r` 真实卸载、签名渠道验收——按 P1（签名证书）/P4（系统安装）授权节奏，本地模拟不冒充公开发布验收（RFC §8）。

### 4.2 macOS / Windows 未支持渠道与理由（引 [M1 §2 矩阵](../2026-10-22-m1-inventory-baseline/README.md)）

| 平台 | 已有面 | 未支持面与理由 |
|---|---|---|
| macOS | .app+zip 打包脚本（`package-macos.sh`）；ad-hoc 签名默认、`MACOS_SIGN_IDENTITY` 可真签 + `--notarize` 公证 | 更新/崩溃报告/反馈导出管线未实现（`update.rs` 依赖 dpkg-deb 的 Linux 布局）；本 Rally 环境为 Linux x86_64，无 macOS 运行时，无法产生真实产物证据。归 P1–P5 授权后的后续里程碑 |
| Windows | 目录+zip 打包（`package-windows.ps1`）；exe 资源版本信息 | NSIS 仅生成脚本不编译、无 Authenticode 签名；更新/崩溃报告管线同样未实现；无 Windows 运行时。同上挂账 |

RFC D1 已定 Linux 为首平台，macOS/Windows 的渠道/签名/报告源决策在 M2 RFC §8 待决清单，不属于本轮验收语义。

### 4.3 清理审计

leg8 四项断言（staging / 进程 / 真实配置目录 / 仓库工作树）全 PASS；串测工作目录 `/tmp/zeroweb-m4-chain/` 运行后删除；.deb 产物保留在 `target/packages/`（gitignore 域，goal 契约"保留旧安装产物"）；无残留进程。

## 5. 限制与挂账

- 真机系统安装/卸载/签名（DC-1 剩余面）NOT_RUN：P1/P4 授权门控。
- 反馈包为目录形态（IF-4 偏差，RFC §3 预授权回退，无 zip 依赖）；GUI 菜单导出入口未做。
- G3（无 `--profile` CLI）/G4（headless 无优雅关停）/G5（headless 不消费启动 URL）沿用 M1 挂账；本轮串测走 GUI + env 隔离，不依赖三者。
- 更新后新版本经重启生效：本轮 leg3→leg4 的重启即该语义的真实验证（S2 §5 挂账项在 GUI 路径闭环；headless 路径仍属 G4）。

## 6. 门禁

fmt PASS / `cargo test -p zero-browser --bin zero-browser install_root` 2/2 PASS / `make test` 311 passed / 0 failed（test-guard 包裹）/ clippy `--workspace --all-targets -D warnings` 零告警 / 提交前 lei-pre-commit-guard PASS（修复提交 `9ce652d88` 与本文档提交各一次）。
