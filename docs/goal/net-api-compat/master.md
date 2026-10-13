# 网络 API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../net-api-compat.md](../net-api-compat.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-10-14（**M5-S10 执行**——XHR timeout 族 3 腿销账 +
setter 重排附带 2 腿：`_zwFetchSimDelay` 共享延迟模型 + timeout setter 重排 +
finite 模型静态判定 + on* handler 抛上报 + runner preflight 应答/到期序派发；
六 corpus 受控 A/B **xhr +6P/−4F/−2T，其余逐字节持平**）。
此前：同日 M5-S9（bad-chunk 注入 2 腿销账）；M5-S8（formdata 簇 +14 / sync XHR
销账）；M5-S7（URL 反射基址接线）；2026-10-13 M5-S6（url 基址链 +215）；
M5-S5（scheme 翻转 +179）；M5-S4（P6 二期收口）；M5-S3（eventsource + WS
cookie jar）；2026-10-13（M5-S2/S1）；2026-10-01（M4-S29 收口判定）

---

## 当前状态

**专项定位**：网络 API（fetch/XHR/URL/mimesniff/streams/EventSource）一致性收敛，
WPT 六 corpus 为验收标尺。WebSocket 二期挂账（需宿主 socket 面 + 帧协议）。

**P6 重入核记（2026-10-13）**：Mission 约束 2 重入条件「zero-net WS 底座就位」已兑现——
zero-web 流 2026-10-08~09 落地 WebSocket 页面 API + 底座（tungstenite 实现
`crates/net/src/websocket.rs` + `engine/ws_bridge.rs` + shim part06.js，commit
b2a0e502c / 2b95db8ba / 395a2ea00 / 95072f30b / 9a1f1c1e1 / 030bcebd7，
ZRG-2026-10-09-01 巡检在案）。可执行前置：runner fixture 通道为请求/响应拦截合成、
无 WS 持久端点——**M5-S1 = runner 内嵌 tungstenite fixture 端点 + `websockets/`
corpus 拉取甄别 + 基线**；WS 语义修齐若触 shim part06.js（engine 共享面），
与 zero-web 流按 run-rules §9 碰头核对。
**M5-S1 执行（同日）**：通道落地——手写 RFC6455 fixture 端点（21 端点，语义基准 =
上游 `websockets/handlers/*.py` @ WPT_REV 31597693 逐文件核读）+ FixtureWsHost
（`*.wpt.test` → 127.0.0.1 改写委派 `default_net_ws_host` 真栈）+ `.sub.js` 替换
三通道补全 + websockets 案 http 页面（上游 ?default 变体语义）。基线**子测试
334/375 = 89.1% / case 78/106 = 73.6%**（自零起步）；失败聚类 → close()/构造器
IDL 校验 19（shim）、`ws.protocol` 属性面 4 案（shim+bridge）、WS cookie jar 4 案
（storage 域挂账族）、basic-auth/ws-301 2 案（zero-net）、状态机 3 案、runner 工件
1（017 执行序）。证据
[evidence/2026-10-13-m5-s1-ws-fixture-baseline.md](evidence/2026-10-13-m5-s1-ws-fixture-baseline.md)。
不测面：wss/h2 变体、`.https.*` 5 案（TLS）、multi-globals/unload-a-document（结构面）。

**M5-S10 执行（同日）**：XHR timeout 族 3 腿销账（timeout-multiple-fetches /
thrown-error-in-events / xhr-timeout-longtask——M4-S21 记账）+ setter 重排附带
2 腿（overrides/overridesexpires）。五处修齐：① `_zwFetchSimDelay` 共享延迟模型
（fetch wrapper 与 XHR send 判定共用；bare delay.py 缺省 1000ms——subtest 2 期望
面钉死；redirect.py 链折叠一次延迟）；② timeout setter send 中改值重排（spec：
deadline = send 起点 + 新值，`_zwArmTimeout` token 失效旧计时器）；③ finite 模型
静态判定（D < T 不挂计时器——longtask busy loop 伪 ontimeout 根因）；④ part05
R114 on* handler 抛按 spec report（`_zwReportListenerError`——原吞掉致 promise 永不
结算页级 Timeout）；⑤ runner delay.py OPTIONS preflight 应答 + 泛化兜底 ACAM/ACAH
回显（探针三级定位：delay.py 分支先于兜底返回裸 200 → preflight 拒 → onerror）+
timer 到期序派发。六 corpus 受控 A/B **xhr 565P/26F/3T → 571P/22F/1T（+6P/−4F/
−2T），其余五 corpus + websockets 逐字节持平**。探针页已清理。证据
[evidence/2026-10-14-m5-s10-timeout-family.md](evidence/2026-10-14-m5-s10-timeout-family.md)。

