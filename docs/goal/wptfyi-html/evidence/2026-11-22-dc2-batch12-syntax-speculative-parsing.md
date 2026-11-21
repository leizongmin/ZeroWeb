# DC-2 · 批十二：html/syntax/speculative-parsing 族备料腿（leg-1）

2026-11-22。预加载扫描器（preload scanner）执行面 127 键子树接入：
`testharness-html-speculative` 新通道（3 个平铺扫描目录，63 脚本）+
`stash.py`/`slow.py` 两 py-route 端点 + 语料 255 文件入盘。分母
（summary-chrome @4b26db8978）127 键 / 127 子测试 / chrome 过 118（9 键
chrome 亦未过，负案 chrome 本身过）——leg-2 扫描器刀兑现上限 +118。

**实测形态（leg-1 首跑，无扫描器）：126 执行键 50 Pass / 76 Timeout，
墙钟 ~15 min**——推翻开工时「0 Pass / 63 min TO」预估（预估按顶层文档
kill 语义外推；实跑证伪见下）。A/B 复跑稳定性见文末。

## 变更（六文件）

1. `tests/wpt-runner/scripts/goals/63-wptfyi-html-speculative-parsing.sh`（新）
   ——子树整拉（fetch_dir_deep，4 次 API 列目录 + jsdelivr 逐文件）；marker
   幂等（FORCE=1 重拉）。实测入盘 255 文件：page-load 63 顶页 + 63 framed 助页、
   document-write 63 顶页、generated/resources 62 nonspec 助页、负案 1 顶页 +
   sub + .headers、util.js；`.py` 端点与 tools/ 生成器随扩展名分流不入盘。
2. `Makefile`——`fetch-wpt-html-speculative`（discovery 约定名自动命中）
   + `testharness-html-speculative`（test-guard 包裹，TIME_LIMIT 缺省 5400）。
3. `tests/wpt-runner/src/main.rs`——`testharness-html-speculative` 分发臂
   + 子命令（退出码语义同 syntax：有用例非 Pass 或空集 → 1）。
4. `tests/wpt-runner/src/testharness.rs`——`HTML_SPECULATIVE_CORPUS_SUBDIRS`
   3 平铺目录（generated/page-load、generated/document-write、
   expect-no-linked-resources；`resources/` 助页不列即不扫，抽样实证非递归）
   + `speculative_case_skipped` 排除规则（助资产目录 / 无 testharness 引用 /
   `window.open` popup——负案经 test_driver.bless + window.open 弹窗，runner
   无弹窗管理，skip 记账）+ `run_html_speculative_cases` + 案级
   `speculative_stash` 接线（per-case 构建、take 取走清空、跨案零泄漏）。
5. `tests/wpt-runner/src/testharness_py_routes/misc.rs`——
   `route_speculative_stash`（put 记 `param-encodingcheck: <&encodingcheck=
   之后 query 原文>` + 请求 Origin/Accept/Referer 头行；标记缺省 500 1:1
   保真；take 取走清空，miss 回空 200——wptserve server.stash 语义）+
   `route_slow_py`（delay 即返，页断言不依赖 delay 值）。
6. `tests/wpt-runner/src/testharness_py_routes/mod.rs`——两 Exact 路由注册 +
   `PyEndpointCtx.speculative_stash` 字段。

## 通道机制验证（探针实证）

写 6 个一次性探针页（跑后即删）直问 stash 管道四个问题：

- **put/take roundtrip**：路由读写正常（take 返回 put 记录）。
- **顶层文档 kill**：顶层 `document.write('<plaintext>')` 后续标记**不再
  解析**（stash 空）——kill 在主文档管道生效。
- **iframe 管道 kill**：framed 助页（iframe 载入）同构 kill **不生效**，
  助页 put 照常落 stash——与真语料 50/76 镜像分布（两形态同标题同结果）
  互证：page-load 形 put 出 iframe 管道，document-write 形 put 出嵌套
  write（顶层 write 写入的 script 内再 write）管道，两管道 kill 均不生效。
- **img 载入语义**：`new Image()` 指向 stash put 响应（200 / text/plain /
  空体）load/error 永不完成——非图片响应的 img 载入挂起（引擎 img 管道），
  TO 面的主要构成之一。

## 50 Pass 的机理归属（loader 怪癖，非扫描器语义）

pass 键在两形态镜像分布（各 25），逐类对照上游扫描器语义：

| 类 | 键形 | 上游扫描器应取 | runner 实际 |
|---|---|---|---|
| 反常取数面（runner 比扫描器更激进） | template/xmp/svg/math 内资源、nomatch-media/unsupported-type/disabled/alternate 样式表、nomodule、loading=lazy、picture-source 无 img 形 | 按扫描器规则部分取 | 取（template 惰性/xmp RAWTEXT/类型与 media 门控未拦） |
| 平凡取数面（扫描器必取） | img-src/script-src/link-stylesheet/srcset/poster/preload 族 | 取（chrome 过） | TO（取数后 img/script 载入完成事件挂起或头序列不等，见下） |
| chrome 亦未过面 | alternate/nomatch-media/viewport-media/csp-asterisk/charset-script 9 键 | — | 3 键 6 变体 runner 反超 chrome Pass（同反常取数面机理） |

TO 面（76 键）构成：取数发生但页面 done 不达——img 类资源指向 text/plain
空体 stash 响应载入永不完成 → 页面 load 事件不触发（探针实证）→
`expect_fetched_onload` 不 resolve；script 类取数/比较臂异常归入同面待
leg-2 定谳。

**会计口径**：50 Pass 为 runner 实测通过（断言面＝首 take 非空＋两 put 头
序列逐行相等，全部满足），按批三 window 变体通道同例计入净增；但该面依赖
引擎 loader 怪癖（template 惰性缺失、iframe/嵌套 write kill 缺失、非图片
img 载入挂起）而非扫描器语义——leg-2 engine 修（扫描器刀＋loader 语义）
落地后此面会重排，届时以 A/B 重记。

## CI 落位（ci_discover_shards.py 本地复跑）

147 套件（+1）：`html-speculative` 落 **catch-all-4**（伴 import-maps），
fetch 目标 `fetch-wpt-html-speculative` 约定名自动命中、零 CI 编辑；
实体矩阵随零头重排漂至 catch-all-10（编号不锁，同预检注记）。预算核对：
本地 ~15 min（50 pass 快速面 + 76×30s TO ≈ 8 min 计时器型）→ catch-all
缺省 5400s 充裕，无需手调提升。

## 门禁与稳定性

- scoped `cargo clippy -p zero-wpt-runner --all-targets`：runner 本域零警告
  （既有 25 条全在渲染流域文件，同批九以来既状）。
- 拉取脚本幂等重跑：marker 命中跳过深挖，失败 0，计数 63/63/62 逐项吻合。
- A/B 复跑（同二进制连跑两轮全量）→ 见逐文件 JSON `ab_check` 面。

[逐文件 JSON](2026-11-22-dc2-batch12-syntax-speculative-parsing.json) ·
[批十一证据](2026-11-22-dc2-batch11-syntax-template-leg.md) ·
[接入预检](2026-11-22-dc3-weekly-integration-preflight.md)
