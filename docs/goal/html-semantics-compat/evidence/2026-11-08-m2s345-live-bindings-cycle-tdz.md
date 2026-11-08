# M2-S345：live bindings + import 提升 + 循环 TDZ 三面收口——导出 getter 活绑定发射面，module/ 簇 5 案 F→P

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（头部状态链 s345 段）。
本切片收口 s344 挂账前三项：live bindings 缺失（duplicated-imports-1/2 + late-*/import
提升面）、循环依赖 TDZ ReferenceError（evaluation-error-2）。duplicated-imports-1/2、
evaluation-error-2、late-namespace-request、late-star-export-request 共 5 案 F→P，语料口径
the-script-element 29F→24F（本轮 A 侧 modulepreload unsafe-url flake 落绿，s292 口径）。
spec 依据：
- 导出绑定是活绑定（读时取当前值） https://tc39.es/ecma262/#sec-exports
- import 声明静态提升 https://tc39.es/ecma262/#sec-moduledeclarationinstantiation
- 循环依赖求值： ill-founded cycle 在依赖读未初始化 let/const 导出时抛 ReferenceError，
  求值中止 https://tc39.es/ecma262/#sec-moduleevaluation

## 定谳（本切片新增事实）

1. **prelude 急切求值入口图在循环依赖面破 TDZ + 体序**：s344 入口入册后 registry 对
   prelude 构建可见入口源 → prelude postorder 急切求值把入口源码内联进依赖 IIFE（a↔b 环
   内联展开不闭合）→ 入口体先于依赖体完成、未初始化 let 导出读 undefined 不抛（
   evaluation-error-2 红态 got 体序倒置 5 项）。修复 = 入口图（含入口）不再 prelude 急切
   求值，求值由入口转换体**内联图**承担：体首提升 import 统一走 memoized 发射，内联递归
   本身即入口图后序；循环在入口 IIFE 内闭合——导出 getter 先于提升 import 发射，依赖体经
   活对象读入口未初始化 let/const 自然 TDZ 抛 ReferenceError。prelude 只发射入口图不可达
   余量（entry = None 契约即全量，v8_runtime 助手面同型，契约不变）。
2. **导出活绑定 = 体首 getter，声明位快照接线废止**：两遍扫描——首遍分类（import 子句
   收集提升、本地导出名收集），体首 `Object.defineProperty(_exports, K, { get: () => 本地名 })`
   ，声明形态只发裸声明。duplicated-imports 判据 `set_foo(43); assert_equals(foo, 43)`
   旧快照接线恒读 42；getter 闭包读本地名取当前值。未初始化 let/const 闭包读自然 TDZ
   （定谳 1 的循环面）——TDZ 无需显式建模。var/function 提升先于 getter 可调用。
3. **import 提升发射**：import 子句全部发射于体首（源内声明序），先于普通语句——
   late-namespace-request 判据形 `log.push(m.foo)` 在源内 import 之前不再 ReferenceError。
4. **导入绑定 globalThis getter**：具名/默认导入绑定由 `var alias = exports.src` 快照读
   改为 `Object.defineProperty(globalThis, name, { get: ... })`（安装失败回落既有全局值，
   FIXME；strict 赋值抛 TypeError 近似导入绑定不可赋值面）——依赖侧 `setN(7)` 后消费方
   再读取当前值。命名空间导入仍绑导出对象本身（属性经 getter 活读）。
5. **再导出活委托**：`export * from` / 具名 `export { x } from` 由语句位值拷贝改为委托
   getter（读时穿透依赖导出活对象）；星号不遮蔽具名导出（`hasOwnProperty` 守卫；同名多
   星号首见优先 vs 旧末见覆盖，语料无钉面 FIXME）。late-star-export-request 判据
   undefined→43。
6. **深层菱形 Visited 臂跨 IIFE 不可见 → `__moduleCache` 回落**（返工定谳）：首版全量
   join 抓到 imports.html「Import the same module multiple times」P→F（`Cannot read
   properties of undefined (reading 'from')`）——菱形共享模块首份内联在兄弟 IIFE 作用域，
   Visited 臂 `typeof _mod_{safe}` 守卫依赖 prelude 平铺脚本作用域（旧实现成立），s345 改
   嵌套 IIFE 作用域后守卫不可见回落空对象。修复 = 守卫后追加 `__moduleCache[key]` 回落
   （首份内联先行已完成，命中同一实例——INC_A === INC_AB_A 同一性判据恢复）；自导入/循环
   重入仍走词法外层 `var _mod_`（求值中环缓存未及入账，语义不变）；仍缺席退化空对象
   （与旧占位一致）。修复后全域 join P→F 归零。