**M5-S9 执行（同日）**：bad-chunk 注入 2 腿销账（error-after-response
0P/2F → 2P/0F）——`bad-chunk-encoding.py` 内置等价 fixture（TEST_CHUNK×count +
garbage，trickle.py 同型生成器）+ 内部标记头 `x-zero-body-error-after` →
`_bodyErrorAfter` 字段 → 流中段网络错误面（前 N 字节单 chunk enqueue；错误在
**首个 read 兑现后**经 getReader 包装触发——start 内同步 error 清空队列令首
read 拒，首版即此翻车；consume 族 text()/arrayBuffer() 同拒 TypeError，spec
一致）。六 corpus 受控 A/B **+2P/−2F 零回归**；make test 全绿（**20661P/0F**）
+ clippy + fmt 干净；bench-gate 同族噪声延续（paint_simple 128µs↔381µs 跨轮，
并行流域负载）。证据
[evidence/2026-10-14-m5-s9-bad-chunk.md](evidence/2026-10-14-m5-s9-bad-chunk.md)。

**M5-S8 执行（同日）**：formdata 簇修齐 + sync XHR 账面修正。修齐七处：ctor
WebIDL TypeError（null/'string'）；ctor nameless image submitter bare x/y +
selected coordinates 消费；`_zwImgRecordClickCoords`/`_zwSubmitBtnDefaultAction`
共享 helper（.click() 门内联 walk 抽出）；dispatchEvent 派发 click 默认动作扩面
（submit 按钮——R106 锚面同语义）；`_zwFormSubmitNavigate` 坐标 entry；Request
FormData/Blob body 共享序列化（CT 派生）；multipart parser part 头大小写不敏感 +
filename → File。fixture 补上游 200×200 svg + 1x1 png（期望值 23/46 几何标定源，
`round(clientX−left)` 推导实证）。六 corpus 受控 A/B **+14P/−14F 零回归**
（formdata 4 腿 + Request body 路径附带解锁 fetch/api 10 腿）；make test 全绿
（20657P/0F）+ clippy + fmt 干净；bench-gate 同代码跨轮 FAIL 集 0/5/10 摆动
（并行流域负载噪声实证，不放宽）。**「sync XHR 结构面 4」实测全绿销账**
（FILTER=sync 全 PASS；残 2 案 = permissions-policy API 面 tentative，security-
hardening 邻域跨域记账）。残余 5F 精确记账：xhr/formdata.html 2（populateForm
iframe 编排 structural——iframe 字面量在外部资源内绕过通道筛减，观察项）、
constructor-formelement 1（unpaired surrogate wire 层 lossy，R121 同型）、坐标
2 腿（input[type=image] 图片加载 + **layout-engine replaced sizing（渲染流域
§9 碰头记账）** + SVG intrinsic sizing——激活/坐标/消费接线已就位，几何对齐
即通）。证据
[evidence/2026-10-14-m5-s8-formdata-cluster.md](evidence/2026-10-14-m5-s8-formdata-cluster.md)。

