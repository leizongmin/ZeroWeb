# Renderer 信任边界控制面

**日期**：2026-10-20
**状态**：Active（启动门控）；规划资产已创建，产品开发与验收 NOT_RUN。
**当前卡点**：尚未启动消息权限盘点；静态源码路径不代表已确认漏洞。
**下一步**：明确启动后执行 M1，核对可信身份来源和真实生产接收方。
**入口**：[目标契约](../renderer-trust-boundary.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 消息身份、站点、角色与存储所有者矩阵 | 追踪 protocol → browser → net/security/storage | NOT_STARTED |
| 非法消息与合法请求配对反例 | 优先 Cookie/HSTS，确认缺陷后修复 | NOT_STARTED |
| helper 沙箱/线程时序与平台降级 | 实测 macOS/Linux，记录 FD/socket/文件权限 | NOT_STARTED |
| 平台缺沙箱能力的产品策略 | M1 证据后列选项，必要时用户决定 | 待核查 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 权限矩阵与双向基线 | NOT_STARTED | 无 |
| M2 | 确认反例、共享路径修复与回归 | NOT_STARTED | 无 |
| M3 | helper 实际权限与时序验收 | NOT_STARTED | 无 |
| M4 | DC 审计、门禁、清理和剩余风险 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | IPC-01-forged-origin-session | NOT_RUN | 无 |
| DC-2 | IPC-02-cookie-hsts-privilege | NOT_RUN | 无 |
| DC-3 | IPC-03-automation-role | NOT_RUN | 无 |
| DC-4 | IPC-04-sandbox-thread-order | NOT_RUN | 无 |
| DC-5 | 全部回归与生产接收方复测 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：protocol、engine/ws、browser、net/security/storage。实际占用 **NOT_CHECKED**；启动前按 run-rules §9–11 核对，不因本表空白推断无并行流。只写本 goal 控制面。

用户本轮授权文档与脚本；没有提交、推送、通知或发布授权。后续运行按统一协议登记 HEAD/diff、隔离身份、命令、结果和清理；失败证据追加保存到 `evidence/`，完成里程碑追加到 `archive/`。当前没有运行记录。
