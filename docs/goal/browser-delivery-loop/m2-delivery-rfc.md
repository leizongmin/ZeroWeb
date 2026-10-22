# M2 交付 RFC：渠道/签名/更新可信来源/隐私/回滚决策

**日期**：2026-10-22
**状态**：草案（M2 交付物；M3 实施输入）
**输入**：[M1 盘点证据](evidence/2026-10-22-m1-inventory-baseline/README.md)（缺口 G1–G5）、[目标契约](../browser-delivery-loop.md)、[控制面](master.md)
**授权边界**：goal 契约条款——只做本地模拟，无签名证书/更新服务/报告服务授权；购买、公开发布、上传需另获用户授权。

## 0. 执行摘要

**目标**：在 Linux（.deb、本地文件源、未签名渠道）上以最小实现达成 DC-1..DC-4 的机制层验收；真实系统安装、真实签名与外部服务全部列待决。

**已定决策**（依据：goal 契约 + M1 盘点事实，见 §5 取舍）：

| # | 决策 | 依据 |
|---|---|---|
| D1 | 渠道 = Linux .deb；更新源 = 本地目录 manifest；macOS/Windows 本 goal 记未支持渠道 | M1 基线 .deb 链路已验证；无外部服务授权 |
| D2 | 签名 = 无证书，验收语义限定"安装机制"（SHA256 完整性，非 PKI 真实性）；DC-1"签名按渠道验证"按未签名本地渠道解释 | 契约明文允许本地未签名模拟 |
| D3 | 更新应用 = `dpkg-deb -x` 暂存 + 前缀 symlink 原子翻转；旧前缀保留即回滚；平台 `dpkg -i`/apt 记为真实安装路径，无授权不执行 | 平台工具优先、无常驻服务；中断点可测（DC-2） |
| D4 | 崩溃报告 = 本地 JSON 白名单字段（无 URL/正文/Cookie/token），本地保留上限 10 份；导出须用户显式动作 | 契约隐私条款（DC-3） |
| D5 | 会话恢复复用既有 session.json 基座；崩溃循环守卫经 per-tab 崩溃计数落 session.json（URL 只进本地 profile，不进报告） | M1 盘点：基座已存在；DC-4 循环打开条款 |

**待决**（§8）：真实签名证书、公开更新源/服务、报告上传服务、真实系统安装验收、保留策略终值。

**下一步**：M3 四切片（§7）——S1 版本追溯（G1/G2）→ S2 更新/回滚 → S3 本地崩溃报告 → S4 反馈导出 + 循环守卫。

## 1. 背景与现状（现状分析）

M1 盘点结论（细节与 file:line 见 [M1 证据](evidence/2026-10-22-m1-inventory-baseline/README.md)）：

- **已有可复用基座**：打包三脚本（Linux .deb/AppDir、macOS .app+zip、Windows zip+NSIS 脚本）；产品版本 `YY.M.D` 构建期注入（`crates/product-version`，`build-support/product_version.rs`）；会话恢复全套（`crates/browser-shell/src/session.rs`：`SessionState.clean_exit` 标记、崩溃气泡、`apps/browser/src/app.rs:2648-2740` 恢复逻辑）；renderer 崩溃自动重生（`apps/browser/src/process_backend.rs:1511-1567`）；profile 隔离经 `HOME`/`ZERO_STORAGE_DIR`/`ZERO_CACHE_DIR` 环境变量已验证可行。
- **全缺**：`--version` CLI（G1）、git hash 构建信息（G2）、任何更新/校验/回滚代码、崩溃报告写入、反馈导出；`--profile` CLI（G3）、headless 优雅关停（G4）与启动 URL 消费（G5）为周边缺口。
- **架构落点**：更新与崩溃报告属 shell 层用户服务——扩展 `crates/browser-shell`（session/bookmarks/settings 同层），不新增 workspace member（避免 Cargo.toml 共享面碰撞，见 run-rules §9）；browser 进程负责接线（CLI flag、信号、UX 入口）。

