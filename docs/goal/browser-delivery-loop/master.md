# 浏览器交付闭环控制面

**日期**：2026-10-20
**状态**：Active（启动门控）；规划已创建，开发/安装/更新/发布 NOT_RUN。
**当前卡点**：渠道、签名、更新可信来源与反馈服务尚未选定。
**下一步**：明确启动后盘点现有打包、版本和恢复，先做隔离本地基线。
**入口**：[目标契约](../browser-delivery-loop.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 安装/版本/卸载真实能力 | 选单个平台隔离验收，保留产物哈希 | NOT_STARTED |
| 更新校验/中断/原子替换/回滚 | 本地源与损坏包 fixture，不损坏 profile | NOT_STARTED |
| 本地崩溃报告与会话恢复 | 数据脱敏、预览、拒绝导出和循环崩溃测试 | NOT_STARTED |
| 平台/渠道/签名/报告源与保留策略 | M1/M2 定义，外部服务和发布另获授权 | 待决 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 交付能力矩阵与隔离基线 | NOT_STARTED | 无 |
| M2 | 渠道/信任/隐私/回滚 RFC | NOT_STARTED | 无 |
| M3 | 单平台安装、更新、报告与恢复 | NOT_STARTED | 无 |
| M4 | 真实产物、故障、完整性与清理审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | DELIVERY-01-install-and-version | NOT_RUN | 无 |
| DC-2 | DELIVERY-02-update-interruption-rollback | NOT_RUN | 无 |
| DC-3 | DELIVERY-03-local-crash-report-consent | NOT_RUN | 无 |
| DC-4 | DELIVERY-04-recover-session-and-feedback | NOT_RUN | 无 |
| DC-5 | 所选平台交付与其余平台审计 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：packaging/browser/profile/storage 与现有发布流。实际占用 **NOT_CHECKED**；启动前按 run-rules §9–11 核对，不安装覆盖用户实例。

本轮不授权购买证书、开服务、上传、通知、提交、推送或发布。后续记录隔离目录/profile、产物来源与签名、故障点、数据完整性和清理到 `evidence/`；里程碑追加 `archive/`。本地模拟不冒充公开发布验收；当前没有运行记录。
