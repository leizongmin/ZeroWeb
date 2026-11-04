# M2 s274 — lazy 零面积细分 + after-load 延后装载（型 D image-loading-lazy 整页绿 + picture 红利，3 文件 7 刀）

日期：2026-11-04。切片号 s274（承 s273 下一步候选①：data:/blob: deferred
唤醒装载面勘根）。3 文件 7 刀（engine shim 单车道，零 runner 改动）。

## 勘根

1. **触发链探针**（zz-s274-probe v3，验后删）：s273 sweep → revalidate →
   select → runupdate 对 dv（data:）/bl（blob:）**全部到达**、src 正确
   ——断点在 runupdate 内部；v4 fetch 链 instrument：runupdate lazy 门
   `__zwImgNearViewport5277(dv/bl)` **false → defer return，fetch 从未
   发出**（bv true → fetch → 装载对照）。
2. **rect 形态定谳**（v5 gBCR dump + v6 not-rendered 形态）：
   - dv/bl：`t=6068 b=6068 w=784 h=0`——**位置真实的零面积**（未装载
     img 无 intrinsic 尺寸；avail 解析期预抓不含 data:/动态 blob: →
     布局零高，http 图预抓命中 rect 40×30 对照）——s273 挂账「avail
     预抓不含 data:/blob:」的布局面后果实锤；
   - not-rendered（display:none/hidden/js display:none）：gBCR **全零
     (0,0,0,0)**——两种形态精确可分；
   - 零面积拦截（s271 `!(r.width>0 && r.height>0)` 判不近）误杀前者
     ——位置在视口内（sy=6068、ih=600、t=b=6068）仍被拦。
3. **zero-intersection-area 页复核**（s271 反向面）：断言实为「零面积
   （被 overflow:hidden 裁剪）图**也装载**」（Mozilla bug 1785186）；
   s271 时绿因 rect_bridge 不裁剪（rect 100×100 非零）——s271 零面积
   拦截从未生效于任何 corpus 断言面（纯保守防御），细分刀无反向面。
4. **真页面 r1 定谳**（solo image-loading-lazy，零面积细分刀后）：
   dv/bl **onload 到达**（装载成功——fetch → scheme dispatch → GIF 解
   码 12×12 → load 事件全链通），仅剩窗序 assert 反向红（img load 早
   于 window load）——四子测窗序断言面：spec lazy 图 delay load =
   false 不阻 window load，装载完成序须在 load 派发之后。
5. **窗序修复刀**（settle 落穿臂延后）：settle 在 window load 派发前
   同步落穿装载近端 lazy 图（s271 距离分流语义）→ img load 早于
   window load → 「do not block the load event」assert false。落穿装
   载改 after-load 队列延后（window load listener 跑完后 drain 装载）。
6. **multicol 回归定谳**（族归一唯一回归）：该页断言方向相反——
   in-viewport lazy 图 load 须**先于** window load（页内 slow.py 慢
   async script 拖住 window load——async script 阻 load 语义）。runner
   无 trickle/slow.py 真实时序差（同步快回）→ 「图装载赶在 load 前」
   与 image-loading-lazy「装载晚于 load」同一窗口两端都要求，**结构
   性不可两全**（runner 时序建模限制，与 s273 挂账② trickle 缺席同族
   ——仅 settle 同步装载或延后装载二选一）。裁定保 image-loading-lazy
   （主干 [4,4] 双引擎绿真缺口 +4 子测），multicol 1 子测挂账。

## 施工（3 文件 7 刀）

1. **part03 `__zwImgNearViewport5277` 零面积细分**：not-rendered 形态
   （gBCR 全零——Chromium IO 对 not-rendered 不回调，lazy 图不装载，
   not-rendered-image-loading-lazy「should never load」断言面含
   below-viewport「even when scrolled into view」）判不近；位置真实
   零面积（未装载 img 高 0——data:/blob: 无预抓尺寸面）落穿到位置距
   离判定（Chromium LazyLoader 几何相交无面积要求）。
2. **part03 runupdate fetch 在飞键册**（`_zwImgFetching5277`）：fetch
   臂启动登记；终点清理 = `_zwImgDispatch` 入口（load/error 唯一出口
   ）+ lazy 门 defer return（stale 取代链断点防挂互锁）。
3. **part03 sweep skip 在飞**：`_resourceStates` 无记录但 fetch 在飞
   图跳过（drain 与 scrollIntoView sweep 重入防双 fetch 双 load——
   step_func_done 二次调用 Fail 面）。
4. **part03 新增 `__zwImgDrainAfterWindowLoad5277`**：drain
   `_zwImgAfterLoadQueue5277`（settle 登记的近端 lazy {sel,handle}）
   ——幂等 skip 已装载/在飞图；队列取空防重入。
