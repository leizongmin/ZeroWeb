# M2 s151 · stylesheet MIME/nosniff 判定矩阵——css.py 端点 + @import 内层链 + compatMode 真值化

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s150（动态 link rel=stylesheet
事件面）

## 入场定刀位

s150 下一步候选①：Face B MIME/nosniff 判定矩阵（link-style-error 系 12F 定向
假象——勘根证实真值 36F：s150 后三文件 74 行中 42P/36F，`<link>` 形 12F 全部
期望 load 案（SO 404 error / CO mixed error 假阳反向）、`@import` 形 24F 全部
期望 error 案（内层缺失外层 data: 恒 load）。

## 勘根（上游源码 + 探针实证定谳）

1. **上游 css.py 真值**（pin master 逐字）：`content_type` 参数原样作
   Content-Type 头（**缺席 → 无 CT 头**）、`nosniff` 旗标 →
   `x-content-type-options: nosniff`、`cors` 旗标 → `ACAO: *`（语料 URL 未携带）、
   体恒 `body { background:red }`、status 200。runner fetch handler 响应
   headers 通道已有（s138/s150 端点先例；`wpt_add_fetch_metadata` 不加 CT，
   no-CT 案可保真）。
2. **shim Response headers 通道现成**：`__zw_fetch` wire 含 headersWire，
   `_parseHeadersWire`/`_takeHeaderValue`（part01）+ Content-Type 大小写不敏感
   ——MIME/nosniff 判定无需桥扩展。
3. **期望公式定谳**（语料判式 `nosniff || (wrong-CT && (CSS1Compat ||
   cross-origin))` → error 的 spec 映射，css-syntax §stylesheet-content-type-error）：
   no-CT+非nosniff→load；wrong-CT(text/plain)+no-nosniff+quirks+同源→load
   （legacy 宽容），其余→error；broken-CT(oops 不可解析)+非nosniff→load。
   **limited-quirks 文件 compatMode 也是 CSS1Compat**（期望与 01 同形），唯
   quirks 文件（BackCompat）激活宽容分支。
4. **runner 通道 compatMode 缺位实证**（探针 zz-s151-probe-quirks）：quirks
   doctype 页 `document.compatMode` 实得 CSS1Compat（shim part06 常量遮蔽，
   part06:4896 既有注记「headless 恒 CSS1Compat」）——quirks 宽容分支从未激活，
   三文件判式实际全按 01 语义跑。宿主侧真值链已备：load_html → dom_bindings
   `DOM_SOURCE`（zero_dom parse_html，parser quirks 判定 set_quirks_mode 在位）。
5. **页锚 scheme 撕裂**：上游语料假设 http 页（wptserve 无 mixed-content 阻断、
   HTTP_REMOTE_ORIGIN `http://www1.wpt.test:8000` 相对判定）。runner 页锚
   https 时 CO 案被 s150 helper mixed 门拦死 error（https 页 http 子资源），期望
   load 的 CO 案到不了判定面。runner 页锚 http 特例有 net-api/webcrypto/referrer
   谓词先例（`*_case_uses_http_page`）。
6. **fetch shim CORS 两截面**（part01:3955–4014）：默认 cors 模式跨源无 ACAO →
   reject「Failed to fetch」；ACAO 通配放行后 cors 过滤响应剥非 safelisted 头
   （`x-content-type-options` 读不到）——端点须 ACAO + ACEH 双头（`*` 通配在
   非 include 分支生效，3988–4001 实码确认）。

## 施工（shim 2 文件 + 宿主桥 2 文件 + runner 端点，净 +155/−8）

1. **宿主桥 `__zw_get_compat_mode`**（callbacks.rs）：经 live Document
   `quirks_mode()` 求值（Quirks → "BackCompat"、其余 → "CSS1Compat"）；映射
   语义封装 `dom_bindings::document_compat_mode_string()`（document.rs 新增，
   mod.rs `pub(crate) use` 导出——gc/`document` 子模块私有，桥不可直入）。
2. **shim part06 compatMode getter**：常量 'CSS1Compat' → getter（桥读、
   值白名单 BackCompat/CSS1Compat、桥缺席回落 CSS1Compat）。scrollingElement
   不读该属性（恒 documentElement），本刀不动滚动面。
3. **shim part03 判定 helper ×2**：`_zwStylesheetJudge`（nosniff 严检 /
   wrong-CT standards-strict quirks-宽容 / broken-CT 无头放行——同步全判定，
   fetch then 链内直调；origin 判定 `new URL().origin` 比较页锚）+
   `_zwStylesheetImports`（`url('..')`/`".."`/`'..'` 三书写形，去重保序，
   绝对化失败项跳过）。
4. **shim part03 fetch 回调改造**（s150 helper）：ok → 判定 → 通过后
   `response.text()` 展开 @import 内层 URL 逐个 fetch + 同判定链，任一 error →
   外层 error、全部 load → load（一层递归——内层响应体不再展开，语料形态；
   pending 计数聚合）。s150「@import 内层解析不在本刀面」注记同步更新。
