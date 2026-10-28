# Network API 兼容二期 — url/mimesniff/fetch 残余失败收敛

> **二期立项（2026-10-27，coverage 对账驱动）**：一期（2026-10-04 收口归档，
> [归档文档](archive/net-api-compat.md)）收口时残余失败挂账、WebSocket 二期
> 挂账。coverage 对账显示 url/mimesniff/fetch 三族在已覆盖面仍有连片失败子
> 测试，本 goal 承接失败簇收敛 + WebSocket 挂账重估。
> **一期零回归底线继续有效**（一期 corpus 分母逐 subtest 对照）。

**版本**: v2.8
**日期**: 2026-10-29（M 收口 + 收口后小片 M2-S5）
**状态**: Done——DC-1/2/3/4 全部满足（[收口档](net-api-compat/evidence/2026-10-29-m-closure-dc4-retest-zero-regression.md)）；一期基线 9354 → **10882**（+1528，含收口后 M2-S5 小片 +39），全轮零真回归；request-cache 98F 口径待决项挂用户，不阻塞判定
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
- **待决项（用户）**：request-cache 族（98F）是否维持「HTTP 栈本体」
  排除口径（DC-4 fetch/api ~300 目标按剔除口径已达标；立项设计则需
  HTTP 缓存层方案，挂用户拍板）。
- **后续可推进小片**（不阻塞收口，重开时按此入列）：abort/general
  引擎入口同步 cancel（4F）、SRI 切片（integrity 显影后重排）、
  request-headers 方法面余 22F（runner 域，M2-S5 首腿清偿后）、
  request-cache HTTP 缓存层（待决项立项后）。

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
