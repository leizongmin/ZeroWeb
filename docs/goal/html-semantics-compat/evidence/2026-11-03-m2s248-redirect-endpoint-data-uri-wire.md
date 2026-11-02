# M2 s248 — redirect.py 真 302 端点 + host wire data:/重定向臂 + finalUrl 污染通道（security 全族 24 文件收口 + toDataURL 往返 4 面 bonus，16F 翻绿）

日期：2026-11-03。切片号 s248（承 s247）。根：s247 残余挂账双面——
canvas security `.redirect` 族 11 文件（需 redirect 端点 + final-URL
污染通道）+ security.dataURI（data: 图运行时解码通道）。

## 入场定刀位

s247 收口后 security 族形态 = 12P/12F+1T：`.cross` 11 文件已绿，
`.redirect` 11 文件 + dataURI 挂；solo 残余签名——redirect 族
`drawImage: source image failed to load`、dataURI 像素断言
（toDataURL 豁免面已过，图未绘制）。

## 勘根（代码链逐环定谳）

- **redirect 族三环**：
  ① fixture 形态——`addCrossOriginRedirectYellowImage`（canvas-tests.js）
  src = `HTTP_ORIGIN + "/common/redirect.py?location=" + HTTP_REMOTE_ORIGIN
  + "/images/yellow.png"`——**同源 redirect.py 跳跨源终图**，断言污染
  （toDataURL/getImageData SECURITY_ERR）；
  ② runner `common/redirect.py` 臂现存实现为 **200 伪重定向**（目标体
  直出 + `X-Zero-Final-URL` 头——R360 Document-URL face 遗产）：跨源
  location（`http://www1...`）按文件路径读盘恒 miss → 空体 → 图不解；
  且**无真实 302 跳**——shim fetch 的 HTTP-redirect fetch 管道
  （`_zwFetchRedirectStatus` + Location 跟随）从未触发；
  ③ 消费面封闭——R360 伪重定面的消费方经核为 `js_dom_bridge_tests/
  part24.rs` 单测 **mock**（直填 wire 头，不经本臂）；subset 内
  `common/redirect.py` 其余引用 = css-module 对（走
  `fetch/api/resources/redirect.py` 独立臂）+ a-download（无状态行）——
  就地升级零回归面。
- **污染通道缺口**：s247 的 `_zwCanvasSrcCrossOrigin(srcAttr)` 按属性
  串判源——redirect 族属性 = 同源 redirect.py → 恒判同源不污染（即使
  图加载成功）。spec 口径：origin-clean 按响应 URL origin
  （HTML §4.12.5.1.1）；shim fetch 重定向跟随后 `response.url` = 最终跳
  URL（`finishFetch` `if (hopUrl) response.url = hopUrl` 已在）——缺的
  只是 settle 时把终 URL 记账 + 源判消费。
- **dataURI 单点**：settle 链已通（shim fetch `_zwFetchSchemeDispatch`
  data: 臂 200 → blob → createImageBitmap → resourceState loaded，
  s247 solo 像素签名佐证）；缺口在 **drawImage wire 抓取**——img 臂
  `__zw_get_image_wire(srcAttr)` 直连 host fetch handler：data: 无
  `://` → `wpt_url_path` 全串当路径 → 落盘 miss → 404 空体 → 空wire
  no-op（红底残留，像素断言 0≠255）。
- **bonus 面同根预判**：toDataURL.png/jpeg *colours 3 文件 + the-img-
  element/data-url.html 同为 `toDataURL → new Image(data:) → drawImage
  → 像素比对` 往返形态——同一 wire 缺口，修复应连带翻绿（验证节证
  实）。

## 施工（2 rust + 2 shim）

- **runner（testharness.rs）**：`common/redirect.py` 臂升级真 302
  （Location 绝对地址——相对值按请求原点解析保留；删目标体直出 +
  X-Zero-Final-URL 伪面；上游 wptserve handlers.py 行为等价锚）。
- **webview.rs（`__zw_get_image_wire`）**：① data: 臂——
  `decode_data_uri_bytes`（render-foundation 既有，forgiving-base64 +
  percent-decode）→ `decode_image_bytes` → wire；② 快照 miss 抓取臂
  增 **3xx 跟随循环**（≤4 跳，Location 头 case-insensitive，相对值按
  页 URL 解析——HTTP-redirect fetch 直连面收窄，语料单跳）；编码循环
  抽 `encode_image_wire` 模块级 helper（data: 臂与抓取臂共享）。
