# Network API 兼容二期 — url/mimesniff/fetch 残余失败收敛

> **二期立项（2026-10-27，coverage 对账驱动）**：一期（2026-10-04 收口归档，
> [归档文档](archive/net-api-compat.md)）收口时残余失败挂账、WebSocket 二期
> 挂账。coverage 对账显示 url/mimesniff/fetch 三族在已覆盖面仍有连片失败子
> 测试，本 goal 承接失败簇收敛 + WebSocket 挂账重估。
> **一期零回归底线继续有效**（一期 corpus 分母逐 subtest 对照）。

**版本**: v2.21
**日期**: 2026-10-29（收口后小片 M2-S5/S6/S7/S8/S9/S10/S11/S11b/S12/S13/S14/S15/S16/S17/S18）
**状态**: Done——DC-1/2/3/4 全部满足（[收口档](net-api-compat/evidence/2026-10-29-m-closure-dc4-retest-zero-regression.md)）；一期基线 9354 → **11040**（+1686，含收口后 M2-S5 +39 / M2-S6 +5 / M2-S7 +6 / M2-S8 +4 / M2-S9 +27 / M2-S10 +25 / M2-S11 +40 / M2-S11b +6 / M2-S12 +5 / M2-S16 +11 / M2-S17 +28 / M2-S18 +1；M2-S13 语料巡检红腿 −1 分母 12078→12077；M2-S14 spec 对齐零净账；M2-S18 +1 为基线漂移修正，切片净账 0/0/0，见 S18 档记账 3），全轮零真回归；request-cache 98F 口径待决项挂用户，不阻塞判定
**执行模式**: WPT 驱动（上游 corpus 为验收标尺）+ 失败簇归因收敛
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——网络面）

## 当前状态（2026-10-29）

- **M1 完成**：通道全量基线七域 9354/12078 = 77.4%；失败簇逐簇归因落
  [M1 归因档](net-api-compat/evidence/2026-10-29-m1-failure-attribution.md)，
  WebSocket 重估**维持挂账**落
  [DC-3 档](net-api-compat/evidence/2026-10-29-m1-ws-reassessment.md)。
- **M2-S1 完成**（[证据档](net-api-compat/evidence/2026-10-29-m2-s1-blob-type-direct-set.md)）：
  blob() 直设 **+270 零回归**（mimesniff 263→残余 602 即上游语料冲突记账面；
  xhr overridemimetype-blob 7）；−1 派生 CT 大小写面语料内部互斥，记账。
- **M2-S2 完成**（[证据档](net-api-compat/evidence/2026-10-29-m2-s2-idna-punycode-fallback.md)）：
  url IDNA IgnoreInvalidPunycode 回退（`js_dom_bridge/url_idna.rs` 占位标签
  回退 + 组件回写）**+789 零回归**——IdnaTestV2 2671/2671 = 100%，url 通过率
  81.1% → 94.1%；url 残余 357F（setters-stripping 54 + USP BOM 28 + 单案面）
  与 M1 预测一致。
- **M2-S3 完成**（[证据档](net-api-compat/evidence/2026-10-29-m2-s3-request-response-constructor.md)）：
  Request 构造校验链/Headers record 逐 trap/Response 消费面/wire 头合并/
  流式上传分型 + echo-content fixtures **+160 零回归**——13 文件语义族
  91/220 → 219/220；r1 过程轮 28 回归全部清偿。记账 4 项落证据档。
- **M2-S3b 完成**（[证据档](net-api-compat/evidence/2026-10-29-m2-s3b-true-regression-clearance.md)）：
  **真回归 3 legs 全清**（xhr json UTF-16 → UTF-8 decode；ws
  constructor/016 → `/echo-query_v13` fixture 路由；xhr CORS 同源重定向 →
  页锚对齐）+ net-api 页锚 `wpt.test`→`wpt.test:8000` 结构面 + shim redirect
  语义两项（cors 跳 Origin 恒归一、Location credentials network error）：
  全通道 A/B **0 回归 +11**（10574→10585，8 MISSING 逐条核对为 subtest
  改名 Pass→Pass）。
- **M2-S3c 完成**（[证据档](net-api-compat/evidence/2026-10-29-m2-s3c-fetch-defaults-txt-unlock.md)）：
  fetch fixture `.txt` 三件解锁（top.txt/cors-top.txt/empty.txt，46F 面）+
  `fetch/api/resources/status.py` 路由同语义接入（10F）+ 浏览器默认请求头
  UA/`Accept: */*` 缺省补齐（2F，预检名单零变化）+ null body status
  204/205/304（9F）+ preflight UA 一致性对齐：fetch/api 404F → **334F**。
