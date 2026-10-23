# M1：交付能力盘点与隔离本地基线（Linux 候选）

**日期**：2026-10-22
**HEAD**：`ac4c7fc59d286c0c9b28f78465301ca5a5933f1f`（工作树干净，无本地改动）
**平台**：Linux x86_64（WSL2 / WSLg，kernel 6.18）
**环境**：39G 内存（守卫 24G systemd scope）、磁盘 749G 可用；cargo release 工具链
**范围**：goal M1——安装/更新/恢复能力矩阵 + 隔离本地基线 + 首个平台候选定版

## 1. 盘点方法

- 源码盘点：`scripts/package-{linux,macos,windows}` 三脚本、`apps/browser/src/main.rs` CLI 面、`crates/browser-shell`（session/profile）、`crates/storage`、`crates/net`（cookie/disk cache/UA）、`crates/product-version`、`build-support/product_version.rs` 全文核对，引用均标注 file:line。
- 运行基线：`./scripts/package-linux.sh --deb` 从 HEAD 构建 .deb（systemd scope 守卫 `zw-delivery-m1-pack`，MemoryMax=24G），隔离前缀解包后以隔离 HOME/env 启动 headless，`/json/version` 查询版本。

## 2. 交付能力矩阵（盘点结论）

| 能力 | Linux | macOS | Windows | 证据（file:line） |
|---|---|---|---|---|
| 打包产物 | .deb（dpkg-deb，装 `/usr/bin`）+ AppDir（appimagetool 可选） | .app + zip（ditto） | 目录 + zip；`-Installer` 仅生成 NSIS 脚本不编译 | scripts/package-linux.sh:206-317 / package-macos.sh / package-windows.ps1 |
| 安装 | `dpkg -i` 系统级（无隔离选项）；AppImage 免安装 | 拖 .app | 解压；NSIS 未接 | 同上 |
| 卸载 | `dpkg -r`（仅删包内文件，不触用户数据） | 删 .app | 删目录 | .deb control 无 privesc/postrm 用户数据操作（package-linux.sh:238-252） |
| 版本方案 | 构建日期 `YY.M.D`（`ZERO_BUILD_VERSION` → `SOURCE_DATE_EPOCH` → 当日本地日期） | 同左 | 同左 + exe 资源 FileVersion/ProductVersion | build-support/product_version.rs:16-39；apps/browser/build.rs:9-27 |
| 版本查询面 | UA `ZeroWeb/<ver>`（crates/net/src/client.rs:516-524）、CDP `Browser: ZeroWeb/<ver>`（apps/browser/src/headless/discovery.rs:129）、内部页脚注（apps/browser/src/pages.rs:705）；**`--version` CLI 未实现**（main.rs:180-466 无分支，未知 flag 静默忽略） | 同左 | 同左 | **缺口 G1：无可执行版本查询；G2：版本不含 git hash/构建号，同日多构建不可区分** |
| 签名 | 无 | ad-hoc 默认，`MACOS_SIGN_IDENTITY` 可选真签 + `--notarize` 公证 | 无 Authenticode | package-macos.sh:24-27；linux/windows 无签名步骤 |
| 更新 | 无（全仓无 updater/version-check 代码） | 无 | 无 | grep "updater/update-check" 仅命中 ServiceWorker 规范机制 |
| 中断回滚 | 无 | 无 | 无 | 无对应代码面 |
| 崩溃报告 | 无 panic hook、无报告文件、无 minidump/breakpad/sentry | 同左 | 同左 | 全仓 grep 无命中；唯一 `catch_unwind` 在 js_dom_bridge.rs:4061（页面 JS 边界） |
| 崩溃可恢复性 | 会话级：`session.json` `clean_exit` 标记 + 重启"恢复标签?"气泡 + renderer 崩溃自动重生重导航 | 同左 | 同左 | crates/browser-shell/src/session.rs:21-123；apps/browser/src/app.rs:2648-2740；apps/browser/src/process_backend.rs:1511-1567 |
| 会话恢复 | 有（见上；每 tab 快照 url/title/history） | 同左 | 同左 | session.rs:21-90 |
| 用户反馈导出 | 无 | 无 | 无 | 无对应代码面 |
| 隔离运行支持 | `HOME`/`XDG_*` 走 dirs（配置落 `~/.config/zeroweb/`）、`ZERO_STORAGE_DIR`、`ZERO_CACHE_DIR` 环境变量可隔离；**无 `--profile`/`--user-data-dir` CLI** | 同左（无 ZERO_* 约定的平台化路径待核） | `%LOCALAPPDATA%` | crates/browser-shell/src/session.rs:91-94；crates/storage/src/storage_manager.rs:23-71；crates/net/src/disk_cache.rs:499-506；**缺口 G3：CLI 级 profile 隔离缺失** |

