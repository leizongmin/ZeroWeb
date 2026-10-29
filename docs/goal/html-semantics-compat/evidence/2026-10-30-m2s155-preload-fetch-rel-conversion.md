# M2 s155 · rel=preload fetch 面 + rel→stylesheet 转换 preload cache 复用收口
## （link-multiple-load-events 文件级 Timeout 键消——6/6 全 Pass）

日期：2026-10-30 · 切片：html-semantics M2 · 前序：s154（adopt/innerHTML 身份
双表示 stylesheet 触发收口）

## 入场定刀位

s154 下一步候选①：rel=preload fetch 面（link-multiple-load-events.html——
基线形态案 1-4 Pass + 文件级 Timeout 键，案 5（rel=preload→stylesheet 转换）
与案 6（逐 href load）未打印）。

## 勘根（先解 Timeout 形态再定刀）

1. **页级 Timeout 机制定谳**：定向诊断行「testharness page timeout completed
   the file with 2 test(s) unreported (state={loaded:true,pending:2,phase:4,
   tests:6})」——语料无 meta timeout → testharness 全局默认超时收文件
   （s149 早收形同路径）。案 1-4「幂等属性写不重请求」的正确语义是第二个
   race 等**满 2s** 超时窗（load 不再 fire = 正确）→ 4×2s=8s 吃掉全局预算，
   案 5/6 到点前被强收——s153 账本「案 6 页级 10s 时序必然」注记的机制来源。
   修复路径必须让案 5/6 毫秒级完成（8s+ε<10s 全报），纯计数面不足。
2. **案 5 两段语义**：①rel=preload as=style 设 href → 发起网络取回 +
   onload（runner stylesheet.py 端点 take+put(count+1) → count "1"）；
   ②rel 改 stylesheet → 转换后按 stylesheet 处理模型从 **preload cache**
   采用资源（零网络）但 **load 事件再 fire**（count 保持——末段断言 "0"）。
   上游明示注释「loads from the preload cache and never touches the network,
   but the load event does fire again」。
3. **触发面缺口**：href 变更有 part04 setAttribute 臂 + part05 IDL 臂（s150），
   **rel 变更无任何臂**；helper rel 门只认 'stylesheet'（preload bail）——
   ①②两段均无通道。
4. **`as` IDL 反射已有**（part05 reflected 列表 p==='as'）——`link.as='style'`
   落属性，helper `_ssReadAttr('as')` 可读，无需新反射。
5. **位移面收敛**：html/semantics 语料 rel=preload 唯一使用点 = 靶面案 5
   （modulepreload 是 s124 已收面、video/audio preload 是 media 属性面）——
   helper 分流新增对全域的波及面仅此一形。

## 施工（3 文件净 +60/−3）

1. **part03 helper rel 门分流 + preload 取回臂 + 复用臂**：rel 分词双旗标
   （stylesheet/preload 不互斥扫描，双无 bail）；preload 分支（`!sheet &&
   preload`）= as 非空门（missing/invalid as 上游不发起——FIXME 分型不建模）
   + 同身份印记去重（`_zwSsFiredMap` 复用——重入同 URL 静默返）+ 页面
   fetch（plain，无 s153 通道旗标——preload 是 cors 模式请求，同源语料过
   check；404 等非网络失败 fetch resolve → load，error 仅网络失败）→
   完成播种 `_zwPreloadCache[key]=abs` + 派 load / 拒绝派 error。sheet 分支
   印记门前插复用臂：`_zwPreloadCache[key] === abs` → setTimeout(0) 派 load
   零网络返（转换面语义）。
2. **part04 setAttribute rel 臂**：LINK TAG 门内甄别，helper(sel,handle,null)
   ——prevHref=null 语义跳过幂等门走 abs 比较；幂等 rel 写由 sheet 分支印记
   静默拦截（case 1 idempotent rel 写不重取不重派——join 实证零位移）。
3. **part05 IDL rel 臂**：同款（link.rel= 走 R3069 reflected 臂不经
   setAttribute——案 5 转换面写入即此通道）。

## 过程勘根（复用臂位置取舍）

复用臂若置于身份印记命中点之后会自伤：s154 取回印记命中本是静默返（append
直取后的重查再入路径）——若改为派 load，插入臂已 fetch + 重查派 load =
双 load，case 1-4 的「second load unexpectedly fired」断言全翻红。故复用臂
**先于**印记门、且只认 `_zwPreloadCache`（preload 取回完成才播种，sheet 取
回不播种）——case 1 重查路径 preload cache miss → 印记静默返不变；案 5 转
换路径 cache 命中 → 派 load 返。

## 验证

- **语法**：规范序拼接 node --check SYNTAX_OK 110401 行（s154 110325，净
  +76）。Rust 面零改动（clippy/fmt 零波及，未跑）。
- **靶面定向**：link-multiple-load-events **6/6 全 Pass**、MAKE_EXIT=0、
  文件级 Timeout 键消。
- **定向 the-link-element 全目录案级 join（对 s154 全量基线）**：LOST 1
  （靶面文件级 Timeout 键）+ GAINED 2（案 5/6 Pass）、其余 113 键零位移。
- **全量案级 join（对 s154 /tmp/zw-s154-full.log，(key,state) 对集直接
  去重）**：s154 8084 行/8081 对（7196P/797F/88T）→ s155 8085 行/8082 对
  （7198P/797F/87T）——**F2P 0（本轮修复面以 Timeout 键消形态计：LOST 1 +
  GAINED 2 全落靶面）、P2F 0、other 位移 0**；全域零位移（fetch API/XHR 面、
  s152 静态扫描面、s153 CORS 面、s154 修复面全保持）。口径注记：s154 账本
  「8080 对/7195P」系上轮 join 脚本差一（直计 8081 对/7196P，多为一同态重
  复行的计入差异——pair 集差结论不受影响），本轮起统一直接去重计数。
- 探针零语料（本轮勘根全部走 runner 诊断行 + 语料/日志直读，无临时页面）。

## 挂账（本切片更新）

- **收口：rel=preload fetch 面 + rel→stylesheet 转换复用面**（link-multiple
  文件级 Timeout 键销案；「案 5 + 案 6 页级 10s 时序必然」注记销案——时序
  形态已解，6/6 全报）。
- **FIXME 新记**：preload 资源级共享缓存（跨元素同 URL 复用）与 as
  destination 分型不建模（语料无形态）；rel 双类型（stylesheet+preload 并
  存）sheet 分流优先（语料无形态）。
- **承袭不变**：LinkStyle sheet 属性面 1F；stylesheet-media-change-no-fetch
  1F；meta long × corpus 通道放宽失效；details 族 lib flake；promise_test
  串行启动面；innerHTML 身份分裂表示面（查询/树视图统一）；sel 重解析同槽
  同 href 印记失配边角；待决三项（css/text 语料资源、green-2x2.png、parser
  form pointer 缓行）须用户拍板。

## 下一步（s156 候选）

①LinkStyle sheet 属性面 1F（the-link-element 在册）；②link-load-event
document load 阻塞面（Check if the stylesheet's load event blocks the
document load event 1F）；③stylesheet-media-change-no-fetch 1F（media 变更
门——helper rel 门同型扩展面）。
