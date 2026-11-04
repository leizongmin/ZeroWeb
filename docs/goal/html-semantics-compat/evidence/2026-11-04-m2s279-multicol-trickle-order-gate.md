# M2 s279 — multicol 窗序反向面收口（trickle 分流建模：after-load 队列按 pipe=trickle 门控）

日期：2026-11-04。切片号 s279（承 s278 下一步候选①：runner trickle/async
慢流时序勘根——multicol 窗序反向面 + s273 挂账②族解除面）。1 文件
1 块（engine shim 单刀，零 Rust 改动）。

## 勘根

1. **三页断言面矩阵**（测试全文逐行核）：
   - image-loading-lazy-multicol.html：in-viewport lazy img（**无
     trickle**，160×120 固有尺寸）onload 置 `has_in_viewport_loaded`；
     window load listener 断言 `assert_true(has_in_viewport_loaded)`
     ——**图装载须先于 window load**（真引擎语义：页内 slow.py 慢
     async script 拖住 load，快图在拖住窗内完成）。
   - image-loading-lazy.html：in-viewport img `pipe=trickle(d2)`
     onload 断言 `has_window_load_fired`——**图装载须晚于 window
     load**（真引擎语义：2s 慢流，load（lazy 不阻）先发）。
   - picture-loading-lazy.html：in-viewport <picture> lazy img **无
     窗序断言**（子测只要求装载发生；窗序断言只在 below-viewport
     lazy_attribute `complete=false` @load——远图距离门 deferral 面）。
2. **两窗序要求在同一 runner 时序下结构性互斥**：runner 同步快回
   无慢流/慢 script，settle 与 window load 同相位——「图先于 load」
   与「图后于 load」只能二选一。s274 全量队列化（近端 lazy 图一律
   after-load 队列）选了后向序 → multicol 反向红（s274 A/B
   REGRESSED=multicol 1 键，裁定挂账）；**A 侧证据回查**
   （s274-fulltree-a.log）：multicol 该子测 A 侧 **Pass**——settle
   内联装载（pre-s274 语义）天然满足 multicol。
3. **两页的判别特征 = trickle pipe 参数**（非 slow.py）：真引擎中
   「settle 时点响应是否已完成」由响应速度决定——trickle(dN) 响应
   dN 秒未完（load 先发、img 后至）；无 trickle 响应即时完成
   （img 先于被 slow.py 拖住的 load）。runner 无真实时序差，用
   **时序级 trickle 建模**（order-level）替代：按 URL 是否携
   `pipe=trickle(` 分流——携 = 响应未完 = after-load 队列（s274
   语义收窄保留）；不携 = 响应已完成 = settle 内联装载（pre-s274
   语义恢复）。
4. **slow.py 缺席定性**（勘根副产品）：`/common/slow.py` 语料包
   不存在（ls 实证，multicol/picture 两页引用之）——它只是真引擎
   测试健壮性机制（拖住 load 使反向形态 dependably fail），非断言
   机制；runner 的 async script 404 不持有任何断言面。本切不需要
   补语料（与 s227/s238 资产缺口不同形——那两处缺资产直接改变断言
   结果，本处 404 无断言消费）。runner 现存 trickle 基建盘点：
   s112 `trickle_delay_ms`（testharness.rs:11141，脚本单元真实
   setTimeout 延迟）+ part01 fetch 通道 trickle 仿真臂（6028 起）+
   s256 dynamic_import_fetch_delay——img 面此前唯一缺的恰是 settle
   门分流判据，本切补齐（时序级，不做真实秒级延迟——trickle(d100)
   语料在案（delay-load-event-until-move-to-empty-source），真实
   延迟撞页脚本 90s 上限必假红）。
5. **picture-loading-lazy 面澄清**（s274 账面回查）：A 侧文件级
   Timeout 非窗序根，是 s271 零面积拦截误杀（in-viewport picture
   img 无 width/height 属性 → 未装载零面积 → 误判远 → 永不装载 →
   子测 pending）；s274 刀1 零面积细分（位置真实零面积落穿距离判
   定）已修分类，队列化只是当时顺带路径。本切该图（无 trickle）改
   走内联装载，子测仅要求装载 → 保持绿。
6. **幂等性核**（settle 多轮）：内联装载臂既有 `if (_zwPrior)
   continue` 守卫（part04:1378）+ SVG 臂 `!_zwPrior` 派发守卫
   ——settle#2 零重派；sweep（s273）按 `_resourceStates` 有记录
   skip → 已装载图零 double-load；multicol `assert_false(has_in_
   viewport_loaded)`（load only once）精确满足。