**M5-S7 执行（2026-10-14）**：URL 反射基址接线（M5-S6 残余记账清偿）——五处
逐点接 `_zwDocBaseURL`：① `_zwResolveFetchUrl` 共享解析器（一次覆盖 fetch 请求
派发 / XHR 相对 URL / track·SOURCE·media·iframe src 加载链 / img CSP·media 钩子
路径）；② form.action getter；③ `_zwFormSubmitNavigate` 提交导航基；④
`_REFLECTED_URL_TAGS` 通用 URL 反射 getter（img·iframe·script·frame·input·track
src / cite / poster / object.data / lowsrc / longDesc / codeBase）；⑤ formAction +
Attr.baseURI。探针 6/6（img.src 首版期望写错非缺陷，双形态基址复跑全绿；探针
页含 `createElement("iframe")` 字面量会被通道 iframe 筛减静默跳过——须避开）。
六 corpus 受控 A/B 对 M5-S6 树**逐页字节级持平**（无 base 元素时全部接线点纯
回落，预期零位移成立）；make test 全绿（**20657P/0F**）+ clippy + fmt 干净；
bench-gate FAIL 五项全为纯 Rust 微基准（M5-S6 轮已 stash 对照实证机器态漂移，
本轮 FAIL 集扩大属跨轮噪声）。BASE/LINK href getter 维持 fallback 解析（spec 防
自引用）；`_zwCurrentHref`（Origin/Referer）不改。残余：fixture 通道 URL 改写
第三形态基址域（语料无此形态，观察项）。证据
[evidence/2026-10-14-m5-s7-url-reflection-base.md](evidence/2026-10-14-m5-s7-url-reflection-base.md)。

**M5-S6 执行（同日）**：url 基址链清偿（M5-S5 残差记账落账）——探针实证四处
缺口（`a.href` 无视 `<base>` / `document.baseURI` undefined / about:blank 基解析
失败面）后接线：`globalThis._zwDocBaseURL` helper（首个带 href 的 `<base>` 元素
frozen base URL，树序 + 解析失败回落文档地址）+ A/AREA accessor 组件族 + element
baseURI（part04 R130）+ document.baseURI getter（part06）。受控 A/B（同树 stash
对照六 corpus 全通道）：**8826P/3239F/10T → 9041P/3024F/10T = +215 子测试零回归**
（a-element 412→544P、a-element-origin 316→399P，origin 簇 96F→13F 随基址对齐；
其余五 corpus 逐页字节级持平；a-element 两页分母不变）。语料漂移注记：wpt-data
未跟踪随 fetch 刷新（IdnaTestV2.json 2671 条吻合本树分母），跨轮次对比以同树
A/B 为准。门禁：make test 全绿（**20649P/0F**）+ clippy + fmt 本流 crate 干净；
bench-gate（zero-engine 定向）FAIL 两 paint 指标经 stash 对照实证为机器态漂移
既有（无修改对照同 FAIL，基线 2026-09-15），不放宽。残余：其余 URL 反射
（form.action/img.src/script.src 等）仍以页面 location 为基（helper 已就位，
逐点接线独立切片）。证据
[evidence/2026-10-13-m5-s6-base-url.md](evidence/2026-10-13-m5-s6-base-url.md)。

**M5-S5 执行（同日）**：M4-S27 两前置审计实证清偿，scheme 翻转落定。① url 基址
回退「×20 回归」未复现（M4-S26 后 shim 演进消化）——url **+186P/+10F（净 +176）**；
② fetch −9P 集中于 Authorization 重定向保留流 21 case（cors-redirect-credentials
15 + cors-redirect 5 + authentication-redirection 1）= M4-S27 评估一已回流
security-hardening 邻域的既有记账簇（非新增缺陷）；解锁 cors-origin +8 /
cors-basic +2 / integrity +1 / response-url +1。实施 = 页锚 scheme 域内双态
（net-api corpus http / 其余通道 https）+ fetcher strip 双 scheme + 响应类型
同源判定双 scheme。六 corpus 总账 **+179**。url 基址链残差（锚点 accessor
`<base>`/document.baseURI 未实现，+5 子测试）记账独立切片。证据
[evidence/2026-10-14-m5-s5-scheme-flip.md](evidence/2026-10-14-m5-s5-scheme-flip.md)。

