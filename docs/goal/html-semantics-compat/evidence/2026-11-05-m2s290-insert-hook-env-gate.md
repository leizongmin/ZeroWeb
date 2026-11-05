# M2 s290 — 插入 hook env 门卫（reacting-to-environment-changes 三级早退，un-gate 双前置件齐备；family 逐键零位移、全树两键 forms 域离群经撤片实验定谳非本片位移）

日期：2026-11-05。切片号 s290（承 s289 下一步候选①——插入 hook env 门
卫，s289 账本 un-gate 第二前置件）。纯 spec 忠实面：allowsAuto 臂内
守卫，零红面依赖。

## 勘根

1. **spec 基座反转澄清**（images.html #relevant-mutations，2026-11-05
   版全文下载）：现行 spec 明文「The img or source HTML element
   insertion steps … count the mutation as a relevant mutation」——
   插入**是** relevant mutation（s284 注释引用无误）；image-base-url
   .html 自带注释「insertion is not a relevant mutation」是旧 spec
   语义、corpus 按 pinned 快照冻结。同节新增「If the element allows
   auto-sizes: the element starts being rendered … set the maybe omit
   events flag」——allows-auto-sizes 图 starts-being-rendered 亦
   relevant mutation（s284 臂的 spec 基座实为该条）。两条机制并存：
   insertion→update-the-image-data（文本层）与 env 算法「encouraged
   … when an img element is inserted」（可选层）；引擎可观测契约 =
   pinned corpus（Chromium 实测插入已加载 plain-src 图不重取），
   env 算法 guard 形状恰好给出该语义——s290 语义基座 = env guard
   形状 + corpus 可观测面。
2. **guard ④比较域定谳**：spec update-the-image-data 对非
   srcset/picture 图就地写 last selected source（microtask 选源半）；
   srcset 路径仅 env finishUpdate 写该对。引擎 s223 定谳选源产物
   就地绝对化（avail 键/请求 URL/currentSrc 同源）——比较域用引擎
   resolved URL（pk.url === state.url），image-base-url 保护不依赖
   ④（该面 plain src，①即拦截）。
3. **last-selected 引擎基线 = settle 态**（_resourceStates
   url+density，loaded 才算）：broken 态 spec 不写该对；deferred 冻结
   请求（_zwImgDeferredReq）与在飞册不写——lazy 重插唤醒面不被④
   拦截，sizes-auto 插入点亮语义保持。
4. **活面零位移勘根**：script 插入 allowsAuto 语料两处
   （sizes-auto-dynamic-attributes 的 makeAutoImg 先设属性再 append
   ——插入时点 srcset 已非空；relevant-mutations-lazy 全静态解析图
   不走 hook）——①②④对现语料全部惰性，活内容全是 un-gate 前置件。
5. **探针时序定谳**（三轮红换来的机制账）：script 节点插入后未过
   布局（gBCR 零 rect）→ lazy 门走 defer（登记冻结+视口旗标，fetch
   由 observers_tick/sweep 消费链异步启动）——①属性 update 半同步
   派发（__zwImgAttrMutated5277 直调 Revalidate，仅 s220 记录半微任
   务化）但零 rect 下不同步 fetch；②裸 append allowsAuto img 落穿
   dormant 旗标会把 sweep 唤醒冻结请求变成第二 fetch 源；③旗标链
   与冻结链先后唤醒经 s289 重启语义收成单 load 双请求（既有引擎行
   为，非守卫失效）。探针最终形：等 30ms 让消费链真启动 fetch 再打
   在飞窗，旗标计数替代 waitLoad。

## 施工（shim 1 文件）

- `crates/engine/src/js_dom_shim/part03.js`：allowsAuto 插入臂内
  （SelectionQueue+Revalidate 之前）加 env 门卫三级：
  - ⓵ srcset 属性空 → 早退落穿 s285 lazy dormant 臂（本臂父已非
    picture，判据即「does not use srcset or picture」；plain-src 面
    image-base-url 在此级拦截）。
  - ⓶ 在飞册 truthy（s289 {url,pending} 形态；avail 臂/defer 两清除
    点保证非空窗=真在飞）→ return（spec env 不取代在飞请求）。
  - ⓮ 选中源 null → return；⓯ settle 态 loaded 且 pk.url===state.url
    且 pk.density===state.density → return（不重验、不重派 load、
    不入队记录）。选源两次算（此处+Revalidate 内）仅同态重插罕见面
    付出（guidelines #26 计成本域：每次插入 +1 attr 读 + 命中态才
    +1 次全选源，事件驱动路径非热循环）。
  - fully active（引擎单活跃文档模型）/multipart（引擎无该资源类
    型）两级不适用，FIXME 注记；s284 臂注释同步补 s289/s290 收口
    交叉引用。