Cookie 为纯内存（crates/net/src/cookie.rs 无持久化路径）；桌面端 `history.json`/`downloads.json` 已定义但无调用方（browser-shell/src/profile.rs:12-42 仅 Android facade 消费）——matrix 记档，不影响本 goal 范围判断。

## 3. 隔离本地基线（Linux = 首个平台候选）

候选理由：本 Rally 环境即 Linux x86_64；.deb 链路工具齐备（dpkg-deb/strip 在位，appimagetool 缺失仅影响 AppImage 成品）；DC-1..DC-5 全部场景可在无根权限、不触系统包数据库的前提下以"解包前缀 + 隔离 env"推进。

### 3.1 产物构建

- 命令：`systemd-run --user --scope --unit=zw-delivery-m1-pack -p MemoryMax=24G -p MemorySwapMax=0G ./scripts/package-linux.sh --deb`
- 结果：见 §3.3 记录（产物名、SHA256、大小）。
- 版本：`26.10.22`（构建日期方案，无 `ZERO_BUILD_VERSION` 覆盖）。

### 3.2 隔离安装与启动

- 安装形态：`dpkg-deb -x <deb> <隔离前缀>`——不改系统包数据库、不写 `/usr`；等价"隔离安装"机制验收（真机 `dpkg -i` 留 M3/M4 真实安装切片）。
- 启动：`HOME=<隔离目录> ZERO_STORAGE_DIR=<隔离目录>/storage ZERO_CACHE_DIR=<隔离目录>/cache <前缀>/usr/bin/zero-browser --headless --remote-debugging-port=<空闲端口>`；子进程经 `current_exe()` 同目录解析（apps/browser/src/process_backend.rs:98,159），前缀内自洽。
- 版本查询：`GET /json/version` → `Browser: ZeroWeb/26.10.22`。
- 卸载：删前缀目录；隔离 HOME 内用户数据保留核对。

### 3.3 结果记录

**产物**：`target/packages/zero-browser_26.10.22_amd64.deb`，35,377,932 字节（34M），SHA256 `9cb178f6ac5cef447a45b6642f49ecb7100f1861c1a390659df2306afd8b7309`（哈希文件 `target/packages/sha256-26.10.22.txt`）。构建 20.37s 增量 release，守卫 scope 正常退出（exit 0）。已知 cosmetic：dpkg-deb rootless 构建 owner 警告（官方提示 `--root-owner-group`，不影响产物）。包内四二进制（browser/renderer/compositor/image-decoder）+ desktop 文件 + 图标 + copyright/changelog 齐备。

**隔离安装与版本查询 PASS**：`dpkg-deb -x` 解包至 `target/packages/instroot`；`HOME=<隔离> ZERO_STORAGE_DIR=<隔离>/storage ZERO_CACHE_DIR=<隔离>/cache <前缀>/usr/bin/zero-browser --headless --remote-debugging-port=19222` 启动成功（日志：headless server 监听、五字体加载、renderer 子进程模式 auto）。`GET /json/version` 原文：

