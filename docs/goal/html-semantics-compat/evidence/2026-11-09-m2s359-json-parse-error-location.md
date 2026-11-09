# M2 切片 359：JSON parse 失败定位传播——parse-error-location F→P

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/json-module/parse-error-location.tentative.html`（s358 挂账残 8F 之一，json-module 族近邻）
**结果**: 单案 F→P；域 join det **F→P=1 / P→F=0 / T 双向 ∅**；语料口径 the-script-element **8F→7F、5T 不变**（320 键）；残 7F 全部在册零新增红。

## 勘根定谳

1. **测试面**：静态 `import v from "./parse-error.json" with { type: "json" }` → window error 事件断言 `e.filename === parse-error.json 的绝对 URL`（红态 `expected ".../parse-error.json" but got ""`）。spec 锚：extract error information 的 message/filename/lineno/colno 属 **implementation-defined derived values**（curl webappapis.html 原文逐字），测试注释同口径（"preferable if implementations provide the correct file location"）——按抛出模块定位 = 浏览器一致行为。
2. **根因双层**（自建 probe17 诊断探针，失败消息携带现场全量，跑后已删）：
   - **表层**：reportError 调用 filename 恒传 `''`（script_gen `script_module_eval_error_report` s255 求值错原值上报面，s347 lineno 轮留下）。
   - **真根因**：JSON parse 失败 synth 源为纯 `throw new SyntaxError(msg);`——**无导出面**。入口静态 import 的 compile 期 export 扫描（es_module `ensure_module_export` → `module_provides_export`）先于本体求值判 Missing → `CompileError("Module ... does not provide an export named default")` → 入口编译失败 → s102 包装级失败面重构上报——**synth 体永不求值，SyntaxError 被链接错误遮蔽**（probe17 实证：error.message = "Compile error: ... does not provide an export named default"，errFileName undefined）。parse-error.html（绿）靠弱断言（`not-undefined` 满足 ''）未暴露此遮蔽。
3. **修面推导**：链接门要成立 → synth 首行补 `export default undefined;`（静态扫描 Provides，求值期 throw 先于导出消费）→ 抛错走求值期（spec：JSON parse 失败属 internalize 期失败经模块记录传播）→ s255 哨兵原值上报（s344-r2 祖先记账跨键同对象，fileName 保真）→ reportError 第二参从原值读 `fileName`。

## 施工（Rust 双点，JS shim 零改动）

1. **webview.rs JSON synth 面**（collect_module_deps_recursive parse-fail 臂）：synth 改 `export default undefined;var __zwJsonParseErr=new SyntaxError({message});__zwJsonParseErr.fileName={url};throw __zwJsonParseErr;`——`url` = registry 键（解析后绝对 URL，与测试 `new URL(...)` 期望同形）。
2. **engine script_gen.rs `script_module_eval_error_report`**：`reportError(__zwErr, '', lineno)` → `reportError(__zwErr, (__zwErr && __zwErr.fileName) || '', lineno)`——有 `fileName` 透传，无则回落 '' 旧行为（普通 Error 实例无该属性 → 全部既有求值错上报零位移）。s347 lineno 面不变。

## Blast radius / 回归定谳

- **corpus 普查**：域内断言 error 事件 filename 的页面仅 3——muted-errors.sub.html（**classic 面**，s255 面被 `if is_module` 门隔离，零接触；filename='' 断言由 classic muted 臂承担）、parse-error.html（`not-undefined` 弱断言，真 filename 更满足）、parse-error-location（本切片目标）。
- **script_module_eval_error_report 唯一调用点** = webview.rs 5464（is_module 门内）；engine 单测零引用该 script（grep 净）。
- **s255 求值错上报面漂移普查**：evaluation-error-5（lineno 16 判据）、choice-of-error-3（跨键同对象 `log[1]===log[3]`）、checkpoint-after-window-onerror-module（reportError 位次）、error-type-3（原值透传）solo 全 Pass——fileName 回落面零位移。
- synth 形态变化面：parse-fail JSON 模块从「编译失败+链接错误文本」改「求值失败+原 SyntaxError」——parse-error.html 断言面（ErrorEvent + instanceof SyntaxError + 四字段 not-undefined）全满足且 message 升级为真实 JSON parse 消息；crossorigin-import-parse-error 双页不在语料（上游 webappapis 通道）。
- css-module/text-module synth 臂零接触（本刀仅 json parse-fail 分支）。

## 验证

- **红态**：solo `expected ".../parse-error.json" but got ""`（s358-b 域账）。
- **probe17 诊断**（跑后已删）：修复前 errFileName undefined + Compile error 遮蔽；修复后 errFileName/evFilename = parse-error.json 绝对 URL、errMsg = "EOF while parsing an object at line 2 column 0"（真实 parse 消息）、failKeys 三键同对象 fileName 保真。
- **绿态 solo**：parse-error-location + parse-error + evaluation-error-5 + choice-of-error-3 + checkpoint-after-window-onerror-module + error-type-3 全 Pass。
- **域 join**（LC_ALL=C + (key,state) sort -u）：A=s358-b 321 键 vs B=s359-b 320 键——A-only 恰 2（目标 Fail + probe16 已删探针 Pass）、B-only 恰 1（目标 Pass）；**P→F=0、T→F=0、零意外位移**；残 7F = probe12/13 自建单列 2 + 143/052 长挂 2 + import.defer 三案。
- **clippy zero-webview + zero-engine --all-targets 净 + fmt 净**（Rust 改动轮全检）。
- **make test**：3816P/7F EXIT=2（失败集与 s356-s358 基线逐名同构 = 并行流组合态既有集，本流不入侵）。

## 挂账 / 下一步

- import.defer 三案（basic-deferred-evaluation/cyclic-tla-defer-eval/tla-deferred-evaluation tentative，JS 语言特性面独立切片）；143/052 流式物化长挂；probe12/13 自建单列；T=5 长尾（load-error-events-3、code-cache-base-url、dynamic-imports-credentials-setTimeout、instantiation-error-8、script-crossorigin-network）。
- s97/R387 prepare 缺口（s357 在册）无钉面不扩刀；外链模块 lineno 面属模块自身 URL 内坐标（s347 FIXME 语料无钉面维持）。
- 动态 import 的 JSON parse 失败上报（`__zw_load_module` catch → __moduleFailures 面）与静态面同 synth 源，fileName 随对象传播——无独立钉面，随语料扩展再收。

## 日志

- evidence/logs/s359-b-script-element.log（域跑 B 面，join 基线 s358-b-script-element.log）
