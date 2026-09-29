# 网络 API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../net-api-compat.md](../net-api-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-30（M4-S8 收口）

---

## 当前状态

**专项定位**：网络 API（fetch/XHR/URL/mimesniff/streams/EventSource）一致性收敛，
WPT 六 corpus 为验收标尺。WebSocket 二期挂账（需宿主 socket 面 + 帧协议）。

**当前进度**：M1（25.8%）→ M2 收口（66.1% / 67.9% / 70.8%）→ M3 收口（73.6% /
75.7% / 76.9% / 收尾 74.6% 分母重锚）→ M4-S1（74.7%）→ M4-S2（零净变化）→
M4-S3（74.7%）→ M4-S4（75.8%）→ M4-S5（76.0%）→ M4-S6（76.8%）→ M4-S7（76.8%）→
M4-S8（2026-09-30）——**8625/11232 = 76.8%**（fetch 69.6% / xhr 53.2% / url 73.1% /
mimesniff 100% / streams **80.9%**（825/1020——tee composite cancel + 源 error 前向 +
prototype 方法委托，tee 两页页级 Timeout 全解除）/ eventsource 6.2%）。下一切片
M4-S9：byte tee 深水（ReadableByteStreamTee——分支字节流身份 + chunk 克隆）+ pipeTo
read-ahead 泵；transfer/detach 族结构性挂账（需宿主 V8 detach）。并轨 P7 fixture 续件
与 DC 判定收口。

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
| P4 | URL 边缘语义 + mimesniff 对齐 | ✅ mimesniff 100%；url 73.1%（新分母 6065——M3 收口 USP/URL.parse/port/live 迭代/UTF-8 解码器/helper 解锁）——残余：urltestdata 全量解析深水（IDNA/toASCII）+ setters-stripping host c0 保留面（Chromium bug-compat，url crate 不可表达）挂账 |
| P5 | streams 底座一致性（fetch body 依赖） | 🔶 M4-S8 收口（80.9%——tee composite cancel + 源 error 前向 + prototype 方法委托，tee 两页页级 Timeout 全解除）；**M4-S9 残余：byte tee 深水（ReadableByteStreamTee 31 腿）+ pipeTo read-ahead 泵（3 腿）+ sink.abort 事件序（1 腿）；transfer/detach 族结构性挂账（需宿主 V8 detach——14 腿）** |
| P6 | WebSocket 二期切片（宿主 socket + 升级握手/帧协议） | 🚫 挂账，用户点名重入 |
| P7 | runner fixture 通道（.py 端点最小 fixture 集） | 🔶 首件已落（M3-S1 `inspect-headers.py`）；待评估：event-source.py / echo-content.py / delay.py 族（eventsource Timeout 23 / request-upload echo / xhr delay 簇公共解锁件） |

## 已完成切片

- **M4-S8（2026-09-30）**：Tee 复合 cancel + 源 error 前向 + prototype 方法委托
  （**8625/11232 = 76.8%**，streams 794/971 → 825/1020 = 80.9%，Δ+31；tee 两页页级
  Timeout 全解除、零 pass 回归）。分支 cancel 聚合（[reason1, reason2] 双序面）+
  reader.closed rejection 前向 error 两分支 + canonical 构造器捕获（页面换全局不受扰）
  + prototype 方法委托（getReader 等 .call(rs) 面）。make test 全绿 + clippy/fmt 干净。
  见 [evidence/2026-09-30-m4-s8-tee.md](evidence/2026-09-30-m4-s8-tee.md)。
- **M4-S7（2026-09-29）**：BYOB controller 深路径（**8594/11183 = 76.8%**，streams
  774/971 → 794/971 = 81.8%，Δ+20；全量丢腿仅 1 记账）。BYOB read **pull-into 描述符
  化**（read(view,{min}) 校验链/队列容量填充+余量回队/最小填充契约/close-partial/
  error-reject）；ReadableByteStreamController.byobRequest（view/respond 续拉/
  respondWithNewView RangeError 校验/flushPull 纳入 pull-into 条件——read-min 挂死
  根因）；autoAllocateChunkSize benign 形 + ===0 TypeError；字节流 enqueue（零长度
  TypeError/描述符填充/尾随续拉/等待读直喂——内存爆涨根因修复）；字节流 close 双模
  （普通交还已填视图/cancel given undefined）；全局 BYOBReader/BYOBRequest 类。make
  test 全绿 + clippy/fmt 干净。见
  [evidence/2026-09-29-m4-s7-byob-controller.md](evidence/2026-09-29-m4-s7-byob-controller.md)。
