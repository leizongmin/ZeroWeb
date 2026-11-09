# Network API 兼容三期 — fetch 族未导入目录 + websockets 全量 + xhr 残余

**版本**: v1.2
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）；
2026-11-06 M0 对账甄别完成（三甄别档落 evidence/，DC-4 数值门按 <10k 分支落定）；
2026-11-07 M4 coverage 复测收口——**goal 收口 ✅**
**状态**: **Done（收口）**——DC-1～DC-4 全落（判定见
[证据档](net-api-compat-v3/evidence/2026-11-07-m4-coverage-retest-eventsource.md)
§五）。批次轨迹：M0 甄别（三档 + 数值门 <10k 分支）→ M1a 枚举器窗体
解锁（+1,624P）→ M1b redirect 批次（+264P + `.h2.` 排除）→ M2 三批
全落 → M3 六簇收口（13,256P）→ M4 coverage 复测三面全升（fetch
17.07% / xhr 41.88% / websockets 4.40%）+ eventsource 簇收口
（10F+4T → 76P/0F/0T：初始失败语义 engine knife——「连接从未 establish
→ fail the connection」+ credentials mode 通道 + MessageEvent origin
序列化；fixture 五端点态机 + preflight×redirect 复合面探针定谳；
request-cancellation 排除记账——无 window.stop→在途 fetch 中止管道）。
累计 **13,275P**/1,412F/9T，逐 subtest A/B 零 P→F，
[证据档](net-api-compat-v3/evidence/2026-11-07-m4-coverage-retest-eventsource.md)。
挂账移交口径见证据档 §六（redirect-keepalive / request-cancellation
重入、cookie jar、responsexml 族）。
**执行模式**: WPT 驱动（上游 corpus 为验收标尺）+ 语料扩面 + fixture 甄别
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——网络面）
**前账**: 一期 `docs/goal/archive/net-api-compat.md`（2026-10-04 收口）；
二期 `docs/goal/archive/net-api-compat-v2.md`（2026-10-29 收口，通道七域
基线 9,354 → 11,040，收口后小片 M2-S5..S19 持续 +40 级增量）

## coverage 对账（coverage-2026-11-02 基线 → 2026-11-07 M4 复测，全量头条 67.54%）

| 面 | 全量分母 | 基线 zw 覆盖 | M4 复测 | 未覆盖（复测后） |
|---|---:|---:|---:|---:|
| fetch | 13,274 | 133/957 文件，1,894P / 14.27% | **2,266P / 17.07%** | **11,008** |
| websockets | 2,389 | 8/799 文件，102P / 4.27% | **105P / 4.40%** | **2,284** |
| xhr | 2,333 | 110/440 文件，554P / 23.75% | **977P / 41.88%** | **1,356** |

三族合计未覆盖 **15,446**。通道现状：`testharness-net-api` 已入
NET_API_CORPUS_SUBDIRS 十四目录（fetch/api/{headers,request,response,
body,basic,abort,credentials,cors} + data-urls/content-type/content-length/
h1-parsing/images/redirects 等）——**fetch/api 之外的上层目录与
fetch/api 内 py 端点密集目录从未导入**（http-cache、redirect py 面、
privacy、corp、streams、nosniff、range、multipart、schemes、destination、
vintage 等，M0 逐目录甄别）；websockets 仅 8 文件（?wss 变体面被 scheme
门筛减）；xhr 余量未收。

前账挂账锚点（2026-11-04 用户拍板拆分落位）：

- **WebSocket wss 面**（TLS 端点 fixture + 通道 scheme/端点寻址）——
  **已拆基建 goal 46 号 runner-net-infra**（v2 挂账的两前置审计随其 M0
  落地）；本 goal 只收**非 wss 依赖面**，wss 面以 46 号 DC-2 为前置。
  **已入账（2026-11-08）**：46 号 DC-2 落地——TLS 端点 fixture + D3-D5
  寻址接线，wss 池 101 案 360 subtest 全绿过批次门（[46 号 M2 档](./runner-net-infra/evidence/2026-11-08-m2-wss-wiring.md)）。