**M5-S4 执行（同日）**：① constructor/017 归因翻案——M5-S1 所记「runner 预处理
工件」实为**产品缺口**：classic 页面脚本经间接 eval 的顶层声明导出扫描
（scan_top_level_decl_exports）只认 `let x =` 初始化形态，`let x;`（无初始化器
后续赋值——constants.sub.js 即此形态）不导出 → 跨脚本蒸发。声明符接受集
`= | ; | ,`（accessor 导出，getter 闭包持活绑定）；② basic-auth：net 预置
`Authorization: Basic`（URL userinfo 手工解析 + 零依赖 base64）+ fixture
/basic_auth（401 挑战/凭据 101）+ 宿主改写 userinfo 感知（旧 split('@').next()
把 userinfo 当 host）。**websockets 376/376 = 100% / case 106/106 = 100%——P6
二期收口**。make test 全绿（**20649P/0F**）。证据
[evidence/2026-10-14-m5-s4-ws-complete.md](evidence/2026-10-14-m5-s4-ws-complete.md)。

**M5-S3 执行（2026-10-14）**：① eventsource request-cache-control Timeout 归因——
探针页三级定位（跨源 fetch 同 URL 正常 → ES onerror → `Cache-Control: no-cache`
（非 safelisted）触发 preflight、runner cors.py fixture 无 ACAM/ACAH 应答被拒）→
fixture 增 OPTIONS 应答（ACAM/ACAH 回显——「支持 ES 跨源的合规 CORS 服务端」语义），
eventsource 回归 35/35 = 100%；② WS cookie jar 集成——双 jar 断缝（document.cookie
走 shim JS jar、WS 握手读 zero-net Rust jar）三面修齐：net 101 响应 Set-Cookie
透出 + bridge `WsEvent::SetCookie`（Open 前派发，007 执行序面）+ 握手 Cookie 双源
合并 + shim `_zwCookieStoreSet` scope 化（WS 面 HttpOnly 按 bis §5.3 置位）+
`_zwCookieHeaderForUrl` 出站检索（含 HttpOnly）+ globalThis 跨闭包桥（part06 双
IIFE 结构）。websockets **373/375 = 99.5% / case 104/106 = 98.1%**。证据
[evidence/2026-10-14-m5-s3-es-cookie.md](evidence/2026-10-14-m5-s3-es-cookie.md)。

**M5-S2 执行（同日）**：shim/bridge 归属簇修齐——close 码域 InvalidAccessError
（spec step 1 现行语义，9 子测试）、构造器 fragment 门（spec step 4，3 子测试）、
协议 RFC 7230 token 文法（Create-asciiSep）、A/AREA stringifier→href 三层落点
（trap @@toPrimitive + trap toString + 接口原型直装——前轮死代码定位经探针页实证）、
`ws.protocol` 透出（net 层 selected_protocol）、per-host 连接串行 + close 握手等待段
（serialize-establish / Close-delayed）。**369/375 = 98.4% / case 100/106 =
94.3% / Timeout 0**；六 corpus 零回归逐域实证（fetch 1499P / xhr 562P / url 4429P /
mimesniff 1033P / streams 905P——与收口分母漂移内持平；eventsource 1 Timeout 经
stash 对照实证为先在漂移）。残余 6 案全为跨域记账（cookies ×4 storage、basic-auth
zero-net、017 runner 工件）。碰头核对：part06 WS 段最后触碰 = zero-web 流 caf3b3373，
无并行编辑。证据
[evidence/2026-10-13-m5-s2-ws-semantic-fixes.md](evidence/2026-10-13-m5-s2-ws-semantic-fixes.md)。

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

**收口后漂移（2026-10-11 记档，2026-10-13 补录）**：fileapi-compat M2-P1 经 chromium
实测仲裁改 Blob.type 判据（范围判据 + ASCII 小写化，取代本 goal M2-S1 落的 MIME
parse-serialize——上游 mimesniff parsing.any.js 与 FileAPI 语料对 Blob.type 断言互斥，
chromium 站 FileAPI 侧），net-api 通道 mimesniff (Blob/File) **865 腿随之转为失败**
——上游语料间冲突，非回归；复跑 fetch+xhr 2060 ≈ 收口 2046（corpus +16 漂移内持平），
streams/url/eventsource/xhr 与收口一致。逐字节仲裁证据见
[fileapi-compat evidence/2026-10-11-m2-p1-constructor-webidl.md](../fileapi-compat/evidence/2026-10-11-m2-p1-constructor-webidl.md)。
本 goal 终态数字仍指 2026-10-01 收口快照；重读通道现状时以本注记为准。

