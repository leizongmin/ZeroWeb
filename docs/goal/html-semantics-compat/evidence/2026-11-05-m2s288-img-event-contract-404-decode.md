# M2 s288 — img 事件契约（Event 实例化 + historical non-element handler）+ 404 状态解码门（双面 F→P，family/全树 REGRESSED=0 第二十二个连续完美）

日期：2026-11-05。切片号 s288（承 s287 下一步候选①——404-response-with-
actual-image-data / historical-progress-event 勘根，s286 挂账④）。双面
同域施工：historical-progress-event.window.js 1F + 文件悬挂（子测 A
语料缺资产）；404-response-with-actual-image-data.html 1F。

## 勘根

三面独立根因，同属 update-the-image-data / img 事件域：

1. **historical 子测 B「onloadend is not exposed」**：
   `document.body.onloadend` 应为 undefined，实返 null。机制位 =
   R2933 element 级 generic on* get trap 分支（part04 ~5971）的
   `|| null` 回落——「on+小写字母」无白名单泛化把 spec 已废止的
   non-element handler 名（onloadstart/onprogress/onloadend，仅
   XHR/FileReader 等非 Node 目标持有）也当 IDL handler 处理。
2. **historical 子测 A 悬挂（文件级 Timeout）**：双层。①语料缺
   `/images/rrgg-256x256.png`（wpt-data images/ 仅 11 文件，s227
   同型资产缺口）——async_test 等 img load 永不 done。②事件形态：
   断言 `e instanceof Event && !(e instanceof ProgressEvent)`，img
   派发走裸 `_makeEvent`（产物原型 Object.prototype），instanceof
   Event 恒 false（Event 构造器形态才有 prototype 挂载，part05
   R109）。子测 B 断言 body/window onloadend undefined（window 侧
   已天然满足）。
3. **404-response**：img fetch 臂（part03 ~3464）`if (!resp.ok)
   → _imFail` 把 HTTP 状态门槛进 load 判决——spec update the
   image data 只门槛网络错误，非网络错误响应按体解码决断（404 +
   可解码体亦 available）。语料 .py 端点缺失（wptserve handler 不
   入静态语料）→ 缺文件 404 + 空体 → onerror unreached Fail。
   上游 .py（wpt pin 1eecaa0c30）行为 = 固定返 404 + image/png 体
   （16x16 PNG 140B base64 内嵌）。fetcher wire（`__zwfr:` 前 4
   字段含 status）本就携带状态通道，s267「无 status 通道」注记指
   img 机具的历史判决面非 fetcher 契约。

## 施工（shim 3 文件 + runner 1 文件 + 语料 1 资产）

- `crates/engine/src/js_dom_shim/part01.js`：
  - **`_ZW_NON_ELEMENT_ON` / `_ZW_NON_ELEMENT_ON_MEDIA` 表**（
    `_ZW_BODY_FORWARD_ON` 邻位）：三名 historical non-element
    handler（loadstart/progress/loadend）；media 豁免子集 = 
    loadstart/progress——HTMLMediaElement 仍有此二 IDL（part06
    `_zwMediaFire` 经 `_listenerStore` 派发，全排会断 media 面；
    corpus media-elements 族实证 `v.onloadstart =` 形态在用）。
- `crates/engine/src/js_dom_shim/part04.js`：
  - **get trap 排除分支**（R107 tag 读取后）：命中表且非 media
    豁免 → expando 语义（hasOwnProperty 命中返存储值，未设返
    undefined——R3042 同口径），不再 `|| null`。
  - **set trap 排除分支**（R107 tag 读取后）：命中表且非 media
    豁免 → 落普通 expando（`_expando[key]`），不入 `_onHandlers`
    /`_listenerStore`（事件派发不触达 = unreached 断言面）。
  - **sweep/drain img 事件派发 4 处转 `__zwImgMakeEvent5277`**
    （sweep load ×2 / sweep error / pending drain）。
- `crates/engine/src/js_dom_shim/part03.js`：
  - **status 门改判**：`!resp.ok` → `!resp.ok && !resp.status`——
    网络错误判据 = status 0（`__zw_fetch_error:` wire / abort /
    opaque 过滤响应均 ok:false + status:0 同形；`__zwfr:` wire 的
    真实 HTTP 响应恒 ≥200）。HTTP 状态错误放行进 blob → 
    createImageBitmap 解码决断（解码失败仍 reject → _imFail）。
  - **`__zwImgMakeEvent5277` helper**（载入期赋值，`_makeEvent`
    产物挂 Event.prototype）+ 运行核内派发 3 处转用（直派 load/
    error、CSP error、null-selection error）。
- `tests/wpt-runner/src/testharness.rs`：
  - **`ZW_S288_404_PNG` 常量**（16x16 PNG 140B，上游 .py 
    decodebytes 内嵌体逐字拷贝）+ **404-response 端点臂**（404
    状态 + image/png 体——上游 wptserve handler 仿真，wpt pin
    1eecaa0c30）。
- 语料：`tests/wpt-runner/wpt-data/images/rrgg-256x256.png`（上游
  wpt images/ 原件，github.com sparse clone，120B）+
  `scripts/update-wpt-data.sh` 已知缺口清单挂账（含 .py 端点为
  runner 仿真臂、无语料资产注记）。