- **request-cache 族 98F**——**已拍板立项 47 号 http-cache**（引擎
  HTTP 缓存层）；本 goal 沿 v2 剔除口径，47 号交付后由其承接收敛。
- 一期/二期零回归底线继续有效（通道分母逐 subtest 对照）。

## Mission

1. **对账甄别**（M0）：fetch 未导入目录逐目录甄别——纯 window 面、
   `.py` wptserve 端点依赖面（fixture shim 可行性——v2 已有 echo-content/
   /echo-query 等先例）、worker 执行面（如实排除）；websockets 全量文件
   清单与 ws fixture 端点需求；xhr 残余失败簇。
2. **fetch 目录批量导入**（M1/M2）：过 net-api 模板（DIRS 扩面 + skip
   规则 + Makefile target + fixture shim 按需），py 端点面分批。
3. **websockets 非 wss 面入账**（M2/M3）：非 wss 依赖面全量入账；wss
   依赖面清单移交 46 号（fixture 需求 + 寻址形态），不阻塞本 goal。
4. **xhr 残余收敛 + 复测收口**（M3/M4）：失败簇归因；coverage 复测落
   数值门。

## Done Criteria

- **DC-1（M0 对账）**：fetch 未导入目录 + websockets 全量 + xhr 残余的
  构成甄别落 `docs/goal/archive/net-api-compat-v3/evidence/`（逐目录：文件数、
  py 端点依赖、批次归属）；wss 依赖面清单与 46 号交接档落档；DC-4
  数值门落定并回填本文件。
- **DC-2（通道扩面）**：M1/M2 批次目录入 testharness-net-api 常驻通道
  （DIRS + skip + Makefile + fixture 四件套）；新增 fixture 走 v2 先例
  （路由表 + echo 族）。
- **DC-3（websockets/xhr）**：websockets 非 wss 依赖面入账（covered > 0
  且通过数达数值门）；wss 面交接 46 号（清单 + fixture 需求）落档；xhr
  残余簇归因落档。
- **DC-4（coverage 复测）**：fetch/websockets/xhr 三面升至数值门；
  一期/二期通道既有账零回归（分母逐 subtest 对照）；头条不回落。

**DC-4 数值门（2026-11-06 M0 落定，走 <10k 分支）**：M0 甄别（三档见
evidence/）判定三族地址池（window 可执行 + fixture 可 shim）≈ 1.5k–3k
子测试，**远低于 10k**——不设单点总量门，改按批次落门：

1. **批次门**：每批导入面通道实跑 pass ≥ 该批 ZW 实跑子测试分母 ×
   98.5%（M0 实测通道既有通过率下限：websockets 100% [377P/0F]、xhr
   98.7% [586P/7F/1T]、fetch 腿 90.9% [1,900P/2,091，含 request-cache
   族 98F 等划界挂账面——批次门只对新增导入面计，不与挂账面混算]）；
   批次导入面 wpt.fyi den 口径含变体膨胀，不做 den 口径对齐承诺（xhr
   档第五节记账）。
2. **DC-3 面门**：websockets 非 wss 依赖面 covered > 0 且 interfaces
   批次（den 291）pass ≥ 实跑分母 × 98.5%；wss 变体面移交 46 号兑现，
   不入门。
3. **零回归门**：一期/二期通道既有账逐 subtest 对照（M0 基线
   11,035P/12,078 见
   [基线档](net-api-compat-v3/evidence/2026-11-06-m0-channel-baseline.md)；
   M1a 后换代为 12,659P/14,221——换代经 A/B 交集 12,070 对逐对字节一致
   证零回归，见 [M1a 档](net-api-compat-v3/evidence/2026-11-06-m1a-enumerator-forms-unlock.md)；
   后续批次 A/B 以同树重跑对照为准）零回归；头条不回落（三族 coverage
   通过率不低于 2026-11-05 基线：fetch 14.25% / websockets 4.27% / xhr
   23.75%）。

## 划界

