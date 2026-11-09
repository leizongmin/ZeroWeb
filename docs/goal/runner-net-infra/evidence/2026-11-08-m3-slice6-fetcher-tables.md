# M3 切片六：fetcher 闭包三分表迁移——逐臂普查 + 分表设计

日期：2026-11-08。前置 = [清单档](2026-11-08-fetcher-closure-inventory.md)
（拓扑与硬约束）+ 切片五档（主表迁移态）。本档 = 切片六执行前置：
行位基于当前树（clippy 收敛后 HEAD `548cfa3d5`，testharness.rs 15,885
行态；清单档行位基于 15,630 态，已漂移 ~50 行，以函数名定位为准）。
分两子批执行：**6a service-worker fetcher 分表** → **6b script fetcher
分表**，各守机械等价 + 全量 A/B。

## 一、逐臂普查（当前树实测）

### script fetcher（`wpt_data_script_fetcher_with_stash`，testharness.rs:5975，
闭包体 5984–6681，非纯 if 链——前置步骤 + 早退臂 + 表达式链混交）

链序普查（执行序，注释锚随臂）：

| # | 臂 | 判据（现码原形） | 迁移形态 |
|---|---|---|---|
| P0 | check-cookie.py（s338） | `src.contains("check-cookie.py")`——**host 剥离前**，自带 authority 解析 + cred_cookies | **留守 dispatch 前置**（src 键非 clean 键，且必须在路径解析前；不进表） |
| P1 | 路径解析 + clean 归一 | `strip_wpt_host_prefix` 四分支 | 留守前置（结构性） |
| P2 | MIME 派生 | `mime_from_pipe`/`mime_from_sidecar`（src query + `{clean}.headers` sidecar） | 留守前置；值经 env 传 handler |
| P3 | json_intent 计算（s107/s115） | `.json` 后缀 / serve-with-content-type.py 文件名——**计算含早退 Err**（fn/ct 缺失 6138） | 留守前置（其 Err 时序在 module-fetch-tester 臂前，保序不可后移）；值经 env 传 |
| T1 | module-fetch-tester.py（s346） | `clean == "html/…/module-fetch-tester.py"` | 表行 Exact（委派 `module_fetch_tester_serve`） |
| T2 | delayed-modulescript.py（s344） | `clean.ends_with("/module/resources/delayed-modulescript.py")` | 表行 Suffix |
| T3 | serve-custom-response.py（s267） | `clean == "html/…/serve-custom-response.py"`，consume `__zw_mtype` 标记 + serve_custom_stash | 表行 Exact |
| T4 | referrer-checker css/json/text（s268） | `clean.contains(…css-module/…) \|\| contains(…json-module/…) \|\| contains(…text-module/…)`（三路 or） | 表行 Custom（谓词 fn 原式承载） |
| T5 | `__zw_mtype` intent（s249） | src query 标记 `__zw_mtype=<css\|text>`（非 clean 键），体含 redirect.py ≤4 跳环 | 表行 Custom（src 键谓词） |
| T6 | json_intent 消费（s107/s115） | `json_intent.is_some()`——**env 键非路径键** | 表行 Custom（env 谓词） |
| T7 | load-error-events.py（s250） | `Path::file_name()==OsStr("load-error-events.py")`（basename 全等，非 ends_with——谓词逐字保留 file_name 比较式） | 表行 Custom |
| T8 | referrer-checker module（s116） | `clean.contains("the-script-element/module/resources/referrer-checker.py")` | 表行 Contains |
| T9 | echo-referrer.py（s124） | `clean.contains("preload/resources/echo-referrer.py")` | 表行 Contains |
| T10 | script.py security-features（s124） | `clean == "common/security-features/subresource/script.py"` | 表行 Exact |
| R1 | scripty 判定链 | `let scripty = if pipe {…} else if sidecar {…} else if imported_script.py {…return…} else {扩展名}`——**imported_script.py 臂嵌在表达式链内且被 pipe/sidecar 分支遮蔽**（pipe 有值时该臂不执行——行为语义，不可提升为表行） | **留守尾段**（遮蔽序不可重排） |
| R2 | .any.js worker 包装 | `.any.worker.js`/`.any.js` 后缀 + `synthesize_any_worker_source` | 留守尾段 |
| R3 | 通用读 + `{{GET[]}}` 替换 | 残余回退 | 留守尾段（= 主表静态回退尾同位） |

- handler 返回 `Result<String, String>`（fetcher 契约本为 Result——臂体
  `return Ok/Err` **零包装变换**，纯 dedent + ctx 名重绑定；比主表迁移
  少一层 Some 包装，变换面更小）。
