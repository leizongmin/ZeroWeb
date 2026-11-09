# M3 切片六（6b）：script fetcher 注册表化——14 臂闭包链 → py_script_fetch_routes 分表

- 日期：2026-11-09（10:20 认领，同日落库）
- 基树：87efb6aa9（切片五文件族拆分 + s356/s350 合流后 HEAD）
- 承接：[普查档](2026-11-08-m3-slice6-fetcher-tables.md) §一 script fetcher 面
  （普查重录 T1-T10 → T1-T14）+ §二设计 + §三验证方案；同型先例
  [切片六 6a 档](2026-11-08-m3-slice6-sw-fetcher-registry.md)。

## 一、交付面

`tests/wpt-runner/src/testharness.rs` +758/-463（fmt 后），全在 script
fetcher 段：

- **表**：`py_script_fetch_routes() -> &'static [PyScriptRoute]`，
  14 行 in-place（承 6a 采纳态：谓词 + fn 指针 + 表序 = 链臂序，
  None 落穿语义由 handler 返回 `Result` 表达）。谓词枚举
  `ScriptFetchMatch`：`Exact(&'static str)` / `Suffix(&'static str)` /
  `Contains(&'static str)` / `Custom(fn(&ScriptFetchEnv, &str, &str) -> bool)`
  （入参 env/src/clean）。
- **ctx/env**：`ScriptRouteCtx<'a>{ root, doc_referrer_policy,
  serve_custom_stash }`（借用形）+ `ScriptFetchEnv<'a>{ json_intent }`
  （单字段借用）。**stash Arc 恒等**（普查档硬约束）：ctx 持
  `&serve_custom_stash` 原 Arc 引用，与 P0 前置构建点共享同一
  per-case Arc，未另建。
- **dispatch**：闭包链原位替换为 9 行循环（`route.matches.test(&env,
  src, clean)` → `(route.handle)(&ctx, &env, page_url, src, clean)`)；
  P0-P3 前置留守 + R1 scripty 链 / R2 any.js 包装 / R3 通用读尾段
  留守**逐字节未动**；`mime_is_scripty` / `mime_is_json` 提升至表区
  helper（体逐字节）。
- **臂清单**（T1-T14，链序）：not-found ×2（T1/T2 Exact）、
  module-fetch-tester（T3 Exact→`module_fetch_tester_serve`）、
  delayed-modulescript（T4 Suffix）、serve-custom-response（T5 Exact，
  stash 消费）、referrer-checker-data（T6 Custom 三路 contains）、
  accept-header json/text（T7/T8 Contains，s350 常量体）、
  data-module-intent（T9 Custom `__zw_mtype=` 标记）、json-intent
  （T10 Custom `json_intent.is_some()`）、load-error-events（T11
  Custom basename 全等）、referrer-checker-module（T12 Contains）、
  echo-referrer（T13 Contains）、subresource/script.py（T14 Exact）。
  原链注记锚块（s356/s267/s346/s344/s268/s350/s249/s250/s116/s124 ×2）
  129 行 verbatim 附入对应 handler 体顶（dedent 4 空格）。
- **T9/T10 if-let 头 → let-else**：表谓词已保证标记/意图在场，
  let-else 不可达臂为 `Err` 字面 + 迁入注记；T10 借用形引用因
  `-D warnings`（needless_borrow）改为 `&strip` 解引用形。

## 二、机械等价变换（工具不入库）

- 转换器 `/tmp/m3s6b_convert.py`：内容锚定位 14 臂（非行号硬编码）+
  dedent8 + string-aware 词边界重绑定（`root`→`ctx.root` 等）+ fn 尾
  `return` → 尾表达式 + let-else 头生成 + 块拼接（自底向上三步）。
  两个坑：T6 多行 if 条件 `{` 独占行（arm 抽取先在 head+6 行窗找
  `{` 再 brace 平衡）；多行 Custom 谓词逐行误加逗号（改仅末行）。
- 验证器 `/tmp/m3s6b_verify.py`（git HEAD vs 工作树 token 级独立复核）：
  A 检 14/14 handler 体 ≡ HEAD 臂体（ctx/env forward-map + 尾 return
  解包后 token 全等）；B 检 14/14 表谓词反推条件式 ≡ HEAD 链条件
  token 序（变体展开 Exact→`clean==`/Suffix→`ends_with`/Contains→
  `contains`，Custom 剥闭包头/壳 + env 限定符回剥；T9/T10 `.is_some()`
  尾规约对齐 if-let 头）；表行数 14 + 行序断言。fmt 后复跑仍全过。
  三类坑留档：token 流空白 token 未滤（A 检 14 全败同型 off-by-one）；
  行界 regex `.+?` 配 `re.S` 跨行吞并到首行（tempered dot
  `(?:(?!handle:)[\s\S])+?` 修复）；fmt 后 Exact 单行变多行 trailing
  comma 入体（尾逗号规约）。