- **url/mimesniff 不重开**：v2 收口账（IdnaTestV2 100% / url 94.1% /
  mimesniff 残余语料冲突记账）零回归门；重入条件仍归 v2 口径。
- **request-cache 98F 归 47 号**：http-cache goal（2026-11-04 立项）承接
  收敛；本 goal 沿 v2 剔除口径，不重复实现缓存语义。
- **wss 面归 46 号**：TLS 端点 fixture 与 scheme/端点寻址归
  runner-net-infra；本 goal 只消费其交付。
- **CORS 引擎语义不重开**：`crates/security/src/cors.rs` 既有实现（s153）
  零回归门；cors topdir 专面通道归 gap-batch1（39 号）。
- **service-workers fetch 拦截面**归 service-workers 通道既有账；本 goal
  改动同窗复验其零回归。
- **encoding/streams** 归 37 号 encoding-compat-v2 划界（ReadableStream
  面不属本 goal）。

## 里程碑

- **M0 ✅**（2026-11-06）：对账甄别 + WebSocket P3 重估 + 数值门（DC-1）——
  三甄别档落 evidence/（fetch 逐目录 / websockets 清单+wss 交接 / xhr
  残余簇归因）；wss 面不阻塞非 wss 导入（interfaces 74 案零 .https 命名）。
- **M1**：**M1a ✅**（2026-11-06，[证据档](net-api-compat-v3/evidence/2026-11-06-m1a-enumerator-forms-unlock.md)）——
  枚举器 `extra_forms` 门解锁 `.htm`/`.window.js`（默认关，仅 net-api 通道
  开启）：通道 **+1,624P**（11,035 → 12,659P；url +1,272 / xhr +317 /
  eventsource +22 / fetch +12 / streams +1），案 543 → 684，A/B 交集
  12,070 对逐对字节一致零回归；新 Fail 面 = 解锁面真实缺口显影（
  access-control preflight 端点 / h1-parsing 原始线缆端点等），转 M1b/M2
  fixture 分批与 M3 归因输入。**M1b ✅**（2026-11-06，[redirect 批次档](net-api-compat-v3/evidence/2026-11-06-m1b-redirect-batch.md) +
  [簇归因图](net-api-compat-v3/evidence/2026-11-06-m1b-folder-txt-and-xhr-cluster-map.md)）：
  folder.txt 补拉（+2P）；fetch/api/redirect 14 案导入（+364 子测试
  264P/99F/1T）+ redirect.py fixture 升上游 per-token stash 语义（键
  命名空间化防 preflight.py 误读，曾致 cors-redirect 20 案 P→F）+
  redirect-empty-location.py / method.py 新 fixture + fetch shim 六项
  重定向修复（error/empty-location/data-scheme/循环上限/303 HEAD/redirected）；
  全量 A/B 12,655 → 12,924P，changed 15 全 F→P 零 P→F；`.h2.` 形态案
  正式排除（26 子测试移出分母）；redirect 残余 99F+1T 七簇归 M3
  （referrer 63 / mode 11 / origin 10 / method 9 / escape 4 /
  back-to-original 2 / keepalive 1T），xhr redirect 引擎缺口并簇 M3。
