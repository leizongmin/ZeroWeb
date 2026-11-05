# M2 s291 — 插入 hook un-gate（非 picture IMG 臂放宽 allowsAuto 收窄门 + ⓷ detached 延步 run pending 标记；family/全树 img 子树逐键零位移，arm F load 计数 2 经撤片实验定谳 s289 既知决策后果）

日期：2026-11-05。切片号 s291（承 s290 下一步候选①——un-gate 切片，
双前置件已齐备）。纯 spec 忠实面：解除 s284 临时收窄，零红面依赖。

## 勘根

1. **un-gate 三前置件齐备核算**：s289 fetch 核 step-11 守卫（restart
   盲重启子面）+ s290 hook env 门卫（⓵⓶⓮⓯ 插入面）+ 本片 ⓷ detached
   延步 run pending 标记（第三前置件）——attr 变更时 img 未连接，
   update run 已延步排队（fetch 臂 isConnected 检查）；此时插入不另启
   重验，排队 run 承接（插入 run 落 spec step 11 同 URL return，decode
   捕获态不被抢跑 run 的 restart 取代——decode-qa「: srcset」非
   picture 断言面）。s284 当时 5 行 REGRESSED 三面归位：image-base-url
   不重取=⓵（plain src srcset 空落 dormant 臂）；decode attach 时序=⓷
   （detached defer 快 attach 面标记早退）；microtask 任务计数=⓯ 同态
   早退 + step-11。
2. **⓷ 标记机制**：登记点 = fetch 臂 isConnected defer 排队点设
   `_zwImgDeferUpdate5277[key]=true`；清除点 = 延步回调头部（先于
   RunUpdate/NullSelection 分支，两分支同清）。key 派生两侧同 `_elKey`
   （`'@'+handle`/sel）。picture 臂不设此卫（empty-picture srcset 重启
   →EncodingError 断言面维持——s221 unconditional 重验语义保留）。
3. **探针归因纪律两刀**（首跑 A/B/D/F 失败的双根因，中间轮日志被重跑
   覆盖，数值以本段为准）：①仪器化全局计数被 testharness 并发臂污染
   ——六臂同文件并行执行，全局 Revalidate 计数含他臂 attr 路径调用
   （arm A 窗内 rev=3 = 本臂 0 + 他臂 3；改按 img 身份
   `__zwSelector/__zwHandle` 过滤 + 全臂 promise 链串行化后归零）；
   ②arm F「exactly one load」断言 2 load 经**撤片实验**（stash un-gate
   hunk 重建前二进制，`strings` 验证 `_zwImgDeferUpdate5277` 缺席）定谳
   非本片位移：HEAD 同现 loads=2——s289 账本反向约束③「state-less 首
   飞同 URL 重入不得早退」（decode-qa picture 6 面 EncodingError 依赖
   重启语义）的既知后果：detached src+srcset 双 mutation 各排队延步
   run，第二 run 执行时首 fetch 未 settle（state-less）→ restart 不早
   退 → 双 fetch 双 load。corpus 无 load 计数钉面（current-request-
   microtask-002 只钉 currentSrc 时序），F 臂撤 load 断言保留 rev==0 +
   currentSrc settle 钉面；dump 探针勘根后删。
4. **Revalidate-on-frozen 路径核**：mutation 面 Revalidate 重选源 →
   RunUpdate lazy 门重新登记冻结请求 = spec 重设请求（s275 注释既定
   语义，非 double fetch）；un-gate 只扩人口（全部 srcset 图），机制
   不变。

## 施工（shim 1 文件零 Rust）

- `crates/engine/src/js_dom_shim/part03.js`：fetch 臂 isConnected
  defer 排队点登记 `_zwImgDeferUpdate5277` + 延步回调头部清除；插入臂
  去 allowsAuto 收窄门（s284 条件解除）+ env 门卫 ⓶ 判据补 ⓷；
  s284→s291 臂注释改写（spec relevant mutation 双条基座、三层收口
  归位记录）+ ⓷ 注释块。

## 验证

- **probe-s291-ungate 六臂 Pass**（探针已删；日志 s291-probe.log）：
  A loaded srcset 图同态重插 env ⓯ 拦截零重验零追加 load；B trickle
  在飞窗摘插 ⓶ 单请求单 load；C detached srcset+decode 快 attach ⓷
  decode resolve 非 EncodingError + 插入臂零重验；D 摘下换源再插排队
  run 承接 currentSrc 翻转 + 插入臂零重验；E sizes=auto lazy script
  建图插入点亮（s284 正控）；F detached src+srcset 双 mutation 后
  append 微任务 currentSrc 落 resolved + 插入臂零重验。