**与兄弟 goal 的边界**：
- security-hardening — CSP 对 fetch 的策略执行归其；本 goal 提供语义钩子位
- service-workers（已归档）— 其 fetch 通道已收口；拦截扩展另行记账
- zero-web P1a — URL/URLSearchParams 既有实现为本 goal 修齐对象（改动走本 goal 账本）
- rendering-compat 及渲染流 — 无共享 crate 面

**碰头观察（2026-10-13，渲染流域，重开）**：渲染流 PR #154（9223cb5e4，t3
-navigate-effect）在 7f5a46587 自修**之后**重新把 `/tmp/kbtrace.log` 调试残留
带回 `apps/renderer/src/runtime.rs`（6 处，rustfmt 不洁）——本流 `cargo fmt
--all -- --check` 门禁再次被域外文件挡住；按 §9 留域内处置不硬解（本流相关
crate 的 fmt 已单测干净）。渲染流需再次自清。

**碰头观察（2026-10-14，渲染流域，第三轮）**：T9 merge（fb3873802）带入
`apps/renderer/src/runtime.rs` fmt 不洁 2 处 + clippy dead-code 1（
`build_implicit_submit_url`/`submit_form_on_enter_at` never used）；R5108（
4fc38b7d2）带入 `crates/layout-engine/src/engine/postprocess.rs` fmt 不洁 1 处
——workspace fmt/clippy 门禁再次被域外文件挡住；按 §9 留域内处置不硬解
（本流相关 crate fmt/clippy 已单测干净）。渲染流需自清。

**碰头观察（2026-10-14，渲染流域，已闭环）**：渲染流 156219d2d 落地的
`apps/renderer/src/runtime.rs` 曾含 `/tmp/kbtrace.log` 调试残留且未过 rustfmt——
本流 `cargo fmt --all` 触及该文件（未提交，按 §9 留域内处置）；渲染流已自修
（commit 7f5a46587——移除调试行 + 恢复 fmt/clippy 门禁），本流无残留。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 六 corpus fetch 脚本 + 导入 + 基线 | ✅ M1（2026-09-28，基线 1297/5019 = 25.8%） |
| P2 | fetch/Request/Response/Headers/Body 语义收敛 | ✅ M2 主体收口 + M4-S23~S26 cors 域收口（fetch **1491/2081 = 71.6%** / mimesniff 100%）；残余：credentials/cookie 管道腿 ~13（跨域挂账）、response-error 族（10） |
| P3 | XHR 状态机 + EventSource 解析/重连 | ✅ EventSource 100% 收口（M4-S13）；XHR 主体收口；page-scheme 失配已解（M5-S5 翻转）；bad-chunk 注入 2 已销账（M5-S9）；**timeout 族 3 已销账（M5-S10，setter 重排附带 overrides 族 2 腿）**——**残余：XML 文档面（engine XML DOM 深度——template-element 族）、formdata 残余 5F（iframe 编排 structural 2 + surrogate wire 1 + input-image 几何跨域 2——M5-S8 精确记账）**；sync XHR 结构面 4 已实测全绿销账（M5-S8——残 2 案为 permissions-policy API 面跨域记账） |
| P4 | URL 边缘语义 + mimesniff 对齐 | ✅ mimesniff 100%；url 基址链已清偿（M5-S6 +215 零回归）+ URL 反射基址接线已清偿（M5-S7——反射/请求/导航五面接 `_zwDocBaseURL`，无 base 元素行为不变）——残余：urltestdata 全量解析深水（IDNA/toASCII/file: 特例/host 校验）+ setters-stripping host c0 保留面（Chromium bug-compat，url crate 不可表达）挂账 |
| P5 | streams 底座一致性（fetch body 依赖） | ✅ M4-S11 收口（88.6%）+ M4-S28 flow-control 清偿（**88.7%**，piping 全绿含 flow-control 5/5）；**残余：transfer/detach 族结构性挂账（需宿主 V8 detach——14 腿，JS 层不可表达）** |
| P6 | WebSocket 二期切片（runner WS fixture 端点 + websockets corpus 基线 + 语义修齐） | ✅ **收口（2026-10-14，M5-S1→S4 四切片）**：**376/376 = 100% / case 106/106 = 100%**（基线 334/375 = 89.1%）；不测面维持记账（wss/h2 变体、.https.* TLS、multi-globals/unload-a-document/bfcache 结构面） |
| P7 | runner fixture 通道（.py 端点最小 fixture 集） | ✅ 累计 24+ 端点（M4-S21 增 echo-headers/xhr trickle/form/echo-content-type/access-control-origin-header/over-1-meg.txt；M4-S22 增 preflight cache 族 ×3 等九件；M4-S23 增 preflight.py/clean-stash.py——fetch/api/cors 域页拉取已随域导入收口） |

