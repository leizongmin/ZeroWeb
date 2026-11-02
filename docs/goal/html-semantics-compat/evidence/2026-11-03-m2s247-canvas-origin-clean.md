# M2 s247 — canvas origin-clean 面勘根分型 + data: script 内联 + no-cors img 取回通道 + 污染核心（security .cross 11 文件 F→P 翻绿）

日期：2026-11-03。切片号 s247（承 s246）。根：s246 下一步候选 canvase
security 族（the-canvas-element/security.* 24 文件全 Fail 文件级 + 1
Timeout，s246-ab.log 失败签名 `drawImage: missing image source`）。

## 入场定刀位

s246 下一步候选：canvas origin-clean 族（496F 全域 triage 后选定的
in-lane 单根族——customizable-select 177F 挂起待拍板、dialog/img
渲染面归渲染流车道、scripting-1 时序面脏）。

## 勘根（代码链逐环定谳）

- **失败签名溯源**：`drawImage: missing image source` ←
  `getElementById('yellow.png')` 返 null ← 助手
  `addCrossOriginYellowImage` 未定义 ← **data: URL 外链 script 从不
  执行**——语料 `security*.html` 以
  `<script src="data:text/javascript,addCrossOriginYellowImage()">`
  形态加载助手（HTML §2.6 data: URL 处理器语义 = fetch 即脚本字节），
  runner `inline_local_scripts` 只内联 http(s)/相对/根相对 src，data:
  落空 → 元素无脚本体。
- **动态跨源图加载通道核实**：`wpt_data_fetch_handler` host-agnostic
  （`wpt_url_path` 剥 scheme+host+port），`images/yellow.png` 语料
  资产在册（95B）——静态解析面无缺口；动态 `new Image()` 走
  `__zwImgRunUpdate5277` 运行时 fetch 链。
- **污染语义矩阵（上游 fixture 断言逐个核）**：HTML
  #concept-canvas-origin-clean——污染源 = 跨源 img drawImage、unclean
  canvas drawImage、unclean ImageBitmap、unclean pattern 的
  fillStyle/strokeStyle **set 时污染**（fixture 惯用 set pattern 后
  立即换 'red' 仍断言 throw——use-time 判定不成立）；createPattern
  不污染且 pattern **创建时捕获**源清洁度（pattern.canvas.timing
  create-then-taint-later 不污染面）；width/height/reset() 位图重置
  清洁；toDataURL/toBlob/getImageData 污染画布上抛 SecurityError；
  data:/blob: 源不污染。
- **solo 首轮暴露第二根**：data: script 修复后 24 文件签名翻为
  `drawImage: source image failed to load`——`__zwImgRunUpdate5277`
  运行时 img 走页面 fetch() **默认 cors 模式**，跨源图（www1 host）
  被 CORS check 拒（本地静态臂无 ACAO）→ error 事件 →
  naturalWidth 恒 0。上游语义 img 请求 mode 'no-cors'（HTML
  #updating-the-image-data）——跨源无 ACAO 照常 load，仅在绘制进
  canvas 时污染。

## 施工（4 文件 shim + 1 文件 rust）

- **runner（testharness.rs）**：`inline_local_scripts` 增 data: 臂
  （`decode_data_url_script`——RFC 2397 `data:[mediatype][;base64],`
  解析，plain 形 percent-decode、base64 形剥 ASCII 空白严格解码，
  非 data:/缺逗号 None 顺延）；单位测试 ×2（plain/base64/percent/
  None 解码矩阵 + 内联交付断言）。
- **shim part01.js**：`_zwImgNoCorsChannelDepth` 内部取回通道
  （s153 stylesheet 通道同型——调用窗置位/清位 + fetch() 同步段
  快照进闭包），`_zwFetchApplyFilteredResponse` 增第 7 参
  `_zwImgChannel`，cors check 臂门控跳过（响应保持 basic——体字节
  仅供 shim 内部解码 naturalWidth，响应对象不外泄 JS 无 opaque
  暴露面）。
- **shim part03.js**：`__zwImgRunUpdate5277` fetch 调用窗置位通道
  旗标（img 请求 no-cors 语义；同步抛按取回拒绝语义归零在飞计数
  派 error）；crossorigin 属性在场不置位（cors 请求语义维持常规
  CORS check——lazy-crossorigin-change 面）。
