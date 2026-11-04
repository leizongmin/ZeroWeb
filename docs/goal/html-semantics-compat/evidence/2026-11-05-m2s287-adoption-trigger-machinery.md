# M2 s287 — adoption 触发器机具（adoption.html 文件级 Timeout→8P 全绿，family/全树 REGRESSED=0 第二十一个连续完美）

日期：2026-11-05。切片号 s287（承 s286 下一步候选①——adoption 触发器
机具）。adoption.html 8 子测：6 子测 adoptNode 悬挂 + 1 Fail（template
clone 面）+ 1 依赖 append 落地面，文件级 Timeout（s286 勘根移交）。
上一轮 session 已完成施工与 solo/全树验证、被 429 限额打断在 make test
编译段；本轮接续完成 make test 与 family A/B 归档收口。

## 勘根

spec 面：img 的 node document 变化（`document.adoptNode`）是
relevant mutation → update the image data
（https://html.spec.whatwg.org/multipage/images.html#relevant-mutations
「the img element's node document changes」；测试名 "Adopting an image
updates the image data" 即此）。四个机具缺口串联成悬挂链：

1. **快照 on* handler 无派发链**：querySelector 每查询产新
   `_zwParseEl` 实例，`i.onload = fn` 落实例 expando，物化/盖章产的
   副本实例不可达 → load/error 事件无人接（R5254 ontoggle 身份键
   通道同构先例）。
2. **adoptNode 物化产副本**：spec concept-node-adopt 返回同一节点
   （IDL handler 随行、load 可观测）；快照架构物化（t8q R3019 桥
   + `_wcRebuildAsHandle`）必产副本——handler 与 currentSrc 都需
   跨副本寻址接线。
3. **template.content 快照视图缺**：`d.querySelector('template').content`
   恒 undefined（快照无 content 槽）→ 'adopt a cloned img in
   template' 首查询链抛错 = 唯一 Fail 面；contents 在独立 inert
   fragment（host 解析器真实化），`__zw_parse_html_child_nodes` 对
   template 定位恒返空数组。
4. **盖章/物化 img 零 update run**：src 经 R4004 属性回填直写
   host，不走 wrapper setter 钩子；append/adopt 落地面无 relevant
   mutation 触发 → 永挂 load。

## 施工（shim 3 文件，零 Rust）

- `crates/engine/src/js_dom_shim/part02.js`：
  - **`_ensureMutTree` template contents 臂**：读
    `__zw_parse_template_contents`（参数化 outerHTML 深 JSON，
    `_zwMBuildNode` template 分支同数据源）本地构建进 childNodes，
    inert 印章随既有分支同口径——WC-M1 content 视图（part03 工厂
    template 门）数据源就位。
  - **快照 onload/onerror IDL accessor**：R5254 ontoggle 同款身份
    键跨实例通道（`_zwParsedImgH5287` 表，`_zwEvKey` id/sig 键），
    spec event handler IDL 赋值即注册。
  - **currentSrc 读转发 accessor**：查 adoptNode 转移时刻登记的
    twin 表（`_zwParsedImgTwin5287`），未物化恒 undefined（与旧
    expando 同观测，非 img 快照零回归）。
  - **content accessor**：template 快照转发 `_ensureMutTree().content`
    （WC-M1 视图）；非 template 恒 undefined（`<meta content>` 属性
    面走 getAttribute 不受影响）。
- `crates/engine/src/js_dom_shim/part04.js`（R368 盖章点四件）：
  - **`_attrs` 直读回填**：parse-el 查询产物无 getAttributeNames
    （XML 工厂独有），属性快照落 `_attrs` 直读回填 host（R4004
    既有源回填语义同口径）。
  - **快照 on* handler 转 live**：盖章点读 child.onload/onerror，
    有 handler 才建 canonical proxy 转接 IDL setter（二分实证：
    无 handler 子提前建 proxy 破坏派发父链——label 面 20F 栈溢出，
    门控后零副作用）。
  - **盖章 img currentSrc accessor**：消费面转发 canonical proxy
    （spec current request 反射）。
  - **append 尾部盖章 img update 触发**：门控有 src/srcset，s69
    共享选源核 walk（picture/source 上下文经 host 树选源）+
    `__zwImgRunUpdate5277`；不扩 `__zwImgTreeMutated5277` 通用
    点亮（s284 收窄门 allowsAuto——fetch 核缺「同 URL 已可用不重
    取」守卫挂账）。函数域 `_s287ImgStamped` 旗标连接盖章点与
    append 尾读点（同 trap 函数作用域）。
