# 网络 API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../net-api-compat.md](../net-api-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-01（M4-S29 **收口判定**——DC-1~4 全满足，目标 Completed；
同日清理下一步计划收口残文并对齐缺口清单至收口口径）

---

## 当前状态

**专项定位**：网络 API（fetch/XHR/URL/mimesniff/streams/EventSource）一致性收敛，
WPT 六 corpus 为验收标尺。WebSocket 二期挂账（需宿主 socket 面 + 帧协议）。

**当前进度**：M1（25.8%）→ M2 收口（66.1% / 67.9% / 70.8%）→ M3 收口（73.6% /
75.7% / 76.9% / 收尾 74.6% 分母重锚）→ M4-S1（74.7%）→ M4-S2（零净变化）→
M4-S3（74.7%）→ M4-S4（75.8%）→ M4-S5（76.0%）→ M4-S6（76.8%）→ M4-S7（76.8%）→
M4-S8（76.8%）→ … → M4-S26（79.7%）→ M4-S27（评估记账）→ M4-S28（79.7%）→
**M4-S29 收口判定（2026-10-01）——DC-1~4 全满足，目标 Completed**。终态
**9292/11659 = 79.7%**（M1 基线 1297/5019 = 25.8%）：fetch 71.6% / xhr 95.4% /
url 73.0% / mimesniff 100% / streams 88.7% / eventsource 100%（隔离跑）。门禁：
make test 19547P/0F + clippy -D warnings + fmt 干净 + **reftest 700/700 = 100%**
（收口核对实跑，零渲染回归）。收口判定逐项核对见
[evidence/2026-10-01-m4-s29-closure.md](evidence/2026-10-01-m4-s29-closure.md)。
**M4-S23 收口**：fetch/api/cors 域导入（M1 记账重入条件兑现）+ shim preflight 语义
修齐（Accept: */*、ACAM/ACAH 覆盖判定 spec 化、Max-Age 0 不缓存、跨源请求重定向跨源
→ Origin opaque + 跳间 Origin 替换）+ preflight.py/clean-stash.py fixture（须置于
S18 泛化 OPTIONS 兜底门之前——假阴性根因）+ redirect.py 对齐上游（redirect_preflight
落穿 + Location 全量 query）+ 路径点段归一。见
[evidence/2026-10-01-m4-s23-cors-domain.md](evidence/2026-10-01-m4-s23-cors-domain.md)。

**与兄弟 goal 的边界**：
- security-hardening — CSP 对 fetch 的策略执行归其；本 goal 提供语义钩子位
- service-workers（已归档）— 其 fetch 通道已收口；拦截扩展另行记账
- zero-web P1a — URL/URLSearchParams 既有实现为本 goal 修齐对象（改动走本 goal 账本）
- rendering-compat 及渲染流 — 无共享 crate 面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 六 corpus fetch 脚本 + 导入 + 基线 | ✅ M1（2026-09-28，基线 1297/5019 = 25.8%） |
| P2 | fetch/Request/Response/Headers/Body 语义收敛 | ✅ M2 主体收口 + M4-S23~S26 cors 域收口（fetch **1491/2081 = 71.6%** / mimesniff 100%）；残余：credentials/cookie 管道腿 ~13（跨域挂账）、response-error 族（10） |
| P3 | XHR 状态机 + EventSource 解析/重连 | ✅ EventSource 100% 收口（M4-S13）；XHR 主体收口（xhr **557/584 = 95.4%**——状态机/事件序/upload ProgressEvent/preflight cache/redirect/preflight/expose/FormData/timeout 面 spec 化）——**残余：page-scheme 失配 2（http vs https 宇宙——需 runner 页面 scheme 对齐）、sync XHR 结构面 4、XML 文档面（engine XML DOM 深度——template-element 族）、bad-chunk 注入 2、formdata FormDataEvent/submitter 族 5、timeout 族 3** |
| P4 | URL 边缘语义 + mimesniff 对齐 | ✅ mimesniff 100%；url 73.0%（新分母 6058）——残余：urltestdata 全量解析深水（IDNA/toASCII）+ setters-stripping host c0 保留面（Chromium bug-compat，url crate 不可表达）挂账 |
| P5 | streams 底座一致性（fetch body 依赖） | ✅ M4-S11 收口（88.6%）+ M4-S28 flow-control 清偿（**88.7%**，piping 全绿含 flow-control 5/5）；**残余：transfer/detach 族结构性挂账（需宿主 V8 detach——14 腿，JS 层不可表达）** |
| P6 | WebSocket 二期切片（宿主 socket + 升级握手/帧协议） | 🚫 挂账，用户点名重入 |
| P7 | runner fixture 通道（.py 端点最小 fixture 集） | ✅ 累计 24+ 端点（M4-S21 增 echo-headers/xhr trickle/form/echo-content-type/access-control-origin-header/over-1-meg.txt；M4-S22 增 preflight cache 族 ×3 等九件；M4-S23 增 preflight.py/clean-stash.py——fetch/api/cors 域页拉取已随域导入收口） |

## 已完成切片

- **M4-S28（2026-10-01）**：flow-control 背压编排（**9292/11659 = 79.7%**，
  streams 903→**905 = 88.7%**，flow-control **5/5 全绿**，+2 零回归）。shim：
  pipeTo 泵**串行读门**（`pendingRead` 在飞时不再入泵——watchReady/write 完成路径
  重入并发双读、超额消费背压余量，pump 日志探针定位；spec pipeTo 逐 chunk
  read→write→desiredSize 检查串行）。M4-S11 flow-control 挂账清偿；M4 里程碑除
  结构性挂账（V8 detach）外全部清偿。make test 全绿（19547P）+ clippy/fmt 干净。
  见 [evidence/2026-10-01-m4-s28-flow-control.md](evidence/2026-10-01-m4-s28-flow-control.md)。
- **M4-S27（2026-10-01，双评估记账零实施）**：① cors 域 credentials/cookie 腿
  ~13（需真实 cookie jar + Authorization 重定向保留流——HTTP 栈/存储状态面非本
  goal 范围）→ 回流 storage goal / security-hardening 邻域；② runner 页面 scheme
  对齐（https→http）全量实测——**解锁 +29**（两腿长挂账 + cors-origin/cors-basic
  端口协议变体 + url http: 形态）vs **耦合回归 −41**（url a-element about:blank
  基址回退 ×20 + credentials userinfo 流 ×21），净 −12 → 记账不定稿，重入条件 =
  url shim 文档基址回退审计 + credentials/Authorization 流程审计两前置完成。
  试验已回退（零代码残留）。门禁沿用 M4-S26。见
  [evidence/2026-10-01-m4-s27-scheme-assessment.md](evidence/2026-10-01-m4-s27-scheme-assessment.md)。
- **M4-S26（2026-10-01）**：CORS-safelisted request-header 值面（**9290/11658 =
  79.7%**，fetch 1484→**1491 = 71.6%**；not-cors-safelisted **9 腿全绿** + 2 腿
  spec 必要回归记账）。shim：`_zwPreNamesOf` 重写为 CORS 变量头单一事实源
  （accept/CT 值条件入列——长 <128/无 forbidden 字节/accept 无 `"`/CT essence 三形；
  accept-language/content-language 2024 spec 移出安全名单恒入列；range 恒入列）+
  `_zwFetchNeedsPreflight` 收敛为 preNames 非空 + preflight 请求头最小集（不转发
  原自定义头——accept 值遮蔽 UA Accept 误拒根因；首版漏 referer 一轮补齐）。make
  test 全绿（19547P）+ clippy/fmt 干净。见
  [evidence/2026-10-01-m4-s26-safelist-values.md](evidence/2026-10-01-m4-s26-safelist-values.md)。
- **M4-S25（2026-10-01）**：cors 域 Referer 面（**9283/11658 = 79.6%**，fetch
  1468→**1484 = 71.3%**，+16 零回归；cors-preflight-referrer **12 腿全绿**）。shim：
  fetch 消费 referrerPolicy/referrer 计算 Referer（origin-only 走 scheme 宽化——
  HTTP_ORIGIN 期望面；注入先于 preflight 构建——首版位置错误致 x-preflight-referrer
  恒 ''）+ ACRH 空值头过滤 + 无非空自定义头时整体省略（「should be omitted」面——
  恒携带首版 7 腿回归即改）。runner：preflight.py control_request_headers 缺省
  b"" 恒发射。make test 全绿（19538P）+ clippy/fmt 干净。见
  [evidence/2026-10-01-m4-s25-referrer.md](evidence/2026-10-01-m4-s25-referrer.md)。
- **M4-S24（2026-10-01）**：重定向再 preflight + Origin opaque 修齐（**9267/11658 =
  79.5%**，fetch 1449→**1468 = 70.5%**，+19 零回归）。shim：hop 循环重定向后对
  新 URL 重跑 preflight（主 fetch 递归语义——ACAO 含 opaque `null` 形态/ACAM/ACAH
  覆盖 + cache + 失败拒绝；`_zwPreNamesOf` 抽取共享）+「当前跳相对文档跨源」改
  scheme 宽化比较（runner https 页锚 vs WPT http 宇宙——same-origin 跳误判跨源
  根因，cors-redirect same-origin→cors 5 腿）+ ACAH `*` 不覆盖 authorization
  （主/hop 检查 + cache 命中三处）。make test 全绿（19538P）+ clippy/fmt 干净。见
  [evidence/2026-10-01-m4-s24-redirect-repreflight.md](evidence/2026-10-01-m4-s24-redirect-repreflight.md)。
- **M4-S23（2026-10-01）**：fetch/api/cors 域导入 + preflight 语义修齐（**9248/11658
  = 79.3%**，分母 +320；域内 **+191 腿零回归**，fetch 1258→1449）。shim：preflight
  带 Accept: */* + ACAM/ACAH 覆盖判定 spec 化（缺 → 拒）+ Max-Age 0 不缓存（max_age=0
  用例组连锁误命中根因）+ 跨源请求重定向跨源 → Origin "null"（跳间替换）。
  runner：preflight.py/clean-stash.py（须置于泛化 OPTIONS 兜底门前——假阴性根因）+
  redirect.py 对齐上游（redirect_preflight 落穿 + Location 全量 query）+ 路径点段
  归一。**回归插曲**：_preNames 重构漏 CT 跳过致 PUT 族 preflight 误拒——preflight
  日志探针一轮定位修复。make test 全绿（19536P）+ clippy/fmt 干净。见
  [evidence/2026-10-01-m4-s23-cors-domain.md](evidence/2026-10-01-m4-s23-cors-domain.md)。