- **shim part05.js（污染核心）**：`_zwCanvasSrcCrossOrigin(src)`
  源判 helper（src 对文档 base 绝对化与文档 origin 比较，data:/blob:
  恒 false）；ctx `_zwUnclean` 状态（`_zwResetCtxMirrors` 5 位图
  重置点归零——DOM set-trap/standalone/OffscreenCanvas ×4/ctx.reset()）；
  fillStyle/strokeStyle setter pattern 臂 set 时污染（
  `_zwPatUnclean`）；drawImage img 臂（src 判跨域）/canvas 臂（源
  `_zwUnclean` 传播）/ImageBitmap 臂（`_zwBitmapUnclean`）三捕获点；
  createPattern img 臂与 canvas 臂创建时捕获（`_zwPatUnclean`）；
  createImageBitmap img/canvas 源清洁度捕获；standalone toDataURL/
  toBlob/getImageData 污染画布 SecurityError。
- **shim part04.js**：DOM canvas 代理 toDataURL/toBlob 同
  SecurityError 臂。

## 验证

- node --check（exact include 顺序 18 part 拼接）SYNTAX_OK；
  `cargo fmt` + guarded clippy（zero-wpt-runner）干净；单位测试
  `r5388` 2 passed；runner 重建三轮（43.9s/47.9s/30.5s）。
- solo security 族（FILTER=the-canvas-element/security）三轮：
  首轮 24F+1T 不动但签名 `missing image source`→`failed to load`
  （暴露第二根）；二轮 **12P/12F+1T**——11 个 `.cross` 文件全部
  F→P（drawImage image/canvas、pattern create/canvas
  fillStyle+strokeStyle/timing/cross、image fillStyle+strokeStyle、
  reset、resize）+ pattern.fillStyle.sub 文件内 cross-origin
  HTMLImageElement 子测试翻绿（文件仍 Timeout——SVG/video 源不
  settle 在册形态）；三轮（crossorigin 修正后复跑）**12P/12F+1T
  精确维持**。
- 首轮全树 A/B 暴露 P→F 1 键（image-loading-lazy-crossorigin-change
  「deferred image uses latest crossorigin」）——通道豁免未区分
  crossorigin 属性形态：属性在场（anonymous）须维持 cors 请求语义
  过 CORS check，无属性才豁免。修正（`__zwImgRunUpdate5277` 读
  `_zw5277Attr(sel, handle, 'crossorigin')`，空才置位通道旗标）后
  solo 复跑该键 Pass 复绿 + security 族零位移。
- 全树 A/B（evidence/logs/s247-ab.log vs s246-ab.log）：见下节。

## 全树 A/B（evidence/logs/s247-ab.log vs s246-ab.log）

- 行级 7719P/496F/81T → **7731P/484F/81T**（ΔP=+12、ΔF=−12、
  ΔT=0）。行级 (key,state) 对（expected 尾截断 + LC_ALL=C
  sort/comm 同序）8293/8293，**LOST 0 / GAINED 0**；状态翻转 **12
  全部 F→P**——11 个 security `.cross` 键 + pattern.fillStyle.sub
  的 cross-origin HTMLImageElement 子测试键，逐键吻合勘根推演，
  **P→F 0 / P→T 0，第六个连续完美 A/B**（承 s241/s242/s243/s244/
  s245/s246 线）。
- 中间轮（crossorigin 修正前）曾记 P→F 1（lazy-crossorigin-change）
  ——修正后终轮归零，过程账见验证节。
- 环境注记：首轮全树跑 customizable-select
  select-home-end-pagedown-pageup-detailed.optional.html 撞 90s
  页脚本执行上限（文件级 Fail 替代 4 子测试 Fail，LOST 4/GAINED 1
  全聚此文件）；solo 复跑（同一二进制）精确复现 s246 形态（4 子测试
  Fail）——环境负载时序归因非代码通道，终轮全树该文件形态同 s246。


## 残余挂账（本切不动）

- **redirect.py 族 11 文件**（security.*.redirect 全部）：需
  redirect 端点 + final-URL 污染通道（fetch handler 建
  /common/redirect.py 等价端点建模 302→最终 URL origin 判定）→
  s248 候选。
- **security.dataURI**：data: 图运行时解码通道（data: src 的
  drawImage 像素面——`__zw_get_image_wire` 对 data: src 现走
  wpt_url_path 落文件 miss）→ s248 一并勘根。
- **pattern.fillStyle.sub 文件级 Timeout**：forEachCanvasSource
  SVG image/video 源不 settle（load 永挂）——媒体源 settle 面，
  非 security 面。
- **pattern.cross.cross 已 Pass 但 redirect 变体在挂账面**。

## 下一步候选

s248 redirect.py 端点 + data: 图解码双面勘根；customizable-select
177F（挂起待用户拍板，GB-20261031 飞书征询在途）；dialog/img 渲染
面归渲染流车道。
