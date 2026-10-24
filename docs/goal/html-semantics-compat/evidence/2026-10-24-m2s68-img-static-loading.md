# M2 切片 68（s68）——F1 img 静态加载管线收口：静态 sweep settle + R56h 生命周期闸门 + window load 等待（R5275）

日期：2026-10-24。s67 八族地图的 **F1 首靶**（原估 ~260F，实测归因后
naturalWidth 228F 大头属 SVG 固有尺寸/密度校正语义——F1b 尾片，raster src
克隆可翻部分本轮收口）。

## 1. 根因

img 资源 settle 只接 `.src` IDL setter 通道（R56h `__zw_fetch` 同步抓取 →
createImageBitmap → `_resourceStates`），parser 静态 `<img src>` 无
fetch/settle 通路：naturalWidth 恒 0、load/error 恒不派。另有三个叠加缺口：

- **template 内容漏抓**：zero_dom `<template>` 内容解析进独立 inert
  DocumentFragment（不在文档树），`extract_img_srcs` 的
  get_elements_by_tag_name 不达 → naturalWidth-naturalHeight.html 的 raster
  克隆臂（SVG 内嵌 template 里的 `<img src=cat.jpg>`）宿主快照恒空。
- **broken 终态缺失**：error 事件派发不写 `_resourceStates` →
  img.complete 对 broken 图恒 false（spec：当前请求 failed → complete true）。
- **运行期事件时序**：R56h 在页面脚本阶段即时派 load/error——spec 语义是
  DCL 不等图片、图片 load 事件延迟 window load（delay-load-event 两案的
  断言序）；`__zwReadyState` 未注入时 readyState getter 缺省 'complete'
  （契约沙箱零回归的关键，闸门只在 runner 注入 'loading' 时生效）。

## 2. 修复面（R5275，四刀 + 生命周期）

1. **part04 `__zwFireStaticImgSettle`**（window load 派发钩子调用，part06
   dispatchEvent('load') 分支，R139/R5254 同款扫描位）：sweep
   querySelectorAll('img')——已 settle/排队中/loading=lazy/无空 src 跳过；
   `__zw_get_image_size` host 快照命中 → loaded + load 事件 + 写
   `_zwAvailableImages`；未命中 → broken + error。随后 drain
   `_zwImgPendingEvents`（error 派发前补写 broken）。同文件
   `__zwImgLoadBlocked`：在飞计数/队列非空 → window load 等待判定。
2. **part04 getter 面**：`_zwImgNaturalDim` helper（loaded → 宽高；lazy 未
   settle → 0——host 快照回落不得绕过 lazy 门）；IMG width/height 属性缺失
   回落自然尺寸（rendered ≈ intrinsic headless 近似，#dom-img-width）；
   currentSrc IMG 绝对 URL（页锚解析，#dom-img-currentsrc）；complete getter
   改读 latest-wins 视图（R3204——`__zw_get_attr` 是解析期陈旧快照）。
3. **part05 R56h src 臂重写**：restart 语义（src 变更作废旧 current
   request——先读 `_imPrev`→算 `_imLoadedSame`→条件 delete，顺序关键）；同
   URL 已 loaded 幂等不重派；available-images 命中同步 settle；
   `_zwImgInflight` 在飞计数（fetch 臂 +1，终点 `_imInflightEnd` 消费旗标
   -1——available-images/空 src 臂不减，InvalidState 守卫失败的无派发终点
   也消费）；`__zwReadyState === 'loading'` 时事件入队 `_zwImgPendingEvents`
   （CSP 臂保持 `_defer` 直派不进队列）；error 派发写 broken 终态。
4. **extract.rs**：`extract_img_srcs` 递归收集 template contents fragment 内
   的 `<img>`（复用 effective_img_src）。