- **M2 ✅**（三批次全落，批次差额随 M3 复验）：
  **ws interfaces 批次**（[证据档](net-api-compat-v3/evidence/2026-11-06-m2-ws-interfaces-batch.md)）：
  websockets/interfaces 74 案导入（+132 子测试 88P/45F，全量 A/B
  changed 0 零回归）+ ws_fixture `/echo_raw`（裸字节回写，保留 opcode
  注入面）与 `/sleep_10_v13`（握手延迟 10s）两新端点——close-connecting
  10s 时序面一次通过；45F 九簇全引擎侧归 M3（接口常量 ReadOnly 16F /
  EventTarget dispatchEvent 9F / 属性描述符 6F / 事件处理器 handleEvent 形
  4F / send 状态机 3F / 协议错误断连 3F / CONNECTING fail 语义 2F /
  事件序 1F / CloseEvent ctor 1F），批次门 66.2% 随 M3 收敛复验。
  **auth 面批次 ✅**（2026-11-06，[证据档](net-api-compat-v3/evidence/2026-11-06-m2-auth-face.md)）：
  auth fixture 套件（xhr authentication.py + auth1-11 三变体 token stash
  挑战流 / fetch authentication.py 固定凭据版 / dump-authorization-header
  OPTIONS strip_auth_header 500 探测臂）+ 引擎 cors check 按 spec 步序
  重排（`*` 通配先行，opaque origin 仅换比对基准——`null`；曾致
  cors-redirect 十案回退的中间迭代见
  [learning](../../learnings/bugs/2026-11/2026-11-06-cors-check-asterisk-step-precedes-origin-compare.md)）。
  全量 A/B changed 17（16 F→P + 1 诚实翻转 send-authentication-basic-cors
  vacuous P）零 P→F 外溢，+15P 至 13,005P；fetch auth 面全绿 7P，
  xhr auth 12F 五簇（URL/open() userinfo 凭据 8F / 401 挑战会话 2F /
  跨源透明 auth 拒绝 1F / preflight ACAH enforcement 1F）归 M3。
  **getResponseHeader 响应头控制面 + preflight 端点扩表批次 ✅**
  （2026-11-06，[证据档](net-api-compat-v3/evidence/2026-11-06-m2-response-headers-preflight-batch.md)）：
  fixture 十一端点（headers.py / chunked.py / headers.asis / parse-headers.py /
  safelisted-request/response-headers / no-custom-header-on-preflight /
  lowercase / sorted / headers-origin / invalid-status）+ 引擎三刀
  （E1 响应头值含 NUL → 网络错误、E2 sync preflight 最小头集镜像异步
  M4-S26、E6 sync 终响应跨源 cors check + cors-filtered 过滤）。
  xhr 面定向 +33P（33 F→P 零 P→F：normalize 15 / getresponseheader 族 5 /
  preflight 端点 7 / E6 语义暴露 6）；全量 A/B 分母 14,692 不变
  changed 33 零外溢，13,031P → **13,064P**；invalid-status ×3 与 status/
  unsent-opened ×2 vacuous P 转真实 P；批次门 97.4%（38/39）差一线，
  `__custom` 排序簇 1F（wpt.fyi Firefox 7/7 定谳的浏览器头序 vs 引擎
  字节升序）挂账随 M3 复验。
