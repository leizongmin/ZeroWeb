# M2-S188 frame content navigable——submission-checks 三兄弟 3F 精确翻绿，shim 5 刀零 rust

## 入场定刀位

s187 账本「下一步候选」首位：form-submission-0/submission-checks
cannot-navigate 三兄弟（连接性面）。

## 勘根

三案直跑取证：失败形态**不是**「提交乱导航」，而是测试体自身抛异常
（`Cannot read properties of undefined (reading 'location'/'body')`）——
`frame.contentWindow`/`frame.contentDocument` 读返 undefined。单根：
三案全部以 **`document.createElement("frame")`** 承载 contentWindow/
contentDocument 访问面（上游 WPT 原文如此，HTMLFrameElement 接口带此二
IDL + Reflect src），而 shim 的嵌套浏览上下文面四处 IFRAME 硬门全不认
FRAME：

- part04 R115 臂（contentDocument/contentWindow/getSVGDocument）
- part01 `_zwStartConnectedIframe`（动态插入初始 navigable + load 派发）
- part05 set trap IDL src/srcdoc 写 → `__zw_reload_iframe` 钩子
- part04 setAttribute src/srcdoc 同款钩子
- part05 URL 反射分支排除清单（FRAME.src 在 `_REFLECTED_URL_TAGS.FRAME`
  在册，先被 verbatim 吞写、到不了带导航钩子的 R3069 尾部）

spec 勘定（obsolete.html §16.3.2 #the-frame-element）：「The frame
element has a **content navigable similar to the iframe** element」；
插入步建 child navigable + process the frame attributes（about:blank
初始插入派 load）；「Whenever a frame element … has its src attribute
set, changed, or removed … process the frame attributes」。

submission 管线本身无需动：`_zwFormSubmitNavigate` 的 connectedness
近似（s144 遗留，`globalThis.document.contains` 判定）对三案全部给出
正确裁决——test 1/2 detached/移除后不可导航、test 3 form 在被换掉的
frame 文档内不在主文档亦被拦。fully-active 完整定义仍是 FIXME 挂账
（近似过宽：活 iframe 文档内 connected form 也会被拦，无语料面）。

## 施工（shim 5 刀零 rust）

- **part04 R115 臂**：tag 归一 `FRAME → IFRAME`（臂内局部
  `_zwIframeTag`，NS fallback 逻辑原位保留）——contentWindow/
  contentDocument/getSVGDocument + named-window 注册 + fallback win
  全管线复用。
- **part01 `_zwStartConnectedIframe`**：门放宽 `IFRAME || FRAME`——
  动态插入建 navigable + 初始 load 派发（spec frame insertion steps）。
- **part05 IDL src/srcdoc 写钩子**：门放宽（spec src attribute set/
  changed/removed → process the frame attributes）。
- **part04 setAttribute src/srcdoc 钩子**：同款门放宽（内容属性路径
  与 IDL 路径同管线）。
- **part05 URL 反射分支排除清单**：增 `FRAME && p === 'src'` 排除——
  FRAME.src 放行至 R3069 尾部 reload 钩子（longDesc 无导航语义留
  URL 分支 verbatim）。

## 验证

- **探针闭环**（临时页，已删）：首轮 src 导航未发生（URL 分支截走）→
  补排除后全链取证 `cd≠null/body≠null/append ok/onload href=
  https://wpt.test/common/blank.html/submit 被拦/pathname=/common/blank.html`。
- **定向 submission-checks：3/3 全 Pass**（3F→3P 精确；首轮 2P +
  test 3 Timeout 中间态，排除清单补刀后全绿）。
- **forms 全树：4365P/335F/12T → 4368P/332F/12T**（ΔP=+3/ΔF=−3
  精确闭合）；文件粒度状态表 diff 仅 submission-checks.window.js
  翻面（3F→3P），其余 411 文件零变化。
- **interactive-elements 483 键零位移**（439P/44F bit-exact）。
- **engine lib 3641P/1F**（1F = ua_display_li 渲染流在册同键）。
- **shim 拼接序 node --check OK**（分片不独立成文，按 bridge
  include_str! 顺序 cat 后检查）；cargo fmt --all --check 过 +
  guarded workspace clippy 零告警。

## 残留与边界

- cannot-navigate 的 fully-active/sandboxed 完整定义仍 FIXME（s144
  挂账承袭）——现 connectedness 近似对在册语料全对齐，过宽面（活
  iframe 文档内提交被拦）无语料需求。
- `/the-frame-element/` 目录为 runner 设计内排除域（navigation M3
  门控，防跨 goal 重复计账）——document-getters-return-null-for-
  cross-origin.html 的 frame 变体不在本 goal 跑集，本次改动的该面
  无基线可验（记账不扩跑）。
- form-data-set-empty-file.window.js 不进跑集：内容判定 skip 规则
  命中源码 `createElement("iframe")` 字样（设计内排除）；三兄弟能进
  跑集恰因上游用词是 `createElement("frame")`——排除规则按内容不按
  案名，两案同为嵌套浏览上下文依赖面，归属判定口径差异在册待议。

## 下一步候选

customizable select 177F 挂起待拍板（GB-20261031 飞书征询在途）；
fix#19c override builtins 1F 挂待决承袭；s186-era FIXME（wrapper
select/textarea/output `.value` 家族）在册。
