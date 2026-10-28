# M2 s124 · link rel=modulepreload 取回面收口（shim 触发臂 + 端点仿真 ×2 + classic 动态 import() module map 记忆化——modulepreload 家族 6 行→15/15 全 Pass）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s123（FormData 条目构造收口）

## 入场定刀位

取 s123 下一步首选：module 网络面挂账项勘根——modulepreload referrerpolicy
族 / module-import-referrer TLA / dynamic import() with 透传 / classic 外链
SRI，逐一辨 shim/runner/host 契约面。

## 勘根定谳（四项逐一，探针页 gitignored corpus 验后删）

1. **modulepreload referrerpolicy 族（三页 15 测）——三面缺口，本切片收口**：
   ① runner fetcher 无 `echo-referrer.py` 端点仿真（判据回体通道缺）；
   ② 引擎 shim 无 link rel=modulepreload 取回面（插入后不抓模块、无
   load/error 事件、不播种 module map）；③ 元素级 referrerpolicy 无通道
   （fetcher 契约 `(page_url, src)` 无策略形参）。另勘明 cross-origin 页
   堵点**不是 https 传输**——`strip_wpt_host_prefix` 已剥
   `www1.wpt.test:8444` 别名，实缺 `script.py` 端点仿真。
2. **module-import-referrer（TLA）——挂账保留，根因精确化**：入口模块
   TLA 可用（`compile_module_script` 对含 await 源 async IIFE 包裹），
   **依赖模块 TLA 不可用**（`build_dep_iife` 同步 IIFE → SyntaxError
   "await is only valid in async functions..."）——深转换架构面，挂账。
3. **dynamic import() with 透传——挂账保留**：语料所需页全部是 css-module
   页（与 CSSStyleSheet/CSSOM data-module 面复合），单修 import 透传不翻绿。
4. **classic 外链 SRI——FIXME 保留**：本 suite 无语料页需要。

## 施工（engine shim part01.js + part03.js / script-sandbox v8_runtime.rs / runner testharness.rs）

1. **shim 取回面**（part03.js `globalThis._zwModulePreloadMaybeStart`）：
   门序 = nodeType 1 → `_zwScriptPrepConnected`（script prepare 同门——
   innerHTML 重建产物解析代理 isConnected 假阴性经父链爬升复核）→ rel
   whitespace-split 含 modulepreload → href 非空 → `__zw_fetch_script`
   已注册（未注册嵌入方 no-op）。元素级 referrerpolicy 合法 label 以
   `__zw_lp` 内部查询参数携入（fetcher 侧验证剥离；非法 label 不携带——
   spec invalid 值顺延文档策略）。取回成功：体全局求值 + 以
   `__zw_parse_url` 解析的绝对 URL 键播种 `__moduleCache`（
   `=== undefined` 守卫先到者胜）→ **异步**派 load/error。
2. **同步内联取回（时序勘根后定刀）**：初版 setTimeout(0) 延后取回——
   探针 mp5（忠实复刻 inline 页）实证延后回调在断言执行前从未轮转
   （ZW 文档脚本 + testharness 起跑间不泵 timer 队列），body 的
   `Promise.all([import()×4])` 先行重取回，以文档策略覆写 referrers，
   no-referrer 臂翻车。终版取回+播种同步内联（插入时生效，匹配页面
   契约），事件保持异步派发（spec 事件语义；onload 赋值 append 前后
   均可见）。
3. **触发臂**（part01.js `_mo_notify` childList 汇流点 s111 臂）：顶层
   LINK 分支 + `_walk111` 下降 LINK 分支（SCRIPT 臂同型，inclusive
   descendants 文档序）。innerHTML 重建产物为 body-context 平铺解析代理
   （`<head>`/`<body>` 标签丢弃），顶层迭代即命中；连接门经
   `_zwScriptPrepConnected` 父链爬升（重建产物 parentNode 已 R136/R304
   重指活代理）。
4. **classic 上下文动态 import() module map 记忆化**（v8_runtime.rs
   `dynamic_import_callback`）：成功条目读 `__moduleCache[resolved]`
   命中即以缓存值 resolve（模块体恰好一次求值——fetch-a-single-module-
   script 命中复用）；成功条目写同表同键（表缺席兜底自建）；失败三态
   不入缓存（取回失败逐次新 TypeError——s87 口径保持）。读/写均
   best-effort（失败回落原行为）。与 preload 播种同表同键（绝对 URL），
   import() 命中播种条目不再重求值——referrers 先写者胜。
