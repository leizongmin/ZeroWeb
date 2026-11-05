# M2 s289 — fetch 核冗余更新守卫（update-the-image-data step 11 pending 同 URL 早退，family/全树逐键零位移第二十三个连续完美）

日期：2026-11-05。切片号 s289（承 s288 下一步候选①——fetch 核冗余更新
守卫，s284 挂账①）。纯 spec 忠实面：守卫面 5+1 行现状绿（s285/s286
定谳），本轮为机制补齐 + un-gate 前置件之一，零红面依赖。

## 勘根

1. **spec 原文勘根**（html.spec.whatwg.org/multipage/images.html
   #update-the-image-data，2026-09-03 版全文下载逐步核对）：算法在
   microtask 段自带两级冗余更新早退——step 11「If the pending request
   is not null and urlString is the same as the pending request's
   current URL, then return.」；step 12「If urlString is the same as
   the current request's current URL and the current request's state
   is partially available: … return.」（含 restart animation 任务）。
   引擎 fetch 臂对同 URL 在飞重入一律重启：双网络请求 + s224 代纪
   bump 把原链 settle 判 stale 丢弃（单 load 双 fetch 形态）。
2. **pending 判据映射**（关键设计约束）：引擎无 progressive 解码态
  （createImageBitmap 全量解码、settle 即 completely available），
   step 12 的 partially available 近似不可达——不建（FIXME 口径注记
   落守卫注释）。step 11 pending = flight 启动时 current request 已
   完全可用：①launch 时态 loaded（crossorigin 变更同 URL 重飞形）；
  ②restart 删除的 loaded 内存——spec restart 只作废请求、不回退
   current request 完全可用态，而 detached defer 重入（_imViaDefer）
   里 prev 已被 run 头删除，需内存表（_zwImgPrevLoaded5277）跨入列传。
3. **反向约束（state-less 首飞同 URL 重入不得早退）**：spec current
   unavailable → step 13 换新请求重启——image-decode-with-quick-
   attach 的 picture 6 断言面 expectReject EncodingError 依赖该重启
   语义（在飞 decode 被新请求取代）。盲「在飞即早退」6 面 P→F，
   probe 控制臂实证守卫按 pending 判据分臂。
4. **s284 五行 + empty-src 真面再定性（un-gate 前置分解）**：插入
   hook 通用点亮回归的根因是 hook 自身缺 spec reacting-to-environment-
   changes 门卫（插入非 relevant mutation；env 算法自带「不用 srcset/
   picture → return」+「selected source 与 pixel density 均同 →
   return」两级早退——image-base-url 的 `<base>` 变更不改变 selected
   source 字符串，env 门卫即拦截）。fetch 核 step-11 守卫只覆盖
  「合法 update run 对 pending flight 的盲重启」子面。**un-gate 需
   双前置：hook 层 env 门卫（待施工）+ fetch 核守卫（本片）**。
5. **s70 张力核销**：relevant-mutations「src set to same value」行
   期望 load 重派（relevant-mutations.html:194-196 expect 'load'），
   与 spec avail-hit 臂「If maybe omit events is not set or
   previousURL is not equal to urlString … fire an event named load」
   一致——引擎 avail 臂无条件重派对该面 spec 忠实，守卫不动 avail 臂。

## 施工（shim 2 文件）

- `crates/engine/src/js_dom_shim/part03.js`：
  - run 头部 restart 删除点记 `_zwImgPrevLoaded5277` 内存表（被删
    请求是否完全可用）。
  - fetch 臂守卫（视口旗标 commit 消费后、rt initiator/代纪 bump/
    在飞 +1 前）：`registry.url === _imSrc && pending → return`——
    不重启 fetch、不 bump 代纪（s224 取代语义仅**新请求**触发，在飞
    decode 链存活）、事件由原链终点照常派发；视口旗标已在 commit
    点消费（s286 一次性语义不变——留旗标会让后续换源 run 经陈旧
    旗标绕过距离门）。
  - 在飞登记值升级 `{url, pending}`（原 boolean）；消费点
   （ViewportRevalidate/sweep/drain/旗标救 skip）均真值判，形态
    升级零位移。