## 验证

- **probe-s290-env-gate 四臂**（探针已删；日志 s290-probe-env-gate
  .log）：A loaded 同态重插不重验不重派 load（⓯）Pass；B 在飞窗重
  插单请求单 load（⓶）Pass；C srcset 空落穿 dormant 旗标同步置位 +
  sweep 复苏单请求（⓵）Pass；D 换源重插仍重验翻转 currentSrc（⓯
  负控）Pass。
- **哨兵 solo**：relevant-mutations 126P 0F；image-base-url 关键断
  言 Pass；decode-quick-attach 0F；update-the-image-data 目录
  18P+2F=not-broken（s283 勘根⑤布局移交基线原样）；empty-src 0F；
  sizes-auto-dynamic 3P。
- **family A/B**（A = s289-imgfamily-b，B = s290-imgfamily-b，探针
  已删后重跑；(key,state) 去重 join）：**1031 键逐键全同**
  （982P/44F/5T），CHANGED/GAINED/LOST 全空，REGRESSED=0。B 原始行
  983P 的 +1 为 parse-a-srcset-attribute 含换行名子测试的 runner 双
  发射（同键同态，s139 先例同类，非事件）。md5 0045ae67。
- **全树 A/B**（A = s289-fulltree-b，B = s290-fulltree-b，双方
  RAF_FRAME_DRIVEN=1/TIME_LIMIT=14400 同旗标）：**2 键 forms 域差
  异，撤片实验（stash s290 hunk 重建前二进制）定谳非本片位移**——
  ①checkable-active-onblur「following label」行：前二进制 solo
  3/3 Fail = 后二进制 solo 3/3 Fail（二值行为同现，基线轮 Pass 为
  8 负载并行时序离群）；②select-home-end-pagedown-pageup-detailed
  .optional：前二进制 solo 4F/0T = 后二进制同形（基线轮文件级
  Timeout 为同源离群）。img 语料面零位移维持；**第 24 个全树轮非
  严格逐键全同，连续完美 streak 以 img 语料子树口径延续**（forms
  两键为负载时序二值面，不属 img 域口径）。B = 7962P/400F/57T，
  md5 83fe384c。
- **make test**：78 套件 9109P/5F——5F 恰为 s283 勘根⑤五红测原键
  （p56/p99s/r2809/r2810/r3276，CSSOM cssText 序列化域兄弟流在途，
  rule 9/10 记账）；判定套件 js_dom_bridge 3727P/5F 与 s288/s289
  逐键同红非结果位移，P 总数持平（日志 s290-make-test.log）。

## 挂账更新

- **un-gate 双前置件齐备**：fetch 核 step-11 守卫（s289，d23e46427）
  + hook 层 env 门卫（本片）——un-gate 切片可独立立项，点亮时只需
  放宽 allowsAuto 收窄门；picture 父 IMG 臂（s221）env 化为独立评估
  面（current-request-microtask 时序面依赖现 unconditional 重验，
  本片不动）。
- **口径注记**：①全树「连续完美」streak 自本轮起分口径：img 语料
  子树逐键全同口径延续；forms 域 checkable-active-onblur「following
  label」行与 select-detailed.optional 为负载时序二值面（s290 撤片
  实验存证），后续全树 A/B 命中该两键不再单独立案，连续两轮同向
  位移才立案。②maybe omit events 旗标（allows-auto-sizes
  starts-being-rendered relevant mutation 携带）引擎未建模——avail
  臂 load 无条件重派 vs spec 条件派发，随 un-gate 切片评估。

## 下一步候选

1. **un-gate 切片**（插入 hook 放宽 allowsAuto 收窄门——双前置件已
   齐备，image-base-url/decode/microtask 五面由 env 门卫与 step-11
   守卫承接）。
2. **SVG 纵横比-only 填充**（naturalWidth 18F，rendering 域跨切）。

## 证据

- `evidence/logs/s290-probe-env-gate.log`（探针四臂）
- `evidence/logs/s290-relevant-mutations.log` /
  `s290-image-base-url.log` / `s290-decode-quick-attach.log` /
  `s290-utid-dir.log` / `s290-empty-src.log`（哨兵 solo）
- `evidence/logs/s290-imgfamily-b.log`（family B；A = s289-imgfamily
  -b）
- `evidence/logs/s290-fulltree-b.log`（全树 B；A = s289-fulltree-b）
- `evidence/logs/s290-make-test.log`（全量 make test）
- 日志 md5：probe ef2c9f56、relmut 42ed1f57、base-url e22bca18、
  decode-qa 6fc366e0、utid-dir f5469062、empty-src 58568220、
  imgfamily 0045ae67、fulltree 83fe384c、make-test 88261d4d
