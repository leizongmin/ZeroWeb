# 网络 API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../net-api-compat.md](../net-api-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-29（M3-S3 收口）

---

## 当前状态

**专项定位**：网络 API（fetch/XHR/URL/mimesniff/streams/EventSource）一致性收敛，
WPT 六 corpus 为验收标尺。WebSocket 二期挂账（需宿主 socket 面 + 帧协议）。

**当前进度**：M1（25.8%）→ M2 收口（66.1% / 67.9% / 70.8%）→ M3-S1（73.6%）→
M3-S2（75.7%）→ M3-S3（2026-09-29）——**4006/5212 = 76.9%**（fetch 70.1% / xhr 53.2% /
url 79.1% / mimesniff 100% / streams 42.5% / eventsource 6.2%）。下一切片：M3 收尾
（setters-stripping protocol/host c0 面 + a-element 装载诊断）与 M4（streams 底座 +
WebSocket 挂账定稿 + DC 判定）。

**与兄弟 goal 的边界**：
- security-hardening — CSP 对 fetch 的策略执行归其；本 goal 提供语义钩子位
- service-workers（已归档）— 其 fetch 通道已收口；拦截扩展另行记账
- zero-web P1a — URL/URLSearchParams 既有实现为本 goal 修齐对象（改动走本 goal 账本）
- rendering-compat 及渲染流 — 无共享 crate 面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 六 corpus fetch 脚本 + 导入 + 基线 | ✅ M1（2026-09-28，基线 1297/5019 = 25.8%） |
| P2 | fetch/Request/Response/Headers/Body 语义收敛 | ✅ M2 主体收口（S1 scheme dispatch + S2 Headers 校验 + S3 consume body 层；fetch 62.7% / mimesniff 100%）；残余：response-error 族（10）、request-upload echo（.py——P7）、错误面余量甄别 → 并轨 M3/P7 处理 |
| P3 | XHR 状态机 + EventSource 解析/重连 | 🔶 M3 进行中——S1（header 校验）+ S2（responseType/同步/overrideMime）已收口（xhr 53.2%）；**残余：blob-range（27，blob:+Range）、progress 事件序、.asis/eventsource fixture 联动 P7** |
| P4 | URL 边缘语义 + mimesniff 对齐 | 🔶 mimesniff ✅ 100%；url 79.1%（S3 收口 USP 容错解码/URL.parse/port/live 迭代）——残余：setters-stripping protocol/host c0 面（62）+ a-element 装载诊断 |
| P5 | streams 底座一致性（fetch body 依赖） | ⏳ M4（主簇：ReadableStream.from 32 / BYOB view 16 / queuing strategy 11 / pull 时机 8） |
| P6 | WebSocket 二期切片（宿主 socket + 升级握手/帧协议） | 🚫 挂账，用户点名重入 |
| P7 | runner fixture 通道（.py 端点最小 fixture 集） | 🔶 首件已落（M3-S1 `inspect-headers.py`）；待评估：event-source.py / echo-content.py / delay.py 族（eventsource Timeout 23 / request-upload echo / xhr delay 簇公共解锁件） |

## 已完成切片

- **M3-S3（2026-09-29）**：URL 修齐（4006/5212 = **76.9%**，Δ+60 零回归）。
  urlencoded 容错 percent-decode（URIError 根因消除）+ USP sequence 构造（严格二元组）
  + live 光标迭代/size getter/delete 可选参 + URL.parse 静态（16/16）+ host port
  setter 剥离解析（setters-stripping 168→198/260）。残余记账：setters-stripping
  protocol/host c0 面（62）、TextDecoder FFFD 合并（encoding 域相邻）、a-element
  装载诊断、opaque-path search 序列化。见
  [evidence/2026-09-29-m3-s3-url.md](evidence/2026-09-29-m3-s3-url.md)。
- **M3-S2（2026-09-29）**：XHR 状态机主片（3946/5212 = **75.7%**，Δ+122 零回归）。
  responseType 状态机（responsetype.any **50/50**）+ 同步 XHR（host 同步契约 +
  `_zwFillFromResponse` 共享填充）+ overrideMimeType（octet-stream parse 回落 /
  charset 优先 / final MIME text/xml 回落——overridemimetype-blob **77/77**、
  done-state 1/1）+ responseType 效果（json/arraybuffer/blob）+ getResponseHeader
  空值修复 + ProgressEvent init IDL 转换（**10/10**）+ P7 续件（status.py 原始字节
  content + 站点根页）。残余记账：shift-jis 解码归 encoding-compat goal、IDL
  interface 保真挂账、.asis fixture 待评估。见
  [evidence/2026-09-29-m3-s2-xhr-state.md](evidence/2026-09-29-m3-s2-xhr-state.md)。
- **M3-S1（2026-09-29）**：XHR setRequestHeader 校验 + SAB send 守卫 + P7 fixture
  首件（3824/5198 = **73.6%**，Δ+142 零回归）。setRequestHeader：OPENED/未发送状态校验
  （InvalidStateError）+ name/value 校验（SyntaxError，先 Normalize 后 validate）+
  combine 合并；send(SharedArrayBuffer) → TypeError（14/14）；runner `inspect-headers.py`
  内置 fixture（上游逐字等价生成器，零 .py 落盘）解锁 fetch headers 回读腿
  （fetch 926→1209）。见
  [evidence/2026-09-29-m3-s1-xhr-headers.md](evidence/2026-09-29-m3-s1-xhr-headers.md)。
- **M2-S3（2026-09-28）**：consume body 统一层 + 流 disturbed/locked 语义（3682/5198 =
  **70.8%**，Δ+155 零 pass 回归）。`_zwConsumeBodyBytes`（unusable 拒绝 / 用户流源
  error+chunk 类型传播 / 字节路径 + 消费后锁定扰动）；Response(ReadableStream) 源 +
  read/cancel 反向标记 bodyUsed；clone() unusable TypeError + 最小 tee（原分支引用 +
  结构化克隆拷贝 + cancel 隔离）；formData essence 白名单 + RFC2046 分隔符行校验；
  body 派生默认 Content-Type；then 投毒防护（null 原型 read-result + 字节结果自有
  then:undefined）；bytes() 补全。disturbed-1..6 全绿、error-from-stream 14/14、
  bad-chunk 6/6、clone 5→20/21、form-data 7→13/14、request-consume 14→43/45。
  5 处存量引擎测试随 consume-once 语义翻新（clone 先于消费 / formData 补 essence 头）。
  门禁注记：make test 两轮各触在册争用窗 flaky 族（stale_etag / skip_waiting——与本
  轮零涉），隔离复跑双绿 + 空闲全量 68 套全绿收口（R4867 判例同型）。见
  [evidence/2026-09-28-m2-s3-body-streams.md](evidence/2026-09-28-m2-s3-body-streams.md)。
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

1. **M3 收尾**：setters-stripping protocol/host/hostname c0 面（62——scheme/host
   setter 剥离语义）+ a-element 装载诊断（url corpus 页级 0/1 案）+ TextDecoder
   FFFD 合并保真（与 encoding-compat 域协同）
2. **M3-S4 评估**：blob-range（blob:+Range XHR 27）+ P7 续件（.asis 原始 HTTP
   fixture / event-source.py / echo-content.py——eventsource Timeout 23 / xhr delay 簇）
3. **M4**：streams 底座（ReadableStream.from 32 / BYOB view 16 / queuing strategy 11 /
   pull 时机 8）+ WebSocket 二期挂账定稿 + DC 逐项判定收口

**待用户决策清单**：（空）