7. **祖先失败记账链改由内联发射承担**：s344 的 DFS 祖先链（choice-of-error-3 中间模块
   记账）原挂 prelude 急切发射；prelude 不再求值入口图后由入口体内联发射携带（
   DepInline::Fresh 第三元 = 入口→导入者路径，`path` 递归维护）。evaluation-error 簇
   4P（1/2/3/4）保绿验证记账面等价。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| 两遍扫描发射体 | script-sandbox `es_module.rs` build_module_script | import 子句收集提升体首 + 本地导出名收集；体首导出 getter（`export_getter_line`）；次遍非 import 语句按源序发射，export 声明只发裸声明；`build_dep_iife` 收敛为同一发射体（入口/依赖 TDZ/活绑定面同型） |
| import 提升 + globalThis getter | 同上 transform_import | `install_import_binding_getters` / `import_binding_getter_line`（具名/默认绑定 getter 安装）；import 发射挪体首 |
| 再导出委托 getter | 同上 transform_export | 星号（hasOwnProperty 守卫）与具名再导出 `Object.defineProperty(_exports, …, { get })` |
| 循环闭合 + 菱形回落 | 同上 inline_dep_once / DepInline | `path` DFS 链贯穿 build_module_script/build_dep_iife/transform_import/transform_export；DepInline::Fresh(+ancestors)/Visited(+cache key)；exports_expr 追加 `__moduleCache` 回落 |
| prelude 余量化 | 同上 build_module_runtime_prelude | 入口图（含入口）postorder 可达集不再急切求值；余量急切求值维持全量预热契约 |
| 单测 | 同上 tests | +5 s345 行为钉（活绑定/提升/循环 TDZ/星号委托/发射形态）+1 深菱形缓存回落钉 `test_deep_diamond_visited_cache_fallback_s345`；s344 prelude 发射断言迁入口体；6 处 build_module_script 调用点 path 参数回填 |

## 验证（429 打断后由接续 session 重锚定——原稿引用的 s345-{a,b,b2}-domain.log 未落盘，
## 全部数字改以现存日志重跑证实；结论与原稿一致，A 侧 flake 状态差异单列）

- join（the-script-element 子域、子测试键级 stable 集语义——`LC_ALL=C`、`grep -a`、
  (key,state) sort -u）：A = `s344-domain-final.log`（s344 收口 scoped 跑，键级
  **1087P/32F/8T**——s344 头部 1088P 为原始行数口径，含 1 重复 Pass 键
  execution-timing/149，`sort | uniq -d` 实证）；B2 = `s345-b2-domain-rerun.log`
  （本 session 重跑，误未带 FILTER=全 html-semantics 域快照 **8111P/312F/45T**——
  首份全域基线，其 the-script-element 子集 **1093P/26F/8T** 用于 join）。
  - det F→P=**6**：5 目标案逐案吻合（duplicated-imports-1/2、evaluation-error-2、
    late-namespace-request、late-star-export-request）+ modulepreload-referrerpolicy
    unsafe-url flake 1 案（A 侧红 B2 绿；s292 口径 ±1，原稿记 A 侧落绿方向相反）。
  - det P→F=**∅**——菱形修复（定谳 6）域级证实：imports.html 5 子测试 A/B2 同绿
    （含「Import the same module multiple times」，首版 WIP 曾 P→F 面）；T=8 同集
    零位移（execution-timing 063/fetch-src empty 双案/load-error-events-3/
    credentials setTimeout 变体/code-cache-base-url/instantiation-error-8/
    script-crossorigin-network 在册长尾）。
  - 语料口径：**29F → 24F**（32F − 5 目标 − 1 flake = 26F；自建 probe12/13 恒红
    2F 扣除承前序切片口径）。
  - solo 四轮 b log（16:13，首版 WIP 二进制、菱形修复前）在盘：
    `s345-b-{evaluation-error,cycle-tdz,duplicated-imports,evaluation-error-2}.log`；
    菱形修复后 solo 未重跑，域级 join 独立证实同结论。
