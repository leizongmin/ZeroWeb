# M2 切片 67（s67）——embedded-content 开片分诊轮：域 458P/819F/40T 八族地图 + object/embed null 反射窄面收口（R5274）

日期：2026-10-24。embedded-content 开片（M1 基线 424P/595F/41T 40.0%），
分诊轮先行。本轮域刷新 **458P/819F/40T**（1317 subtest，执行案 231 与 M1
一致；子测试总数较 M1 +257 为 R5235 帧驱动 rAF 后此前 Timeout 悬挂案完成
注册，非语料增长），F7 窄面 null 反射收口 **461P/816F/40T**（GAIN 3 /
REG 0）。

## 1. 八族地图（819F 分诊归因）

- **F1 img 静态加载管线 ~260F**：naturalWidth-naturalHeight-width-height
  228 + img.complete 5 + update-the-image-data 邻案（fail-to-resolve 4 /
  not-broken 2 / current-request-microtask 2 / update-src-complete 1）+
  empty-src 2 + data-url 1 + 404-response 1 + available-images-onload 1 +
  delay-load-event 2 + invisible-image 1 + img-src-in-synthetic-document 1 +
  adoption 1 + Image-constructor 2。根因：img 资源 settle 只接 `.src` IDL
  setter 通道（R56h fetch→createImageBitmap→_resourceStates），parser 静态
  `<img src>` 无 fetch/settle——naturalWidth 恒 0（探针钉定
  `naturalWidth expected 320 but got 0`）、load/error 事件恒不派、
  `__zw_get_image_size` host 桥仅 webview 面注册（runner 无此回调）。修法
  方向：DCL 扫描 settle（R5254 两段式先例 + R56h 既有链复用）+
  loading=lazy 跳过 + setAttribute('src') 通道接入。**下一片首靶**。
- **F2 select-an-image-source / currentSrc / srcset / sizes ~300F**：
  update-the-source-set 84 + sizes-auto 74 + parse-a-srcset-attribute 49 +
  relevant-mutations 46 + current-pixel-density 21 + select-an-image-source
  13 + svg-img-density-vs-zoom 8 + img-picture-ancestor 4 + source-relevant-
  mutations 1 + srcset-media-dynamic 1 + sizes-auto-dynamic-attributes 1 +
  sizes-auto-resize-observer 1 + implicit-sizes-ignores-width 1。根因：
  currentSrc IDL 缺失（恒 ''，全族信号 `ref currentSrc was ""`）+ 整条
  select-an-image-source 算法（srcset 解析/sizes 解析/density-width 选择/
  picture-source 语义/relevant mutations）未实现。最大单族，独立大切片
  （F1 后）。
- **F3 img.decode() 56F**：decode/* 七文件。`decode is not a function`
  （M1 信号延续）——promise 面依赖 F1 的 settle 状态机（loaded→resolve /
  error→EncodingError reject / 挂起等待）。
- **F4 image-map 命中测试 84F**：area 三文件 63 + usemap-casing（img 8 +
  object 8）+ image-maps 5。全族信号 `elementFromPoint(3,3) expected
  <area> but got null`（area-coords/shape/processing + usemap 大小写匹配 +
  hash-name-reference 动态匹配经 testdriver click）——几何/hit-test 底座
  缺口，**渲染流记账回流**（s49 bounds-clicking / s57 inert 族
  elementFromPoint 同桶）。
- **F5 lazy-loading 语义 ~49F**：image-loading-lazy 族 12 + src-then-lazy-
  load 4 + lazy-out-of-band 2 + relevant-mutations-lazy 1 +
  image-loading-lazy-use-list 1 + disconnected-image-loading-lazy 1 +
  image-loading-eager 1 + video 15 + audio 12。loading=lazy 可见性延迟——
  runner 无视口几何，「不加载」半边因无管线碰巧绿、「加载」半边红；依赖
  F1 管线落地后重分诊。video/audio 属媒体域边界（media goal 已归档，本
  goal 只做元素属性面——loading 属性反射在、加载行为面挂账）。
- **F6 canvas 53F**：getContext options 字典转换顺序（host 桥面
  `["alpha","colorSpace",...]` 操作序可观测）+ descendants-focusability
  （canvas 后备内容 Tab 序——s48 editing host 同族 runner 插焦面）+ 余案
  待逐案分诊。
- **F7 object/embed 窄面 25F**：null 反射 3F（**本轮收口**）+ 计算样式
  4F（embed/object width/height gCS "100px" vs "auto"——渲染面挂账）+
  资源加载 fallback 7F（embed-in-object-fallback 加载计数——加载管线族）
  + usemap-casing 8F（F4）+ constraint-validation 1F + svg 尺寸 1F
  （渲染面）。
- **F8 杂项底座 ~2F**：historical-progress-event.window /
  image-alternate-no-store.window 等 .window 变体通道面。

## 2. 修复面（R5274，一刀）

part04 get-trap IFRAME 分支后新增 OBJECT/EMBED 分支：contentDocument/
contentWindow 恒 null（OBJECT；spec #dom-object-contentdocument /
#dom-object-contentwindow「does not represent a nested browsing context →
null」尾步——runner 不建 object/embed 浏览上下文，null 恒正确）+
getSVGDocument 返 null 函数（OBJECT+EMBED；GetSVGDocumentSupport 非 SVG
document 臂，#dom-getsvgdocument）。EMBED 无 contentDocument/contentWindow
成员（fallthrough 保持 undefined）。契约 part84 两测试（钉①五臂 + 钉②
IFRAME 活浏览上下文面不回归——R130 无 src iframe contentDocument 空
Document 非 null）。

## 验证

- embedded-content 域 **458P/819F/40T → 461P/816F/40T**（GAIN 3 = null 面
  三案：object contentDocument / object getSVGDocument / embed
  getSVGDocument；object-attributes t1/t2 因 obj1 非 null 断言在前不翻面，
  null 化零回归）
- 回归：interactive-elements **439P/44F/0T** 逐位持平；zero-engine lib
  **3469P/0F**（+2 契约 part84；余 +4 为并行流新增）
- 契约翻转力实证：撤 part04 分支（git stash 二分同法）→
  object_embed_null_browsing_context_r5274 红；恢复后字节一致
  （sha256 核对）
- 拼接 shim node --check 通过

## 未解决项

- F1–F6/F8 按上地图挂账；下一片 **F1 img 静态加载管线**（~260F 最大可
  收口单族，共享路径修）
- F4 渲染流记账回流（几何/hit-test 底座，与 s49/s57 同桶）
- F5 video/audio 半边媒体域边界，F1 后重分诊
- object-attributes t1（obj1 HTML 型 contentWindow 须 Window 对象）/
  embed-dimension / embed-svg-navigation-resets-size 等渲染面挂账