## 已完成切片

- **M5-S10（2026-10-14）**：XHR timeout 族 3 腿销账 + setter 重排附带 2 腿（xhr
  565P/26F/3T → **571P/22F/1T = +6P/−4F/−2T**，其余 corpus 逐字节持平）：共享延迟
  模型 `_zwFetchSimDelay`（bare delay.py 缺省 1000ms + redirect 链折叠）+ timeout
  setter send 中改值重排（token 失效旧计时器）+ finite 模型静态判定（D < T 不挂
  计时器）+ on* handler 抛按 spec report + runner preflight 应答（delay.py OPTIONS
  分支 + 泛化兜底 ACAM/ACAH 回显）+ timer 到期序派发。make test 全绿（20664P/0F，
  +3 = 随 pull 并入的渲染流 R5108 测试）。见
  [evidence/2026-10-14-m5-s10-timeout-family.md](evidence/2026-10-14-m5-s10-timeout-family.md)。
- **M5-S9（2026-10-14）**：bad-chunk 注入 2 腿销账（error-after-response 2/2）：
  内置 bad-chunk-encoding fixture + `x-zero-body-error-after` 流中段网络错误面
  （首 read 兑现后 error + consume 族同拒）。六 corpus 受控 A/B **+2P/−2F 零
  回归**；make test 全绿（20661P/0F）+ clippy + fmt 干净。见
  [evidence/2026-10-14-m5-s9-bad-chunk.md](evidence/2026-10-14-m5-s9-bad-chunk.md)。
- **M5-S8（2026-10-14）**：formdata/FormDataEvent/submitter 簇修齐（ctor WebIDL
  TypeError / nameless image submitter + selected coordinates / dispatchEvent
  提交扩面 / Request FormData·Blob body 序列化 / multipart part 头解析）+ sync
  XHR 账面销账。六 corpus 受控 A/B **+14P/−14F 零回归**；make test 全绿
  （20657P/0F）+ clippy + fmt 干净。见
  [evidence/2026-10-14-m5-s8-formdata-cluster.md](evidence/2026-10-14-m5-s8-formdata-cluster.md)。
- **M5-S7（2026-10-14）**：URL 反射基址接线（form.action / `_REFLECTED_URL_TAGS`
  全族 src·cite·poster·data / formAction / Attr.baseURI / `_zwFormSubmitNavigate`
  / `_zwResolveFetchUrl` 共享解析器逐点接 `_zwDocBaseURL`）。六 corpus 受控 A/B
  逐页字节级持平 + 探针 6/6；make test 全绿（20657P/0F）+ clippy + fmt 干净。
  见 [evidence/2026-10-14-m5-s7-url-reflection-base.md](evidence/2026-10-14-m5-s7-url-reflection-base.md)。