- **shim part03.js（settle）**：`__zwImgRunUpdate5277` settle 成功臂把
  `resp.url`（重定向跟随后最终响应 URL）记入
  `_zwAvailableImages[src].finalUrl`——无重定向时 = 请求 URL，同源
  零位移。
- **shim part05.js（源判）**：`_zwCanvasSrcCrossOrigin` 查
  `_zwAvailableImages[src].finalUrl` 优先（有则按终 URL origin 判），
  无记录回落 src 属性——data:/blob: 豁免前置保持。

## 验证

- node --check（18 part exact 顺序）SYNTAX_OK；`cargo fmt` 双 crate +
  guarded clippy（zero-wpt-runner + zero-webview）干净；runner 重建
  35.5s。
- solo security 族：**24P/1T**——11 个 `.redirect` 文件 + dataURI 全部
  F→P（承 s247 的 12P/12F+1T）；残 1T = pattern.fillStyle.sub 文件级
  Timeout（SVG/video 源 settle 媒体面，在册维持）。
- 敏感邻接 solo：image-loading-lazy-crossorigin-change Pass 维持；
  css-module redirect 对 3F 零位移（其根在 module 链非端点，独立
  挂账）。
- 全树 A/B（evidence/logs/s248-ab.log vs s247-ab.log）：见下节。

## 全树 A/B（evidence/logs/s248-ab.log vs s246/s247-ab.log）

- 行级 7731P/484F/81T → **7746P/469F/81T**（ΔP=+15、ΔF=−15、ΔT=0）。
  (key,state) 对（expected 尾截断 + LC_ALL=C sort/comm 同序）
  8293/8293，**LOST 0 / GAINED 0**；翻转 **16 全 F→P + 1 P→F**：
  - 16 F→P = 11 个 `.redirect` 键 + security.dataURI + **4 bonus 往返
    面**（the-img-element/data-url + toDataURL.png.primarycolours/
    toDataURL.png.complexcolours/toDataURL.jpeg.primarycolours——
    `toDataURL → new Image(data:) → drawImage → 像素比对` 往返形，同
    wire 缺口勘根预判逐键兑现）+ pattern.fillStyle.sub cross-origin
    img 子测试键（s247 已翻，本轮 join 对 s247 基线仍计）——零通道
    外溢。
  - 1 P→F = checkable-active-onblur-with-click「Checkboxes should
    clear :active when the user tabs away」——**s238/s239/s245/s246
    在册 checkbox `:active` 交互漂移族**（双向漂移，s246 轮同键
    F→P 漂回；本切通道 = canvas 污染/data: 解码/redirect 端点，与
    checkbox 指针键盘时序无涉），同跑日志 checkable-active 族
    14P/3F 混态自证 flaky，按 s239 先例记基线态销账。
- 对 s246 基线全量 join：28 F→P（22 security + dataURI + 4 bonus +
  sub 键）+ 1 P→F，净 +27——第七个连续完美 A/B（承 s241-s247 线）。

## 残余挂账（本切不动）

- **pattern.fillStyle.sub 文件级 Timeout**：forEachCanvasSource 的
  SVG image/video 源不 settle（load 永挂）——媒体源 settle 面，非
  security 面（security 断言腿已全绿）。
- **css-module redirect 对 3F**：CSS module 动态 import 链
  （module script + CSS 资源加载），redirect 端点已真跳仍挂——独立
  根，scripting-1 车道候选。
- **redirect 跨→同跳形态**：语料无 face（11 文件全同源→跨源）；
  `_zwCanvasSrcCrossOrigin` 终 URL 模型对「跨源起跳收同源」会判不
  污染（fetch spec response tainting 跨跳持久语义未建模）——语料
  出现该形态时再勘根，FIXME 注记在源判处。

## 下一步候选

scripting-1 module 面（css-module redirect 对 3F + muted-errors 等
~124F 域勘根分型）；customizable-select 177F（挂起待用户拍板，
GB-20261031 征询在途）；dialog/img 渲染面归渲染流车道。