- **M2-S4a 完成**（[证据档](net-api-compat/evidence/2026-10-29-m2-s4a-url-setters.md)；
  前轮遗留未提交工作本轮补验收口）：url 组件 setter 语义面（no-throw
  分型、cannot-have 守卫、host/hostname state override 机、opaque path
  SPACE 归一、hash getter 空串面 + ws 构造器 fragment presence 连带回归
  修复）：url 357F → **169F**（setters +110 / setters-stripping +54 /
  a-element、url-constructor 各 +11）；url crate panic stderr 噪声 0 次
  （基线每轮 2 次）。
- **联合终态 A/B（S3c + S4a 一轮全通道）**：`netapi-m2s3c-r4.json` vs
  `netapi-m2s3b-r3.json` 逐 subtest——**+258 增益 / 0 真回归 / MISSING 0**
  （10585 → 10843）。唯一 Pass→Fail 为 integrity「CORS invalid integrity」
  **语义显影**记账（基线靠 Failed to fetch 假绿，top.txt 解锁后显影为
  SRI 真缺口，机械证据落 S3c 档记账 1）。
- **DC-4 判读（复测数据待 M 收口终稿）**：mimesniff 865→**602** ✓ 精确
  命中；url 1146→**169** ✓ 超额（目标 ~360）；fetch/api 572→**334**
  （目标 ~300：剔除 request-cache 族 98F「HTTP 栈本体」standing
  exclusion 后 236F 已达线，含族则差 34F——口径决策见待决项）。
- **一期零回归对账**：各轮 A/B 均零回归；真回归 3 legs（M2-S3b）+ 过程
  轮 2 legs（S3c/S4a）全部清偿；1 语义显影记账（非行为回归）。
- **M 收口完成**（[收口档](net-api-compat/evidence/2026-10-29-m-closure-dc4-retest-zero-regression.md)）：
  DC-4 终稿（url 1146→169F 超额 / mimesniff 865→602F 精确命中 /
  fetch/api 572+1T→334+1T，剔除 request-cache standing exclusion 达线）+
  **一期零回归声明**（全轮 A/B 零真回归，显影 1 记账）。二期 Done
  Criteria 全部满足。
- **M2-S5 完成（收口后重开小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s5-header-values-wire-escaping.md)）**：
  header-values-normalize 36F 三分解收敛——fetch 头 wire `\x1e`/`\x1f`
  边界对称转义（`fetch_bridge` + shim 六点）、fixture `inspect-headers.py`
  POST 豁免、空值头回显：normalize **62/62 收口**；全通道 vs M 收口参考
  **+39 零回归**（10843 → 10882），fetch/api 334F → **295F**，url/mimesniff
  零扰动。
- **M2-S6 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s6-abort-entry-sync-cancel.md)）**：
  abort/general 引擎入口同步取消面——fetch 入口已 aborted 信号同步
  cancel 流体 body + 置 Request bodyUsed（abort-a-fetch 语义，M2-S3
  记账 3 清偿）+ fetch settle 时序去包装（RT 发射侧信道化）+ stash
  仿真 fixture 扩注入 abort/general：**+6 零真回归**（10882 → 10887），
  general.any.js 29P/32F → 35P/26F；A/B 唯一 Pass→Fail 为 SRI 腿
  **语义显影**记账（旧 RT 包装 +1 microtask 假绿被去包装显影，与 S3c
  记账 1 同根因 SRI 未实现，机制落证据档）。
- **M2-S7 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s7-sri-integrity-verify.md)）**：
  SRI integrity 校验门（fetch verify 步，shim 单点）——opaque+integrity
  fail / null-body+integrity fail / metadata 解析（最强算法组 + base64url
  宽容面）/ 摘要直调 `__zw_crypto_subtle_digest`（http 页锚下
  crypto.subtle 未挂载）：**+6 零回归**（10887 → 10893），
  integrity.sub 13P/5F → **18P/0F 收口**，response-null-body SRI 腿
  翻绿；S3c 记账 1 与 S6 记账 4（同根因显影两腿）双双清偿。
- **M2-S8 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s8-signal-identity-clone.md)）**：
  Request signal 身份族——构造器克隆原语 `_zwAbortSignalFollow`（独立对象
  + `_zwDependents` 依赖登记）+ `_zw_abort_signal` 依赖级联（dom spec
  signal abort 步骤 3-6：依赖先统一置 aborted、abort steps 源先依赖后派发）
  + 构造器/fetch 两处 signal 三态（AbortSignal / 显式非 AbortSignal 移除 /
  缺省回落载体）：**+4 零回归**（10893 → 10897），abort/general
  35P/26F → 39P/22F，「Clone aborts with original controller」顺序腿
  保持 Pass（过程轮 listener 方案翻红未落提交，dependent 模型清偿）。