- **哨兵 solo 基线原样**：relevant-mutations 126P 0F；image-base-url
  1P 0F；decode-quick-attach 10P 0F；utid 18P+2F（not-broken-while-
  load-task-scheduled src/srcset 同键，s283 勘根⑤基线）；empty-src
  3P 0F；sizes-auto-dynamic 3P 0F。
- **family A/B**（A = s290-imgfamily-b，B = s291-imgfamily-b，
  (key,state) sort -u join）：**1031 键逐键全同**（982P/44F/5T），
  CHANGED/GAINED/LOST 全空，REGRESSED=0。md5 3801beda。
- **全树 A/B**（A = s290-fulltree-b，B = s291-fulltree-b，双方
  TIME_LIMIT=14400 RAF_FRAME_DRIVEN=1）：**REGRESSED=0**（P2F/F2P/
  T-changed 全 0）；唯一差异键 = select-detailed.optional 文件级
  Timeout（A）→ 4 子测 Fail（B）——forms 负载时序二值面，s290 撤片
  已证双二进制 solo 同态 4F/0T；连续第二轮同向（s289→s290 同形），
  按 s290 口径注记升级常设负载时序面：A 侧文件 Timeout = 8 负载并行
  伪态、B 侧 4F = solo 真态，非引擎位移。img 语料子树 join same=1031
  零位移——**img 子树逐键全同口径延续**。B = 7960P/402F/57T，
  md5 ef2334c2。
- **make test**：78 套件 **9109P/5F**——5F 恰为 s283 勘根⑤五红测原键
  （p56/p99s/r2809/r2810/r3276，CSSOM cssText 序列化域兄弟流在途，
  rule 9/10 记账），判定套件 js_dom_bridge 3727P/5F 与 s288/s289/s290
  逐键同红非结果位移，P 总数与 s290 持平（日志 s291-make-test.log）。
- **施工后微改确认**：s291 hunk 两处 catch 局部变量名 `_e5290dm/dc`→
  `_e5291dm/dc`（纯命名对齐本片惯例，零行为面）——concat node --check
  干净 + release runner 重建 + relevant-mutations 复跑 126P/0F/0T 与
  前跑全同（日志 s291-relevant-mutations-postrename.log）。

## 挂账更新

- s290 挂账①（un-gate 可独立立项）销案 = 本片。
- s290 挂账②（maybe omit events 旗标随本片评估）**销案**：维持不建
  模——corpus relevant-mutations「src set to same value」行期望 load
  重派（s289 勘根⑤ s70 张力核销同源），spec avail-hit 臂条件含
  「maybe omit events is not set OR previousURL ≠ urlString → fire」
  的 not-set 分支，语料全部落 not-set 臂；starts-being-rendered 判别
  （渲染完成时机）语料无钉面，建模零收益有红面风险。口径注记转常设。
- picture 父 IMG 臂 env 化维持独立评估面（s221 current-request-
  microtask 时序面依赖现 unconditional 重验，本片不动）。
- 宿主 documentElement 物化缺口根修（s285 挂账沿用）。

## 下一步候选

1. SVG 纵横比-only 填充（naturalWidth 18F，rendering 域跨切——s290
   候选②顺承）。
2. picture 父 IMG 臂 env 化独立评估（s221 microtask 时序面）。
3. 宿主 documentElement 物化缺口根修（s285 挂账）。

## 证据

- `evidence/logs/s291-probe.log`（六臂终跑；首跑污染+归因过程数值见
  勘根③）
- `evidence/logs/s291-head-dump-ungate-attribution.log`（撤片实验：
  HEAD 二进制 dump 探针 loads=2）
- `evidence/logs/s291-relevant-mutations.log` / `s291-image-base-url
  .log` / `s291-decode-quick-attach.log` / `s291-utid-dir.log` /
  `s291-empty-src.log` / `s291-sizes-auto-dynamic.log`（哨兵 solo）
- `evidence/logs/s291-imgfamily-b.log`（family B；A = s290-imgfamily
  -b.log）
- `evidence/logs/s291-fulltree-b.log`（全树 B；A = s290-fulltree-b.log）
- `evidence/logs/s291-make-test.log`（全量 make test）
- `evidence/logs/s291-relevant-mutations-postrename.log`（微改后复跑
  确认）
- 日志 md5：probe f40ccf79、head-dump 7092cede、relmut b15e5a1e、
  base-url 9e3b9224、decode-qa 24051e42、utid ecb6ffc7、empty-src
  5bb94227、sizes-auto-dyn ab6e73e6、imgfamily 3801beda、fulltree
  ef2334c2、make-test d21f517d、postrename 359b6313