- **M4-S22（2026-10-01）**：preflight cache + 跨源重定向语义（**9059/11338 =
  79.9%**，xhr 545/583 → **557/583 = 95.5%**，Δ+12 全 corpus 零回归）。shim：
  CORS-preflight cache（(origin|credentials) 键 + ACAM/ACAH 覆盖判定（trim 修复——
  invalidation/cors-upload 双腿根因）+ Max-Age 过期）+ 重定向 credentials 升级
  （Location userinfo → include，ACAO `*` 失效面）。runner：.sub 脚本替换面收口
  （{{ports[http][0]}} 残留 → expose-headers 页 302 不跟随根因）+ fixture 九件
  （reset-token/preflight-cache ×3/preflight-request ×2/echo-content-cors/
  put-allow/common blank.html）+ WPT_TOKEN_STASH（Python falsy 语义）。make test
  全绿（19534P）+ clippy/fmt 干净。见
  [evidence/2026-10-01-m4-s22-preflight-cache.md](evidence/2026-10-01-m4-s22-preflight-cache.md)。
- **M4-S21（2026-10-01）**：XHR upload ProgressEvent 面 + send() invoked 状态机
  （**9047/11337 = 79.8%**，xhr 514/583 → **545/583 = 93.5%**，Δ+31 全 corpus 零
  回归；eventsource 双剥 BOM 误伤即修归位 35/35）。shim：send 重入 InvalidStateError
  + 11.6 return（loadstart handler abort/open 面——重入递归页 Timeout 根因）+
  upload listener flag 快照 + upload loadstart 同步/transmission 挂 fetch 成功路 +
  error 路径 upload.{event}+loadend（request error steps 步骤 6）+ error/loadend
  ProgressEvent (0,0,false) + fill 补 progress + load/loadend 按 Content-Length 取
  total + LOADING rs 双发 + GET/HEAD 体前置置 null + ArrayBuffer 体 byte-wire +
  responseType document 非 XML → null + TextDecoder UTF-8 decode 剥流首 BOM
  （EventSource `_process` 原地 replace 移除）。fixture 六件（echo-headers/xhr
  trickle/form/echo-content-type/access-control-origin-header/over-1-meg.txt）+
  探针残留三页清理。make test 全绿（19533P）+ clippy/fmt 干净。见
  [evidence/2026-10-01-m4-s21-xhr-upload.md](evidence/2026-10-01-m4-s21-xhr-upload.md)。
