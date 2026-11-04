# M2 s286 — lazy 视口门 restart 语义修复（multiple-times F→P，family/全树 REGRESSED=0 第二十个连续完美）

日期：2026-11-05。切片号 s286（承 s285 下一步候选①——fetch 核冗余更
新守卫）。勘根转向：候选①域内真红面勘根定位到 lazy 门 restart 旁路
缺陷，守卫本身（同 URL 已可用不重取）现状无红面依赖，保持挂账。

## 勘根

1. **candidate ① 域内红面清单勘根**（family 54F 逐面归类）：候选①
   fetch/update-the-image-data 域内有 6 个真红面——adoption（6 子测
   悬挂 + 1F，文件 Timeout）、not-broken-while-load-task-scheduled
   （2F）、image-loading-lazy-multiple-times（1F）、list-of-available-
   images-does-not-coalesce（1F）、404-response-with-actual-image-
   data、historical-progress-event。
2. **multiple-times 根因双层**（solo + 断言面定位）：①s273 lazy 视口
   门 `!_imHadPrev` 旁路——已装载 lazy img 换 src 走 restart 语义绕过
   距离门 eager 装载，与 spec「lazy 可多次装载」相悖（face 断言滚出视
   口后二次换源不得装载）；②视口旗标 `_zw5277ViewportFlag` sticky
   永不消费——第一次 scrollIntoView 置位后所有后续 update run 恒直通
   门（face 第二次换源 eager 装载的真机制位；删旁路后仍因陈旧旗标
   eager）。
3. **域内移交/挂账归位**（rule 9 工作面）：not-broken-while-load-task-
   scheduled 失败机制 = clientWidth pre-load 布局面（attr 尺寸
   presentational hint 未落 layout box，`expected 50 but got 0`）——
   layout-engine 属渲染流工作面，移交记账；adoption = adoptNode 触发
   器机具整体缺失（DOMParser 文档 img adopt 后无 update run，6 子测
   悬挂）+ template clone 插入面，独立候选；coalescing = host 网络层
   pipe 语义（同 URL 在飞请求去重破坏 trickle(d1) 每请求管道），网络
   域候选。

## 施工（shim 1 文件）

- `crates/engine/src/js_dom_shim/part03.js`：
  - **删 lazy 门 `!_imHadPrev` 旁路**：已有请求的图过视口门逐 run 重
    判（spec update-the-image-data lazy 分支逐 run 生效；同 URL 已可
    用 run 走 available-images 命中臂在门前，不受影响）；旁路删后
    `_imHadPrev` 定义行无读点（门禁扫描附注），准则 4 同片清理。
  - **视口旗标一次性消费**：commit 点清位（avail 命中 settle 臂 +
    fetch 臂启动位）——置即开语义在本次 run 提交时消费，滚出视口后的
    后续换源 run 回落距离门；消费点在门后 commit 位，s274 after-load
    drain 的「旗标救」路径不受影响（drain 经 viewport-revalidate →
    fetch 臂即消费）。

## 验证

- **multiple-times solo**：F→P（日志 s286-multipletimes-fix.log，
  md5 805b1fd4）。
- **image-loading-lazy 族 solo**：33P/0F/0T（日志 s286-lazy-family.log，
  md5 3a64a149）——lazy 门核心改动全域复绿（含 below/in-viewport-
  dynamic 窗序、crossorigin-change、base-url ×2、available/use-list、
  not-rendered、to-eager）。
- **fetch 守卫哨兵复绿**：update-the-image-data 目录 18P，唯一 2F =
  not-broken（layout 移交面，非本片位移；日志 s286-utid-dir.log，
  md5 1d7707ef）；empty-src 2P（s285 面保持）；relevant-mutations
  零 Fail。
- **family A/B**（A = s285-imgfamily-d.log，B = s286-imgfamily-e.log）：
  73P/14F/4T，键级 **FIXED=1 / GAINED=0 / LOST=7 / REGRESSED=0 /
  T-changed=0**。LOST=7 全为 s285 基线中已删探针页陈旧键（良性）。
- **全树 A/B**（A = s285-fulltree-d.log；run1 s286-fulltree-e-run1.log
  唯一 P→T = forms/focus-dynamic-type-change-on-blur 文件级 Timeout，
  solo 两跑 160 子测全 Pass 实证并行负载时序噪声——forms 域零 img
  机具耦合；run2 为正式证据）：917P/203F/52T，键级 **FIXED=1 /
  GAINED=0 / LOST=0 / REGRESSED=0 / T-changed=0** 第二十个连续完美。
  （日志 run1 e0d94401 / run2 ea134801 / focus 归因 360be6b6）
- **make test**：9092P/5F——5F 恰为 s283 勘根⑤五红测原键（p56/p99s/
  r2809/r2810/r3276，CSSOM cssText 序列化域兄弟流在途，rule 9/10
  记账）；对比 s285 9088P/5F，P 增量含本片目标面（日志
  s286-make-test2.log）。
- 探针页无新增；临时 trace 无残留。

## 挂账更新

- **候选①（fetch 核冗余更新守卫）保持挂账**——现状无红面依赖，纯
  spec 忠实面（un-gate 插入 hook 的前置）。
- **新挂账（本片勘根定性）**：
  ①adoption 触发器机具（adoptNode 跨文档 img 无 update run——6 子测
  悬挂文件 Timeout + template clone 插入面，engine shim 域独立候选）；
  ②not-broken-while-load-task-scheduled 客户Width pre-load 布局面
  （attr 尺寸 presentational hint 未落 layout box——**渲染流工作面
  移交**，rule 9 记账）；
  ③coalescing：同 URL 在飞请求去重破坏 trickle 管道（host 网络层域）；
  ④域内余红面 404-response-with-actual-image-data /
  historical-progress-event 待勘根。
- **保持**：SVG 纵横比-only 填充、clone density、display:none rect
  清零、宿主 documentElement 物化缺口（s285 新挂）、sizes-auto 残
  4F+1T（各归其域）。

## 下一步候选

1. **adoption 触发器机具**（新挂账①，engine shim 域，多子测悬挂面）。
2. **fetch 核冗余更新守卫**（候选①保持，spec 忠实 + un-gate 前置）。
3. **SVG 纵横比-only 填充**（naturalWidth 18F，rendering 域跨切）。

## 证据

- `evidence/logs/s286-multipletimes-fix.log`（目标面 F→P solo）
- `evidence/logs/s286-lazy-family.log`（lazy 族 33P 全绿）
- `evidence/logs/s286-utid-dir.log`（update-the-image-data 目录 +
  not-broken 移交面证据）
- `evidence/logs/s286-imgfamily-e.log`（family B；A = s285-imgfamily-d）
- `evidence/logs/s286-fulltree-e-run1.log` / `s286-fulltree-e2.log`
  （全树 run1 噪声归因 + run2 正式证据；A = s285-fulltree-d）
- `evidence/logs/s286-focus-noise-solo.log`（focus 面噪声 solo 归因）
- `evidence/logs/s286-make-test2.log`（全量 make test）
- 日志 md5：multipletimes 805b1fd4、lazy-family 3a64a149、utid
  1d7707ef、imgfamily e2241e8f、fulltree-run1 e0d94401、fulltree-e2
  ea134801、focus-noise 360be6b6、make-test cc5a1210