- **M5-S6（2026-10-13）**：url 基址链（`<base>` 元素 + document.baseURI）：shim
  `_zwDocBaseURL` helper（首个带 href 的 base 元素 frozen base URL，树序 +
  失败回落）+ A/AREA accessor 组件族 + element/document baseURI 接线。受控 A/B
  六 corpus **+215 子测试零回归**（a-element +132P / a-element-origin +83P，
  origin 簇 96F→13F；其余逐页持平）。make test 全绿（20649P/0F）+ clippy + fmt
  干净。残余记账：其余 URL 反射（form.action/img.src 等）独立切片。见
  [evidence/2026-10-13-m5-s6-base-url.md](evidence/2026-10-13-m5-s6-base-url.md)。
- **M5-S5（2026-10-14）**：页面 scheme 对齐翻转（M4-S27 两前置审计实证清偿——
  重入条件闭合）：net-api 六 corpus 页锚 http（NET_API_CORPUS_SUBDIRS 前缀判定，
  其余通道 secure-context https 不动）+ fetcher strip 双 scheme + 响应类型同源
  双 scheme。六 corpus 总账 **+179 子测试**（url +186P/+10F、fetch −9P、xhr +2P、
  mimesniff/streams/eventsource 持平）。①「×20 基址回退回归」未复现（残差 +5
  子测试 = url 基址链记账：锚点 accessor 基址 = 页面 location、`<base>` 元素/
  document.baseURI 未实现——探针实证，独立切片）；② fetch −9P = Authorization
  重定向保留流 21 case 转入既有 security-hardening 邻域记账簇；解锁 cors-origin
  +8 / cors-basic +2 / integrity +1 / response-url +1。make test 全绿
  （20649P/0F）+ clippy + fmt 干净。见
  [evidence/2026-10-14-m5-s5-scheme-flip.md](evidence/2026-10-14-m5-s5-scheme-flip.md)。
- **M5-S4（2026-10-14）**：websockets corpus 收口（**376/376 = 100% / case
  106/106**）。① constructor/017 翻案为产品缺口：classic 间接 eval 顶层声明导出
  扫描（script_gen.rs scan_top_level_decl_exports）`const|let` 分支声明符接受集
  补 `;`/`,`——`let x;` 无初始化形态 accessor 导出（getter 闭包持活 eval 词法
  绑定，017/constants.sub.js 断言面）；part50 双态回归测试。② basic-auth：net
  预置 Authorization（userinfo 手工解析 + 零依赖 base64 编码——url crate panic
  面规避）+ ws_fixture /basic_auth（401 挑战/凭据 101）+ rewrite_wpt_authority
  userinfo 感知；net known-answer 单测。P6 二期四切片（M5-S1 通道 89.1% → S2
  IDL/状态机 98.4% → S3 cookie jar 99.5% → S4 收口 100%）。make test 全绿 +
  clippy + fmt 干净。见
  [evidence/2026-10-14-m5-s4-ws-complete.md](evidence/2026-10-14-m5-s4-ws-complete.md)。
- **M5-S3（2026-10-14）**：eventsource 漂移归因修复 + WS cookie jar 集成（websockets
  **373/375 = 99.5%** / case **104/106 = 98.1%**；eventsource 35/35 = 100% 回归）。
  ① request-cache-control Timeout：探针页三级归因（`Cache-Control: no-cache` 非
  safelisted → preflight 须过、runner cors.py fixture 缺 ACAM/ACAH 应答）→ fixture
  OPTIONS 应答补齐（ACAM/ACAH 回显）；② 双 jar 断缝修齐：net `response_cookies()`
  透出 + bridge `WsEvent::SetCookie`（Open 前派发）+ `merge_handshake_cookie` 双源
  合并 + shim `_zwCookieStoreSet` scope 化（WS 面 HttpOnly 置位）+
  `_zwCookieHeaderForUrl` 出站检索 + globalThis 跨闭包桥（part06 双 IIFE——首版
  直引 ReferenceError 定位）。product 面：net/bridge/shim + runner fixture；零回归
  （cookies 006 原 Pass 面保持）。make test 全绿（**20646P/0F**）+ clippy + fmt 干净。
  见 [evidence/2026-10-14-m5-s3-es-cookie.md](evidence/2026-10-14-m5-s3-es-cookie.md)。
