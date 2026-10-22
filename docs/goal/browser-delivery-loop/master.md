# 浏览器交付闭环控制面

**日期**：2026-10-22
**状态**：Active；M1 完成（能力矩阵 + Linux 隔离基线落账，[证据](evidence/2026-10-22-m1-inventory-baseline/README.md)）；M2–M4 未启动；安装/更新/发布 NOT_RUN（本 goal 未做真实系统安装，M3/M4 切片按排期）。
**当前卡点**：M2 待做——渠道、签名、更新可信来源与隐私/保留策略未定（RFC 输入已备：盘点缺口 G1–G5）。
**下一步**：M2 交付 RFC：渠道（Linux .deb 本地源为默认候选）、签名语义（无证书渠道下"未签名本地模拟"限定验收）、更新可信来源与回滚决策（本地源 + 原子替换）、崩溃报告隐私/保留策略；外部配置列待决。
**入口**：[目标契约](../browser-delivery-loop.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 安装/版本/卸载真实能力 | Linux 已定候选：.deb 解包前缀隔离基线 PASS（版本查询 + 隔离边界核验）；真机安装验收留 M3/M4 | ✅ 基线完成（2026-10-22）；真实安装 NOT_RUN |
| 更新校验/中断/原子替换/回滚 | M2 RFC 定可信来源；全仓现无任何 updater 代码 | NOT_STARTED |
| 本地崩溃报告与会话恢复 | 会话恢复已有基座（session.json + clean_exit 标记 + 崩溃气泡，browser-shell/session.rs）；崩溃报告/反馈导出全缺 | NOT_STARTED（恢复基座已盘点） |
| 平台/渠道/签名/报告源与保留策略 | M2 定义，外部服务和发布另获授权 | 待决（M2 首步） |
| 盘点缺口 G1–G5（--version CLI、版本无 git hash、无 --profile、headless 无优雅关停、headless 不消费启动 URL） | M3 切片排布输入，见 [证据 §4](evidence/2026-10-22-m1-inventory-baseline/README.md) | 已落账（2026-10-22） |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 交付能力矩阵与隔离基线 | ✅ 完成（2026-10-22：三平台打包/安装/卸载/版本/签名/更新/崩溃/恢复/隔离九维矩阵；Linux 定首个平台候选；.deb 从 HEAD 构建哈希在账、解包前缀隔离启动 + `/json/version` 版本查询 PASS、隔离边界核验 PASS；G1–G5 缺口定版） | [2026-10-22-m1-inventory-baseline](evidence/2026-10-22-m1-inventory-baseline/README.md) |
| M2 | 渠道/信任/隐私/回滚 RFC | NOT_STARTED | 无 |
| M3 | 单平台安装、更新、报告与恢复 | NOT_STARTED | 无 |
| M4 | 真实产物、故障、完整性与清理审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | DELIVERY-01-install-and-version | NOT_RUN（M1 基线已覆盖机制层：隔离启动 + 版本查询 PASS；真机安装/卸载验收未做） | [M1 证据 §3](evidence/2026-10-22-m1-inventory-baseline/README.md) |
| DC-2 | DELIVERY-02-update-interruption-rollback | NOT_RUN | 无 |
| DC-3 | DELIVERY-03-local-crash-report-consent | NOT_RUN | 无 |
| DC-4 | DELIVERY-04-recover-session-and-feedback | NOT_RUN（恢复基座代码已盘点，未验收） | [M1 证据 §2](evidence/2026-10-22-m1-inventory-baseline/README.md) |
| DC-5 | 所选平台交付与其余平台审计 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：packaging/browser/profile/storage 与现有发布流。2026-10-22 核对：本 clone 工作树干净（HEAD `ac4c7fc59`）；并行渲染流占用 css-parser/style-system/layout-engine/render-foundation + rendering-compat，与本 goal 域（scripts/package-*、apps/browser、browser-shell、docs/goal/browser-delivery-loop）无重叠。本基线运行用 `HOME`/`ZERO_STORAGE_DIR`/`ZERO_CACHE_DIR` 隔离，未触真实 `~/.config/zeroweb/`（既有文件为并行流产物，未动）。

本轮不授权购买证书、开服务、上传、通知、提交、推送或发布。后续记录隔离目录/profile、产物来源与签名、故障点、数据完整性和清理到 `evidence/`；里程碑追加 `archive/`。本地模拟不冒充公开发布验收。本轮提交/推送按 Rally 常规授权执行（文档与打包产物记录，无产品代码变更）。