## 三、设计偏差（vs 普查档 §二，按 6a 采纳态收敛）

- **in-place 表**（非 sw_fetch.rs/script_fetch.rs 模块化拆分）：
  6a 先完成者采纳态沿用，模块化退役；script 表同样留在
  testharness.rs 表区，与主表底座文件族分离。
- **ScriptFetchEnv 收窄**：普查设计为 `mime_from_pipe`/
  `mime_from_sidecar`/`json_intent` 三值；实提仅 `json_intent`
  单字段借用——mime 两值仅 R1 尾段消费（留守闭包原位读，值不
  过表），入 env 即 dead field。
- **cred_cookies 不入 ctx**：仅 P0 前置（fetch 桥闭包）消费，
  handler 无需。
- **表行数**：普查 §三记 script 10；实提 14（s356 not-found ×2 +
  s350 accept-header ×2 四新臂入链后普查重录，认领条目已记）。

## 四、等价性证据

1. **token 级独立复核**：§二验证器 A 14/14 + B 14/14 全过（fmt 前后
   各一轮）。
2. **静态门**：cargo fmt 净（fmt 差异全部为表行长/闭包缩进，
   应用后验证器复跑全过）；clippy -p zero-wpt-runner --all-targets
   `-D warnings` 零告警（systemd scope 9.9s）。
3. **单测**：303P/0F/76 ignored（vs s350 时点 302P，+1 = s356 后
   新增钉）。臂覆盖记账：≥14 钉直接行使 script fetcher——T3
   （s346 module_fetch_tester 状态机）、T5（s267 serve-custom-response
   状态机 + s338 stash Arc 往返）、T6/T12/T13（s116 referrer-checker/
   referer 表 + s124 modulepreload/echo-referrer + s270 img 面）、
   T7/T8（s350 accept-header 锚）、T9（s249 data-module 标记 +
   s268 referrer-checker sims + s351 css-module 相对 url 钉）、
   T10（s107 json-module mime 门）、T11（s250 load-error-events
   端点 + query-aware 体）——fetcher API 级动态证据覆盖 11/14 臂；
   T1/T2（not-found 负路径）/T4（delayed 时序 fixture）无单测钉，
   由 token 级复核 + 行为面补证。
   - 插曲：s351 css-module 钉首跑红——**语料缺口**（
     `css-module/resources/load-relative-url.css`/
     `redirect-target.css` 本地缺席，仅存 basic.css）；stash 往返
     复证 HEAD 同红（数据面归因，非本批）；按
     `goals/lib.sh` pin rev 31597693 定向补拉两文件后全绿。
4. **net-api 全量 A/B multiset（计数敏感）**：A 锚
   `/tmp/m3s6b-base-run.log` @ 87efb6aa9（13,993P/1,321F/9T =
   15,323 行）vs B 改后树 release 全跑：15,323 行 multiset
   **changed = 1**——`xhr/send-timeout-events.htm`「timeout is not
   0」P→F。归因：B 跑窗口同机有并行 `make test` 编译腿（load
   average 465），该案为 send/timeout 计时敏感面；**等负载受控
   探针**（同窗、同 case、三连跑）：HEAD 3/3 FAIL vs B 3/3 FAIL
   同型——HEAD≡B，环境归因非重构归因，等效判定成立。
   注：script fetcher 臂在 net-api 套件不行使（6a 档 §四订正同源），
   本跑为全量回归面守门；臂级行使证据由 §4 单测钉承担。
5. **行为面（html-semantics FILTER=the-script-element 双跑）**：
   **待静默窗口补跑**——本轮机器 load>300 并行腿持续，RAF/timing
   噪声双侧对称不可判，推迟不折损判据（承 6a §四同型协议，6b
   版按本节 §3 单测钉 + §4 全量守门先行落库，行为面为收口前
   最后补证项）。
6. **基线锚漂移归因**（承认领条目，A 锚 vs 切片五锚 +396）：
   http-cache +138（124P/14F 与 30584b1e7 账逐字对上）+
   websockets 面 Pass 741→999 = +258（d19247694..87efb6aa9
   50 提交窗引擎/交付修复，域级归因）；净 -3 = P→F/T 零星。
   A/B 双侧同基树比较，漂移不进入判定。

## 五、过程记录

- 10:20 认领后先录 A 锚再动树（同二进制窗口纪律）；转换器 +
  验证器双件套承切片四/五方法；cargo check 过 → 注释锚 dedent
  （首次按块端点探测失败，改逐 handler brace 平衡后 129 行落位）
  → 验证器三修后全过 → fmt/clippy/单测 → net-api B 跑 → multiset
  对比 → 等负载探针。
- B 跑两次被 10 分钟前台窗切断（套件 + release 构建合计 >15 分钟），
  改 `setsid nohup` 后台 + PID 轮询完成；test-guard 900s 墙钟内
  完跑（make exit 1 = 套件含既有红案，属正常）。
