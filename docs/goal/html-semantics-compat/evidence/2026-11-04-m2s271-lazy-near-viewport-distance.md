# M2 s271 — lazy 视口门距离分流（in-scroller ×7 + multicol + zero-intersection-area 收口，9 文件红→绿）

日期：2026-11-04。切片号 s271（承 s270 下一步候选①：lazy 族既有红群
勘根分型）。2 文件 2 刀（engine shim 单车道，零 runner 改动）。

## 勘根

1. **红群分型**（s270 family A 侧 19 红，referrerpolicy-change 已于
   s270 收口故实余 18）：型 A in-scroller 近端 ×7（in-scroller /
   -horizontal / -nested / -nested-2..5，签名「lazy-loaded images
   load when near viewport」）；型 B base-url ×2（parse-time base
   URL）；型 C eager 变更交互 ×4（in-viewport-dynamic /
   below-viewport-dynamic / to-eager / multiple-times）；型 D 文件级
   Timeout ×4（image-loading-lazy / srcset / zero-intersection-area /
   below-viewport-image-loading-lazy-load-event）；型 E different-
   crossorigin（avail 元组面）；另 multicol（视口交互）。
2. **距离模型反向页安全论证**：negative-margin 页 `margin-left:
   -10000vw`（水平 ~7.8M px 远）；not-rendered 零面积；below-viewport
   1000vh（y≈6008）；in-scroller-far ×2（大 spacer 远距）——统一
   「rect 与视口矩形外扩 margin 求交」模型天然排除。
3. **gBCR 通道探针**（zz-s270b-probe，验后删）：复刻 in-scroller
   结构，三时点（同步脚本 / setTimeout 100ms / window load）gBCR
   全可解析，返回**文档坐标**未被 overflow:scroll 容器裁剪（target
   y=138 ≈ body margin 8 + spacer 130），innerHeight=600 / innerWidth
   =800。rect_bridge 快照查询通道对未装载 lazy img 可用。
4. **刀位误判与再定谳**：初版假设解析静态 img 初始 fetch 走
   `__zwImgRunUpdate5277` 门（3190 lazy 视口门）——门内距离分支 +
   布局后 sweep 双刀 solo 七页仍红。调试计数器实证门**从未执行**
   （_zwDbgNearCalls undefined、册空）：解析静态 img 的初始 settle
   通路是 part04 `__zwFireStaticImgSettle`（R5275，window load 派发
   前扫描），其 lazy 分支直接 `continue`（1225，惰性记账面）；run
   update 门只接 mutation/revalidate 触发的 run（relevant mutations
   / scrollIntoView / eager 变更面）。sweep 机构撤銷——settle 时点
   （load 派发前）layout 快照已落地，距离判定单点即可，最小实现。
5. **host 预抓覆盖核实**：`extract_img_srcs`（extract.rs:470）无
   lazy 过滤——webview fetch_page_images 预抓全量 img src（含
   srcset 候选）→ settle 走 host `__zw_get_image_size` 快照必命中，
   放行不落 broken。
6. **type C 交互预判**：in-viewport-dynamic / multicol 近端图经新
   通路装载；multicol 实测翻绿（红利）；in-viewport-dynamic 仍红
  （assert 窗序语义，另轮勘根）。

## 施工（2 文件 2 刀）

1. **part03 `__zwImgNearViewport5277`**（scrollIntoView helper 后）：
   文档坐标 gBCR（rect_bridge 快照）与视口矩形 (0,0,innerWidth,
   innerHeight) 外扩 M=1250px（spec 未定值，Chrome 实现近似，FIXME
   净速分档注记）求交；零面积（width/height ≤0）与 rect 不可得一律
   判不近（保守 = 现状 deferred，零回归方向）。滚动位移未建模注记
  （scrollIntoView 旗标路径不受影响，s70 口径保留）。
2. **part03 lazy 视口门距离分支**（3190 门）：门条件追加
   `!__zwImgNearViewport5277(sel, handle)`——mutation/revalidate 触发
   的 run 近图放行（视口触发近似第二臂）。
