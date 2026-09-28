# 网络 API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../net-api-compat.md](../net-api-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-28（M1 基线落地）

---

## 当前状态

**专项定位**：网络 API（fetch/XHR/URL/mimesniff/streams/EventSource）一致性收敛，
WPT 六 corpus 为验收标尺。WebSocket 二期挂账（需宿主 socket 面 + 帧协议）。

**当前进度**：M1 完成（2026-09-28）——六 corpus window 可执行子集导入 + 常驻通道
+ 分类通过率基线 1297/5019 = 25.8%。下一里程碑 M2（fetch 面逐簇修齐）。

**与兄弟 goal 的边界**：
- security-hardening — CSP 对 fetch 的策略执行归其；本 goal 提供语义钩子位
- service-workers（已归档）— 其 fetch 通道已收口；拦截扩展另行记账
- zero-web P1a — URL/URLSearchParams 既有实现为本 goal 修齐对象（改动走本 goal 账本）
- rendering-compat 及渲染流 — 无共享 crate 面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 六 corpus fetch 脚本 + 导入 + 基线 | ✅ M1（2026-09-28，基线 1297/5019 = 25.8%） |
| P2 | fetch/Request/Response/Headers/Body 语义收敛 | ⏳ M2（当前里程碑；主簇：错误面 262 / Body 消费 92 / forbidden header 72 / 构造校验 16 / clone 流 14） |
| P3 | XHR 状态机 + EventSource 解析/重连 | ⏳ M3（xhr 主簇：send 错误面 30；url 66.9% 修齐：percent-encoding 39 / default port ~40 / URL.parse 8；mimesniff 参数序列化 232；eventsource 依赖 fixture 通道） |
| P4 | URL 边缘语义 + mimesniff 对齐 | ⏳ M3（并入 P3 窗） |
| P5 | streams 底座一致性（fetch body 依赖） | ⏳ M4（主簇：ReadableStream.from 32 / BYOB view 16 / queuing strategy 11 / pull 时机 8） |
| P6 | WebSocket 二期切片（宿主 socket + 升级握手/帧协议） | 🚫 挂账，用户点名重入 |
| P7 | runner fixture 通道（.py 端点最小 fixture 集——eventsource Timeout 23 / fetch status-0 簇 / xhr delay 簇公共解锁件） | 📋 M2 起评估（fetch 簇 164 + 98 记账 + xhr Timeout 16 + eventsource 23） |

## 已完成切片

- **M1（2026-09-28）**：六 corpus 导入 + 基线。资产腿 = `goals/20-net-api-compat.sh`
  显式 DIRS（拉取面/记账不拉面逐域定稿）；runner 腿 = `testharness-net-api` 通道
  （`NET_API_CORPUS_SUBDIRS` + `net_api_case_skipped` + `run_net_api_subdirs`）+
  Makefile `fetch-wpt-net-api`/`testharness-net-api`；lib.sh 扩展（GITHUB_TOKEN 认证
  列目录 / `GOAL_PULL_ANY_JS=1` 拉 .any.js / .json 数据依赖随拉，opt-in 语义既有
  goal 零行为变化）。基线：fetch 29.3% / xhr 28.3% / url 66.9% / mimesniff 6.5% /
  streams 42.9% / eventsource 6.2%，聚类与修齐方向见
  [evidence/2026-09-28-m1-baseline.md](evidence/2026-09-28-m1-baseline.md)。
  经验注记：goals/lib.sh 幂等快路径对「历史轮已拉过 .html 的目录」在 opt-in any.js
  后会永久漏拉（需删 .html 触发重列）；目录含 .html 且曾有瞬时 fetch 失败不自愈。

## 下一步计划

1. **M2**：fetch 面逐簇修齐——顺序：错误面（`Should have rejected` 262）→ Body mixin
   消费（arrayBuffer/json/text 92 + request-consume 10）→ forbidden header 72 →
   Request 构造校验 16 → response-clone 流缓冲 14；runner fixture 通道（P7）随簇
   评估最小 fixture 集
2. **M3**：XHR 状态机/事件序 + EventSource + URL/mimesniff 修齐
3. **M4**：streams 底座（ReadableStream.from/BYOB/queuing strategy）+ WebSocket
   二期挂账定稿 + DC 逐项判定收口

**待用户决策清单**：（空）