```json
{"Browser":"ZeroWeb/26.10.22","Protocol-Version":"1.3","User-Agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36 ZeroWeb/26.10.22","V8-Version":"12.0","WebKit-Version":"0.1","webSocketDebuggerUrl":"ws://127.0.0.1:19222"}
```

`GET /json/list` 返回单 tab `about:blank`（type=page）。**隔离边界核验 PASS**：运行期间真实 `~/.config/zeroweb/`（既有文件 mtime 均早于本次运行）零写入；隔离目录外无 `.zero-browser-storage`/`.zero-browser-cache` 兜底目录生成——`HOME`/`ZERO_STORAGE_DIR`/`ZERO_CACHE_DIR` 三个环境变量足以完成 CLI 级隔离（G3 的短期替代路径成立）。

**启动渲染腿 INCONCLUSIVE→改判**：headless 位置参数 `<url>` 不生效（`run_headless` main.rs:732-760 仅传 port/viewport，`headless/mod.rs` 无 startup URL 消费，初始 tab 恒 `about:blank`）——归档为缺口 G5；打包产物页面渲染证明改由既有 `make product-smoke` 门禁承担（同 HEAD PASS 记录见 goal master 里程碑 M4 行、合成动画控制面 2026-10-22 记录），本基线不重复造渲染证据。

**关停腿 FAIL（记缺口 G4）**：headless 模式无优雅关停——`shutdown_signal::install()` 在 Unix 为 no-op（shutdown_signal.rs:51-56），`run_headless` 不接信号、不落 `session.json`（SIGTERM 后隔离 HOME 内无任何文件）。含义：更新流"退出旧版→换新→重启"若走 headless/信号路径，会话以 crash 语义留存（`clean_exit=false`），DC-4 恢复验收时按崩溃重启分支设计；GUI 正常关窗路径（main.rs:822/1161/1231/1649 → `persist_user_data`）不受影响。

### 3.4 清理

已删除本次创建的 `target/packages/isolate/`（隔离 HOME/前缀/日志）与 `probe.html`；保留 .deb 与哈希文件作为旧安装产物（goal 契约"每批保留旧安装产物"）。无残留进程（两个 headless 进程分别 SIGTERM/SIGKILL 后确认退出）。

## 4. 缺口与 M2/M3 入口

| 缺口 | 影响 DC | 建议归属 |
|---|---|---|
| G1 `--version` CLI 缺失 | DC-1 | M3 版本追溯切片：`--version` 输出产品版本 + git hash（需构建期注入，评估 build-support 扩展） |
| G2 版本无 git hash/构建号 | DC-1 | 与 G1 同切片；`SOURCE_DATE_EPOCH`/CI 环境注入可选 |
| G3 无 `--profile` CLI | DC-1/DC-2/DC-4 隔离验收 | M3 安装修复切片；短期以 env 隔离替代（本基线已验证可行） |
| G4 headless/信号路径无优雅关停（Unix no-op，不落 session） | DC-2/DC-4（更新换版重启、崩溃恢复语义） | M2 RFC 回滚决策输入；M3 与更新重启流一并处理 |
| G5 headless 不消费启动 URL（初始 tab 恒 about:blank） | DC-2 验收便利性 | M3 更新切片验收设计时绕过（CDP WS 导航）；是否补齐届时定 |
| 更新/回滚全缺 | DC-2 | M2 RFC 定可信来源与回滚决策后 M3 实现 |
| 崩溃报告/反馈导出全缺 | DC-3/DC-4 | M2 RFC 定隐私/保留策略后 M3 实现（panic hook → 本地报告 → 脱敏 → 导出） |
| Linux/Windows 无签名 | DC-1（签名按渠道验证条款） | M2 记待决：Linux 候选渠道下以"未签名本地模拟"限定验收语义（goal 契约已允许） |

## 5. 清理

构建产物留在 `target/packages/`（gitignore 域，不入库）；隔离前缀与 HOME 目录运行核验后删除并在 §3.3 回填确认；无残留进程。