- **M3 ✅**（2026-11-07 在账余项清零收口）：
  **ws interfaces 九簇 ✅**（2026-11-06，[证据档](net-api-compat-v3/evidence/2026-11-06-m3-ws-interfaces-clusters.md)）：
  part06.js 八刀——CloseEvent 内部槽 + getter-only accessor + ctor code 缺省
  0 + toStringTag、WebIDL 常量双挂 ReadOnly、url/readyState/bufferedAmount
  内部槽、on* 双语义（原样存 + 哨兵注册序）、dispatchEvent、send 三态、
  fail-connection error 恰一次（err wire 标记与派发门拆双标记——018 二轮
  根因，015 的 P 不证明派发）；定向 88P/45F → 133P/0F；全量 A/B
  13,064 → **13,109P**（+45P，分母 14,692 不变，changed 45 全 F→P 零
  P→F 零 T 变化）；批次门 100%，DC-3 interfaces 面门达成。
  **fetch redirect 七簇 ✅**（2026-11-06，[证据档](net-api-compat-v3/evidence/2026-11-06-m3-redirect-engine-clusters.md)）：
  part01.js 六刀 + testharness.rs 两刀——referrer 链式逐跳重算（Referrer-Policy
  替换 + 上一跳外发值为源，双引擎 128/128 定谳）、mode 三刀（sidecar 全量头 /
  no-cors 跨源 manual 入口门 / cors check 先于 opaqueredirect 转换）、hop
  Origin 发射门（clean 同源跳不发射 / 已污染 sticky 'null'）、hop CL 空串体门
  + 303 任意非 GET/HEAD 改写、response tainting 持久（whatwg/fetch#737）、
  Location 字节序列保真（fixture `wpt_query_value_bytes` + shim `%XX` 单字节
  转义——字节语义根因落
  [learning](../../learnings/bugs/2026-11/2026-11-06-response-header-values-are-byte-sequences.md)）；
  定向 338P/25F/1T → 363P/0F/1T（批次门 99.7%）；全量 A/B 13,109 →
  **13,211P**（+102P，分母 14,650 键不变，changed 102 全 F→P 零 P→F 零
  T 变化，链外搭车 3 行逐行溯源）。
  余项：xhr auth 五簇（URL/open() userinfo 凭据解析 8F / 401 挑战流 UA 会话
  2F / 跨源透明 auth 拒绝 1F / preflight ACAH enforcement 1F）+
  getAllResponseHeaders 排序簇（浏览器头序 vs 引擎字节升序，1F）+
  重定向终跳 sync cors-filter 同规 + xhr 在账失败簇收敛
  （formdata/submitter 簇 4F + overrideMimeType Shift-JIS 接线 1F +
  cookies.http.html 挂起 1T + redirect-keepalive 1T；permissions-policy 2F
  策略面排除）（DC-3）。
  **xhr auth 五簇 ✅**（2026-11-06，[证据档](net-api-compat-v3/evidence/2026-11-06-m3-xhr-auth-clusters.md)）：
  part02.js + part03.js 五刀——open() 非 null 覆写矩阵（8F）、URL 凭据延迟
  注入 + 401 Basic 挑战同源透明重试一次 + UA 会话缓存（2F）、跨源挑战不透传
  JS（async error / sync throw，1F）、sync preflight ACAH 覆盖镜像（失败
  分型：HTTP 级失败同步抛 vs 覆盖失败 error 事件——语料双面标尺，1F）、
  同步预检名单对齐 `_zwPreNamesOf` 单一事实源（首刀 PUT P→F 回归由全量 A/B
  拦截后根因修复）；定向 auth 面 23/23（wpt.fyi Edge 159 auth 9 文件 8 绿
  基准，cors-no-cred auth8 腿语料标尺反超）；全量 A/B 13,211 → **13,223P**
  （+12P，分母 14,692 行不变，changed 12 全 F→P 零 P→F 零 T 变化）。
  余项：`__custom` 排序簇（1F）+ 重定向终跳 sync cors-filter 同规 +
  xhr 在账失败簇（formdata/submitter 4F + overrideMimeType Shift-JIS 1F +
  cookies 1T + redirect-keepalive 1T）（DC-3）。
  **getAllResponseHeaders 排序簇 ✅**（2026-11-06，[证据档](net-api-compat-v3/evidence/2026-11-06-m3-getallresponseheaders-sort.md)）：
  part03.js 单刀——序列化排序落实 xhr.spec **legacy-uppercased-byte less than**
  比较器（名 byte-uppercased 后 byte 比较，`_`=0x5F 恒大于大写字母域，
  `__custom` 排小写字母名后——与 fetch Headers 迭代面纯 byte 排序两套口径，
  headers-sorting 族既有绿面零扰动；wpt.fyi「Firefox 7/7 浏览器头序」注记
  落实为规范文本，M2 批「浏览器头序」假设证伪回退留痕证据档）；定向
  getallresponseheaders 10/10；全量 A/B 13,223 → **13,224P**（+1P，分母
  14,692 行不变，changed 1 全 F→P 零 P→F 零 T 变化）。
  余项：重定向终跳 sync cors-filter 同规 +
  xhr 在账失败簇（formdata/submitter 4F + overrideMimeType Shift-JIS 1F +
  cookies 1T + redirect-keepalive 1T）（DC-3）。
  **send-redirect-to-cors 八簇 ✅**（2026-11-07，[证据档](net-api-compat-v3/evidence/2026-11-07-m3-send-redirect-to-cors-fixture.md)）：
  testharness.rs corsenabled.py fixture 两处（`?safelist_content_type` 追加
  第二条 ACAH content-type——上游逐字核对补齐；`x-request-data` 回显回落文本
  体 `body`——fetch_bridge 文本/二进制互斥契约），**零引擎改动**。逐跳请求
  探针归因：在账「重定向终跳 sync cors-filter 同规」标签证伪——引擎
  preflight/redirect/cors-filter 语义本已齐备（期望 0 的对称案全绿互证），
  8F 全部为 fixture 缺口；跨案状态污染假设经最小序列复刻页排除。定向
  send-redirect-to-cors 23/23；全量 A/B 13,224 → **13,232P**（+8P，分母
  14,692 行不变，changed 8 全 F→P 零 P→F 零 T 变化）。
  余项：xhr 在账失败簇（formdata/submitter 4F + overrideMimeType Shift-JIS 1F +
  cookies 1T + redirect-keepalive 1T）（DC-3）。
  **overridemimetype 族 + 响应解码链 ✅**（2026-11-07，[证据档](net-api-compat-v3/evidence/2026-11-07-m3-overridemimetype-encoding-chain.md)）：
  part03.js 两刀（open() 去 override 重置——spec 明注 overrideMimeType 可先于
  open 调用；`_zwXhrDecode` 落 **get a final encoding** 链——response CT charset
  → override charset 覆写，spec 明注不用 final MIME type）+ part02.js
  `_zwXmlNode` tagName getter（XML 树 responseXML.tagName 断言面）+
  testharness.rs status.py content 双重解码修正（`wpt_query_value_bytes` 字节
  保真——lossy 曾把 %83/%FF/%FE%FF 载荷毁成 U+FFFD，shiftjis 族 got 串指纹
  锚定 fixture 根因）。在账标签修正：M0「1F」为 `.htm` 解锁前口径，该族实际
  5F+1T 全数清零。定向族内 13/13；全量 A/B 13,232 → **13,253P**（+21P，
  分母 14,621 键不变，changed 21 = F→P 20 + T→P 1 零 P→F；链外搭车
  responsetext-decoding 13F + send-receive-utf16 1F 同根因逐行溯源；
  responsexml 族 19 行在账外存量缺口不动）。
  余项：xhr 在账失败簇（formdata/submitter 4F + cookies 1T +
  redirect-keepalive 1T）（DC-3）。
  **formdata/submitter 簇收口 ✅**（2026-11-07，[证据档](net-api-compat-v3/evidence/2026-11-07-m3-formdata-closest-click-coords.md)）：
  在账 4F 逐案归因四分。修复双案：F1 FormDataEvent（三刀 shim——part03
  `Element.prototype.closest` plain 世界新装（s82 单 compound 口径）+
  part02 FormData 构造器 owner 判定分源（②源子树查询结构性属主免 closest
  复验）+ 去重键碰撞子树源原位顶替——tag 级模糊 sel 身份下 doc 级逗号列表
  占位会吞结构正确字段）；F4 submitter 坐标（webview `dispatch_planned_event`
  click/dblclick in-view 中心探测经既有 DomEventDetail 坐标通道 + part06 泛型
  click clientX/Y floor 注入与 image 按钮记账臂——host 通道此前无坐标且不跑
  页面默认动作，recorder 只在页面事件路径可达）。挂账双案归因重分类移交：
  F2 unpaired surrogate（DOMString/UTF-16 引擎桥语义欠账）/ F3 图像固有尺寸
  （render-foundation/layout-engine 渲染流域，run-rules §9 工作面不重叠），
  不计 DC-3 门。定向 formdata 族 87P/2F；全量 A/B 13,253 → **13,255P**
  （+2P，分母 14,621 键不变，changed 2 全 F→P 零 P→F 零 T 变化）。
  余项：xhr 在账失败簇（cookies 1T + redirect-keepalive 1T）（DC-3）。
  **cookies/keepalive 尾批 2T 收口 ✅**（2026-11-07，[证据档](net-api-compat-v3/evidence/2026-11-07-m3-cookies-keepalive-tail.md)）：
  双 T 探针定谳分派。cookies.http.html T→P：testharness.rs get-set-cookie.py
  fixture cookie 状态机（handler per-case 表——Set-Cookie 双枚 wire 保真 +
  请求 cookie 回显体逐枚前置空格 + Secure cookie 不随 http 送达；此前 POST
  撞非 GET 兜底门 handler Err → onerror 挂起案级 Timeout）；
  redirect-keepalive.any.js（window 变体）排除记账：多窗口编排面
  （window.open(NOTSAMESITE) 子窗 + opener postMessage——探针实证
  openRet=null/msg=null，六案 setting up 全挂），M0 甄别档 multi-globals/
  unload-a-document「runner 无多文档管道」同因口径，multi-window 管道落地后
  重入；.https 变体走 iframe flow 不在此门维持既有绿面。定向双案
  cookies.http.html 1P / redirect-keepalive 仅余 .https 双 P；全量 A/B
  13,255 → **13,256P**（+1P，分母 14,621 → 14,620 键——排除移出 1；added 1
  = cookies 子测试，removed 2 = 双案级 Timeout 键；零 P→F 零 T 变化）。
  **M3 在账余项清零** → M4 coverage 复测收口（DC-4）。