- 单测（test-guard 包裹，本 session 重跑）：script-sandbox **417P/0F**（411 基线
  +5 行为钉 +1 菱形钉；`s345-unit-rerun.log`）。
- clippy（script-sandbox --all-targets -D warnings）净；rustfmt --all --check 净
  （含跨流五文件机械 fmt 修复独立提交 3f2d80d27——net/wpt-runner/layout-engine
  并行流 fmt 债阻塞共享 make test 门禁，碰头记入控制文档）。
- make test workspace：**3817P/6F**——6 案 engine js_dom_bridge **既有回归，非本切片
  引入**（WIP stash 后 HEAD 同红实证）。bisect 首坏归因：t8k websocket 对
  （error-nonterminal + event-flow-open-message-close）= `ba03fd7ff`（46号 M3 ws
  interfaces——err wire 即派改 close 统一派，t8k 钉旧「err 非终结即派」契约，emit
  无 close wire → error 永不派）；img deferred-settle-r5275(c2)/svg
  settle-density-r5277/fetch filtered-response-type-matrix/iframe SW messages 四案 =
  `f021e11fb`（46号 M3 redirect 七簇 part01.js 重构，机制待勘）。两提交各自 A/B
  分母为 runner corpus，未含 engine lib 单测。s345 面（script-sandbox 单测 + runner
  语料域 join）全绿；碰头按 run-rules #9 记入控制文档挂账，不在本切片硬解。

## 挂账（承继 + 更新）

1. **live bindings 缺失——销案**（duplicated-imports-1/2、late-namespace-request、
   late-star-export-request 收口）；**循环 TDZ——销案**（evaluation-error-2）。
2. **包装级 parse 原子性（per-record 归因缺失）**：choice-of-error-1 + compilation-
   error-2——坏依赖源内联致整单元 parse 失败无法按模块记录归属；需沙箱 compile-only API
   或静态依赖改走宿主取回 eval 面（s87 三态记账既有）。承 s344 挂账③原样。
3. **内联 module 错误行号**：evaluation-error-5 判据 lineno 16 得 0——错误事件行列
   传播面（源位 → 错误对象注入）。承 s344 挂账④原样。
4. **导入绑定 globalThis getter 安装失败回落**（同名全局不可配置如 `window.location`）
   ——bare 读回落既有全局值，FIXME 在码；语料无钉面。
5. **同名多星号首见优先 vs 旧末见覆盖**——spec 对齐改首见，语料无钉面 FIXME 在码。
6. s341-s343 既有承继原样：iframe 子文档 currentScript、window.onerror window.event、
   credentials setTimeout 变体、engine fetch 完成驱动 ASAP 臂、renderer 产品路径
   deferred parity、execution-timing 041/056/063/068 文档写入面、fetch-src empty 双案、
   concurrent-failed-imports 双案、tla/deferred tentative 三案（域长尾 24F 在册面）。
7. **make test workspace 6 案 engine 既有回归**（本切片归因定谳，非本切片引入）：
   t8k websocket 对 = `ba03fd7ff` 契约改（err 只标记、close 统一派 error；t8k emit
   无 close → 永不派，需按新契约迁钉或恢复非终结即派臂——spec 面待勘）；img/svg/
   fetch/iframe 四案 = `f021e11fb` part01.js redirect 重构碰撞（img c2 面=「0;
   checkpoint _defer 派 load」handler 未跑；机制待勘）。修复须与 46号 M3 上下文
   合流（该流 runner 基线落账后），或单开切片；修复前共享 make test 门禁维持
   3817P/6F 红态记录。
8. 下一步候选：s344 挂账②包装级 parse 原子性（choice-of-error-1/compilation-error-2
   双案 + concurrent-failed-imports 同族勘向）或 evaluation-error-5 行号传播面（单钉
   面小）——下轮先勘 concurrent-failed-imports 簇聚归因再定；engine 6 案回归修复
   （挂账 7）视 46号 流排期插队或并行为 s346 备选。
