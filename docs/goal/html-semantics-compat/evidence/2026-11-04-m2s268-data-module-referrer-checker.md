# M2 s268 — css/json/text-module referrer-checker 端点仿真 + 元素级 referrerpolicy 图穿参（三页 3T→24P 收口）

日期：2026-11-04。切片号 s268（承 s267 下一步候选；s116 挂账
"text/css/json-module referrer-policies（3T）"收口）。3 文件 3 刀
3 钉（engine + webview + runner 三车道同切）。

## 勘根

1. **入场承 429 中断现场**：上轮 session API 429 中断于 mid-flight
   diff——`extract_script_referrer_policies` 已落 extract.rs、
   `collect_module_deps_recursive` 签名已加 `referrer_policy` 形参
   但 4 个调用点未跟上（E0061 ×4 编译红）。本切先恢复现场再续勘。
2. **上游 checker 体形**（curl 三端点原文镜像，语料端点文件保持缺
   席 runner 仿真）：css-module/referrer-checker.py 回
   `.referrer { content: "<R>" }`（页断言
   `.rules[0].style.content` 含引号形，charset.html 语料钉先例同
   口径）；json-module 回 `{"referrer": "<R>"}`；text-module 回裸
   `<R>`，no-referrer 臂**裸空体**。有效策略三层取：
   script 元素 `referrerpolicy` 属性 > `?pipe=header(Referrer-Policy,X)`
   > `<meta name=referrer>` > SWCO 默认。
3. **端点臂放置**：referrer-checker.py 臂须前置在 `__zw_mtype`
   intent 臂**之前**（s267 同理由——css 端点取回若落 intent 臂被
   Err 接管，端点语义断裂）。
4. **text 空体 blocker**：no-referrer 臂裸空体撞 fetcher 契约
   「空体=取回失败」既有防线（语料无空体 .js/.json/.css 钉面）
   → 图收集放宽：`source.is_empty() && type_attr != Some("text")`
   才维持失败语义；text 数据模块空体合法（`export default "";`
   合成面照常成立）。
5. **方案伪影（scheme artifact）定谳**：首跑 solo 21P/3F——3 个
   **跨源 default 策略臂** expected `https://wpt.test/` got `""`。
   根因非语义缺口：runner 文档锚 `https://wpt.test` 合成 origin，
   而跨源取回落 `http://www1.wpt.test`（语料 token URL）→
   https 源→http 请求触发 `wpt_referrer_is_downgrade` 降级臂回空。
   **曾试合成 host 豁免改 → 8 job 族跑 strict-policies strict-*
   子测 P→F 回归**（HEAD stash solo 实证 strict-policies 原本
   Pass，s116 原口径不可侵）→ revert 豁免，改走 s116 先例
   `referrer_case_uses_http_page` 扩展：三页上游期望值锚
   `(new URL(location.href)).origin` 的 http 形（页断言字面即
   `http://wpt.test:80` 形语义），文档锚改 `http://wpt.test/`
   后 SWCO 默认策略跨源臂不再落降级臂，24 子测全对位。

## 施工（3 文件 3 刀 3 钉）

1. **engine extract.rs**：`extract_script_referrer_policies`
   （镜像 `extract_script_csp_info` 枚举：gEBTN("script") + trim
   空体→None，序数对位页面脚本单元）；钉
   `extract_script_referrer_policies_aligns_with_script_ordinal_s268`
   （5 script 含 harness/JSON 占位，trim 断言 + 序数配对）。
2. **webview.rs**：`collect_module_deps_recursive` +`referrer_policy`
   形参递归穿参（`#[allow(clippy::too_many_arguments)]` 8 参形）；
   静态顶层图位 `script_referrer_policies.get(idx)` 穿入；动态
   import() 三调用点（`__zw_compile_module` shim / native resolver /
   `__zw_compile_inline_module`）传 `None`（dynamic import() 元素
   级策略面不在语料，FIXME 注记挂账）；text 空体放宽；钉
   `script_referrerpolicy_graph_marker_and_empty_text_module_s268`
   （no-referrer 图取回 spec 含 `__zw_lp=no-referrer` +
   `__zw_mtype=text`，空体合成 `globalThis.__t === ""`）。