5. **testharness.rs 生命周期拆三任务链**（原单 timer 任务 interactive+DCL+
   complete+load+pageshow+pagereveal）：①interactive+DCL（spec：DCL 不等
   图片）；②img settle 独立任务——img load 派发后的 promise 回调
   （invisible-image `Promise.all` 的 have_images_loaded）是微任务，须在任务
   边界排水，同任务直派则 window load listener 同步断言先于微任务；③load
   等待任务——先 settle 再查 `__zwImgLoadBlocked`，图片在飞（wptserve
   trickle(dN) 慢图 shim 侧延迟，delay-load-event-detached）→ 100ms 重试，
   就绪 → complete+load+pageshow+pagereveal（严格序不变，readyState
   complete 相位随 load 后移——spec the-end complete 即 fully loaded）。
6. **available-images 臂事件 `_defer` 化**（域跑发现的伪影——probe14 实证
   `i2-set;comp=true` 后 i2-load 永不触发）：available-images 命中时状态
   同步 settle（naturalWidth/complete 同步可读——already-loaded-image-sync-
   width 面），load 事件 `_defer`（microtask）派发 + 在飞计数——spec：
   relevant mutation 的 image loading task 异步派 load，setter 同步任务内
   src 后注册的 onload 不得丢（available-images-onload 的 i2 链
   `i2.src="3.jpg"` 后才挂 onload）；等待循环使 window load 排在事件之后
   （probe14 终序 `i2-load → WINLOAD`，spec the-end 序）。

契约 part85 两测试：`img_static_sweep_settle_r5275`（快照命中 loaded 面/
lazy 跳过/无空 src complete/broken 面，经 dispatchEvent(load) 触发钩子）+
`img_runtime_deferred_settle_r5275`（loading 闸门入队 + drain 补写 +
available-images 同步 settle + currentSrc 绝对 URL + 同 URL 幂等）。

## 验证

- embedded-content 域 **461P/816F/40T → 549P/738F/35T**（GAIN 88 = 78
  F→P + 5 T→P + 5 新注册 subtest；三数守恒无 P→F 回归空间；长尾为 sweep
  对 raster src img 的跨文件 settle 收益，tracked 明细如下）
- 翻面明细：naturalWidth-naturalHeight **30P/228F → 48P/210F**（+18，raster
  src 克隆臂全过，210F 归因 SVG 固有尺寸/密度校正= F1b）；delay-load-event
  与 -detached **7 案全绿**（detached 的 trickle(d2) 经在飞等待收口）；
  invisible-image F→P（微任务任务边界排水）；already-loaded-image-sync-width
  F→P；available-images-onload T→P（defer 刀收口）；img.complete 17P/0F
  持平（srcset 两案 pending 属 F2 依赖，baseline 同 bug）
- 回归：interactive-elements **439P/44F/0T 逐位持平**（defer 刀后终态复
  跑）；zero-engine lib **3474P/0F**（+2 契约 part85；余为并行流新增）
- 契约翻转力实证：撤 part06 settle 调用 → part85 两测试双红；撤 sweep 体
  （早退）→ 双红；各自恢复后字节一致 + 复跑全绿
- 拼接 shim node --check 通过；fmt clean

## 未解决项

- **F1b naturalWidth SVG 尾片**（210F 余量）：SVG 固有尺寸/密度校正语义
  （intrinsic dimensions 算法）——raster src 已由 sweep 覆盖
- img.complete srcset 两案（F2 currentSrc 完整算法依赖）
- data-url 像素断言（canvas drawImage 面，F6 记账）；404-response-with-body
  （fixture handler 限制挂账）；img-src-in-synthetic-document src getter
  about:blank 回落 1F；Image-constructor prototype 身份 2F
- F3 img.decode() 56F → F2 select-an-image-source/currentSrc/srcset ~300F
  → F6 canvas 53F → F5 lazy ~49F（控制面挂账序）
- 生命周期等待循环仅覆盖 img 在飞（`_zwImgInflight`）——iframe/媒体资源
  不阻塞 window load（corpus 现无观测面，spec the-end 全资源等待挂账）