## 施工（1 文件 1 块）

part04.js settle 扫描 lazy 门 s274 队列块改造：近端 lazy 图先做
trickle 判定（`/[?&]pipe=trickle\(/` 语法同 part01:6035——判据
URL = s275 冻结源 `_zwImFz.url` 优先、raw src 属性回落，picture
srcset 面覆盖）：trickled → 旗标 + after-load 队列（原语义原样）；
非 trickled → 落穿下方内联装载（s277 曾-lazy 判据继续适用——曾
defer 的图仍跳过同步装载）。注释记 s279 分流语义 + wptserve pipes
规范锚。

## 验证

- **family**（FILTER=image-loading-lazy，39 结果行）：**37P/2F →
  38P/1F**——multicol **F→P**（目标），multiple-times F 维持
  （s278 销案签名逐字同——assert_unreached 换 src eager 装载），
  其余 36 文件零位移（日志 s279-family.log vs s279-family-base.log）。
- **picture-loading-lazy solo**：4/4 Pass 零位移（s279-picture.log）。
- **image-loading-lazy 核心页**：4/4 Pass 双侧逐子测同（trickled
  路径零位移）。
- **全树 A/B**（stash 法，test-guard + RAF，同机安静窗）：A 8345 /
  B 8345 全键（(state,path,subtest) 归一 sort -u），**LOST 0 /
  GAINED 0**，**FIXED=2 / REGRESSED=0 / T-changed=0**，行级
  7862P/424F/62T → **7864P/422F/62T**（ΔP=+2/ΔF=−2/ΔT=0）——
  **第十四个连续完美 A/B**：
  - multicol F→P（目标键，恰靶）；
  - checkable-active-onblur-with-click「following label radios」
    F→P = 在册 checkbox `:active` 漂移族（s238/s239/s245/s246/s248
    记档）有利向侧摆——同跑混态自证（A 侧该页 4P/4F、B 侧 5P/3F
    八子测混态），改动臂（img settle 门）与 checkbox :active 零通
    道，按先例记基线态不归因。
  （日志 s279-fulltree-{a,b}.log）
- **make test**（test-guard，后台 setsid）：结果见日志
  s279-maketest.log（engine R-t2qq adopted-stylesheets 既有 5 红
  同集基线外零位移预期，实测见账面）。
- 语法验证：runner 重建 3 轮（31.7s/35.4s/33.4s）+ family/全树
  7864 Pass 执行面证（单文件 node --check 不适配拼接设计，s273 起
  口径）。

## 挂账更新

- **multicol 窗序反向面：销案**（trickle 分流建模收口；s274 裁定
  「保 image-loading-lazy」的两难前提解除——分流后两页各得其所，
  无取舍）。
- **s273 挂账②族解除面更新**：runner 时序建模缺口在 **img settle
  门面收口**（时序级 trickle 建模——携 trickle = 响应未完 = 队列
  后置；无 trickle = 响应已完 = 内联先行）；**余面维持**：
  - below/in-viewport-dynamic Chrome-only 口径（s272/s277 运行期
    src 变更面，非 settle 静态图，本切不涉）；
  - 真实时序级 trickle（秒级慢流）/slow.py 慢 async script：语料
    无断言消费面（slow.py 健壮性机制 + trickle 秒级延迟语料
    delay-load-event 页对时序序断言不敏感——eager 图 settle 内联
    装载先于 load 恒满足 delay-load 断言），维持不建模记账。
- **slow.py 语料缺席注记**：multicol/picture 两页引用的
  `/common/slow.py` 语料包不存在——404 无断言消费（健壮性脚本），
  不补（区别于 s227/s238 断言消费型资产缺口）；update-wpt-data.sh
  无需挂账。
- 承前不动：multiple-times 销案（签名级钉死）；runner 全量钉内存
  大户二分（s270 挂账①）；root margin 净速分档（s271 挂账①）；
  dynamic import() referrerpolicy 穿参；decode() 兜底元组细化
  （s276 注记）。

## 下一步候选

1. runner 全量钉内存大户二分（s270 挂账①——runner 域延续）。
2. root margin 净速分档（s271 挂账①——lazy 视口门精度面）。
3. below/in-viewport-dynamic Chrome-only 口径勘根（s272 挂账延续
   ——inflight 阻塞链调度 vs spec delay load 字面冲突面）。