- **M4-S20（2026-09-30）**：timeout 族 + 404 语义 + ES object 面（**9042/11366 =
  79.6%**，xhr 494/586 → 514/583 = 88.2%，Δ+20 + fetch +22 零回归）。shim：timeout
  IDL（sync setter/open → InvalidStateError）+ 计时器（TimeoutError request error
  steps：DONE + reset + fire('timeout') + loadend）+ delay.py shim 侧延迟（host
  sleep 冻结 JS 根因绕开——setTimeout 延迟 host 发题）+ ES object（send 同步 String
  化上抛 + XHR/ReadableStream toStringTag 品牌化置 part06 尾段）+ FormData.forEach
  live 化 + File 转换规则 spec 化（File+filename 复制/File 无 filename 保留）。runner：
  文件缺失 → 404 响应（HTTP 语义——fetch 连带 +22）+ delay.py/image.gif fixture。
  存量 encoding.py.bak 断言随语义翻新。make test 全绿（19518P）+ clippy/fmt 干净。见
  [evidence/2026-09-30-m4-s20-timeout.md](evidence/2026-09-30-m4-s20-timeout.md)。
- **M4-S19（2026-09-30）**：FormData 面收口（**9000/11369 = 79.2%**，xhr 477/586 →
  494/586 = 84.3%，Δ+17 零回归）。shim：live cursor 迭代（WebIDL value pairs）+
  Blob/File → File 化副本（filename ?? name、lastModified 保真）+ (form, submitter)
  双参构造（TypeError/NotFoundError 校验、文档树序 + form owner 过滤枚举、`_elKey`
  稳定身份、image x/y、append filename 校验）+ FormDataEvent（**eval 序根因**：
  初置 part02 引用 part05 的 Event → ReferenceError 装载中止 → webdriver
  「document is not defined」五连；二分定位迁 part05 修复）+ 重入 InvalidStateError。
  fixture：upload.py multipart 解析回显（boundary off-by-one 探针定位）。make test
  全绿（19518P）+ clippy/fmt 干净。见
  [evidence/2026-09-30-m4-s19-formdata.md](evidence/2026-09-30-m4-s19-formdata.md)。
