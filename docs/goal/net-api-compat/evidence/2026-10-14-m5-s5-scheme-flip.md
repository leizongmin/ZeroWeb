# M5-S5 — 页面 scheme 对齐翻转（M4-S27 两前置审计清偿，2026-10-14）

**背景**：M4-S27 记账（2026-10-01）——页锚 https→http 翻转当时净 −12，重入条件 =
两项前置审计。本切片实施域内翻转（net-api 六 corpus）+ 双审计实证清偿。六 corpus
总账：**+179 子测试**（url +186 / fetch −9 / xhr +2 / mimesniff·streams·eventsource
持平）。

## 翻转实施（testharness.rs 三处）

1. **页锚 scheme**：`page_scheme` 判定改 `net_api_case_uses_http_page(case_name)`
   （NET_API_CORPUS_SUBDIRS 前缀枚举——与 run_net_api_cases 通道同口径）；其余
   通道（service-workers 等 secure-context 面）维持 https 页锚不动；websockets
   特例被覆盖（同 http，语义不变）。
2. **资源 fetcher strip 双 scheme**：image/script fetcher `strip_prefix(
   "https://wpt.test")` 增 http 回落。
3. **响应类型过滤**：`wpt_fetch_response_type` 同源判定由单 `page_origin` 常量改
   wpt.test 权威双 scheme 成员（本 runner 页面域恒 wpt.test）。

SW 重定向 fixture 解析（resolve_service_worker_fixture_redirect）不动——SW 通道
页锚保持 https。

## 审计①（url 基址回退）——×20 未复现，净 +176

- url corpus：4429P/1636F（https 锚）→ **4615P/1646F（http 锚）= +186P/+10F**。
- 回归面仅 **15 子测试**（a-element 12 + a-element-origin 3），形态 = `<https:…>`
  special-scheme 相对输入（`https:example.com/` against `<about:blank>` /
  `<//foo/bar>`）；对称地 20 个 `<http:…>` 形态子测试在新锚下**翻绿**。
- M4-S27 所记「×20 基址回退回归」未复现——M4-S26 后 shim 演进（M5-S2 A/AREA
  stringifier 等锚点面工作）已消化大半；残差 +5 子测试归 url 基址链
  （探针实证：锚点 accessor 基址 = 页面 location 而非 `<base>` 元素 /
  document.baseURI 未实现——**url 基址链记账保留**，非本翻转引入）。

## 审计②（credentials/Authorization 流程）——−21 案即已回流记账簇

- fetch corpus：1499P/589F/3T → **1490P/598F/3T = −9P**。
- 回归集中三案 21 个 case：**cors-redirect-credentials 15 + cors-redirect 5 +
  authentication-redirection 1**——形态 = 「Redirect 301 from remote to same
  origin with user/password」期望 reject 实际 resolve（Authorization 重定向保留流
  在 http 页锚下的逐跳 CORS/凭据剥离语义）。
- **归属**：M4-S27 评估一已将「userinfo/Authorization 重定向保留流」记账回流
  security-hardening 邻域（goal 排除「HTTP 栈本身」）——本翻转把该簇从「https 锚
  下偶合通过」转为既有记账项的真实缺口面（−9 子测试），非新增产品缺陷。
- 解锁：**cors-origin +8 case、cors-basic +2、integrity +1、response-url +1**
  （M4-S27 预估 +29 的 fetch 面）+ url a-element/http: 形态面（+186）。

## 判定

M4-S27 重入条件两审计**实证清偿**：净账从 −12 反转为 **+179**，翻转落定（域内
scheme 双态，跨通道零影响）。Authorization 重定向簇维持跨域记账（security-hardening
邻域）。

## 门禁

- cargo fmt 干净；clippy（wpt-runner，-D warnings）零告警；make test 全量结果见
  master.md 本切片条目。
- 六 corpus 逐域数字：url 4615P/1646F、fetch 1490P/598F/3T、xhr 564P/28F/3T、
  mimesniff 1033P/865F、streams 905P/108F/5T、eventsource 35P/0F/0T。

## 下一步候选

1. Authorization 重定向保留流（security-hardening 邻域，跨域记账——本 goal 不实施）
2. url 基址链残差（锚点 accessor `<base>` 元素 / document.baseURI 面——+5 子测试，
   独立切片评估）
3. go back and re-run 全量基线回填 suites CSV（url corpus 大幅位移后）