5. **runner css.py 端点**（testharness.rs，stylesheet.py 臂后）：content_type /
   nosniff / cors 参数 + 体 `body { background:red }`；无条件补 ACAO/ACEH `*`
   （上游 wptserve stylesheet 加载不走 CORS 检查，两头是通道妥协——语料不读
   ACAO/ACEH，期望面不变）。
6. **页锚谓词 `link_style_error_case_uses_http_page`**（三文件名尾缀匹配，
   referrer 先例同构）接入 page_scheme 判定。

## 验证

- 探针：quirks doctype compatMode = BackCompat **Pass**（真值化链路通）。
- **定向 link-style-error 三文件：74P/0F/0T**（行集守恒 26+24+24；基线
  42P/36F）——36F 全转绿：`<link>` 形期望 load 案（no-CT/broken no-nosniff
  SO/CO）判定放行 + 期望 error 案从 404/mixed 假阳变真判定；`@import` 形期望
  error 案内层判 error 传播、期望 load 案保持。quirks 文件宽容分支激活后
  wrong-CT SO 案 load ✓（三文件判式不再同判）。
- **定向 document-metadata 全目录 + link-multiple 复验**：link-multiple-load-
  events 4P + 文件级 Timeout（案 6 preload 页级 10s 必然面——s150 在册挂账
  不变）；media 触碰案稳定 P（案 1–4 与 s150 同形）。
- **全量 join（案级全键，对 s150 合并树基线 /tmp/zw-s150-rebase-full.log）**：
  s151 **805P/396F/88T = 1289 文件粒度键**（s150rb 805P/399F/89T = 1293）；
  案级全键 lost 44 / gained 70，归因逐条闭合：
  - **真修复 41 案 F→P**：link-style-error 36 + link-load-error-events 5
    （non-CSS 三案 = 内层 404 error 传播；http Import 两案 = 跨源 CORS reject
    error 传播——`success: !isHttps` 页 https 时期望 error，s150 外层恒 load
    恒 F）。
  - **假阳消除 2 案 P→F**：https Import 两案——s150 假阳 Pass（内层未取、外层
    data: 恒 load 凑巧命中期望）；本刀内层真取 `https://wpt.test:8444`（跨端口
    = 跨源）撞 fetch CORS check（通用静态臂无 ACAO）→ reject ≠ 期望 load。
    挂账（见下）。
  - **flake 形态（非本刀位移）**：select-event 26P 键增 + 文件级 Timeout 键消
    （在册全量负载 flake 复绿形，s148「负偏差恢复 39」/s150 合并树复验
    270/270 先例同族）；link-multiple media 案 1P（s150rb 全量早收形态少报，
    定向复验案 1–4 稳定 P）。
  - **文件级键**：-3F（link-style-error 三文件）-1T（select-event）。
- **门禁：clippy workspace --all-targets -D warnings 零告警；runner bin
  265/0F；fmt 过**（本轮 .rs 干净）；shim 规范序拼接 node --check
  SYNTAX_OK 110138 行。

## 挂账（本切片更新）

- **新记：跨源静态资源 ACAO 面**——httpsSheet 内层链（`https://wpt.test:8444`
  跨端口）被 fetch CORS check 拒（通用静态臂无 ACAO）。修法候选：通用静态臂
  加 ACAO/ACEH（影响全语料静态资源语义，需影响面盘点）或 helper 通道对
  stylesheet 加载改 no-cors 语义仿真（opaque 头剥更严，不可行）。2 案（link-
  load-error-events https Import ×2）假阳消除后挂账，上游期望 load。
- **承袭不变**：Face C script/render-blocking 时序面（conditionally-block 2F、
  link-error-fired-before-scripting-unblocked 等）；adopt/innerHTML 身份双
  表示深水面（document-without-browsing-context 2F）；LinkStyle sheet 属性面
  1F；link-load-error-events 余 https Load 案 1F（同跨源 ACAO 面，非 import 形
  一直 Fail）；rel=preload fetch 面（link-multiple 案 5 + 案 6 页级 10s 时序
  必然）；stylesheet-media-change-no-fetch 1F；@import 多层递归链（本刀一层，
  语料无多层形态）；meta long × corpus 通道放宽失效；details 族 lib flake；
  promise_test 串行启动面；待决三项（css/text 语料资源、green-2x2.png、
  parser form pointer 缓行）。
- **观察归档**：link-multiple media 案全量位移属 s150rb 早收形态少报，定向
  稳定 P——非修复非回归。

## 下一步（s152 候选）

①Face C render-blocking 时序面勘根（conditionally-block 2F /
link-error-fired-before-scripting-unblocked——link load 时机 vs script 阻断
语义）；②跨源静态资源 ACAO 面影响面盘点（通用静态臂加 ACAO 的全语料波及
评估——2 案 + https Load 案 3F 修复面）；③adopt/innerHTML 身份双表示深水面
（document-without-browsing-context 2F）；④rel=preload fetch 面（link-multiple
案 5 + 缓存复用）。
