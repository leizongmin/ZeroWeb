# 安装更新与崩溃反馈交付闭环

**版本**：v1.0 / 2026-10-20
**状态**：Active（启动门控：本轮只创建文档和脚本；等待后续明确启动）
**父目标**：[zero-web](zero-web.md) / **控制面**：[master](browser-delivery-loop/master.md)

## 0. 执行摘要与 Mission

让现有浏览器交付具备可验收的安装、版本识别、更新中断恢复和本地崩溃反馈流程。先盘点现有打包与恢复入口，再选定一个平台完成最小闭环。

拆分依据：2026-10-20 用户要求按 [Ladybird 建议](../research/research-ladybird-progress-2026-10-20.md)创建后续 goal，开发另行启动。

## 基线与 Support Envelope

静态基线：已有 `scripts/package-macos.sh`、`scripts/package-linux.sh`、`scripts/package-windows.ps1`，应用入口在 `apps/browser/src/main.rs`。打包脚本存在不代表已具备签名、更新服务或崩溃上报；安装和更新基线 **NOT_RUN**。

范围：现有桌面平台安装/卸载、可追溯版本与构建信息、更新校验与中断回滚、本地崩溃报告、用户同意后导出反馈、会话恢复。

边界：复用 [desktop-browser](desktop-browser.md)、[Android 历史交付](archive/android-browser/master.md)和现有验证工具，不改变支持平台承诺。渠道、签名证书、更新源、报告服务和保留策略在 M1/M2 记录待定；没有对应授权只做本地模拟，不自动购买、发布、上传或发送消息。

碰撞：按 [run-rules §9–11](../rally/run-rules.md)协调 packaging/browser/profile/storage 和既有发布流；只修改本 goal 控制面，安装使用隔离目录与 profile。

## Done Criteria / 验收场景（P1-release）

| ID | 条件与动作 | 期望与验证 |
|---|---|---|
| DC-1 / DELIVERY-01 | 在所选平台隔离安装、启动、查询版本、卸载 | 版本/构建可追溯，签名按选定渠道验证，卸载不误删用户数据；产物哈希与真实安装记录 |
| DC-2 / DELIVERY-02 | 校验有效/损坏/错误版本更新包，在下载与替换阶段中断 | 损坏包拒绝，旧版本仍可启动；原子替换与回滚不破坏 profile；本地更新源故障验收 |
| DC-3 / DELIVERY-03 | 受控崩溃，查看报告，拒绝或同意导出 | 默认本地保留，报告剔除 URL、正文、Cookie、token 等隐私数据；导出前可预览与删除，拒绝不发送 |
| DC-4 / DELIVERY-04 | 崩溃后重启、恢复会话并生成用户反馈包 | 恢复不循环打开崩溃页面、不丢已持久化数据；反馈包版本/复现信息完整且遵守用户选择 |
| DC-5 | 审计所选平台闭环与其余平台状态 | 常驻回归、真实安装/更新/恢复证据和适用门禁齐全；未支持渠道明确，不用模拟验收冒充公开发布 |

DC-1 的渠道与签名要求必须在 M2 固定；本地未签名模拟只能验证安装机制。扩大公开发布范围或调整 DC 需用户确认。

## 里程碑与实施交接

| 阶段 | 文件/职责与依赖 | 产物与验证 |
|---|---|---|
| M1 | package 脚本 → browser 启动/版本 → profile 存储 | 安装、更新、恢复能力矩阵；隔离本地基线与首个平台候选 |
| M2 | 本 goal 交付 RFC 与现有发布入口 | 渠道/签名/更新可信来源、隐私/保留、回滚决策；外部配置列待决 |
| M3 | packaging/browser 既有消费者 | 最小安装更新切片、本地报告与会话恢复；常驻中断和损坏输入回归 |
| M4 | 真实安装产物与本地故障 fixture | 所选平台全流程验收、数据完整性、清理与 DC 审计 |

实现来源：复用现有打包、版本与 profile 存储；平台更新能力优先，不预设自建常驻服务。建议分批安装/版本、更新/回滚、本地报告/恢复；每批保留旧安装产物和可恢复 profile。信任边界校验与防数据丢失不可省略。

## 执行与 Document Control

入口：`./scripts/rally-goal-browser-delivery-loop.sh`；加 `--dry-run` 仅预览，直接执行将启动 Rally。统一授权、守卫与证据协议见 [启动说明](ladybird-followups/README.md)。

入口仅目标变化时更新；master/evidence 持续更新，archive 只追加。本轮安装、发布和产品开发 **NOT_RUN**。启动后 `CONTINUE: <下一切片>` / `BLOCK: <真实阻塞>`；所有 DC 与适用门禁有证才 `DONE`。