- **M4-S6（2026-09-29）**：WritableStream spec 状态机重做（**8572/11162 = 76.8%**，
  streams 635/910 → 774/971 = 79.7%，Δ+139；六 corpus 全量零 pass 回归）。erroring/
  abort/写队列/close sentinel 全 slot 化（aborting **65/65 全绿**——abort 信号同步/
  pendingAbortRequest 与 close-in-flight 共存/queued 写拒绝序；close **26/26 全绿**——
  close 哨兵排队/stream.close 公开方法）；写队列串行推进（AdvanceQueueIfNeeded）；
  pipeTo spec 条件 1-4 优先序 + shutdown-with-action 时序（先排空 in-flight 写）+
  signal null TypeError；writer release 后方法 TypeError 面。make test 全绿（1 处
  桥测 abort 时序随 spec 翻新）+ clippy/fmt 干净。见
  [evidence/2026-09-29-m4-s6-writable-state-machine.md](evidence/2026-09-29-m4-s6-writable-state-machine.md)。
- **M4-S5（2026-09-29）**：TransformStream spec 化重做 + writable in-flight 语义 +
  构造严格校验重做（**8433/11101 = 76.0%**，streams 548/823 → 635/910 = 69.8%，
  Δ+89 pass；页级 Timeout 11→1）。TS backpressure 机制（backpressureChangePromise/
  写链串行/SourcePull/Enqueue 翻转观察——spec §6.2）+ 传播语义（cancel/abort/terminate/
  start 门控）+ 策略分离（writable hwm 1 / readable hwm 0）；writable
  `[[inFlightWriteRequest]]`（error 时 in-flight write fulfill、queued reject——writable
  constructor 13/13 全绿）；严格校验重做（object/null 转换 + type/mode enum + callback
  转换 + 全局 Writer/Controller 类 + locked 上移 prototype——readable general **38/38**
  全绿）；`_bodyToStream` null 原型投毒免疫（fetch +2）。owning-type -2 记账
  （tentative 'owning' type 改 spec 严格拒绝）。make test 全绿（1 处存量桥测随 spec
  翻新）+ clippy/fmt 干净。见
  [evidence/2026-09-29-m4-s5-transform-writable-strict.md](evidence/2026-09-29-m4-s5-transform-writable-strict.md)。
- **M4-S4（2026-09-29）**：M4-S1 两处回退专设重做 + 随基线甄别展开的构造/控制器语义簇
  （**8344/11014 = 75.8%**，streams 271/598 → 548/823 = 66.6%，Δ+281 pass）。values()/
  @@asyncIterator 完整语义（**async-iterator 41/41 全绿**——OngoingPromise 串行 /
  preventCancel / 同步 unlock / errored reject / exhaustive 迭代后可再 getReader）；
  构造 dictionary 转换序（策略转换先行 + ExtractHighWaterMark RangeError 面 +
  UnderlyingSource 成员构造时一次读取/缓存/this 绑定 + start 同步抛错冒出构造器 +
  策略类 required 成员/全局共享 size 函数——bad-strategies 8/8、queuing-strategies
  18/20、cancel 11/11、bad-underlying-sources 21/22）；reader.[[closedPromise]] 全局化
  （页级挂死根因消除——分母 598→823 重锚）；控制器 enqueue/close TypeError 语义 +
  ReadableStreamCancel await 语义；pull 机制 spec 化（微任务 started + pullAgain 完成后
  续拉）；read() 结果 author/%Object.prototype% 形态分离（内部 _zwReadRaw——then 投毒
  防线零回退，broken-then 六腿维持全绿）；pipeTo options/signal（throwing-options 8/8、
  piping/abort 0/1→20/33）+ tee×无限源 pipeTo **4.2GB OOM 根因修复**。make test 全绿 +
  clippy/fmt 干净。见
  [evidence/2026-09-29-m4-s4-values-constructor.md](evidence/2026-09-29-m4-s4-values-constructor.md)。
- **M4-S3（2026-09-29）**：BYOB reader 专设重做（streams 全量复核**零回归** 271/598、
  fetch consume-stream **15/15 全绿**——M4-S2 回退的 6 条 byob 腿全部解锁）。
  reader 本地 `byobPending` 余量（queue/pull 双通道复制面消除——M4-S2 4.2GB 爆涨
  根因）；done 语义三足（填充即返 chunk / 排空后 closed → done / waiting 自定义
  resolve 重入 byobViewRead + close 哨兵）；fill try/catch。M4-S2 的 closed-check
  丢失与 waiting 盲区两处回退伤一并治愈。见
  [evidence/2026-09-29-m4-s2-byob-reader.md](evidence/2026-09-29-m4-s2-byob-reader.md)。
