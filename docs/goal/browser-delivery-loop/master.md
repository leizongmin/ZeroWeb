# 浏览器交付闭环控制面

**日期**：2026-10-22
**状态**：Active；M1 ✅（能力矩阵 + Linux 隔离基线，[证据](evidence/2026-10-22-m1-inventory-baseline/README.md)）；M2 ✅（交付 RFC 定版 D1–D5，[m2-delivery-rfc](m2-delivery-rfc.md)）；M3 ✅ 四切片全部完成——S1 版本追溯（[证据](evidence/2026-10-22-m3-s1-version-traceability/README.md)）、S2 更新/回滚（[证据](evidence/2026-10-22-m3-s2-update-rollback/README.md)）、S3 本地崩溃报告（[证据](evidence/2026-10-22-m3-s3-crash-report/README.md)）、S4 反馈导出+崩溃循环守卫（[证据](evidence/2026-10-22-m3-s4-feedback-loop-guard/README.md)，DC-3 后半 + DC-4 两腿 17 断言 PASS）；安装/发布 NOT_RUN（未做真实系统安装，M4 按排期，依赖 P1/P4 授权）。
**当前卡点**：无阻塞卡点。外部授权待决 P1–P5（签名证书/公开源/上传服务/系统安装/保留终值）已列 RFC §8，不阻塞 M4 机制层串测；真实系统安装与公开发布须先获授权。
**下一步**：M4——DC-1..DC-4 全链路串测（安装→版本→更新→崩溃→恢复→导出串联走查）+ DC-5 审计（所选平台闭环证据齐全性、macOS/Windows 未支持渠道与理由记录、清理审计）；真实系统安装（DC-1 签名/卸载）按 P1/P4 授权节奏。
**入口**：[目标契约](../browser-delivery-loop.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 安装/版本/卸载真实能力 | Linux 已定候选：.deb 解包前缀隔离基线 PASS（版本查询 + 隔离边界核验）；真机安装验收留 M3/M4 | ✅ 基线完成（2026-10-22）；真实安装 NOT_RUN |
| 更新校验/中断/原子替换/回滚 | browser-shell `update.rs` 全管线 + `--check-updates`/`--rollback-update` CLI；DC-2 六腿（正常/回滚/哈希拒/降级拒/自检拒/两中断腿）22 断言 PASS；未签名渠道，真机安装留 M4 | ✅ 机制层完成（2026-10-22）；真机系统安装 NOT_RUN |
| 本地崩溃报告与会话恢复 | 崩溃报告（`crash_report.rs` 白名单 + panic hook + `handle_renderer_ipc_lost` 单一收口 + 保留 10 份删最旧）；反馈导出（`feedback.rs` 目录形态 + CLI 预览/删除/导出，URL 仅显式勾选）；崩溃循环守卫（crash_count + 观察窗 10s + 恢复跳过）；附带补 Unix SIGINT graceful handler | ✅ 完成（2026-10-22：S3 两腿 12 断言 + S4 两腿 17 断言 PASS） |
| 平台/渠道/签名/报告源与保留策略 | M2 定义，外部服务和发布另获授权 | 待决（M2 首步） |
| 盘点缺口 G1–G5 | G1（--version CLI）+ G2（版本无 git hash）已随 M3-S1 关闭；G3（无 --profile）/G4（headless 无优雅关停）/G5（headless 不消费启动 URL）未处理，见 [证据 §4](evidence/2026-10-22-m1-inventory-baseline/README.md) | G1/G2 ✅（2026-10-22）；G3–G5 挂账 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 交付能力矩阵与隔离基线 | ✅ 完成（2026-10-22：三平台打包/安装/卸载/版本/签名/更新/崩溃/恢复/隔离九维矩阵；Linux 定首个平台候选；.deb 从 HEAD 构建哈希在账、解包前缀隔离启动 + `/json/version` 版本查询 PASS、隔离边界核验 PASS；G1–G5 缺口定版） | [2026-10-22-m1-inventory-baseline](evidence/2026-10-22-m1-inventory-baseline/README.md) |
| M2 | 渠道/信任/隐私/回滚 RFC | ✅ 完成（2026-10-22：D1 Linux .deb 本地源、D2 未签名本地渠道验收语义、D3 dpkg-deb 暂存+symlink 翻转+回滚、D4 白名单崩溃报告本地保留、D5 会话基座复用+循环守卫；FR-1..7/IF-1..4/NFR-1..4 与 M3 四切片交接；待决 P1–P5 列档；[RFC](m2-delivery-rfc.md)） | [m2-delivery-rfc](m2-delivery-rfc.md) |
| M3 | 单平台安装、更新、报告与恢复 | ✅ 完成（2026-10-22 四切片全落地。S1：`ZERO_BUILD_COMMIT` 构建注入 + `--version` CLI（IF-1）+ .deb 前缀版本逐字符一致。S2：`update.rs` manifest 校验 + 暂存自检 + 相对 symlink 原子翻转 + 自动/显式回滚；DC-2 六腿 22 断言 PASS。S3：`crash_report.rs` IF-3 白名单 + panic 钩子链 + `handle_renderer_ipc_lost` 单一收口 + 保留上限；两腿 12 断言 PASS。S4：`feedback.rs` 目录形态导出（IF-4 回退，无 zip 依赖）+ CLI 预览/删除/导出 + 崩溃循环守卫（crash_count/观察窗/恢复跳过）+ Unix SIGINT graceful 补齐；DC-3 后半 + DC-4 两腿 17 断言 PASS） | [S1](evidence/2026-10-22-m3-s1-version-traceability/README.md) / [S2](evidence/2026-10-22-m3-s2-update-rollback/README.md) / [S3](evidence/2026-10-22-m3-s3-crash-report/README.md) / [S4](evidence/2026-10-22-m3-s4-feedback-loop-guard/README.md) / [RFC §7](m2-delivery-rfc.md) |
| M4 | 真实产物、故障、完整性与清理审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | DELIVERY-01-install-and-version | NOT_RUN（机制层已推进：M1 隔离启动 + CDP 版本查询 PASS；S1 `--version`+commit 追溯 PASS——真机安装/卸载验收未做） | [M1 证据 §3](evidence/2026-10-22-m1-inventory-baseline/README.md) / [S1 证据](evidence/2026-10-22-m3-s1-version-traceability/README.md) |
| DC-2 | DELIVERY-02-update-interruption-rollback | ✅ 机制层 PASS（2026-10-22：六腿 22 断言——正常应用/显式回滚/HASH_MISMATCH/DOWNGRADE/STAGING_CHECK_FAILED/两 SIGKILL 中断腿，中断后旧版存活且重试收敛；单测 8 项含自动回滚与 staging 残留清理；未签名渠道 + 解包前缀隔离，真机系统安装 NOT_RUN） | [S2 证据](evidence/2026-10-22-m3-s2-update-rollback/README.md) |
| DC-3 | DELIVERY-03-local-crash-report-consent | ✅ PASS（2026-10-22。前半 S3：FR-4/IF-3 崩溃报告——panic 探针真实进程白名单字段集精确 + GUI SIGKILL tab renderer 报告落盘 + 隐私 grep 全 0。后半 S4：FR-5 导出同意——列举/预览/删除/目录包导出，摘要与 URL 各需显式勾选，重复导出拒绝，无报告拒绝且零文件产生；整包隐私 grep 全 0） | [S3 证据](evidence/2026-10-22-m3-s3-crash-report/README.md) / [S4 证据](evidence/2026-10-22-m3-s4-feedback-loop-guard/README.md) |
| DC-4 | DELIVERY-04-recover-session-and-feedback | ✅ PASS（2026-10-22 S4：FR-6 真实进程两腿——双 SIGKILL 同一 tab → crash_count 逐次即时落盘 → SIGINT 干净退出 → 重启自动恢复达阈值 tab 置空 about:blank 不加载崩溃页 → 观察窗 10s 清零；反馈包版本/commit 与宿主一致且遵守用户勾选；app 级恢复跳过回归测 + browser-shell 366 测全绿。注：Unix SIGINT graceful handler 为本次附带补齐的既有缺口） | [S4 证据](evidence/2026-10-22-m3-s4-feedback-loop-guard/README.md) |
| DC-5 | 所选平台交付与其余平台审计 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：packaging/browser/profile/storage 与现有发布流。2026-10-22 核对：本 clone 工作树干净（HEAD `ac4c7fc59`）；并行渲染流占用 css-parser/style-system/layout-engine/render-foundation + rendering-compat，与本 goal 域（scripts/package-*、apps/browser、browser-shell、docs/goal/browser-delivery-loop）无重叠。本基线运行用 `HOME`/`ZERO_STORAGE_DIR`/`ZERO_CACHE_DIR` 隔离，未触真实 `~/.config/zeroweb/`（既有文件为并行流产物，未动）。

本轮不授权购买证书、开服务、上传、通知、提交、推送或发布。后续记录隔离目录/profile、产物来源与签名、故障点、数据完整性和清理到 `evidence/`；里程碑追加 `archive/`。本地模拟不冒充公开发布验收。本轮提交/推送按 Rally 常规授权执行（文档与打包产物记录，无产品代码变更）。