- **M2-S9 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s9-body-abort.md)）**：
  abort body-abort 族 **27F 全收口**（general 61/61、request 18/18）——
  消费门 `_zwAbortErr`（先于 unusable）+ consume 族同步 gate 直返已拒
  promise（`.then` 派生跳破坏 [reject, next-microtask] 排序的过程轮教训落
  档）+ `_zwController` 流外可达 + settle 后 abort 错误体接线 +
  fetch(request) 传输即扰动（空体豁免）+ runner fixture 体改开放流：
  **+27 零回归**（10897 → 10924）。
- **M2-S10 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s10-request-headers-method.md)）**：
  request-headers 方法面 **25F 全收口**（basic/request-headers 22F +
  request-headers-case 2F + request/request-headers 1F）——Origin 第三分支
  （非 GET/HEAD 恒带，FIXME 清偿）+ string 体默认 CT + CL:0 收窄 POST +
  头名大小写线上保真（`_hCase` + wire 同名值合并 + 内部头 Title-Case 31 处 +
  runner echo-headers 逐字回显）+ set-cookie 入 forbidden 名单：
  **+25 零回归**（10924 → 10949）。
- **M2-S11 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s11-cors-preflight-cache-coverage.md)）**：
  CORS preflight 面 **+40 / 0 回归**（10949 → 10989）——preflight 缓存键补
  URL 维度（spec (origin, URL, credentials) 条目）+ settlePre ACAM/ACAH 覆盖
  spec 化（token 语法校验 / ACAM 大小写敏感 / `*` 凭据条件 / authorization
  非通配）+ 缓存 method 精确匹配 + accept-language/content-language 值条件
  safelist + hop 门 taint 补位 + 两臂缓存仅成功写入 + fetch 入口 method
  归一 `_zwNormalizeMethod` + ACRH 无空格连接。六文件收口
  （redirect-preflight 30/30、star 34/34、preflight 22/22、no-preflight
  15/15、cache 2/2、response-validation 2/2）；cors-preflight-redirect 余
  5F、referrer 6F 记账落档。
- **M2-S11b 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s11b-window-origin-var-shadow.md)）**：
  Window.origin 自有 accessor → 数据属性快照（`part07.js` 安装尾）——
  QuickJS 不落实 spec CreateGlobalVarBinding 的 var 重声明转换，cors-preflight-referrer
  顶层 `var origin` 期望值静默丢失（S11 记账 2 的引擎 `_zwRefOut` 假设经
  诊断用例实证推翻，引擎 Referer 本已正确）：referrer **6F 全收口 +6 零
  回归**（10989 → 10995，6/12 → 12/12）；经验落 learnings/bugs。
- **M2-S12 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s12-preflight-redirect-corpus-drift.md)）**：
  cors-preflight-redirect 五腿**归因反转 + 语料对齐收口**——S11/S11b 记账的
  runner stash 互写 KeyError 假设经上游源码核对证伪（wptserve Stash 按请求
  path 作用域，跨 fixture 不碰撞）；真根因是语料漂移（上游 651ffcb03 于
  2026-09-12 把五腿改写为「跟随 + 重定向跳重预检」，本地快照早于该提交，
  Chrome 158 实测 10/10 绿）。语料逐字对齐上游 HEAD，引擎零改动（M4-S24
  重预检机制既有）：cors-preflight-redirect 5/10 → **10/10 收口 +5 零回归**
  （10995 → 11000）；教训落 learnings/bugs。
- **M2-S13 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s13-corpus-drift-sweep.md)）**：
  语料漂移巡检首轮 + request-upload 假红腿清偿——mtime 取证证伪 v1.10 周级
  漂移前提（三域 247 文件已由交互会话于 2026-10-29 00:19–00:39 全量同步上游
  master，无记录），实测漂移仅 9 文件并全部分类处置：request-upload
  上游已删的「421 text retry」腿逐字对齐清偿（引擎零改动，探针 18/18，全通道
  A/B **0 增益 / 0 回归 / 0 翻转 / 删除 1**，pass 11000 持平、分母 12078→
  12077）；request-disturbed 期望翻转（init body 不扰动源）记账挂起（引擎+
  语料须同步翻，单独动互斥翻红）；additive/数据 7 文件不追 HEAD（口径记账 2）；
  巡检结论不常态化立项（S12 记账 3 清偿，收敛到语料仓滚动 tag 单点结算）。
  update-wpt-data.sh 缺口清单补两条（request-upload 无需重补 + disturbed
  翻转预警）。「13h 窗」口径经 S14 修正：语料实有 pin `WPT_REV=3159769`
  （2026-08-12），9 differ 为 pin→HEAD 约 2.5 个月漂移，结论不变。