- 表行谓词序 = 链序 T1→T10（保序硬门同主表）；R1 遮蔽语义是本切片
  最重的行为陷阱：imported_script.py 若提升表行会在 pipe= 有值时提前
  截答——普查即定性留守，不迁。

### service-worker fetcher（`wpt_data_service_worker_script_fetcher`，
testharness.rs:6781，闭包体 6786–7257，纯 if/else 链 + 公共尾）

前置：`service_worker_fixture_path`/`_query` 两步（Err 面在全部臂前）；
`let mut headers = Vec::new()` + 公共尾 `Ok(HttpResponse{…})`。

| # | 臂 | 判据 | 迁移形态 |
|---|---|---|---|
| S1 | invalid-chunked-encoding(.with-flush).py | `ends_with("/resources/invalid-chunked-encoding.py") \|\| ends_with("…-with-flush.py")` | Suffixes ×2 → Err |
| S2 | malformed-worker.py | `ends_with("/resources/malformed-worker.py")`，params 九模式 | Suffix |
| S3 | mime-type-worker.py | `ends_with("…/mime-type-worker.py")`——**不 return**：推 Content-Type 后 body= `Vec::new()` 落公共尾 | Suffix → **Body 臂** |
| S4 | bytecheck-worker.py | `ends_with("…/bytecheck-worker.py")`，state.bytecheck_visits | Suffix |
| S5 | bytecheck-worker-imported-script.py | 同上 imported 面，自定义 HttpResponse（含 ACAO 头） | Suffix |
| S6 | update-registration-with-type.py | `ends_with("…/update-registration-with-type.py")`，key/classic_first | Suffix |
| S7 | test-request-mode/headers-worker.py | `ends_with(…mode…) \|\| ends_with(…headers…)`（headers 臂内含 visit 子判） | Suffixes ×2 |
| S8 | update-max-aged-worker.py | `ends_with("…/update-max-aged-worker.py")`，test 名模式机 | Suffix |
| S9 | update-max-aged-worker-imported-script.py | 同上 imported 面 | Suffix |
| S10 | update-worker-from-file.py | `ends_with("…/update-worker-from-file.py")`，Key/First/Second | Suffix |
| S11 | update-during-installation-worker.py | `ends_with("…/update-during-installation-worker.py")` | Suffix |
| S12 | update-nocookie-worker.py | `ends_with("…/update-nocookie-worker.py")` | Suffix |
| S13 | update-missing-import-scripts-main-worker.py | `ends_with(…main-worker.py)` | Suffix |
| S14 | update-missing-import-scripts-imported-worker.py | `ends_with(…imported-worker.py)` | Suffix |
| S15 | 404.py | `ends_with("/resources/404.py")` → Err | Suffix |
| S16 | import-scripts-404-after-update(-plus-update).js | `ends_with(…plus-update…js) \|\| ends_with(…after-update.js)`，含 `-plus-update-` contains 子判——**不 return**：推 Content-Type 后 body 表达式落公共尾 | Suffixes ×2 → **Body 臂** |
| S17 | import-scripts-version.py | `ends_with("…/import-scripts-version.py")` | Suffix |
| S18 | import-scripts-get.py | `ends_with("…/import-scripts-get.py")`，output/msg | Suffix |
| S19 | import-scripts-echo.py | `ends_with("…/import-scripts-echo.py")` | Suffix |
| S20 | redirect.py | `ends_with("/redirect.py")`，递归解析 + 目标面 | Suffix |
| S21 | update-worker.py | `ends_with("…/update-worker.py")`，visited×mode 状态机 | Suffix |
| R | 静态尾 | else：读文件 + `{{domains[www1]}}`/`{{ports[https][0]}}` 替换 + cache-abort/any-js 合成 | 留守 dispatch 尾段 |

- **Body 臂语义**（S3/S16 与公共尾的耦合）：现码两臂不 return——
  向共享 `headers` 推头、产出 body 表达式，汇入公共尾
  `Ok(HttpResponse{status 200, headers, body, url, redirect 0})`。
  迁移形态：handler 返回 `Result<SwArmOutcome, String>`，
  `SwArmOutcome = Complete(HttpResponse) | Body(Vec<u8>)`；`headers`
  经 `&mut Vec<(String,String)>` 形参穿入（臂内 push 逐字保持），
  dispatch 对 Body 臂按公共尾形状收口。
- handler 返回 `Result`（非 Option）——臂体 `?` 算子（`state.lock()
  .map_err(…)?` / `params.get(…).ok_or_else(…)?` / `read_corpus_to_string
  (…)?`）**逐字保持**，`return Ok(x)` → `return Ok(Complete(x))` 一类
  包装变换 + dedent + `state`/`root` 由 ctx 解构重绑定（切片五
  `let X = &ctx.X` 同法，臂体除包装行外逐字节等）。