- **M4-S18（2026-09-30）**：CORS-preflight + 暴露头过滤（**8983/11369 = 79.0%**，
  xhr 471/586 → 477/586 = 81.4%，Δ+6 + fetch +1 零回归）。shim：preflight（非
  safelisted cors 请求 → OPTIONS ACRM/ACAH + Origin，2xx + ACAO/ACAM/ACAH 覆盖
  判定，异步 + 同步双路）+ cors-filtered-response 暴露头过滤（safelisted + ACXH
  白名单、Set-Cookie 恒排除、`*` 通配 include 仅字面）+ 文档 origin 门控
  （`_zwHasRealPageOrigin`——裸 sandbox 无 CORS 执行语境，存量管路桥测零翻新回绿）+
  redirect 链补强（跨源跳丢 Authorization / 同步逐跳门控 + 入口 Origin）。runner：
  OPTIONS handler（非 GET 拒绝后移 + fixture 自派派 + 200 兜底）+ redirect-cors/
  fetch redirect.py OPTIONS 分支 + top.txt。make test 全绿（19518P）+ clippy/fmt
  干净。见
  [evidence/2026-09-30-m4-s18-preflight.md](evidence/2026-09-30-m4-s18-preflight.md)。
- **M4-S17（2026-09-30）**：redirect 跟随 + credentials-aware CORS（**8976/11369 =
  78.9%**，xhr 459/587 → 471/586 = 80.4%，Δ+12 + fetch +3 零回归）。shim：redirect
  循环（301/302 POST→GET、303 GET、307/308 保持；20 跳；逐跳 content-length 重算；
  **同步返回契约就地结算**——headless `__zw_fetch` 直返 wire，挂等 resolver 为根因，
  intercept 探针定位；主流程 sync-consume arity 同修）+ 逐跳 CORS 门控 + 跨源跳
  Origin 注入 + credentials-aware cors check（include 拒 `*`、须 ACAC true）+ 同步
  XHR 循环 + XHR credentials 接线（读侧闭包变量修正）。fixture 六件（access-control
  族 / redirect-cors / fetch 域 redirect.py + dump-authorization-header）。存量桥测
  test_fetch_passes_request_credentials_to_host 随语义翻新（改相对 URL 保管路断言）。
  make test 全绿（19518P）+ clippy/fmt 干净。见
  [evidence/2026-09-30-m4-s17-redirect.md](evidence/2026-09-30-m4-s17-redirect.md)。