- **M4-S2（2026-09-29，零净变化）**：byob read(view) 首版甄别与回退。queue 路径
  探针单读通过，但 waiting 路径 × `_bodyToStream` 单 chunk 源 × `_RS_DONE` 填充
  退化组合触发 **4.2GB 内存爆涨**（test-guard 拦截）；修复尝试后仍挂 → 回退默认
  读取形态（closed-check 补回复核）。回退后 streams 271/598 与 M4-S1 逐案一致。
  **记账**：BYOB reader 专设切片（view 跟踪/最小填充契约/waiting × pull 重入/
  内存画像）——不并入混合切片。见
  [evidence/2026-09-29-m4-s2-byob-revert.md](evidence/2026-09-29-m4-s2-byob-revert.md)。
- **M4-S1（2026-09-29）**：streams 底座首片（**8063/10789 = 74.7%**，streams +18 /
  fetch -2 byob 残余）。queuing strategies 全局（+13）、`ReadableStream.from` 静态、
  pipeTo/pipeThrough brand 校验（dest 锁定先检不锁源 +6）、byob getReader 守卫 +
  response body 字节流标记（consume-stream 恢复 14/15）。**两处回退注记**：构造严格
  校验与 values() 命名方法均退化基线（test-utils undefined 形态 / values 腿挂
  settle），延后重做。见
  [evidence/2026-09-29-m4-s1-streams.md](evidence/2026-09-29-m4-s1-streams.md)。
- **M3 收尾（2026-09-29，M3 收口）**：helper 解锁 + 深水防护（**8047/10786 = 74.6%**，
  分母重锚 5212→10786，pass 绝对 +4041）。`/common` subset-tests helpers +
  a-element 资源补拉（url-constructor 815/893、a-element 414/892、a-element-origin
  319/412 页级解锁）；WHATWG 逐字节 UTF-8 解码器（每无效子部分一 U+FFFD——
  %FE%FF/%C2x 面）；protocol setter 剥离 + 失败无操作（URL spec scheme setter 失败
  即 return）；url crate username()/password() panic 防护（serialize_url
  catch_unwind——url-setters.any hostile 面，**整跑 abort 消除、url 全 corpus 首次
  完整跑通**）；探针 JSON 孤代理转义消毒（urltestdata 代理串 → serde 拒绝面）。
  1 处存量引擎测试随 spec 翻新（非法 protocol 由返空串改无操作）。残余记账：url
  全量解析深水（IDNA/toASCII）、setters-stripping host c0 保留面（Chromium
  bug-compat，url crate 不可表达）、eventsource/.asis/blob-range（P7 续件）。见
  [evidence/2026-09-29-m3-finisher.md](evidence/2026-09-29-m3-finisher.md)。
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

1. **M4-S9 byte tee 深水**：ReadableByteStreamTee（分支字节流身份 + chunk 克隆 +
   tee×byob 组合——byte tee 31 腿）+ pipeTo read-ahead 泵（3 腿）+ sink.abort 事件序
   （1 腿）
2. **结构性挂账**：缓冲 transfer/detach 族（spec TransferArrayBuffer——需宿主 V8
   detach 能力，JS 层不可表达——bad-buffers 10 腿 + non-transferable 4 腿）
3. **M4 并轨**：P7 续件评估（.asis / event-source.py / echo-content.py）+ blob-range
   （blob:+Range XHR 27）
4. **收口判定**：WebSocket 二期挂账定稿 + DC 逐项判定（DC-1 helpers 账本核对 /
   DC-4 make test+clippy+fmt 门禁与 reftest 零回归核对）
2. **M4 并轨**：P7 续件评估（.asis 原始 HTTP fixture / event-source.py /
   echo-content.py——eventsource Timeout 23 / xhr delay 簇 / request-upload echo）
   + blob-range（blob:+Range XHR 27）
3. **收口判定**：WebSocket 二期挂账定稿 + DC 逐项判定（DC-1 helpers 账本核对 /
   DC-4 make test+clippy+fmt 门禁与 reftest 零回归核对）

**待用户决策清单**：（空）
