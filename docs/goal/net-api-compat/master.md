# 网络 API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../net-api-compat.md](../net-api-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-28（M2-S2 收口）

---

## 当前状态

**专项定位**：网络 API（fetch/XHR/URL/mimesniff/streams/EventSource）一致性收敛，
WPT 六 corpus 为验收标尺。WebSocket 二期挂账（需宿主 socket 面 + 帧协议）。

**当前进度**：M1（25.8% 基线）+ M2-S1（66.1%）+ M2-S2（2026-09-28）——
**3527/5193 = 67.9%**（fetch 53.7% / xhr 23.8% / url 67.3% / **mimesniff 100%** /
streams 42.9% / eventsource 6.2%）。下一切片 M2-S3（response-stream-disturbed 簇）
或并轨 M3（XHR 状态机——header-values 腿依赖）。

**与兄弟 goal 的边界**：
- security-hardening — CSP 对 fetch 的策略执行归其；本 goal 提供语义钩子位
- service-workers（已归档）— 其 fetch 通道已收口；拦截扩展另行记账
- zero-web P1a — URL/URLSearchParams 既有实现为本 goal 修齐对象（改动走本 goal 账本）
- rendering-compat 及渲染流 — 无共享 crate 面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 六 corpus fetch 脚本 + 导入 + 基线 | ✅ M1（2026-09-28，基线 1297/5019 = 25.8%） |
| P2 | fetch/Request/Response/Headers/Body 语义收敛 | 🔶 M2 进行中——S1（scheme dispatch + data: + Body MIME）与 S2（Headers 校验 + guard + 迭代语义）已收口（fetch 53.7% / mimesniff 100%）；**S3 = response-stream-disturbed 簇（33）+ request-upload/clone 流（14）**；错误面余量甄别 |
| P3 | XHR 状态机 + EventSource 解析/重连 | ⏳ M3（xhr 主簇：send 错误面 30；url 66.9% 修齐：percent-encoding 39 / default port ~40 / URL.parse 8；mimesniff ✅ 99.2%；eventsource 依赖 fixture 通道） |
| P4 | URL 边缘语义 + mimesniff 对齐 | 🔶 mimesniff 已收口（99.2%）；url 归 M3 |
| P5 | streams 底座一致性（fetch body 依赖） | ⏳ M4（主簇：ReadableStream.from 32 / BYOB view 16 / queuing strategy 11 / pull 时机 8） |
| P6 | WebSocket 二期切片（宿主 socket + 升级握手/帧协议） | 🚫 挂账，用户点名重入 |
| P7 | runner fixture 通道（.py 端点最小 fixture 集——eventsource Timeout 23 / fetch status-0 簇 / xhr delay 簇公共解锁件） | 📋 M2 起评估（fetch 簇 164 + 98 记账 + xhr Timeout 16 + eventsource 23） |

## 已完成切片

- **M2-S2（2026-09-28）**：Headers 校验 + guard 完整化（3527/5193 = **67.9%**，Δ+178
  零回归）。name/value 校验（TypeError）+ sequence 二元组 + ctor null/1 throw；
  X-HTTP-Method-Override forbidden 值判定（get-decode-split quoted 感知）；request-no-cors
  guard（CORS-safelisted 判定 + Range privileged 清除）；sort-and-combine 迭代（set-cookie
  逐值 + live 光标 + %ArrayIteratorPrototype% 链）；fetch() 同步异常转 rejected Promise；
  runner `self.GLOBAL` shim。mimesniff **100%**。3 处存量引擎测试期望随 spec 翻新
  （sort-and-combine 升序 / no-cors 丢非 safelisted 头）。残余：XHR 腿归 M3、
  headers-record Proxy 观测面（10）挂账。见
  [evidence/2026-09-28-m2-s2-headers.md](evidence/2026-09-28-m2-s2-headers.md)。
- **M2-S1（2026-09-28）**：fetch scheme dispatch + data: URL processor + Body MIME
  type（1297 → 3349 = **66.1%**，Δ+2052 零回归）。bad port 79 端口表（83/83）、
  data: processor + forgiving-base64 + MIME parse/serialize（154/154 + scheme-data
  8/8 含 HEAD null-body）、blob: store 分派（18/18）、about/file/未知 scheme
  network error（7/7 + 16/16）；FileAPI Blob.type parse+serialize + Body blob()
  get-the-MIME-type → mimesniff 99.2%。细节与残余簇见
  [evidence/2026-09-28-m2-s1-scheme-dispatch.md](evidence/2026-09-28-m2-s1-scheme-dispatch.md)。
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

1. **M2-S3**：response-stream-disturbed 簇（disturbed/locked 语义 33）+ request-upload/
   clone 流缓冲（14）+ fetch 错误面余量甄别
2. **M3**：XHR 状态机/事件序（setRequestHeader 校验——header-values 腿 64 subtest
   依赖）+ EventSource（fixture 通道联动）+ URL 修齐（percent-encoding 39 / default
   port ~40 / URL.parse 8）
3. **M4**：streams 底座（ReadableStream.from/BYOB/queuing strategy）+ WebSocket
   二期挂账定稿 + DC 逐项判定收口

**待用户决策清单**：（空）