- `ServiceWorkerFixtureState`（testharness.rs:6684）随分表迁为
  `SwFetchCtx` 字段（清单档 §二：SW fetcher 独占案级状态，无共享面）；
  `state` 由闭包捕获 `Mutex::new(...)` 改 ctx 构建。

## 二、分表设计（两表实例 + 底座复用面）

- 新文件：`testharness_py_routes/sw_fetch.rs`（SW 表 + SwFetchCtx +
  dispatch）、`testharness_py_routes/script_fetch.rs`（script 表 +
  ScriptFetchCtx/env + dispatch）；表实例
  `py_sw_fetch_routes()` / `py_script_fetch_routes()`，沿用
  `PyRoute`/`PyRouteMatch` 保序同构（谓词 + handler + 表序=臂序），
  **不并入主 `py_routes()`**（清单档 §三：fetcher 契约入参
  `(page_url, src)` 无 method/headers 面）。
- ScriptFetchCtx 字段 = 闭包捕获集（root/doc_referrer_policy/
  serve_custom_stash/cred_cookies）；**stash Arc 恒等**（清单档 §二
  硬约束）：ctx 构建点用 `wpt_data_script_fetcher_with_stash` 现参
  Arc，`None` 回落静态面语义逐字保持（`unwrap_or_else(|| Arc::clone
  (&SERVE_CUSTOM_RESPONSE_STASH))` / `unwrap_or_default()`）。
  单测回调面 `wpt_data_script_fetcher`（:5968，#[cfg(test)]）分层不变。
- ScriptFetchEnv（per-call 派生值）：`mime_from_pipe`/
  `mime_from_sidecar`/`json_intent` 三值由 dispatch 前置算出传
  handler（纯派生——pipe/sidecar 读 src query 与磁盘 sidecar，顺序
  在表前与现码一致）。
- dispatch 签名：`dispatch_script_fetch(ctx, env, page_url, src, clean)
  -> Result<String, String>` / `dispatch_sw_fetch(ctx, src) ->
  Result<zero_net::HttpResponse, String>`；闭包体收缩为
  「ctx 构建 + 前置步骤 + dispatch + 尾段」。

## 三、等价验证方案（两子批同构）

1. **机械双件套**（承切片四/五方法）：A——每 handler 体 vs HEAD 原臂
   体经「dedent + Complete 包装（SW）/纯 dedent（script）+ ctx 重绑定
   行剔除」变换后 token 序全等；B——谓词逐臂反推条件式 vs 原 if/else
   条件 token 序全等（链序配对）；C——表行数断言（script 10 /
   sw 21）+ dispatch 前置/尾段顺序锚断言。
2. **全量 A/B**：`make testharness-net-api` 双跑，per-subtest multiset
   空集（对照锚 = 切片五 §三重录基线 `/tmp/m3s5-base-214840.log`
   13,614P/1,305F/8T）；script fetcher 面覆盖锚（module/
   delayed-modulescript、serve-custom-response、referrer-checker、
   echo-referrer、check-cookie）+ SW 面（bytecheck、update-worker、
   import-scripts-*、redirect.py）四域锚从 A/B 日志提取双侧全等。
3. **单测**：runner 单测持平 298P/0F；`wpt_data_script_fetcher` 测试
   回调面（#[cfg(test)]）行为不变。
4. cargo fmt --check 净 + cargo clippy -p zero-wpt-runner 零告警。

## 四、执行序与状态

- [x] 普查 + 设计（本档 §一/§二）——2026-11-08 认领提交 548cfa3d5 后落。
- [x] 6a：SW 分表（生成器 + 验证器 + A/B）——2026-11-08 落库，见[切片六 6a 档](2026-11-08-m3-slice6-sw-fetcher-registry.md)；表设计按先完成者采纳为 in-place 单 suffix 行 ×24（§二 sw_fetch.rs 模块化留 6b 收尾补项）。
- [x] 6b：script 分表——2026-11-09 落库，见[切片六 6b 档](2026-11-09-m3-slice6b-script-fetcher-registry.md)。普查重录：T1-T10 → **T1-T14**（s356 not-found ×2 + s350 accept-header ×2 新臂入链，§三表行数 script 10 → 14）；设计偏差四项（in-place 镜像 6a 采纳态、env 收窄 json_intent 单字段、cred_cookies 不入 ctx、模块化退役）见 6b 档 §三；行为面（html-semantics FILTER=the-script-element 双跑）待静默窗口补跑后收口。
- [ ] 收口：控制面/任务文档 DC-3 行更新 + M3 里程碑 ✅（余 6b 行为面补证）。