- **M4-S16（2026-09-30）**：XHR 状态与事件（**8961/11370 = 78.8%**，xhr 433/586 →
  459/587 = 78.2%，Δ+26 零回归）。abort() spec 化（request error steps：DONE +
  readystatechange 先于响应重置 → abort + loadend → 静默 UNSENT；DONE → UNSENT +
  reset 无事件；重置含 responseURL/响应头表）+ 请求代际守卫（`_zwReqGen`——abort/
  复用后晚到响应隔离）+ `xhr.upload`（XMLHttpRequestUpload + send 有体时同步
  loadstart/progress，total = 体字节长）+ withCredentials IDL setter（send() flag
  已置 → InvalidStateError）+ ProgressEvent 专设定义（length 1 / prototype 描述符 /
  getter-only + brand check / global non-enumerable）。五页 27 腿全绿；fixture：
  corsenabled.py / redirect.py / well-formed.xml / pass.txt。make test 全绿（19508P）
  + clippy/fmt 干净。见
  [evidence/2026-09-30-m4-s16-xhr-state.md](evidence/2026-09-30-m4-s16-xhr-state.md)。
- **M4-S15（2026-09-30）**：XHR send 簇 + content.py/.asis fixture 通道（**8935/11369
  = 78.6%**，xhr 267/451 → 433/586 = 73.9%（分母含 send-usp 128 子案细粒度化），
  Δ+166 + fetch +6 附带解锁，零回归）。fixture：`content.py` 回显端点（POST 面，
  置于 handler 非 GET 拒绝前）+ `.asis` 原始 HTTP 通道（6 文件嵌入，280/444 非常规
  状态 + 空值/重复头逐字）。shim：USP 序列化 spec 化（encodeURIComponent 漏 !'()~）；
  type-less Blob 无 CT 派生（fetch spec）；UA Content-Length 补齐；XHR send GET/HEAD
  体丢弃 + USP charset 替换（xhr.spec）；同步体类型分发（Blob/FormData byte-wire，
  原 String(Blob) 垃圾）。getresponseheader/send-usp/invalid-unicode/no-mime 四页
  全绿。make test 全绿（19508P）+ clippy/fmt 干净。见
  [evidence/2026-09-30-m4-s15-xhr-send.md](evidence/2026-09-30-m4-s15-xhr-send.md)。