5. **part04 settle lazy 门分流改造**：远图 defer（s273 口径维持，含
   `_zwLazyEver` 登记）；近图**不落穿装载**——置 `_zw5277ViewportFlag`
   （drain revalidate 过 lazy 门无条件放行，scrollY=0 远位图 Near
   Viewport false 由旗标救）+ 入 after-load 队列。eager 图照旧落穿
   （阻 load 语义面不动——delay-load-event 断言）。
6. **part04 settle 循环开头队列/在飞册重置**（persistent context 跨
   案键隔离，与 s224 代际表同点）。
7. **part06 window load 派发返回前挂 drain**：`_dispatchWithBubble`
   （listener 同步跑完、has_window_load_fired 已置位）后
   `__zwImgDrainAfterWindowLoad5277()`。window load 派发路径唯一
   （testharness.rs:9889 `globalThis.dispatchEvent(load)` → part06）。

## 验证

- **solo**（image-loading-lazy.html）：四子测 **4/4 Pass**——文件级
  Timeout + in-viewport/below-viewport 两子测 Fail → 全绿；**in-viewport
  窗序子测翻绿含 s273 裁定「结构性不可满足」（trickle 缺席）面**——
  after-load 延后机制使 iv 装载晚于 window load，断言可达（s273 挂账
  ②该子测收口销账）。日志 s274-solo-r2.log。
- **picture-loading-lazy.html**（型 D 族红利）：A 侧文件级 Timeout →
  B 态 **4/4 Pass**（in-viewport <picture> 子测延后装载翻绿；deferred
  子测 scrollIntoView 时序由 load 后 drain 承接）。全树 A/B 可见。
- **族**（FILTER=image-loading-lazy，39 结果行 / 32 文件）归一 diff
  vs s273：**+1 文件全绿**（image-loading-lazy 3 键：文件 Timeout +
  2 子测 Fail → 4 子测 Pass）+ multicol **Pass→Fail**（勘根 6 结构性
  挂账）——其余 6 红（base-url ×2 / below-viewport-dynamic /
  different-crossorigin / in-viewport-dynamic / multiple-times）双侧
  零位移（日志 s274-family.log）。
- **全树 A/B**（stash 同二进制法，test-guard + RAF）：A 8344 / B 8345
  全键（(state,path) 子测级归一），**REGRESSED = multicol 1 键**（已
  裁定刀形取舍非意外回归）；FIXED 5 键：image-loading-lazy 文件级
  Timeout 键（A-only）→ below ×2 Pass 键（B-only）+ In-viewport/
  Below-viewport Fail→Pass（共有键）+ **picture-loading-lazy 文件级
  Timeout 键（A-only）→ in-viewport Pass 键（B-only）**（日志
  s274-fulltree-{a,b}.log）。
- **make test**（test-guard，后台 setsid）：70 test-result，唯一
  FAILED = engine **3724 passed + 5 failed**——R-t2qq
  adopted-stylesheets 既有 5 红同集（s265/s268/s270/s271/s272/s273
  账本记档），本切零位移；错误 101 同源（日志 s274-maketest.log）。
- fmt：`cargo fmt --all --check` 干净（rc=0）；make test 并行 clippy
  臂（-D warnings）零 error 完成。

## 挂账更新

- 型 D **image-loading-lazy.html 整页收口**（文件级 Timeout 销案，
  四子测全绿）；**picture-loading-lazy.html 红利收口**（A 侧文件级
  Timeout，非候选名单内翻绿）；s273 挂账②窗序结构性面**收窄**：
  in-viewport 窗序子测（image-loading-lazy/picture-loading-lazy）
  由 after-load 延后机制收口销账；余 below-viewport-dynamic /
  in-viewport-dynamic Chrome-only 口径挂账维持。
- 新记录：**multicol 窗序反向面**（in-viewport lazy 图 load 须先于
  window load——slow.py 慢 async script 拖 load；runner 无 trickle/
  slow.py 真实时序差，settle 同步装载与延后装载二选一结构性不可两全
  ——保 image-loading-lazy 裁定 multicol 挂账；解除面 = runner 实现
  trickle pipe / async script 慢流时序，与 runner 建模限制挂账并轨）。
- 承前不动：型 B base-url ×2；型 E different-crossorigin；
  multiple-times（主干双红销案维持）；below/in-viewport-dynamic
  （Chrome-only 口径）；runner 全量钉内存大户二分（s270 挂账①）；
  root margin 净速分档（s271 挂账①）；dynamic import()
  referrerpolicy 穿参（零 corpus face）；multicol 上游主干状态待补查
  （wpt.fyi API 本轮网络超时）。

## 下一步候选

1. 型 B base-url ×2 勘根（parse-time base URL 冻结面——s225 延步
   重选源机制与 lazy 门交互；after-load drain/settle 分流新基线态）。
2. 型 E different-crossorigin（avail 元组面——list of available
   images 元组键比对）。
3. below/in-viewport-dynamic Chrome-only 口径勘根（s272 挂账延续
   ——inflight 阻塞链调度 vs spec delay load 字面冲突面）。
