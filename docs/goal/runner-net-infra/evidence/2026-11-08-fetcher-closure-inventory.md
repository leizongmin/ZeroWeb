# 三 fetcher 闭包迁移清单（拓扑 + 约束 + 普查方法）

日期：2026-11-08。用途 = M3 迁移末段（四 fetcher 闭包臂同框架分表迁移，
master 下一步挂账项）的执行前置档；本档只记拓扑与约束，逐臂普查由执行
切片按 §三方法落自己的证据档。行位基于切片一 slim 后
testharness.rs（15,630 行态）。

## 一、闭包拓扑（slim 后行位）

| 闭包 | 位置 | 签名/状态 | py 臂形态 |
|---|---|---|---|
| `wpt_data_script_fetcher_with_stash` | testharness.rs:5943 | `(wpt_root, doc_referrer_policy, serve_custom_stash, cred_cookies) -> Option<ScriptSourceFetcher>`；闭包体 ≈5958–6732 | 臂较散：host 前缀剥离**前置**（check-cookie.py 臂在其前自带 host 解析——www2 cross 臂不进通用剥离），余为表达式链 + 少量 if 臂（delayed-modulescript.py、.json intent、css/txt hop） |
| `wpt_data_service_worker_script_fetcher` | testharness.rs:6733 | `(wpt_root) -> Option<ServiceWorkerScriptFetcher>`；案级 `ServiceWorkerFixtureState` Mutex 新建于闭包外 | `service_worker_fixture_path/query` 预解析后 if/else 表达式链（invalid-chunked-encoding ×2、malformed-worker.py 参数臂族等） |
| `wpt_data_image_fetcher` | testharness.rs:5631 | `(wpt_root) -> Option<ImageSourceFetcher>` | 非路由表（image 资产定位），预计不迁注册表 |

## 二、迁移硬约束（stash 恒等）

- **案级 Arc 共享不可断**：script fetcher 的 `serve_custom_stash` /
  `cred_cookies` 与 fetch 桥闭包（现 PyEndpointCtx）共享同一 per-case
  Arc（testharness.rs 5926、8144、8149 注记锚）——迁移后 fetcher 分表
  必须继续接收 PyEndpointCtx 构建点传出的同一 Arc，不得另建（否则
  serve-custom-response 的 stat/set 与 check-cookie 的 cookie 写入跨
  通道失联，页面 fetch() 与模块取回两读法漂移）。
- `wpt_data_script_fetcher`（5936，无 stash 包装）为单测回调面（生产
  路径走 `_with_stash`）——迁移保持该分层。
- service-worker fetcher 的 `ServiceWorkerFixtureState` 为其独占案级
  状态（fetch 桥不共享）——可随其分表整体迁为独立 ctx，无共享面。
- fetcher 契约无 CORS 层（5869 注记）：Referer/policy 头不建模的简化
  面随臂注释保留，迁移不改语义。

## 三、逐臂普查方法（执行切片用）

- script fetcher：`awk 'NR>=5958 && NR<=6732'` 段内按
  `(clean|hop_clean)\.(==|starts_with|ends_with)` + 注记锚（s338/
  s249/模块类型）双轨清点——表达式链臂（if/else 赋值形）不计入
  `^\s+if ` 口径，需人工按臂注释分界。
- service-worker fetcher：6733 起同法（`clean.ends_with` 链 + params
  参数臂族）。
- 迁移形态：fetcher 分表沿用主表 `PyRoute` 面同构（谓词 + handler +
  保序），但**独立表实例**（`py_script_fetch_routes()` /
  `py_sw_fetch_routes()`）——不并入主 `py_routes()`（语义域不同：
  fetcher 契约入参 `(page_url, src)` 无 method/headers 面）。