## r1 回归与归因（重要教训）

family r1（s288-imgfamily-b.log）**REGRESSED=15**：eager/invisible/
delay-load/in-scroller ×8/multicol/relevant-mutations-lazy ×2/
relevant-mutations/svg-density 全 P→F。根因 = helper 初版定义在
`__zwImgRunUpdate5277` 运行核内——part04 sweep/drain 在**无任何
update run 的页面**上先行执行（parse 期 src 属性回填不走 setter
钩子不触发 run，lazy 未跑图 sweep 直派），`globalThis.
__zwImgMakeEvent5277` 缺位 throw 撕裂 sweep 单 try 包裹体（一抛
全弃），img load 全丢失 → 窗序/装载断言面崩。二分：proto 挂载
摘除仍红（bisect1，排除 Event.prototype 嫌疑）→ helper 提升至
载入期作用域（bisect2 eager Pass）→ family r2 REGRESSED=0。
**教训：被载入期机具（sweep/drain）消费的 helper 必须在载入期
作用域赋值，不可依赖运行核首次执行**（经验沉淀见
docs/learnings/）。

## 验证

- **historical solo**：2/2 Pass（日志 s288-historical-fix.log，
  md5 204f7f3e）。
- **404 solo**：1/1 Pass（日志 s288-404-fix.log，md5 e9ea462f）。
- **eager 二分面**：bisect1 Fail（proto 摘除，md5 2047ea60）→
  bisect2 Pass（helper 提升，md5 7e8b1197）。
- **family A/B**（A = s287-imgfamily-b，B = s288-imgfamily-c，
  (key,state) 去重 join；注：比对脚本 GAINED/LOST 标签互换——
  GAINED 段实为 A-only、LOST 段实为 B-only）：B 唯一键
  **982P/44F/5T**，键级 **FIXED=2 / GAINED=1 / LOST=1 良性 /
  REGRESSED=0**。GAINED=1 = historical 子测 A Pass 行（A 侧文件
  悬挂无行）；LOST=1 = historical 文件级 Timeout 行消失（被 2
  真子测行替代）。ΔP+3/ΔF−2/ΔT−1 守恒闭合。
- **全树 A/B**（A = s287-fulltree-b，B = s288-fulltree-b）：
  B 唯一键 **7960P/402F/57T**，键级 **FIXED=2 / GAINED=1 /
  LOST=1 良性 / REGRESSED=0 / T-changed=0** 第二十二个连续完美
  （归因同 family；Δ 守恒闭合同式；日志 md5 0f821156）。
- **make test**：9104P/5F——5F 恰为 s283 勘根⑤五红测原键
  （p56/p99s/r2809/r2810/r3276，CSSOM cssText 序列化域兄弟流
  在途，rule 9/10 记账）。P 总数 +12 对 s287 9092P 为 cargo
  套件截断顺序差（make 在首败 crate 截断：本轮 77 套件先行
  vs s287 76 套件——判定套件 js_dom_bridge 3727P/5F 两轮
  逐键同红，非测试结果位移；日志 s288-make-test.log）。
- 探针页已删（probe-s288-onend.html）；临时二分仅 helper 内注释
  级改动，无残留。

## 挂账更新

- **s286 挂账④（404-response / historical-progress-event 待勘根）
  销案**（本片双面收口）。
- **口径注记（非挂账）**：part04:4273 attr-recompile 路径（inline
  on* 属性编译）未接入 `_ZW_NON_ELEMENT_ON` 排除——content
  attribute 形态（`<body onloadend="...">`）spec 上亦应无效，现
  仍编译注册；本片双目标测试不涉此形态，留待后续切片按需收口。
- **保持**：not-broken clientWidth 布局面（渲染流移交）、coalescing
  trickle 管道（网络域）、fetch 核冗余更新守卫候选①（un-gate 前置）、
  SVG 纵横比-only 填充、clone density、display:none rect 清零、宿主
  documentElement 物化缺口、sizes-auto 残 4F+1T（各归其域）。

## 下一步候选

1. **fetch 核冗余更新守卫**（s287 候选②保持，spec 忠实 + un-gate
   前置）。
2. **SVG 纵横比-only 填充**（naturalWidth 18F，rendering 域跨切）。

## 证据

- `evidence/logs/s288-historical-fix.log`（historical solo 2/2）
- `evidence/logs/s288-404-fix.log`（404 solo 1/1）
- `evidence/logs/s288-eager-bisect1.log` / `s288-eager-bisect2.log`
  （r1 回归二分：proto 摘除仍红 / helper 提升转绿）
- `evidence/logs/s288-imgfamily-c.log`（family B 正式证据；A =
  s287-imgfamily-b；r1 回归日志 s288-imgfamily-b 留档）
- `evidence/logs/s288-fulltree-b.log`（全树 B；A = s287-fulltree-b）
- `evidence/logs/s288-make-test.log`（全量 make test）
- 日志 md5：historical 204f7f3e、404 e9ea462f、bisect1 2047ea60、
  bisect2 7e8b1197、imgfamily-r1 dceb2159、imgfamily 33a49468、
  fulltree 0f821156、make-test 781d0da9
