# Network API 兼容二期 — url/mimesniff/fetch 残余失败收敛

> **二期立项（2026-10-27，coverage 对账驱动）**：一期（2026-10-04 收口归档，
> [归档文档](archive/net-api-compat.md)）收口时残余失败挂账、WebSocket 二期
> 挂账。coverage 对账显示 url/mimesniff/fetch 三族在已覆盖面仍有连片失败子
> 测试，本 goal 承接失败簇收敛 + WebSocket 挂账重估。
> **一期零回归底线继续有效**（一期 corpus 分母逐 subtest 对照）。

**版本**: v2.4
**日期**: 2026-10-29（M2-S3 收口）
**状态**: Active——M1/M2-S1/M2-S2/M2-S3 完成（累计 +1219），下一步 M2-S3b（真回归 3 legs 清偿）
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
  91/220 → 219/220；r1 过程轮 28 回归全部清偿（abort/general 16 legs 构造
  优先 + wire 头 11 legs + 上传 1 leg）。记账 4 项落证据档。
- **一期零回归对账**：各轮 A/B 均零回归；**真回归 3 legs**（xhr json UTF-16、
  xhr CORS 同源重定向、ws constructor/016）挂 M2-S3b 清偿（xhr json 修复点
  已定位 `part03.js` json 分支 UTF-8 decode）。
- **下一步（M2-S3b）**：真回归 3 legs 清偿；其后 runner 基建切片（记账
  推动：url crate panic、端口锚结构面）、USP BOM/setters-stripping 小片、
  M 收口 coverage 复测。

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

## Done Criteria

- **DC-1（归因表）**：三族失败逐簇归因落 `docs/goal/net-api-compat/evidence/`
  （簇 → 根因 → 归口 → 可收敛性）。
- **DC-2（收敛轮）**：真引擎簇逐域修齐，每轮与一期基线逐 subtest 对照零回归，
  evidence 落档。
- **DC-3（WebSocket 结论）**：挂账重估结论落 evidence（可行切片或维持挂账
  及重入条件）。
- **DC-4（coverage 复测）**：三族已覆盖失败子测试显著下降（M1 归因定量：
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
