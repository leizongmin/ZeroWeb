# M2 切片 356：script src 文档基 URL + 空 src error 面——fetch-src 四案 F/T→P

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/fetch-src/alpha/base.html`（s355 挂账三案并勘之一）+ `script-not-found-not-executed.html`（s350 同族语料资产缺面）
**结果**: 四案 F/T→P（alpha/base F→P、script-not-found F→P、empty T→P、empty-with-base T→P）；域 join det **F→P=2 / T→P=2 / P→F=0 / T→F=0**；语料口径 the-script-element **12F→10F、7T→5T**（306P/10F/5T，321 键）；残 10F/5T 全部在册零新增红。143 upstream 真值定谳收口（chrome+firefox Pass，引擎真缺陷，挂账为写流弹出 SVG script 流式架构面）。

## 勘根与定谳（三案并勘）

1. **script-not-found-not-executed**：语料仅 .html，上游 `script-not-found-not-executed{,-2}.py` 端点件缺席（pin 1eecaa0c30 gh api 逐字核对：.py = 404 + text/javascript 体、-2.py = 200 + 执行体）。双态 {1P,1F} 的 Fail 腿 = 端点缺席——.py 落 fetcher 扩展名非 scripty 臂恒 Err，test2 的 200 执行面不可达（s227/s344 缺资产同形）。修法 = fetcher 端点仿真臂（s344/s346 惯例——wptserve handler 不入语料；404 → Err 契约口径同 s346）。
2. **fetch-src/alpha/base**：`<base href=../beta/>` 下 parser 面 `<script src=test.js>` 红（got "alpha"）。spec [prepare the script element](https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element) src 步：URL 相对 **node document**（文档基 URL）解析。引擎两缝均未吃 base：runner 装配 `inline_local_scripts` 相对 src 按 case_dir 归一（本案消费缝——外链 classic 走内联投递）；webview 交付循环 fetch 缝（真外链透传面——SRI/module/非 JS type/http src）。动态面已净（shim fetch `_zwResolveFetchUrl` M5-S7 基解析）。wpt.fyi chrome 159 Pass 真值在案。
3. **fetch-src/empty + empty-with-base**：动态 `setAttribute("src","")` 双案 Timeout。spec src 步显式短句：**src 空串 → 排 element task 派 error、return**（先于 URL 解析，与 base 无关——unreachable.js 在场亦不 fetch）。引擎经典臂 `if (!url) return false` 静默 bail（Timeout 根因）。chrome 双案 Pass 真值在案。
4. **143 upstream 定谳（s352 延续挂账收口）**：上游原件逐字节一致（gh api pin 1eecaa0c30，blob sha a0d9012f 与 master 同）；wpt.fyi chrome 159 + firefox 162（run 991f40f670）**双 Pass**（1/1 subtest）——s352「疑上游自身失败」判定**推翻**。引擎红态 [#1,#2] 缺 #3 根因：写流为 body 追加流（R5001 口径，无插入点回接原流栈），document.write 注入的游离 `</script>` 无法在流中弹出 SVG script #A——#A 源 = 最终树 child text（含 write 弹出后才 token 化的 `assert_unreached():` 语法错文本）→ SyntaxError 整体不执行。Chrome 真值：#B 端标签同步执行 → write 注入内容于插入点续parse → 游离 `</script>` 弹 #A（源快照 = 弹出时刻 child text，后续文本落 svg 根）→ [#1,#2,#3]。**流式物化架构面（052 同族）长挂**。

## 施工

1. **runner fetcher 端点仿真臂**（testharness.rs）：`wpt_data_script_fetcher_with_stash` 新增 script-not-found 双臂——`script-not-found-not-executed.py` → Err（404 契约面）+ `-2.py` → Ok("test2_token = \"script executed\";")（上游回体逐字）。
2. **runner 装配基解析**（`inline_local_scripts`）：文档基 URL 预扫（首个带非空 href 的 base 元素，`<base` 边界排除 `<basefont`；记录标签尾偏移）——相对 src 按基归一、基前脚本回落 case_dir（spec parse 时刻基位置语义）。根绝对/协议相对/绝对 src 基无关（URL join 语义）。全语料 `<base>`×相对 src 6 页普查：相对 src 均在 base 标签前（位置判定零翻面）。
3. **webview 交付循环基解析**（webview.rs External/ExternalModule 臂）：fetch src 相对面按 `script_bases.get(script_index)`（s90 既有 parse 时刻快照表）`resolve_document_url` 后交 fetcher；绝对形 join 恒等零漂移（module src 恒根绝对）。动态面不走此缝（shim 已基解析）。
4. **shim 空 src error 臂**（part03 `_zwPrepareSrcScriptGate`）：src 属性**在场但空串** → run-once 置位（spec already-started 先于 src 步）+ setTimeout(0) 派 trusted error（`__zwMakeTrustedEvent`，bubbles/cancelable 缺省 false）+ `_zwOnRan` 续链；src **缺席**不进本臂（空性早退不置 already-started——s354 056/057 判据）。module/classic 共用（src 步在 type 分支前）。

## Blast radius / 回归定谳

- **首版空 src 臂 P→F 回归（域 join 暴露）**：execution-timing 022/023/024（late .src 形——append 无 src、后设 `.src=` 触发）被误置 run-once 挡死后设触发臂。solo 同二进制复现（非 flake）；定谳 = src 缺席/在场前提缺失，加 `hasAttribute('src')` 门后 022-024 全 Pass、fetch-src 4/4 保持。
- `<base>` 普查零翻面（6 页相对 src 全在 base 前）。
- 写流面零接触（写流 src 走 shim fetch 通道既有基解析；assembly/webview 缝只覆盖静态解析集）。

## 验证

- **红态**：solo base `expected "beta" but got "alpha"`（/tmp 探针）；双 empty Timeout（s355-b 域账）。
- **域 join**（LC_ALL=C + (key,state) sort -u）：A=s355-b 321 (key,state) vs B=s356-b 320——F→P=2、T→P=2、P→F=0；残 10F/5T 逐项在册。中途一跑无 FILTER 全树（8128P/300F/44T 超域口径未入账，重启 FILTER=script-element 口径）。
- **clippy** zero-webview + zero-wpt-runner `--all-targets` 净（首版 `if_same_then_else` 修并）+ `cargo fmt --all -- --check` 净。
- **make test**：后台跑（结果见控制文档轮次账）。

## 挂账 / 下一步

- **143 施工面**：写流弹出 SVG script 需 live parser 插入点（流式物化）——052 同族长挂；engine 红态 [#1,#2] 与 chrome [#1,#2,#3] 语义差已在案。
- remove-src-attr-prepare-a-script（残 10F 之一，与 src 属性族近邻——下一切片候选）；json-module parse-error-location、custom-element-exception、import.defer 三案、probe12/13 自建单列在册。
- 052 批解析可见性：流式物化架构面长挂（不变）。
- T=5 长尾（load-error-events-3、code-cache-base-url、dynamic-imports-credentials-setTimeout、instantiation-error-8、script-crossorigin-network）在册。

## 日志

- evidence/logs/s356-b-script-element.log（域跑 B 面，join 基线 s355-b-script-element.log）
