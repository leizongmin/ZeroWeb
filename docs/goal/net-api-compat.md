# Network API 兼容二期 — url/mimesniff/fetch 残余失败收敛

> **二期立项（2026-10-27，coverage 对账驱动）**：一期（2026-10-04 收口归档，
> [归档文档](archive/net-api-compat.md)）收口时残余失败挂账、WebSocket 二期
> 挂账。coverage 对账显示 url/mimesniff/fetch 三族在已覆盖面仍有连片失败子
> 测试，本 goal 承接失败簇收敛 + WebSocket 挂账重估。
> **一期零回归底线继续有效**（一期 corpus 分母逐 subtest 对照）。

**版本**: v2.1
**日期**: 2026-10-29（M1 归因收口）
**状态**: Active——M1 完成（DC-1/DC-3 落档），M2+ 未启动
**执行模式**: WPT 驱动（上游 corpus 为验收标尺）+ 失败簇归因收敛
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——网络面）

## 当前状态（2026-10-29）

- **M1 完成**：通道全量基线七域 9354/12078 = 77.4%（url 1146F、mimesniff
  865F、fetch/api 572F+1T 为三大失败族）；失败簇逐簇归因落
  [M1 归因档](net-api-compat/evidence/2026-10-29-m1-failure-attribution.md)，
  WebSocket 重估结论**维持挂账**（残面全为 runner 结构面）落
  [DC-3 档](net-api-compat/evidence/2026-10-29-m1-ws-reassessment.md)。
- **一期零回归对账**：url/streams/eventsource/fetch-other 零回归；mimesniff
  −856 与 fetch/api 13 legs 为一期已记账翻转/trade-off；**真回归 3 legs**
  （xhr json UTF-16、xhr CORS 同源重定向、ws constructor/016）挂收敛轮清偿。
- **卡点**：url 759F 根因在 `url`/`idna` crate 共享路径（FailFast vs spec
  IgnoreInvalidPunycode），收敛点为 `js_dom_bridge.rs` 解析回退，涉及共享
  crate 边界，改动按依赖约束核对。
- **下一步（M2+，按可收敛腿量排序）**：① mimesniff Request/Response 腿
  blob().type parse-serialize（263+xhr 7 连带）；② url IDNA（~783）；
  ③ fetch/api 构造/消费语义簇（~138）+ 真回归 3 legs；④ runner 基建切片
  （cors-filtering/request-cache 族，记账推动）；⑤ USP BOM（28）/
  setters-stripping（54）小片。

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
