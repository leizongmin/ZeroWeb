# 浏览器交付闭环控制面

**日期**：2026-10-22
**状态**：Active；M1 ✅（能力矩阵 + Linux 隔离基线，[证据](evidence/2026-10-22-m1-inventory-baseline/README.md)）；M2 ✅（交付 RFC 定版 D1–D5，[m2-delivery-rfc](m2-delivery-rfc.md)）；M3 ✅ 四切片全部完成——S1 版本追溯（[证据](evidence/2026-10-22-m3-s1-version-traceability/README.md)）、S2 更新/回滚（[证据](evidence/2026-10-22-m3-s2-update-rollback/README.md)）、S3 本地崩溃报告（[证据](evidence/2026-10-22-m3-s3-crash-report/README.md)）、S4 反馈导出+崩溃循环守卫（[证据](evidence/2026-10-22-m3-s4-feedback-loop-guard/README.md)，DC-3 后半 + DC-4 两腿 17 断言 PASS）；M4 ✅（全链路串测 8 腿 31 断言 PASS + DC-5 机制面审计，[证据](evidence/2026-10-22-m4-full-chain/README.md)；串测暴露并修复安装根推导缺陷 commit `9ce652d88`）。机制层全部完成；真实系统安装/签名 NOT_RUN（P1/P4 授权门控）。挂账推进：G4 headless 优雅关停 ✅（2026-10-22：SIGINT + CDP Browser.close 两路径真实进程 11 断言 PASS，[证据](evidence/2026-10-22-g4-headless-graceful-shutdown/README.md)，commit `f424cf7c6`——S4 的 SIGINT 全局 flag 曾使 headless Ctrl+C 从立即死变为永不退出，一并修复 Browser.close 空壳 stub）；G3 `--profile` CLI ✅（2026-10-22：`ZERO_CONFIG_DIR` 入 runtime-config 权威清单 + browser-shell 4 处 config_dir 收口单助手 + main 解析提前于 panic hook 单点注入三环境变量，GUI 覆盖语义/renderer env 继承/崩溃报告随 profile/相对路径 CWD 绝对化三腿 15 断言 PASS，[证据](evidence/2026-10-22-g3-profile-cli/README.md)）；G5 headless 消费启动 URL ✅（2026-10-22：与 CDP navigate 同管线 + 地址栏同一 normalize_url，失败不阻断 server，GUI 辅助会话不双加载；fixture 真网络加载/缺省回归/规范化与失败不阻断三腿 14 断言 PASS，[证据](evidence/2026-10-22-g5-headless-startup-url/README.md)——缺口表 G1–G5 全部关闭）。
**当前卡点**：无自主可推进的机制层切片。剩余判据全部落在外部授权门控：P1（签名证书）、P4（系统安装）挡 DC-1 真机安装/卸载/签名验收；P2/P3/P5 挡公开源/上传/发布。P1–P5 已列 RFC §8 待决。
**下一步**：获 P1/P4 授权后做真机 `dpkg -i`/`dpkg -r` 与签名渠道验收（复用 M4 串测脚本骨架换系统包管理器路径）；授权前可选挂账仅余：GUI 导出入口（RFC §4.4 可选增强，需目录选择器基建，成本超切片）。反馈包 zip 形态已落地（2026-10-22：零依赖自实现 stored ZIP + `--export-format=zip` opt-in，真实崩溃报告 zip 导出深校验三腿 19 断言 PASS，[证据](evidence/2026-10-22-feedback-zip/README.md)；RFC §3 行已闭合）。
**入口**：[目标契约](../browser-delivery-loop.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 安装/版本/卸载真实能力 | Linux 已定候选：.deb 解包前缀隔离基线 PASS（版本查询 + 隔离边界核验）；M4 串测 leg1/2/7 在已安装产物上完成安装→追溯→卸载保数据串联 | ✅ 机制层串测完成（2026-10-22）；真机 `dpkg -i`/`dpkg -r`/签名 NOT_RUN（P1/P4 门控） |
| 更新校验/中断/原子替换/回滚 | browser-shell `update.rs` 全管线 + `--check-updates`/`--rollback-update` CLI；DC-2 六腿（正常/回滚/哈希拒/降级拒/自检拒/两中断腿）22 断言 PASS；M4 leg3 无 env 覆盖串联 PASS（暴露并修复默认安装根推导缺陷 `9ce652d88`） | ✅ 机制层完成（2026-10-22）；真机系统安装 NOT_RUN（P1/P4 门控） |
| 本地崩溃报告与会话恢复 | 崩溃报告（`crash_report.rs` 白名单 + panic hook + `handle_renderer_ipc_lost` 单一收口 + 保留 10 份删最旧）；反馈导出（`feedback.rs` 目录形态 + CLI 预览/删除/导出，URL 仅显式勾选）；崩溃循环守卫（crash_count + 观察窗 10s + 恢复跳过）；附带补 Unix SIGINT graceful handler | ✅ 完成（2026-10-22：S3 两腿 12 断言 + S4 两腿 17 断言 PASS） |
| 平台/渠道/签名/报告源与保留策略 | M2 定义，外部服务和发布另获授权 | 待决（M2 首步） |
| 盘点缺口 G1–G5 | G1（--version CLI）+ G2（版本无 git hash）已随 M3-S1 关闭；G4（headless 优雅关停）已随挂账切片关闭（SIGINT 曾因 S4 全局 flag 变为永不退出，Browser.close 空壳一并修复）；G3（--profile）已随挂账切片关闭（`ZERO_CONFIG_DIR` 收口 4 处 config_dir 站点 + CLI 单点注入三环境变量，GUI 覆盖语义/崩溃报告随 profile/相对路径三腿 15 断言 PASS）；G5（headless 不消费启动 URL）已随挂账切片关闭（CDP navigate 同管线 + 地址栏 normalize_url，三腿 14 断言 PASS），见 [证据 §4](evidence/2026-10-22-m1-inventory-baseline/README.md) | G1–G5 全部 ✅（2026-10-22） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 交付能力矩阵与隔离基线 | ✅ 完成（2026-10-22：三平台打包/安装/卸载/版本/签名/更新/崩溃/恢复/隔离九维矩阵；Linux 定首个平台候选；.deb 从 HEAD 构建哈希在账、解包前缀隔离启动 + `/json/version` 版本查询 PASS、隔离边界核验 PASS；G1–G5 缺口定版） | [2026-10-22-m1-inventory-baseline](evidence/2026-10-22-m1-inventory-baseline/README.md) |
| M2 | 渠道/信任/隐私/回滚 RFC | ✅ 完成（2026-10-22：D1 Linux .deb 本地源、D2 未签名本地渠道验收语义、D3 dpkg-deb 暂存+symlink 翻转+回滚、D4 白名单崩溃报告本地保留、D5 会话基座复用+循环守卫；FR-1..7/IF-1..4/NFR-1..4 与 M3 四切片交接；待决 P1–P5 列档；[RFC](m2-delivery-rfc.md)） | [m2-delivery-rfc](m2-delivery-rfc.md) |
| M3 | 单平台安装、更新、报告与恢复 | ✅ 完成（2026-10-22 四切片全落地。S1：`ZERO_BUILD_COMMIT` 构建注入 + `--version` CLI（IF-1）+ .deb 前缀版本逐字符一致。S2：`update.rs` manifest 校验 + 暂存自检 + 相对 symlink 原子翻转 + 自动/显式回滚；DC-2 六腿 22 断言 PASS。S3：`crash_report.rs` IF-3 白名单 + panic 钩子链 + `handle_renderer_ipc_lost` 单一收口 + 保留上限；两腿 12 断言 PASS。S4：`feedback.rs` 目录形态导出（IF-4 回退，无 zip 依赖）+ CLI 预览/删除/导出 + 崩溃循环守卫（crash_count/观察窗/恢复跳过）+ Unix SIGINT graceful 补齐；DC-3 后半 + DC-4 两腿 17 断言 PASS） | [S1](evidence/2026-10-22-m3-s1-version-traceability/README.md) / [S2](evidence/2026-10-22-m3-s2-update-rollback/README.md) / [S3](evidence/2026-10-22-m3-s3-crash-report/README.md) / [S4](evidence/2026-10-22-m3-s4-feedback-loop-guard/README.md) / [RFC §7](m2-delivery-rfc.md) |
| M4 | 真实产物、故障、完整性与清理审计 | ✅ 完成（2026-10-22：单一脚本在已安装产物上串联安装→版本→更新→崩溃→恢复→导出→卸载→清理审计八腿 31 断言 PASS；版本/commit 经 --version/崩溃报告/environment.json 四环节贯穿一致；串测暴露默认安装根推导被 /proc/self/exe 符号链接解析击穿的缺陷并修复（commit `9ce652d88`，回归测试 2 项）；DC-5 审计落档：Linux 证据链核对 + macOS/Windows 未支持渠道与理由 + 清理审计四项 PASS。真机安装/签名 NOT_RUN 按 P1/P4 授权节奏） | [2026-10-22-m4-full-chain](evidence/2026-10-22-m4-full-chain/README.md) |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | DELIVERY-01-install-and-version | ✅ 机制面 PASS（2026-10-22：M1 隔离安装 + CDP 版本查询；S1 `--version`+commit 追溯；M4 串测 leg1/2/7——.deb 前缀安装、安装内 `--version` 与宿主 git 逐字符一致、卸载后用户数据完整。真机 `dpkg -i`/`dpkg -r`/签名 NOT_RUN，P1/P4 授权门控） | [M1 证据 §3](evidence/2026-10-22-m1-inventory-baseline/README.md) / [S1 证据](evidence/2026-10-22-m3-s1-version-traceability/README.md) / [M4 证据](evidence/2026-10-22-m4-full-chain/README.md) |
| DC-2 | DELIVERY-02-update-interruption-rollback | ✅ 机制层 PASS（2026-10-22：六腿 22 断言——正常应用/显式回滚/HASH_MISMATCH/DOWNGRADE/STAGING_CHECK_FAILED/两 SIGKILL 中断腿，中断后旧版存活且重试收敛；单测 8 项含自动回滚与 staging 残留清理；M4 leg3 在无 env 覆盖的默认安装根推导下串联复验 PASS，暴露并修复推导缺陷 `9ce652d88`；未签名渠道 + 解包前缀隔离，真机系统安装 NOT_RUN） | [S2 证据](evidence/2026-10-22-m3-s2-update-rollback/README.md) / [M4 证据](evidence/2026-10-22-m4-full-chain/README.md) |
| DC-3 | DELIVERY-03-local-crash-report-consent | ✅ PASS（2026-10-22。前半 S3：FR-4/IF-3 崩溃报告——panic 探针真实进程白名单字段集精确 + GUI SIGKILL tab renderer 报告落盘 + 隐私 grep 全 0。后半 S4：FR-5 导出同意——列举/预览/删除/目录包导出，摘要与 URL 各需显式勾选，重复导出拒绝，无报告拒绝且零文件产生；整包隐私 grep 全 0。M4 leg4/6 在安装内 v2 产物上复验：报告与导出包的 version/commit 绑定运行版本） | [S3 证据](evidence/2026-10-22-m3-s3-crash-report/README.md) / [S4 证据](evidence/2026-10-22-m3-s4-feedback-loop-guard/README.md) / [M4 证据](evidence/2026-10-22-m4-full-chain/README.md) |
| DC-4 | DELIVERY-04-recover-session-and-feedback | ✅ PASS（2026-10-22 S4：FR-6 真实进程两腿——双 SIGKILL 同一 tab → crash_count 逐次即时落盘 → SIGINT 干净退出 → 重启自动恢复达阈值 tab 置空 about:blank 不加载崩溃页 → 观察窗 10s 清零；反馈包版本/commit 与宿主一致且遵守用户勾选；app 级恢复跳过回归测 + browser-shell 366 测全绿。注：Unix SIGINT graceful handler 为本次附带补齐的既有缺口。M4 leg5 在安装内 v2 产物上全链复验 PASS） | [S4 证据](evidence/2026-10-22-m3-s4-feedback-loop-guard/README.md) / [M4 证据](evidence/2026-10-22-m4-full-chain/README.md) |
| DC-5 | 所选平台交付与其余平台审计 | ✅ 机制层 PASS（2026-10-22 M4 证据 §4：Linux 平台 DC-1..DC-4 证据链逐项核对成立；macOS/Windows 未支持渠道与理由记档（管线未实现 + 本环境无运行时，挂 P1–P5）；清理审计四项 PASS——staging/残留进程/真实配置目录/仓库工作树。真实发布渠道审计待 P1–P5 授权） | [M4 证据 §4](evidence/2026-10-22-m4-full-chain/README.md) |

## 碰撞与执行记录

候选共享面：packaging/browser/profile/storage 与现有发布流。2026-10-22 核对：本 clone 工作树干净（HEAD `ac4c7fc59`）；并行渲染流占用 css-parser/style-system/layout-engine/render-foundation + rendering-compat，与本 goal 域（scripts/package-*、apps/browser、browser-shell、docs/goal/browser-delivery-loop）无重叠。本基线运行用 `HOME`/`ZERO_STORAGE_DIR`/`ZERO_CACHE_DIR` 隔离，未触真实 `~/.config/zeroweb/`（既有文件为并行流产物，未动）。

本轮不授权购买证书、开服务、上传、通知、提交、推送或发布。后续记录隔离目录/profile、产物来源与签名、故障点、数据完整性和清理到 `evidence/`；里程碑追加 `archive/`。本地模拟不冒充公开发布验收。提交/推送按 Rally 常规授权执行；2026-10-22 M4 轮含产品代码变更（更新安装根推导修复 `9ce652d88`，由串测缺陷驱动，附回归测试与全量门禁）。