## 2. 需求规格（Spec）

### 2.1 功能需求（FR）

| ID | 需求 | 验收场景（Given/When/Then） | 对应 DC |
|---|---|---|---|
| FR-1 | 版本追溯：`zero-browser --version` 向 stdout 输出一行 `ZeroBrowser <YY.M.D> (commit <短 hash>)`；构建期注入 git commit hash；打包产物内二进制与该输出一致 | Given HEAD 构建的安装产物；When 执行 `--version`；Then 输出含产品版本与构建 hash，且与打包记录一致；When 无 git 环境构建（如源码 tarball）；Then hash 段输出 `unknown`，不失败 | DC-1 |
| FR-2 | 更新清单与校验：更新源为本地目录，含 `manifest.json` 与 .deb 产物；更新器校验 SHA256 与版本序；校验失败/源不可达/损坏包 → 拒绝并保持旧版本可启动 | Given 含损坏包（哈希不符）/降级版本/损坏 manifest 的本地源；When 触发更新检查；Then 三者均拒绝，退出码区分原因，旧版本启动不受影响 | DC-2 |
| FR-3 | 原子替换与回滚：更新按 `下载复制 → 校验 → dpkg-deb -x 暂存 → 前缀翻转（symlink rename）→ 健康检查` 执行；任一阶段中断，旧前缀完整可用；提供显式回滚（翻回旧前缀）；profile 数据在前缀外不受损 | Given 正常包；When 在复制/暂存/翻转三阶段分别注入中断（SIGKILL）；Then 旧版本仍启动、无半成品成为活跃前缀、隔离 profile 读写正常；When 显式回滚；Then 活跃前缀回到上一版本 | DC-2 |
| FR-4 | 本地崩溃报告：browser 进程 panic 与 renderer 异常退出时写 JSON 报告至 `<配置目录>/crash-reports/`；字段白名单（§4.3），无 URL/页面正文/Cookie/token/用户路径；本地保留上限 10 份，超出删最旧 | Given 注入 panic 的构建；When 触发；Then 生成白名单字段报告，grep 全文无 URL/敏感串；When 已有 10 份再崩溃；Then 总数仍为 10 | DC-3 |
| FR-5 | 报告预览与导出同意：报告可列举、预览、删除；导出仅经用户显式动作，导出前可预览与删除选中项；拒绝/不操作 = 不产生任何导出文件 | Given ≥1 份报告；When 用户选择导出并确认路径；Then 生成 zip 且内容与预览一致；When 用户取消；Then 无任何新文件 | DC-3 |
| FR-6 | 崩溃后恢复与反馈包：崩溃重启走既有恢复流程；同一 tab 反复崩溃 ≥2 次后，下次恢复该 tab 置空不再自动打开崩溃页；反馈包 = 用户选中报告 + 版本/平台信息（+ 可选会话摘要，默认不含 URL） | Given clean_exit=false 的 profile 且某 tab 连崩两次；When 第三次重启恢复；Then 该 tab 不再自动加载崩溃页，其余 tab 正常；When 生成反馈包；Then 版本/复现信息完整且遵守用户勾选 | DC-4 |
| FR-7 | 更新触发面：更新检查为显式触发（CLI flag / 设置文件字段），默认路径零网络、零后台检查 | Given 默认配置；When 正常启动；Then 无任何更新相关网络访问（源为本地路径，本就不出网）；When 设置了本地源并触发；Then 按 FR-2/FR-3 执行 | DC-2 |

### 2.2 接口需求（IF）