3. **part04 settle 点距离分流**（`__zwFireStaticImgSettle` lazy
   skip 点，主刀）：`if (_zwImLoading === 'lazy' &&
   !__zwImgNearViewport5277(...)) continue;`——近图落穿按常规静态
   settle（host 快照 → loaded + load 事件 + available-images 缓存）
   ，远图维持惰性记账。块注释同步更新。
   （试错的 sweep/registry/debug 计数器机构已全撤——最小实现收敛。）

## 验证

- **solo**：in-scroller 族 9 文件 **9/9 Pass**（型 A ×7 F→P +
  in-scroller-far ×2 反向面保绿「do not load when far」）（日志
  evidence/logs/s271-solo-inscroller-v2.log；v1 为误刀位留证）。
- **族**（FILTER=image-loading-lazy）：**26P/9R**——较 s270 B 侧
  （17P/18R）**+9P**：型 A ×7 + multicol + zero-intersection-area
  全翻绿；余红（base-url ×2 / below-viewport-dynamic /
  different-crossorigin / in-viewport-dynamic / multiple-times /
  srcset / to-eager / image-loading-lazy / below-viewport-load-event
  ）双侧零位移（日志 s271-family-v2.log）。
- **全树 A/B**（stash 同二进制法，test-guard + RAF，4/8 阈值）：
  A 8341 键 / B 8342 键，(key,state) join **REGRESSED=∅**（第十一
  个连续完美 A/B）：
  - FIXED 8 键：型 A ×7 + multicol（Fail→Pass）；
  - zero-intersection-area：文件级 Timeout 键 → 子测键 Pass（整页
    红→绿，键形换代——A-only Timeout 键 + B-only Pass 键同文件对
    销）；
  - image-loading-lazy.html：文件级 Timeout 双侧同态，B 侧新增子测
    Fail 键可见（「In-viewport loading=lazy images load immediately
    but do not block the window load event」——A 侧文件早超时未及
    报告，红态细化非回归，归型 D 后续勘根）。
  （日志 s271-fulltree-{baseline-a,b}.log）
- **make test**（test-guard）：workspace 66 test-result 全绿，唯一
  FAILED = engine **3724 passed + 5 failed**——R-t2qq
  adopted-stylesheets 既有 5 红同集（s265/s268/s270 账本记档），
  本切 shim 改动零位移。
- fmt：`cargo fmt --all --check` 干净；workspace guarded-clippy
  零告警（-D warnings）。

## 挂账更新

- lazy 族型 A（in-scroller ×7）+ multicol + zero-intersection-area
  **销案**（9 文件收口）。
- 新记录：
  ① **root margin 净速分档未建模**（Chrome 按连接类型 1250/2500px
  两档，本切单值 1250px 近似，FIXME 注记在 helper）；
  ② **image-loading-lazy.html 新可见子测 Fail**（型 D 文件 Timeout
  内子测细化——「In-viewport loading=lazy images load immediately
  but do not block the window load event」，settle 分流后子测可跑
  至报告，窗序语义待勘根）。
- 承前不动（lazy 族余红再分型推进）：型 B base-url ×2；型 C
  in-viewport-dynamic / below-viewport-dynamic / to-eager /
  multiple-times；型 D srcset / below-viewport-load-event Timeout；
  型 E different-crossorigin（avail 元组面）；runner 全量钉内存大
  户二分（s270 挂账①）；dynamic import() 元素级 referrerpolicy 穿
  参（零 corpus face）。

## 下一步候选

1. 型 C eager 变更交互勘根（in-viewport-dynamic / to-eager /
   below-viewport-dynamic / multiple-times——window load 窗序 +
   restart 语义面，run update 门/settle 分流交互，shim 车道承本切
   刀位）。
2. 型 B base-url ×2 勘根（parse-time base URL 冻结面——s225 延步
   重选源机制的 lazy 门交互）。
3. 型 D 文件级 Timeout ×2（image-loading-lazy / srcset /
   below-viewport-load-event——挂死根因定位优先于断言修复）。