- `crates/engine/src/js_dom_shim/part04.js`：
  - settle 跨案重置点补 `_zwImgPrevLoaded5277` 表重置（跨案键隔离
    ——本表非在飞账目，不参与 s277 在飞存活恢复）。
  - 在飞册同文档存活 restore 点 `= true` → 值透传（{url, pending}
    原样照搬——s277 解析期在飞 chain 存活面，boolean 残留会破 URL
    比对）。

## 验证

- **probe-s289-pending 双臂**（探针页已删，日志 s289-probe-pending
  .log）：pending 同 URL 重入 rtCount=1（step 11 早退生效）Pass；
  state-less 重入 rtCount=2 + 在飞 decode EncodingError（step 13
  replace 语义未误伤）Pass。
- **哨兵 solo**：update-the-image-data 目录 18P + 2F=not-broken
  （layout 移交面基线原样，s286 记录吻合）；decode-quick-attach
  10 行 0F（含 svg variant）；image-base-url / relevant-mutations
  （4 文件）/ empty-src 均 0 Fail。
- **family A/B**（A = s288-imgfamily-c，B = s289-imgfamily-b，
  (key,state) 去重 join；比对脚本 GAINED/LOST 标签互换——GAINED 段
  实为 A-only、LOST 段实为 B-only）：**1031 键逐键全同**（982P/44F
  /5T），CHANGED/GAINED/LOST 全空，REGRESSED=0。
- **全树 A/B**（A = s288-fulltree-b，B = s289-fulltree-b）：**8419
  键逐键全同**（7960P/402F/57T），CHANGED/GAINED/LOST 全空，
  REGRESSED=0 / T-changed=0 第二十三个连续完美。
- **make test**：9109P/5F（78 套件）——5F 恰为 s283 勘根⑤五红测
  原键（p56/p99s/r2809/r2810/r3276，CSSOM cssText 序列化域兄弟流
  在途，rule 9/10 记账）；判定套件 js_dom_bridge 3727P/5F 与 s288
  逐键同红非结果位移，P 总数 +5 为 cargo 套件截断顺序差（日志
  s289-make-test.log）。
- 探针页已删；临时勘根件无残留（变更仅 shim 2 文件，零 Rust）。

## 挂账更新

- **候选①（fetch 核冗余更新守卫）销案**——step 11 收口；step 12
  （partial 同 URL + restart animation）引擎无 progressive 解码态/
  动画机具，FIXME 口径注记（part03 fetch 臂守卫注释，后续域收口时
  承接）。
- **un-gate 前置修正记账**：插入 hook env 门卫（reacting-to-
  environment-changes：srcset/picture-only + 同 selected source/
  density 两级早退）为独立第二前置件——fetch 核守卫不覆盖
  image-base-url 型（selected source 字符串同、URL 解析结果异的
  env 重验）。un-gate 切片须双件齐备方可点亮 hook。
- **口径注记**：①avail 命中臂对同 URL 在飞链不取代（引擎后写胜出
  vs spec abort current+pending——s224 avail 臂不 bump 代纪的 decode
  面调优语义，维持不扩）；②broken 同 URL 重试的在飞重入 pending=
  false（launch 时态 broken → replace，spec 同形）。

## 下一步候选

1. **插入 hook env 门卫**（reacting-to-environment-changes 两级早退
   ——un-gate 第二前置，engine shim 域）。
2. **SVG 纵横比-only 填充**（naturalWidth 18F，rendering 域跨切）。

## 证据

- `evidence/logs/s289-probe-pending.log`（探针双臂）
- `evidence/logs/s289-utid-dir.log`（update-the-image-data 目录）
- `evidence/logs/s289-decode-quick-attach.log`（decode 哨兵）
- `evidence/logs/s289-image-base-url.log` / `s289-relevant-mutations
  .log` / `s289-empty-src.log`（哨兵 solo）
- `evidence/logs/s289-imgfamily-b.log`（family B；A = s288-imgfamily
  -c）
- `evidence/logs/s289-fulltree-b.log`（全树 B；A = s288-fulltree-b）
- `evidence/logs/s289-make-test.log`（全量 make test）
- 日志 md5：probe 24214ada、utid-dir 548105bf、decode-qa fd9380ed、
  base-url b83d05ed、relmut ce7a585a、empty-src 3b29741b、imgfamily
  fb6f6edb、fulltree 027cb3e4、make-test 62d750d4