| ID | 接口 | 定义 |
|---|---|---|
| IF-1 | CLI：`--version` | 无参数；stdout 单行；退出码 0；不启动 GUI（FR-1） |
| IF-2 | 更新清单 `manifest.json` schema v1 | `{"schema_version":1,"version":"YY.M.D","artifact":"<文件名>","sha256":"<64hex>"}`；`version` 递增才接受（相等/降级拒绝，拒绝原因区分）；`artifact` 仅允许源目录内相对文件名（防路径穿越） |
| IF-3 | 崩溃报告文件 | `<配置目录>/crash-reports/crash-<unix秒>-<角色>.json`；白名单字段：`schema_version, role(browser|renderer|compositor|image_decoder), version, git_commit, os, arch, signal|panic_message, panic_location(文件:行，仅仓内路径相对化), thread, timestamp, renderer_exit_code?`；单文件 ≤ 64 KiB |
| IF-4 | 反馈包 | `<用户指定路径>/zeroweb-feedback-<版本>-<日期>.zip`：`reports/`（用户勾选的报告原样）+ `environment.json`（版本/hash/os/arch/报告清单）+ 可选 `session-summary.json`（tab 计数；含 URL 需用户显式勾选） |

### 2.3 非功能需求（NFR）

| ID | 需求 | 验证 |
|---|---|---|
| NFR-1 | 隐私：报告与反馈包默认不含 URL、页面正文、Cookie、token、用户主目录路径；panic 消息中若嵌页面串，以白名单字段 + 仓内路径相对化兜底 | FR-4 验收 grep 项；报告生成器单测断言字段集 |
| NFR-2 | 防数据丢失：更新全程不写 profile 数据；前缀翻转用 rename 原子性；回滚不删用户数据 | FR-3 验收；隔离 HOME 前后对比 |
| NFR-3 | 默认零行为：不配置更新源则全部新代码路径不激活；崩溃报告写入不改变正常退出路径语义 | 既有默认路径测试全绿 + 无新文件产生 |
| NFR-4 | 信任边界：manifest 哈希校验在暂存前；`artifact` 字段路径穿越校验；报告/反馈包路径仅由用户显式指定 | 单测覆盖恶意 manifest 用例 |

## 3. 约束与假设

**约束（来源：goal 契约 + run-rules）**：
- 只做本地模拟：无证书购买、无外部服务、无上传、无公开发布；DC-1 签名条款按未签名本地渠道验收（D2）。
- 不新增 workspace member；更新/崩溃报告逻辑入 `crates/browser-shell`，browser 进程只做接线。
- 共享面注意：`Cargo.lock` 若因新增 `sha2` 显式依赖变化，属共享面，实现时先 pull 核对（run-rules §9）。
- 版本方案沿用 `YY.M.D` 构建日期（`product-version` crate 不改语义，仅增量加 commit hash 注入）。

**假设**：
- A1：更新目标形态为"隔离前缀安装"（M1 已验证 `dpkg-deb -x` 解包启动）；真实系统级 `dpkg -i` 安装留 M4，需隔离环境授权。
- A2：WSL2/Linux 环境下 symlink rename 原子性成立（POSIX `rename(2)` 保证同目录原子替换）。
- A3：`sha2` crate 已在依赖图（Cargo.lock 两条记录），browser-shell 显式声明不引入新下游树；版本对齐实现时核对。

**实现来源闭合**：
| 能力 | 来源 |
|---|---|
| SHA256 校验 | `sha2` crate（依赖图已有）；hex 编码用 `hex` 或手写格式化 |
| 版本/hash 注入 | 扩展 `crates/product-version` build.rs + `build-support/product_version.rs`，仿既有 `ZERO_BUILD_VERSION` 模式加 `ZERO_BUILD_COMMIT` |
| 前缀翻转原子性 | POSIX `rename(2)`（std `fs::rename`） |
| 崩溃捕获 | std `panic::set_hook`（browser 进程）+ `process_backend` 既有 renderer 崩溃检测点 |
| 会话崩溃计数 | 扩展 `browser-shell/session.rs` `TabSnapshot`（新增可选字段，向后兼容旧 session.json） |
| zip 导出 | 已定（2026-10-22）：依赖图无 zip crate 且 Cargo.lock 为共享碰撞面，不引新依赖——自实现 stored（不压缩）ZIP writer（`browser-shell/feedback.rs`，PKWARE APPNOTE §4.3 手写头+EOCD，CRC-32/ISO-HDLC），`--export-format=zip` 显式 opt-in；目录形态保持缺省（S4/M4 证据可复现） |