- **M4-S14（2026-09-30）**：blob: Range 切片 + 同步 XHR scheme 分派（**8763/11234
  = 78.0%**，xhr 240/451 → 267/451 = 59.2%，Δ+27 零回归；blob-range 页 0/27 →
  **27/27**）。fetch spec blob-url-scheme Range：提取 failure（malformed/多区间/
  起点>终点）或起点 ≥ size → network error（同步 send 抛 NetworkError / 异步
  onerror——现 spec 无 200 回落）；提取成功 → 206 切片（Content-Range + 钳制 +
  OWS 容差四形态）；同步 XHR scheme 分派先行（host 同步契约无 blob store 视角）；
  getAllResponseHeaders 实现（xhr.spec §4.6.5，原空串桩）。make test 全绿（19506P）
  + clippy/fmt 干净 + 五 corpus 逐字节同值。见
  [evidence/2026-09-30-m4-s14-blob-range.md](evidence/2026-09-30-m4-s14-blob-range.md)。
- **M4-S13（2026-09-30）**：eventsource 跨源半 + cors.py fixture（**8736/11234 =
  77.7%**，eventsource 33/34 → **35/35 = 100%**——六 corpus 首个收口，Δ+2 零回归）。
  runner 内置 `eventsource/resources/cors.py`（ACAO 回显 Origin + ACAC + `run=
  cache-control` 模板体——上游逐字等价）；跨域链路（runner path 匹配 origin 无关 +
  shim fetch Origin 注入/ACAO 过滤）零改动即通——request-cache-control 页 4/4。
  fetch/api/cors 域评估定谳：独立切片（多模式 cors.py + OPTIONS preflight 双前置）。
  make test 全绿（19506P）+ clippy/fmt 干净 + 五 corpus 逐字节同值。见
  [evidence/2026-09-30-m4-s13-eventsource.md](evidence/2026-09-30-m4-s13-eventsource.md)。
- **M4-S12（2026-09-30）**：EventSource spec 化重做 + P7 fixture 续件（**8734/11233 =
  77.8%**，eventsource 2/32 → 33/34 = 97.1%，Δ+31 零回归；32 页 31 页全绿）。
  runner 内置 fixture：message.py（mime/message/newline/sleep）/ message2.py（一轮
  循环体）/ last-event-id.py（重连面）/ *.event_stream 模板回显。shim：构造 URL 解析 +
  SyntaxError（globalThis.DOMException 防 wrong global）；status/MIME essence 门 →
  fail the connection；网络错误/body 结束 → reestablish（CONNECTING + error + retry
  时序重连）；Last-Event-ID 经 `__zwInternalHeadersWire` 内部直设（公共 ByteString
  校验对 >U+00FF 拒绝 → fetch 拒绝 → 重连死循环根因；spec 于 header list 直设不经
  JS 校验）；解析派发 spec 化（仅 data 缓冲非空派发 / retry 全数字 / id 缓冲以源上
  string 播种逐流重置 / 末尾终止符弹栈）；WebIDL 常量挂 prototype + isTrusted。
  make test 全绿（19506P——桥测随 spec 翻新 + 重连 wire 断言）+ clippy/fmt 干净 +
  五 corpus 逐字节同值。见
  [evidence/2026-09-30-m4-s12-eventsource.md](evidence/2026-09-30-m4-s12-eventsource.md)。
- **M4-S11（2026-09-30）**：pipeTo/pipeThrough spec 化重做（**8703/11231 = 77.5%**，
  streams 862/1020 → 903/1019 = 88.6%，Δ+41 零回归）。shutdown/shutdown-with-action/
  finalize 三段收尾机（动作执行时序按 dest 态分支——writable 排在飞写后微任务跳、
  erroring 同步执行；动作拒绝优先于原错误；finalize 显式 errGiven 区分 undefined 拒绝）；
  read-ahead 泵（desiredSize 背压门控 + writer.ready 级联续泵）+ 源状态监视
  （reader.closed 兑现/拒绝 → 条件 3/1，pendingRead/activeOps 在飞门控防已读 chunk 丢弃）；
  pipeTo 同步置 disturbed（spec 步骤 11）；abortAction 覆盖 dest erroring +
  signal 路径 writable-only 分离；pipeThrough 成员读取序 = WebIDL 定义序（readable 先）
  + 品牌校验 + options 后复核锁定；原型委托防自递归（别名赋值移至方法定义后——
  'Maximum call stack size exceeded' 根因）。piping 簇 186→227：pipe-through 43/43、
  error-propagation-forward 32/32、general 14/14、error-propagation-backward 35/35、
  multiple-propagation 9/9 全绿。删除前轮未提交探针残留页 zz-abort-probe.any.js。
  make test 全绿（1 处在册 stale_etag 争用窗 flaky 隔离复跑绿 + 全量复跑收口）+
  clippy/fmt 干净。见 [evidence/2026-09-30-m4-s11-pipe-to.md](evidence/2026-09-30-m4-s11-pipe-to.md)。