- **M5-S2（2026-10-13）**：websockets IDL/构造器/状态机修齐（**369/375 = 98.4%**
  子测试 / case **100/106 = 94.3%** / Timeout 2→0；基线 334/375 = 89.1% → +35 子
  测试 +22 case 零回归）。shim part06：close 码域 InvalidAccessError（现行 spec——
  9 子测试）+ 构造器 fragment 门（spec step 4）+ 协议 RFC 7230 token 文法 + options-bag
  /scheme 转换/无参 TypeError/binaryType·on* accessor/send DOMString 兜底（前轮遗留
  变更本轮验证收编）。shim part03：A/AREA stringifier→href 三层落点（trap
  @@toPrimitive 实为吞点——探针页定位，前轮锚点分支加挂系死代码）。net/bridge：
  `Sec-WebSocket-Protocol` 往返 + `ws.protocol` 透出 + per-host 连接串行 +
  close 握手等待段（回关码优先/EOF→1006/12s 兜底）。六 corpus 零回归逐域实证；
  eventsource request-cache-control Timeout 经 stash 对照实证为先在漂移（记账）。
  前轮 streams 探针残留 2 页清理；存量桥测两处随 spec 翻新（constructor http: 转换
  放行 + close 码域 InvalidAccessError；echo mock 子协议选定——RFC 6455 §4.2.2
  tungstenite 客户端强制）。make test 全绿（**20643P/0F**，首跑 2 失败系存量桥测
  断言旧语义、随 spec 翻新后复绿）+ clippy（engine/net/wpt-runner）+ fmt 干净。见
  [evidence/2026-10-13-m5-s2-ws-semantic-fixes.md](evidence/2026-10-13-m5-s2-ws-semantic-fixes.md)。
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
1. **scheme 对齐两前置审计 → 已清偿（M5-S5，2026-10-14）**——翻转落定（六 corpus
   总账 +179）；残差：url 基址链 **已清偿（M5-S6，同日 +215 零回归）**；
   Authorization 重定向保留流 21 case（security-hardening 邻域）
1.5. **url 基址链续片 → 已清偿（M5-S7，2026-10-14）**：URL 反射/请求/导航五面
   基址接线落定；残余仅 fixture 通道 URL 改写第三形态基址域（语料无此形态，
   观察项）。a-element 残余 348F + a-element-origin 13F = 解析器深水簇
   （P4 既有记账，非基址面）
2. **结构性残余跨域认领**：streams transfer/detach 14 腿（spec
   TransferArrayBuffer——需宿主 V8 detach 能力，JS 层不可表达——bad-buffers 10
   + non-transferable 4；engine 宿主能力面）；cors credentials/cookie jar +
   Authorization 重定向流 ~13 腿（storage goal / security-hardening 邻域）；
   WS 残余 **已全部清偿（M5-S4 收口）**——P6 二期记账仅余不测面（wss/h2 变体、
   TLS、结构面）与 userinfo 百分号编码凭据解码（语料无此形态）；eventsource
   request-cache-control Timeout **已修复归档**（M5-S3——runner fixture preflight
   应答面，非产品缺口）；url crate panic 防护调用点审计（一次性 panic 未复现，
   低优先观察项）
3. **goal 内结构性挂账（M5-S10 后余量）**：XML 文档解析（engine XML DOM——
   template-element 族）；formdata 残余 5F（structural/wire/跨域几何——M5-S8
   精确记账，本 goal 不再实施）。timeout 族 3 腿 **已销账（M5-S10，2026-10-14）**
   ——setter 重排附带 overrides 族 2 腿一并回绿。**下一候选切片（M5-S11）**：
   XHR 小残量三簇——response-body-errors 2 腿（XHR bad-chunk 面，可复用 M5-S9
   `x-zero-body-error-after` 机制扩 XHR 路径）+ getallresponseheaders-after-reuse
   1 腿（复用响应头表重置）+ event-error.sub 1 腿（先甄别归属）；XML DOM 深水
   切片在其后

**待用户决策清单**：（空）