## 4. 技术设计（RFC）

### 4.1 更新管线（FR-2/FR-3/FR-7）

```
浏览器/CLI 触发
    │ settings.update.source（本地目录）或 CLI --check-updates=<dir>
    ▼
[读 manifest] ──解析失败──► 拒绝(MANIFEST_INVALID)，旧版继续
    ▼
[版本比较] ──≤ 当前──► 拒绝(UP_TO_DATE/DOWNGRADE)
    ▼
[复制 artifact 到 staging/] ──中断──► staging 残留下次清理，活跃前缀无恙
    ▼
[SHA256 校验] ──不符──► 删暂存，拒绝(HASH_MISMATCH)
    ▼
[dpkg-deb -x → staging/prefix/] ──失败──► 删暂存，拒绝(EXTRACT_FAILED)
    ▼
[暂存自检：zero-browser --version 输出 = manifest.version]
    ▼
[symlink 翻转] current -> staging/prefix   （rename(2) 原子）
    ▼
[健康检查：经 current 启动 headless /json/version 匹配新版本]
    ├─ 通过 ─► 上一前缀保留为 rollback/，报告成功
    └─ 失败 ─► 自动翻回 rollback/，报告失败原因
```

- 前缀布局：`<安装根>/releases/<version>/`（各版本完整前缀）、`<安装根>/current`（symlink→活跃版本）、`<安装根>/rollback/`（上一活跃版本的 symlink 副本）、`<安装根>/staging/`（可丢弃暂存）。安装根由用户指定（隔离验收即隔离目录）。
- 中断安全性：翻转前任何失败只污染 `staging/`（下次触发时先清空）；翻转本身是单次 rename，无中间态；翻转后失败走自动回滚。
- `dpkg-deb -x` 以子进程调用（dpkg 套件已在 M1 验证）；不解析 .deb 内部格式，避免自造解包。

### 4.2 崩溃报告管线（FR-4）

- browser 进程：`main` 早期 `panic::set_hook`——钩子内仅做白名单字段收集 + 原子写文件（写报告自身 panic 时静默放弃，不递归），随后调原钩子保持既有行为。
- renderer/compositor/image_decoder 异常退出：在 `process_backend` 既有崩溃检测点（`handle_crashes`）追加写报告（角色 + 退出码；无 panic 位置）。
- 路径：复用 profile 配置目录解析（`browser-shell` 现有 `dirs` 路径），随 `HOME` 隔离天然隔离。
- 保留策略：写入前列目录，超过 10 份按文件名时间戳删最旧（文件名含 unix 秒，天然可排序）。
- 脱敏：panic location 为仓内源码路径时转项目相对路径（遵循仓规 SL-008）；`panic_message` 截断至 2 KiB；白名单外字段一律不落。

### 4.3 会话恢复协作与循环守卫（FR-6）

- `TabSnapshot` 增加可选字段 `crash_count: u32`（缺省 0；旧 session.json 无此字段照常加载）。
- `handle_crashes` 重生前递增当前 tab 的 `crash_count` 并落盘；恢复成功且 tab 存活一个观察窗（如 10s）后清零。
- `restore_session_at_startup`：跳过 `crash_count ≥ 2` 的 tab 的自动加载（置 `about:blank`），其余照旧；跳过行为进报告/反馈包说明。
- 隐私边界：URL 只存 session.json（既有行为，本地 profile）；崩溃报告不含 URL，二者不交叉。