- **M2-S14 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s14-request-disturbed-init-body.md)）**：
  request-disturbed 期望翻转双侧同步收口（S13 记账 1 清偿）——fetch spec
  现行文本 initBody 非空时不触碰 inputBody 分支，shim 构造器尾步扰动标记加
  `!bodyGiven` 作用域（一行，采纳 input 体形态才标源 used）+ 语料逐字对齐
  上游 HEAD。探针 5/5（新腿绿、无 init 体旧腿绿、unusable 拒绝腿绿）；全通道
  A/B **0 增益 / 0 回归 / 0 翻转**（改名腿 Pass→Pass，pass 11000 持平）——
  spec 对齐零净账，保障语料滚动后该腿不翻红。附带发现：引擎 10 个
  js_dom_bridge 单测长期预存红（V8 桥 about:blank 基址相对 URL 严格拒绝，
  回溯 M2-S11b 同失败；stash 干净树同集合）——非本切片引入，归引擎域
  独立切片（记账 2），共享面不越界。
- **M2-S15 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s15-xhr-streams-drift-sweep.md)）**：
  xhr/streams 语料漂移巡检（S13 记账 4 挂账清偿，巡检范围至此覆盖通道五域）
  ——双基准比对（HEAD + `WPT_REV` pin）：447 文件对 pin **0 差异**（S14 pin
  模型实测成立），对 HEAD 漂移 7 文件全部分类且**零可清偿面**（additive/
  加严/元数据/绿腿上游删除各归记账）；通道 xhr/streams 红腿与漂移无交集，
  `ReadableStream.from` 11F 认证为**有钉真引擎缺口**（streams 域最大收敛面，
  入列后续小片）。巡检不常态化结论维持，收敛到 pin 滚动单点结算。
- **M2-S16 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s16-readablestream-from.md)）**：
  ReadableStream.from **11F 全收口**（from.any.js 39P/11F → **50P/0F**）——
  `part02.js` 按 streams spec §rs.from + WebIDL §3.2.22 async sequence 原语
  重做：参数非 Object 同步 TypeError / GetMethod 语义 / 迭代器结果 Object 校验 /
  **value 展平**（Promise.resolve 后 enqueue）/ cancel 面（return 非_callable
  TypeError 拒绝、reason 透传、同步抛转拒绝、fulfill 非对象校验、拒绝传播）。
  spec 迁移期语义以 Chrome 154 实测标定（node 22 旧 V8 不展平为反例，弃用）。
  语料零改动；全通道 A/B **+11 / 0 回归 / 0 MISSING**（11000 → 11011），增益
  全部集中 from.any.js，其余 487 文件零扰动。
- **M2-S17 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s17-usp-bom-formdata-urlencoded.md)）**：
  USP BOM + formData urlencoded **28F 全收口**（urlencoded-parser 77P/28F →
  **105P/0F**；M1 归因「独立小片」清单最后一项——BOM 本档 / setters S4a /
  integrity S7 三项全部清偿）——URLSearchParams 解码改
  `ignoreBOM: true`（url spec urlencoded parser「UTF-8 decoding without BOM」
  保留前导 U+FEFF）；formData 改字节面解析（fetch spec「parse the
  urlencoded-byte sequence」）：体字节 ignoreBOM lossy 文本化 + 逐 pair
  percent-decode（废弃 decodeURIComponent 的 URIError 面）。语料零改动；
  全通道 A/B **+28 / 0 回归 / 0 MISSING**（11011 → 11039），formData 消费
  面 10 文件零扰动。
- **M2-S18 完成（收口后小片，[证据档](net-api-compat/evidence/2026-10-29-m2-s18-body-arraybuffer-wasm-stream.md)；
  wasm-compat goal M3 挂账移交承接）**：Body mixin `arrayBuffer()` 返 spec
  **ArrayBuffer**（fetch spec #dom-body-arraybuffer；R2978/R2982 旧 Uint8Array
  形态解除，Response/Request 两处 + 单测断言同步）→ wasm streaming 挂账
  **2F 清偿**（enqueue 产物撞既有体消费 chunk 类型门，引擎无新增门）——
  wasm/webapi 全子集 **272/272**（M3 基线 270/272），jsapi 底线 878/878 维持，
  wasm-compat M3 挂账结清。全通道 A/B：**REGRESSIONS 0 / MISSING 0**，
  11039 → 11040 的唯一 GAINS 三重对照定谳为 S17 基线漂移伪影（S17 采数
  runner 早于并行流 M3 静态头去重修复，属基线修正）——**本切片净账
  0 增益 / 0 回归 / 0 MISSING**，fetch/api consume 族零扰动。范围引擎
  测试 3592 过/11 败（名单对账 +1 为 rebase 基座预存红，stash 归并行流域）。
  记账 3 项落证据档（语料 403 限流补盘 / 类型门既有承接 / 基线漂移定谳
  与采数教训）。