- **M4 ✅**（2026-11-07，[证据档](net-api-compat-v3/evidence/2026-11-07-m4-coverage-retest-eventsource.md)）——
  **coverage 复测（DC-4）**：wptfyi-cache 10-29 summaries 同 universe 复测
  三面全升——fetch 1,894P/14.27% → **2,266P/17.07%**（+372P）、xhr
  554P/23.75% → **977P/41.88%**（+423P）、websockets 102P/4.27% →
  **105P/4.40%**（+3P），分母逐字节同口径（fetch 13,274 / xhr 2,333 /
  websockets 2,389）；全量头条由 weekly cron 产出，本 goal 只复测三面。
  **eventsource 簇收口**（M1a 窗体解锁显影的未归因尾巴，71 案 10F+4T →
  76P/0F/0T）：engine knife——初始连接失败语义（spec「连接从未 establish →
  fail the connection」重连例外分支：非 http(s) scheme 预检 + `_everAnnounced`
  分派 rejection + sync throw failConnection）+ MessageEvent origin 属性
  改连接 URL origin 序列化（此前派全 URL）+ credentials mode 通道
  （withCredentials → include，链路既有仅缺传参）；fixture knife——
  es_cookie_stash 表承载 reconnect-fail/status-reconnect/cors-cookie 三态机
  （M3 get-set-cookie.py 先例）+ cors.py run=message/redirect/status-reconnect
  委派三臂 + last-event-id2.py 静态三流 + OPTIONS preflight 先于委派早退
  （探针三腿 bisect 定谳的 preflight×redirect 复合面：委派臂不分手 →
  preflight 落 302 → 2xx 门不过）；request-cancellation 排除记账
  （window.stop 仅 navigation-API abort，无在途 fetch 中止管道，fetch
  cancellation 域落地后重入）。定向 76P/0F/0T；全量 A/B 13,256 →
  **13,275P**（+19P，分母 14,690 → 14,695 键；F→P 9 / F→排除移出 1 /
  Timeout 键移出 4 + 子测试入账 10；零 P→F 零 T 变化零其他变化）。
  **DC-4 判定落定**（批次门随各批落账 + M1a 尾巴补齐 / DC-3 面门 100% /
  零回归门 / 头条不回落）→ **DC-1～DC-4 全落，goal 收口**。

## 依赖约束（run-rules §9）

- net-api 通道改动（DIRS/skip/fixture/shim）后，一期/二期通道基线
  （v2 终账 11,040 + S 系列增量）逐 subtest 零回归复验。
- ws fixture 与 ws_host_factory 接线只挂 websockets 用例（run-rules §10
  通道级隔离先例），CSP/cookies 通道 WebSocket 用例零耦合。
- weekly 分片预算：fetch 大目录单套件耗时对照分片表，必要时套件级分批
  （form-encode 先例：单套件单进程超 14400s 即拆）。
- fixture shim 语义改动（路由/echo 族）先落 evidence 再合，防 v2 通道
  既挂 fixture 的面（S3b `/echo-query_v13` 等）回归。