### 4.4 反馈导出（FR-5/FR-6）

- 导出为显式两步：用户勾选报告 → 预览清单（文件名+角色+时间+大小，可单项删除）→ 确认目标路径 → 生成 zip（或目录，见 §3 实现来源）→ 报告生成结果。
- M3 最小面：CLI/设置驱动（`--export-feedback=<dir>` + 交互式确认经 stdin/参数跳过模式由验收 fixture 驱动）；GUI 菜单入口视既有菜单基建接入成本为可选增强，不阻塞 DC-3/DC-4 验收。

## 5. 设计选项与取舍

| 议题 | 选项 | 取舍 | 决策 |
|---|---|---|---|
| 更新应用机制 | ① 平台 `dpkg -i`（需 root）② `dpkg-deb -x` 前缀 + symlink 翻转 ③ 自写 .deb 解包 | ① 最真实但验收需 root/隔离环境且中断点难注入；③ 重造 dpkg 轮子；② 复用 M1 已验证工具链、中断点可测、无 root | **②**；① 记为 M4 真实安装路径（需授权），③ 拒绝 |
| 新 crate vs 扩展 browser-shell | 新 `zero-delivery` crate / 扩展 `crates/browser-shell` | 新 crate 需改 workspace Cargo.toml（与并行流共享面碰撞）且当前体量不支撑独立 crate；browser-shell 已是 shell 用户服务层（session/bookmarks/settings 同层） | **扩展 browser-shell**，不新增 member |
| 崩溃循环守卫数据落点 | 崩溃报告内记 URL / session.json 记 per-tab 计数 | 前者违反 FR-4 隐私白名单（URL 不进报告）；后者 URL 留在既有本地 session（现状即如此），报告保持无 URL | **session.json per-tab 计数** |
| 更新触发面 | 启动自动检查 / 显式触发 | 自动检查即使本地源也有启动路径耦合与测试面扩大；goal 要"最小闭环"，DC-2 验收是机制层 | **显式触发（FR-7）**，自动检查不做 |
| hash/版本注入 | vergen / 扩展现有 product-version build.rs | vergen 引入新构建依赖；现有 build-support 模式已验证且极简 | **扩展现有 build-support**（加 `ZERO_BUILD_COMMIT`） |

## 6. 安全考虑

- **信任模型边界（D2 的如实声明）**：本地源 SHA256 只保证完整性（传输/磁盘损坏检测），不保证真实性——manifest 与包可被同源篡改。本 goal 验收语义限定为"安装机制与中断回滚"；真实性依赖签名，列待决（§8）。文档与报告措辞不得将该校验表述为安全校验通过。
- **路径穿越**：IF-2 `artifact` 字段拒绝绝对路径与 `..`；暂存/翻转均在安装根内；报告/反馈包输出路径仅来自用户显式指定（NFR-4）。
- **子进程**：`dpkg-deb -x` 仅以固定参数调用（-x、暂存路径），参数不来自 manifest 内容（manifest 只提供文件名，拼接在源目录下并经穿越校验）。
- **崩溃钩子**：钩子内不分配复杂资源、不递归加锁；写文件失败静默放弃，绝不因报告路径不可写而阻断进程退出。
- **验收数据**：所有验收 fixture 使用隔离 HOME/安装根，不触真实用户 profile（M1 已验证隔离三环境变量）。

## 7. 实施交接（M3 切片）