5. **runner 端点仿真 ×2**（testharness.rs fetcher）：`echo-referrer.py`
   （`window.referrers["<uid>"] = "<Referer>";` 回体；策略链 = `__zw_lp`
   → pipe → 文档策略 → spec 默认，逐臂过滤非法 label）；`script.py`
   （非空 JS 回体——引擎 modulepreload 面以空体承载取回失败）。
6. **pin ×2**：runner `modulepreload_endpoint_sims_s124`（__zw_lp 通道 /
   非法 label 顺延文档策略 / uid 记账键 / script.py 经 www1:8444 别名
   剥离）；script-sandbox `test_native_dynamic_import_module_map_
   memoization_s124`（persistent context——非持久模式每 execute 新
   context 钉不到命中面，首版钉自身假阳性已修正：成功恰好一次求值 /
   预播种条目命中不重求值 / 二次 import 命中缓存 / 失败三态不入缓存 +
   resolver 逐调用取键序）。

## 验证

- 定向 `FILTER=modulepreload`：**15/15 Pass**（referrerpolicy 7 + inline 1
  + cross-origin 7）。
- **全量 A/B**（/tmp/s123-clean-full.log → /tmp/s124-full.log，逐行
  status join）：家族基线仅 6 行（4 Fail + 2 Timeout——其余 9 子测试
  因页面挂起从未完成注册不打印）；现 15/15 Pass。恰 **+15P / −4F /
  −2T，全部落在 modulepreload 家族，真翻红 = 0**（6653P/1145F/105T →
  6668P/1141F/103T，总行数 +9 = 此前未完成注册的 9 子测试现完成打印，
  总量口径变化已记账）。
- script-sandbox lib **377/0**（+1 pin）；engine lib **3570/0**；
  wpt-runner 全量 **256/0**（+76 ignored）。
- guard 内 clippy（script-sandbox + wpt-runner + engine，-D warnings）
  零告警（needless_borrow 两处已修）；fmt 净；shim 拼接（真实装配序
  part01+01b+02..08d）node --check 过。
- 语料零改动（探针页 zw-probe-s124-tla/tla2/mp/mp2/mp3/mp4/mp5 +
  dep1/dep2.js 验后删除；调试追踪临时桩已全部移除）。
- 渲染面零依赖——reftest/product-smoke 未跑。

## 挂账（本切片更新）

- **modulepreload referrerpolicy 族收口移出候选**；**modulepreload
  inline 重建路径收口**（同步内联时序刀）。
- **module-import-referrer TLA——根因精确化保留**：入口 TLA 可用、依赖
  模块 TLA 不可用（`build_dep_iife` 同步 IIFE）——依赖图 async 语义是
  深转换架构面（module transform 管线重构级）。
- **dynamic import() with 透传保留**：语料页全部与 css-module 复合。
- **classic 外链 SRI 保留 FIXME**：无语料需求。
- 新勘彼面（不挂账，记录）：`__zw_fetch_script` 每调用现取（无缓存层）
  ——preload + import 双取回端点各命中两次，判据面无观察（referrers
  先写者胜 + import 不重求值）；如未来语料断言取回次数需宿主层取回
  记忆化。
- 承袭不变：form-submission-0 219F、写流执行身份盖章断链 FIXME、
  part02 setAttribute 绕 notify 臂、execution-timing 构成面、041
  nested write、lazy-loading 26F、meta color-scheme 21F、css-module
  corpus 门（待决）、www2 可达性、script-crossorigin-network cookies、
  redirect-relative-urls、export-from with FIXME、143 write、defer 多
  写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索
  引访问、live 树 document 级逗号列表 qSA 根缺口；
  **待决（须用户拍板）**：css/text 数据模块语料资源缺失、parser form
  element pointer 4F 缓行（2026-10-23）。

## 下一步候选（s125）

承袭挂账序复核（form-submission-0 186F POST 传输契约勘根 / 
execution-timing 构成面 056/063+051/052 单根）/ 或 meta color-scheme 
21F 跨流面评估（style-system 协同）。