- `crates/engine/src/js_dom_shim/part06.js`（adoptNode 物化面）：
  - **物化 twin 递归转移**：`_t8qMat` 子树逐节点按身份键（id 或
    sig:TAG|cls|outer 前 64——宿主行派生重解析稳定，与
    `_zwEvKey` 同形）寻址 `_zwParsedImgH5287` 转移 handler 到
    twin wrapper IDL setter（入 live 派发），登记 twin 表供
    holder 实例 currentSrc 读转发。时序安全：物化 setAttribute
    触发的 update run 事件为 _defer 异步派，同 tick 转移不丢
    （R5275② 探针先例）；同 sig 孪生键串取首个（s244 架构限度
    同口径）。

## 验证

- **adoption solo**：8/8 Pass EXIT=0（s286 基线：6 悬挂 + 1F，
  日志 s287-adoption-fix.log，md5 7258dc9f）。
- **family A/B**（A = s286-imgfamily-e.log，B = s287-imgfamily-b.log，
  (key,state) 去重 join）：B 唯一键 979P/46F/6T，键级 **FIXED=1 /
  GAINED=7 / LOST=1 / REGRESSED=0**。LOST=1 = adoption 文件级
  Timeout 行消失（被 8 行真子测行替代，良性）；GAINED=7 = 6 悬挂
  子测 + adoption is from appendChild；FIXED=1 = adopt a cloned
  img in template F→P。
- **全树 A/B**（A = s286-fulltree-e2.log，B = s287-fulltree-b.log）：
  B 唯一键 7957P/404F/58T，键级 **FIXED=1 / GAINED=7 / LOST=1 /
  REGRESSED=0 / T-changed=0** 第二十一个连续完美（同口径 LOST/GAINED
  归因如 family）。CHANGED 另含 2 个 forms
  checkable-active-onblur-with-click F→P：s196 域历史抖动键（在档
  99 跑 83P/25F；s285→s286 同键 P→F、s286→s287 F→P 双向翻转），
  归因并行时序噪声非本片位移（s287 只动 img adoption 面，零 forms
  机具耦合）。
- **make test**：9092P/5F——5F 恰为 s283 勘根⑤五红测原键
  （p56/p99s/r2809/r2810/r3276，CSSOM cssText 序列化域兄弟流在途，
  rule 9/10 记账）；对比 s286 9092P/5F 零位移（adoption GAINED 面
  在 runner 通道，workspace 面零回归；日志 s287-make-test.log）。
- 探针页无新增；临时 trace 无残留。

## 挂账更新

- **s286 挂账① adoption 触发器机具销案**（本片收口）。
- **s286 账本计数勘误**：s286 账本 family「73P/14F/4T」/ 全树
  「917P/203F/52T」为转录错误——(key,state) 去重 join 复算实数：
  family A 972P/54F/7T / B 971P/47F/7T；全树 7947P/407F/59T。键级
  定性结论（FIXED=1/GAINED=0/LOST=探针页良性/REGRESSED=0）复核
  无误，勘误注记见 s286 账本附记。
- **保持**：not-broken clientWidth 布局面（渲染流移交）、coalescing
  trickle 管道（网络域）、404-response/historical-progress-event
  待勘根、fetch 核冗余更新守卫候选①（un-gate 前置）、SVG 纵横比
  only 填充、clone density、display:none rect 清零、宿主
  documentElement 物化缺口、sizes-auto 残 4F+1T（各归其域）。

## 下一步候选

1. **404-response-with-actual-image-data / historical-progress-event
   勘根**（s286 挂账④，update-the-image-data 域内余红面）。
2. **fetch 核冗余更新守卫**（候选①保持，spec 忠实 + un-gate 前置）。
3. **SVG 纵横比-only 填充**（naturalWidth 18F，rendering 域跨切）。

## 证据

- `evidence/logs/s287-adoption-fix.log`（adoption solo 8/8）
- `evidence/logs/s287-imgfamily-b.log`（family B；A = s286-imgfamily-e）
- `evidence/logs/s287-fulltree-b.log`（全树 B；A = s286-fulltree-e2）
- `evidence/logs/s287-make-test.log`（全量 make test）
- 日志 md5：adoption 7258dc9f、imgfamily dcd7aa4b、fulltree cbd4f162、
  make-test 3a12e147