| 切片 | 内容 | 主要文件（拟） | 验证入口 | 依赖 |
|---|---|---|---|---|
| S1 版本追溯 | `ZERO_BUILD_COMMIT` 注入 + `--version`（IF-1） | `build-support/product_version.rs`、`crates/product-version/{build.rs,src/lib.rs}`、`apps/browser/src/main.rs`（flag 分支） | `--version` 输出单测 + 打包产物实跑（进 M1 证据形态的 evidence）；`make browser-build` 后实跑 | 无 |
| S2 更新/回滚 | manifest 解析 + 校验 + 暂存 + 翻转 + 回滚（FR-2/3/7、IF-2） | `crates/browser-shell/src/update.rs`（新模块）+ `Cargo.toml`（sha2 显式声明）、`apps/browser` CLI 接线 | browser-shell 单测（manifest 恶意用例/版本序/中断注入）+ 隔离安装根端到端 evidence（DC-2 三中断腿） | S1（版本比较与自检依赖 --version/hash） |
| S3 本地崩溃报告 | panic hook + 角色报告 + 保留上限（FR-4、IF-3） | `crates/browser-shell/src/crash_report.rs`（新模块）、`apps/browser/src/main.rs`（hook 安装）、`apps/browser/src/process_backend.rs`（检测点追加） | 单测（白名单字段/脱敏/滚动删除）+ 注入 panic 的真实进程 evidence（DC-3 前半） | 无（可与 S2 并行） |
| S4 反馈导出+循环守卫 | 导出两步流 + TabSnapshot 崩溃计数 + 恢复跳过（FR-5/6、IF-4） | `crates/browser-shell/src/feedback.rs`（新模块）、`session.rs`（字段）、`app.rs`/`process_backend.rs`（计数与跳过） | 单测 + 崩溃-重启循环 fixture evidence（DC-3 后半 + DC-4） | S3 |

**修改顺序**：S1 → （S2 ∥ S3）→ S4。每切片独立提交并附对应回归；S2 起实现前先 pull 核对 Cargo.lock 共享面。

**首批提交建议**：S1 单独一提交（`feat(browser): --version 输出产品版本与构建 hash`）；其后每切片一提交，evidence 与切片同提交。

**验收总装（M4 预告）**：S1–S4 后按 DC-1..DC-4 全链路串测 + DC-5 审计（支持矩阵：macOS/Windows 记未支持渠道与理由；清理审计）。

## 8. 待决清单（需用户/后续授权）

| # | 待决 | 影响 | 当前处置 |
|---|---|---|---|
| P1 | 真实签名证书（Linux GPG/apt 源签名；macOS Developer ID；Windows Authenticode） | DC-1 签名条款升级为真实性验证 | 未签名本地渠道验收（D2）；不购买 |
| P2 | 公开更新源 / 更新服务托管 | FR-7 从本地目录扩展到 HTTP 源 | 仅本地目录源 |
| P3 | 崩溃报告上传服务 | FR-4 增加上传分支 | 仅本地保留 |
| P4 | 真实系统级安装验收（`dpkg -i`/apt，隔离环境） | DC-1"隔离安装"升级为系统安装 | M4 前提权，需环境授权 |
| P5 | 保留策略终值（报告 10 份上限、观察窗 10s） | FR-4/FR-6 参数 | 先按本值实现，常量集中便于调整 |

## 9. Spec Lint 报告

| 检查 | 结论 | 证据/说明 |
|---|---|---|
| 每 FR 有可验证验收场景 | Pass | FR-1..FR-7 均为 Given/When/Then 或等价断言 |
| 引用可追溯（IF/FR/§） | Pass | §4 设计仅引用 FR/IF 编号，未复制参数表 |
| 实现来源闭合 | Pass | §3 实现来源表逐项落来源；zip 依赖实现时定并声明回退 |
| 外部事实降级 | Pass | 证书/服务/系统安装均列 §8 待决，未升格为已定 |
| 默认动作显式化 | Pass | FR-5 拒绝=无文件、FR-7 默认零网络、A1 前缀形态、IF-2 版本序均为显式默认 |
| 需求来源明确 | Pass | 授权边界来自 goal 契约明文（本地模拟/隐私条款/M1/M2 记待定条款） |
| 阻塞 TBD | 无 | §8 全部为外部授权待决，不阻塞 M3 机制层实现 |