- **M4-S10（2026-09-30）**：Byte tee 源侧 BYOB pull + tee() spec 结构重做（**8662/11232 =
  77.1%**，streams 826/993 → 862/1020 = 84.5%，Δ+36 零回归；byte tee 页 10/40+页级
  Timeout → **40/40 零超时**）。tee() 重做 spec ReadableByteStreamTee（reading/readAgain
  旗标 + pullWithDefaultReader/pullWithBYOBReader + steps 形源读 API `_zwReadRawSteps`/
  byobReadInto request 形——投递微任务在 dequeue 同步步内排队，先于 pull throw 的错误前向，
  'errors in the source' 双块队列面根因）；分支 cancel 单路化（desc.cancel = composite，
  reader.cancel 同语义）；_doCancel spec 修齐（close given undefined + errored reject）；
  respondWithNewView 校验升 spec（偏移/缓冲长/容量）；errorStream 标记 closedP handled
  （spec ReadableStreamError 步骤 7，双页 Unhandled rejection 根因）。make test 全绿
  （1 处在册 skip_waiting 争用窗 flaky 隔离复跑绿 + 全量复跑收口）+ clippy/fmt 干净。
  见 [evidence/2026-09-30-m4-s10-byte-tee-byob.md](evidence/2026-09-30-m4-s10-byte-tee-byob.md)。
- **M4-S9（2026-09-30）**：Byte tee 分支字节流身份 + 双分支 chunk 克隆（**8626/11205 =
  77.0%**，streams 825/1020 → 826/993 = 83.2%；byte tee 页 9/40→10/13 记录窗收敛）。
  分支 `type: 'bytes'`（byob reader/pull-into 在分支可用——byte tee 31 腿公共前置）+
  spec ReadableByteStreamTee 双分支克隆（原 buffer 属源）。make test 全绿 + clippy/fmt
  干净。见 [evidence/2026-09-30-m4-s9-byte-tee.md](evidence/2026-09-30-m4-s9-byte-tee.md)。
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

目标已收口（Completed，2026-10-01）。重入条件在案：
1. **scheme 对齐两前置审计**（+29 连带入袋，见 M4-S27 evidence）：url shim 文档
   基址回退审计（a-element about:blank 面 ×20）+ credentials/Authorization 流程
   审计（×21）→ 完成后翻页面锚 `https://wpt.test` → `http://wpt.test`
   （testharness.rs 34 处 + shim iframe 解析 1 处统一 http——跨切面，先评估回归
   面再动；解同源判定/Origin 回显跨 scheme 失配 2 腿）
2. **结构性残余跨域认领**：streams transfer/detach 14 腿（spec
   TransferArrayBuffer——需宿主 V8 detach 能力，JS 层不可表达——bad-buffers 10
   + non-transferable 4；engine 宿主能力面）；cors credentials/cookie jar +
   Authorization 重定向流 ~13 腿（storage goal / security-hardening 邻域）；
   WebSocket 二期 P6（宿主 socket 面 + 升级握手/帧协议，用户点名重入）
3. **goal 内结构性挂账**：sync XHR 窗口 onload 阻塞语义 4 腿；bad-chunk 注入
   2 腿（需 .asis + chunked 解析面）；formdata FormDataEvent.entries live 面 +
   submitter 坐标族 5 腿；XML 文档解析（engine XML DOM——template-element 族）

**待用户决策清单**：（空）