3. **runner testharness.rs**：referrer-checker.py 三域端点仿真臂
   （`__zw_lp` > pipe header > doc policy > SWCO 默认，经既有
   `compute_module_referer` 8 策略表回体；css→CSS 规则形 /
   json→JSON 形 / text→裸体，与上游体形逐字对位）；
   `referrer_case_uses_http_page` 扩 3 页；钉
   `data_module_referrer_checker_endpoint_sims_s268`（7 断言：
   css default 全 URL / lp=no-referrer / lp=origin / json 跨源
   origin 形 / text 跨源 unsafe-url 全 URL / text 同源
   no-referrer 空体 / 非法 lp 落 doc policy）。

## 验证

- **钉**：engine pin + webview pin + runner pin 绿；runner bin
  全量 **285P/0F/76I**（s267 284 + 1 新钉）。
- **solo**：首跑 21P/3F（跨源 default 臂，勘根⑤方案伪影）→
  http 页锚扩展后 **24/24 全 Pass**（日志
  evidence/logs/s268-solo-final.log）。
- **豁免改回归证据**（revert 依据）：豁免在树期 8 job 族跑
  79P/2F（strict-strict-\* 子测红 + img-face 既有红；日志
  evidence/logs/s268-family-exemption-regression.log）；
  HEAD stash solo strict-policies Pass。
- **族跑终态**（FILTER 覆盖 css/json/text-module +
  referrer-strict-policies 8 job）：**80P/1F**——唯一残红
  image-loading-lazy-referrerpolicy-change（A/B 基线双红零位移，
  deferred img 最新策略独立面非本切路径，在册既有红；日志
  evidence/logs/s268-family-final.log）。
- **全树 A/B**（stash 同二进制法，test-guard + RAF）：
  **7814P/431F/72T → 7836P/433F/69T**——join 精确 24 目标翻绿
  （3 文件 Timeout→24 子测 Pass）+ checkable-active-onblur ×2
  P→F（在册漂移族 s238/s245/s248 同键混态，B 日志内同文件
  :active 兄弟子测 Pass/Fail 并存自证，销账）+ **REGRESSED=∅
  零其他位移**。
- **make test**（test-guard）：`cargo test -p zero-engine --lib`
  **3722 passed + 5 failed**——5 红为 R-t2qq adopted-stylesheets
  系列既有（s265 账本记档同集，尾分号 Blink 形过期期望，归因
  bisect 至 1f37e08a3 前均红），本流不入侵；3721 基线 + 1 新钉。
- fmt/clippy：`cargo fmt --all` 干净；workspace guarded-clippy
  零告警。

## 挂账更新

- css/json/text-module referrer-policies ×3 文件级 Timeout
  **销案**（s116 挂账该项收口；24 子测全绿）。
- 新记录（零 corpus face，不扩面）：**dynamic import() 元素级
  referrerpolicy 穿参**——webview 三动态调用点现传 `None`
  （spec 动态 import 请求继承 importeer 策略面），语料无断言
  页，随语料扩展补。
- 承前收窄注记：s116 module-import-referrer TLA 挂账——顶层
  await face 保留；本切元素级 referrerpolicy 部分同域不同面，
  不混记。
- 承前不动：image-loading-lazy-referrerpolicy-change 1F
  （deferred img 面，A/B 双红在册）；R-t2qq 5 红（归该流自查）；
  css-module relative-urls / redirect-relative-urls 3F（CSS
  url() 基面，渲染流车道交接）；module map 缺席（s256/s267
  注记）；NotRun 微面（json `;boundary` ×3、css `;params` ×2）。

## 下一步候选

1. module-import-referrer 顶层 await face（s116 挂账余项，
   runner 车道）。
2. image-loading-lazy-referrerpolicy-change 勘根（deferred img
   装载策略切换面，engine/webview 车道；本切 A/B 双红零位移
   证与 referrer 穿参无涉）。
3. script-element-css-src module MIME 检查余项（s265 候选 #1
   承接，zero-net script_mime.rs 助手在册）。
