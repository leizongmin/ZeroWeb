# M2 s273 — lazy 滚动唤起面 + lazy 不阻 window load（型 D 双页 Timeout→Pass 收口，2 文件 6 刀）

日期：2026-11-04。切片号 s273（承 s272 下一步候选①：型 D 文件级
Timeout 勘根）。2 文件 6 刀（engine shim 单车道，零 runner 改动）。

## 勘根

1. **image-loading-lazy.html 页语义**（主干 [4,4] 全绿）：四 lazy 图
   ——in_viewport（near + trickle(d2)）onload 断言 window load 先发后
   scrollIntoView(#bottom)；below 系三图（http / data: / blob:）
   onload 断言 in_viewport 已装载 + window load 已发。solo 签名 =
   in-viewport 子测 Fail（窗序）+ 文件 Timeout（below 系 pending）。
2. **below 系挂死根**（探针 zz-s273-probe 两轮，验后删）：scrollInto
   View(#bottom) 已执行（y=6068）、window load 已派，但 below 系三图
   零装载——`__zwImgScrollIntoView5277`（s70）只对**目标元素自身**置
   旗标（本页目标 #bottom 是 div）→ 三图旗标未置、零 revalidate；settle
   skip 点「惰性记账」无登记册、滚动唤起面完全缺席。
3. **窗序根**（探针 probe3 时序 dump）：`iv-onload@113 winload=false →
   scrolled@116 → bv-onload@117 → winload@125`——lazy 图装载阻
   window load（winload 晚于 bv-onload）→ below 子测断言
   has_window_load_fired 败。spec 自洽定谳：fetch 步「Let delay load
   event be true if the img's lazy loading attribute is in the Eager
   state ... and false otherwise」——delay load 由 loading 态决定，
   **lazy 态不阻**。s272 挂账的 below-viewport-dynamic /
   in-viewport-dynamic Chrome-only 口径同根，本切一并落地。
4. **trickle 缺席为既有建模限制**（非 bug）：testharness.rs:2256
   「WPT trickle pipe 机制不可复现」+ webview.rs:216「runner 同步
   fetcher 忽略 trickle」——探针实证 trickle(d2) 图 113ms 装载（预抓
   通道无慢流）。in-viewport 子测窗序断言（依赖 iv 慢 2s 拉开 load 与
   img load）结构性不可满足 → 维持挂账（连同 settle 同步直派事件早于
   window load 面）。
5. **data:/blob: 唤醒装载面**（探针 v2 定谳）：sweep → revalidate 后
   dv（data:）/bl（blob:）仍不装载——avail 预抓不含 data:/动态 blob:
   URL + fetch 通道 scheme 支持面——独立缺口挂账。

## 施工（2 文件 6 刀）

1. **part03 `__zwImgNearViewport5277` 滚动折算**：rect 快照是文档坐
   标恒定，滚动态减 scrollX/Y 折算视口坐标再与外扩矩形求交（spec
   intersection 按视口坐标；探针实证 scrollIntoView 落位后 scrollY=
   6068 与 _winScroll 联动）。scrollY=0 页判定恒等零位移。
2. **part03 `__zwImgSweepDeferredLazy5277`**（scrollIntoView helper
   后新增）：scrollIntoView 落位后重验未启动 lazy img（近似 spec
   IntersectionObserver 回调）。只扫 `loading=lazy` 且
   `_resourceStates` 无记录（与 s272 loading 刀同 no-op 判据）且旗标
   未置图（旗标路径已触发过，避免 crossorigin-change/referrerpolicy-
   change 直调页双 run）；不置旗标——旗标是无条件放行语义，距离门才
   是「将进入视口」近似。
3. **part04 scrollIntoView wrapper 挂 sweep**：`scrollTo(0, newTop)`
   落位后调用（scrollIntoViewIfNeeded 经委托同受益）。
4. **part03 `_imNoBlock` + `_zwLazyEver5277`**：lazy 装载不阻 window
   load 判据 = loading 现值 lazy 或曾处 lazy 态（deferred 发生点登记
   ——run 门 defer return + part04 settle 分流 continue 两点；
   Chromium once-deferred 永不阻口径，eager 化曾 lazy 图装载同
   noBlock）。eager/缺省装载照旧阻（delay-load-event trickle(d2)
   慢图断言面不动）。
5. **part03 avail 臂 + fetch 臂 inflight 条件化**：`_imNoBlock()` 真
   时跳过 `_imFetched`/inflight +1（终点不消费，+1/-1 配对保持）；事
   件照常派发。
6. **part03 `_zwImgDispatch` 入队 noBlock + part04
   `__zwImgLoadBlocked` 过滤**：lazy 图 pending 事件带 noBlock 标记，
   阻链判定过滤后不计阻；eager 事件照旧阻。

## 验证

- **族**（FILTER=image-loading-lazy，37 结果行 / 27 文件）归一 diff
  vs s272：**+2 文件翻绿**——below-viewport-image-loading-lazy-load-
  event **Timeout→Pass** + image-loading-lazy-srcset **Timeout→Pass**
  （型 D 三页收口两页），其余零位移（日志 s273-family.log）。
- **全树 A/B**（stash 同二进制法，test-guard + RAF）：A 8343 / B
  8344 键，(state+path) join **REGRESSED=∅**（第十三个连续完美 A/B）：
  - FIXED 键形换代 ×2：两页文件级 Timeout 键 → 子测 Pass 键（整页
    红→绿）；
  - image-loading-lazy::Below-viewport 子测 B 侧新可见 Fail 键（红态
    细化：文件 Timeout 未及报告 → 子测 Fail 可报告——s271
    image-loading-lazy 同例口径，非回归）；
  - image-loading-lazy 余键（in-viewport Fail 等）/ multiple-times /
    base-url / different-crossorigin 双侧零位移。
  （日志 s273-fulltree-{a,b}.log）
- **make test**（test-guard）：67 test-result 全绿，唯一 FAILED =
  engine **3724 passed + 5 failed**——R-t2qq adopted-stylesheets
  既有 5 红同集（s265/s268/s270/s271/s272 账本记档），本切零位移。
- fmt：`cargo fmt --all --check` 干净；make test 并行 clippy 臂
  （-D warnings）零 error 完成。
- 注记：shim 单文件 `node --check` 不适配（拼接设计、单文件不平衡
  ——HEAD 版同报，s273 起语法验证以 runner 构建 + 跑页为准）。

## 挂账更新

- 型 D **收口两页**：below-viewport-load-event + srcset（滚动唤起面
  根，本切销案）；image-loading-lazy.html 维持文件 Timeout（余红 =
  in-viewport/below 窗序子测 Fail + data:/blob 子测 pending）。
- 新记录：
  ① **data:/blob: deferred 唤醒装载面**（sweep revalidate 后 run 的
  avail/fetch 臂不覆盖——avail 预抓不含 data:/动态 blob: URL +
  fetch 通道 scheme 支持面勘根）；
  ② **窗序结构性面**（img load vs window load）：trickle pipe 缺席
  （runner 建模限制，testharness.rs:2256/webview.rs:216 在案）+
  settle 同步直派事件早于 window load——in-viewport-dynamic /
  below-viewport-dynamic / image-loading-lazy in-viewport 子测窗序
  断言维持 Chrome-only 口径挂账（s272 挂账收窄归并入此）。
- 承前不动：型 B base-url ×2；型 E different-crossorigin；
  multiple-times（主干双红销案维持）；runner 全量钉内存大户二分
  （s270 挂账①）；root margin 净速分档（s271 挂账①）；dynamic
  import() referrerpolicy 穿参（零 corpus face）。

## 下一步候选

1. data:/blob: deferred 唤醒装载面勘根（avail 预抓 scheme 面覆盖 +
   fetch 通道 data:/blob: 分派——image-loading-lazy 文件 Timeout
   余红 dv/bl 子测）。
2. 型 B base-url ×2 勘根（parse-time base URL 冻结面——s225 延步
   重选源机制与 lazy 门交互，settle 分流 + sweep 后基线态）。
3. 型 E different-crossorigin（avail 元组面——list of available
   images 元组键比对）。