- **待决项（用户）**：request-cache 族（98F）是否维持「HTTP 栈本体」
  排除口径（DC-4 fetch/api ~300 目标按剔除口径已达标；立项设计则需
  HTTP 缓存层方案，挂用户拍板）；语料仓 zeroweb-wpt-data 是否滚动新
  tag（M2-S12 的 cors-preflight-redirect 对齐目前为本地补丁 + update 脚本
  缺口清单记账，见 [S12 档](net-api-compat/evidence/2026-10-29-m2-s12-preflight-redirect-corpus-drift.md)）。
- **后续可推进小片**（不阻塞收口，重开时按此入列）：request-cache HTTP
  缓存层（待决项立项后）、abort 目录外 signal 关联体中止（S9 记账 1-2，
  语料无钉）、Headers 实例/XHR 异步源大小写保真（S10 记账 1-2，语料无钉）。
  **「有钉引擎缺口」存量清零**（M1 归因独立小片三项 BOM/setters/integrity
  已由 S17/S4a/S7 清偿，ReadableStream.from 由 M2-S16 收口）；
  cors-preflight-redirect 五腿已由 M2-S12 语料对齐收口（S11 记账 1 /
  S11b 记账 1 清偿）；request-disturbed 翻转已由 M2-S14 双侧同步收口
  （S13 记账 1 清偿）。

## coverage 对账（coverage-2026-10-27 基线，47 套件干跑口径，终稿待 `docs/compat/wptfyi/coverage-2026-10-27.md` 复核）

| WPT 面 | 全量分母（子测试） | 覆盖文件 | 已覆盖面失败 |
|---|---:|---:|---:|
| url | 12,323 | 22/77 | **877**/4,005（78.1% 通过）|
| mimesniff/mime-types | 3,837 | 1/3 | **865**/1,898（54.4% 通过）|
| fetch/api | 8,037 | 126/526 | **577**/1,923（70.0% 通过）|

失败合计 ~2,319 子测试，是 encoding 之外最大的已覆盖失败连片区。

## Mission

1. **失败簇归因**：三族失败逐簇定性——引擎语义错误 / runner 基建 / 语料边界
   （`zw-static-corpus` 面）/ 上游漂移，归因表落 evidence。
2. **逐簇收敛**：真引擎语义簇按域修齐（预期 url 解析边界、mime-type 解析矩阵
   是大头）；runner 基建簇归口 runner 域。
3. **WebSocket 二期重估**：一期挂账面按当前 runner 通道现状出可行性与切片
   建议，可行则本轮内立项推进。

## Done Criteria（v2.7 全部满足，对账见[收口档](net-api-compat/evidence/2026-10-29-m-closure-dc4-retest-zero-regression.md)）

- **DC-1（归因表）✓**：三族失败逐簇归因落 `docs/goal/net-api-compat/evidence/`
  （簇 → 根因 → 归口 → 可收敛性）。
- **DC-2（收敛轮）✓**：真引擎簇逐域修齐，每轮与一期基线逐 subtest 对照零回归，
  evidence 落档。
- **DC-3（WebSocket 结论）✓**：挂账重估结论落 evidence（可行切片或维持挂账
  及重入条件）。
- **DC-4（coverage 复测）✓**：三族已覆盖失败子测试显著下降（M1 归因定量：
  mimesniff 865→602、url 1146→~360、fetch/api 572→~300，详见 M1 归因档
  「M2+ 收敛排序建议」）；对一期已收口域零回归。

## 里程碑

- **M1**：失败簇归因（DC-1）+ WebSocket 重估结论（DC-3）。
- **M2+**：逐域收敛轮（url → mimesniff → fetch/api 顺序建议按失败量定）。
- **M 收口**：coverage 复测 + 一期基线零回归声明。

## 依赖约束（run-rules §9）

- URL 解析变化核对 `url` crate 与引擎共享路径（20 号一期已核对的边界继续有效）。
- mime/fetch 语义变化不越 `zero-protocol`/网络栈边界；`zw-static-corpus`
  环境受限面维持不计引擎缺口的对账约定。
