# M2-s4 — 元素面指令口径 + WPT .sub.js 模板面（全绿 37→62）

**日期**: 2026-09-25
**corpus**: `make testharness-csp`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-25-m2-s4-csp-final.json](2026-09-25-m2-s4-csp-final.json)
**前序**: [M2-s3](2026-09-25-m2-s3-csp-extensions.md)（37/415，21.4%）

## 结果

| 指标 | M1 基线 | M2-s3 | M2-s4 | Δ（s3→s4） |
|---|---|---|---|---|
| 全绿用例 | 30/415 | 37/415 | **62/415** | **+25 零丢失** |
| subtests Pass | 91/557 = 16.3% | 118/552 = 21.4% | **159/570 = 27.9%** | +41 |

新增全绿 25 案跨五目录：style-src 阻止族 10（style-src-none-blocked /
inline-style-nonce-blocked / hash-blocked / stylehash-basic-blocked /
stylenonce-allowed / inline-style-allowed 族 / stylesheet-nonce-blocked 等）+
script-src 阻止/allowed 面 4（scriptnonce-basic-blocked / scripthash-ignore-
unsafeinline / overrides-default-src / strict_dynamic_in_img-src）+ connect-src
allowed 族 4 + default-src/meta/generic/frame-src/blob 5 + img-src 间接 2。

## 本切片三改动（kill-switch 延续，default off）

1. **元素面指令口径**（csp.rs `effective_style_directive` /
   `effective_script_directive`）：显式 style-src/script-src 存在时上报
   **"style-src-elem"/"script-src-elem"**（Chromium 对齐——style-blocked 族 corpus
   以 `assert_equals("style-src-elem", e.violatedDirective)` 实参交换形断言元素面
   口径；无案断言裸指令名，切换零回退）。
2. **`{{GET[name]}}` 模板替换**（runner `wpt_data_script_fetcher`）：WPT .sub.js
   服务端模板面（上游由 .py handler 按请求查询串注入）——script src query 参数按
   `{{GET[key]}}` 占位符替换（最小百分号解码），无匹配占位符 → 空串。**logTest/
   alertAssert/checkReport 全族 support 脚本解锁**——这是 +25 的主杠杆（此前整个
   .sub.html 族折在脚本装配层 Unex pected token '{'）。
3. **corpus 级 support/ 目录补拉**（fetch 脚本 `content-security-policy/support/**`
   + 本地补齐 37 文件）——stylenonce/logTest 族依赖。

## 剩余缺口（M2-s5+）

| 簇 | 形态 | 切片 |
|---|---|---|
| style 外链 allowed.css 应用 | stylenonce-blocked 期望 allowed.css 生效（green）实得黑——外链 css 应用链（与 CSP 无关的 runner/pipeline 面） | M2-s5 |
| script-src-attr | onclick 内联事件处理器（targeting block2/4） | M2-s5 |
| 运行时 img / eval | createElement('img') src-set 钩子 / blockeduri-eval | M2-s5/s6 |
| connect-src 阻止族 | beacon/fetch/XHR blocked 面（fetch 桥检查点） | M2-s5 |
| report-uri / Reporting API | .sub.html report 端点族 | 记账（infra） |
