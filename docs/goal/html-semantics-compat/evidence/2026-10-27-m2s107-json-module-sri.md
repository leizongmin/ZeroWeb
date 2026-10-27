# M2 切片 107：JSON module 基础执行面 + 顶层模块 SRI + 外链模块成功 load 面

日期：2026-10-27。范围：`tests/wpt-runner/src/testharness.rs`（fetcher JSON 意图分支 + integrity 单元跳过内联 + fetcher 回归钉）+ `crates/script-sandbox/src/es_module.rs`（带属性提取）+ `crates/engine/src/pipeline/extract.rs`（`PageScript::ExternalModule` 携 integrity + 提取钉）+ `crates/webview/src/webview.rs`（collect type 参数 + JSON 合成 + SRI 校验 + 外链成功 load + 5 回归钉）+ 六消费点模式同步（browser/renderer/reftest）。上轮：[M2 切片 106](2026-10-27-m2s106-svg-script-family-static-face.md)。

## 1. 入场定谳

- s107 探针（承 s106 下一步三候选）：非 P 分布——module 域 92 行最大、数据模块域合计 47 行（css 21 + json 17 + text 9）、execution-timing 余 46 行（scheduler 长尾异质）。JSON module 定为最小可收口切片。
- MIME 合法集按**语料判据**定谳（不凭现行 spec 印象）：essence `application/json` | `text/json` | `+json` 后缀、`;参数` 剥离——valid-content-type 8 臂全应成功，而现行 HTML spec 的 JSON MIME type 已收窄至 application/json 与 +json 后缀（text/json 是 corpus 时代合法集）；`text/json;%2Bjson` 须百分号解码后判定。
- `serve-with-content-type.py` 端点**本体不在 runner 语料**（find 全语料零命中）——fetcher 按 handler 语义模拟：`?fn=<相对 .py 所在目录>&ct=<mime>`，ct 交付 fn 指向文件（corpus `fn=json-module/module.json` 对 `the-script-element/` 目录成立）；与既有 imported_script.py 模拟同模式。
- 裸 `.json` import 全语料零分布（排除 with 形态 grep 零命中）——fetcher 放行无负例翻转风险；动态 `.json` import 仅 serviceworker 一处（runner 无 SW，不触）。
- 风险面定谳两处：①`integrity.html` mismatch 行平凡 Pass（fetch 恒败→体不执行）——JSON 放行后必翻 Fail，**须真实现 SRI** 才能保持零 P→非P；②`text-module/script-element-json-src.html` src 是 `file.txt` 非 .json，负例解除。

## 2. 接线

- **fetcher JSON 意图分支**（testharness.rs）：`.json` 静态文件默认 `application/json`（pipe/sidecar 可覆盖）与 serve-with-content-type.py 端点按 `mime_is_json` 判定：合法交付原文、非法 Err（网络错误语义）；ct 值百分号解码（`+` 原样保留——t3/t4/t8 的 `text/html+json` 裸 `+` 在 query 不表示空格）。
- **script-sandbox** 新 `extract_static_module_imports_with_attributes(source) -> Vec<(specifier, Option<type>)>`——复用 s105 `suffix_after_specifier_literal`/`parse_with_clause`（提 pub(crate)）；属性块语法错误由校验面先行报告，此处按无属性；仅 import 语句（export-from/动态 import() 同 s105 口径）。transform 各臂全部重建输出、with 子句被消费后丢弃，无语法残留。
- **webview collect 链**：`collect_module_deps_recursive` 增 `type_attr` 参数（9 形参超 clippy 阈值，函数级 allow 承接，FIXME 挂账）；type=json → serde_json 校验（非法 → DepFetch/TopFetch 网络错误语义）+ 规范化内嵌合成 `export default <字面量>;`（default 导入经既有 export transform 与导出扫描自动成立），数据模块无依赖不递归。
- **SRI**：`PageScript::ExternalModule { src, integrity }` 结构化（classic 外链不带，FIXME）；webview `integrity_metadata_matches`——空格分隔 `algo-digest` 多 hash 任一命中即过，sha256/384/512（`base64_encode` 启用 + webview 增 sha2 workspace dep）；mismatch → 元素 fire error + 体不执行（与取回失败同面）。
- **外链模块成功 load 面**：模块取回+编译+求值成功 → 元素 fire load（按 src 匹配，R2944 同口径）——此前仅失败面有事件派发。**内联模块成功不 fire load**：load 事件属外链 fetch 完成路径（load-error-events-inline 四臂 assert_unreached 判据；首版内联也派发致 2 行 P→F，定向轮即定位改外链限定）。
- **runner 装配面**：`inline_local_scripts` 对带 integrity 属性的单元整段透传不内联——内联投递丢 src/integrity 双锚（webview SRI 校验与 load/error 事件 src 匹配派发都以 src 为锚）。四页 integrity.html 语料全部 module 形，classic 无涉。
- 六消费点同步：browser `tab_scripts`、renderer `page_scripts`/`script_prefetch`、reftest_scripts、testharness 计数臂的模式改形（无行为变化）。

## 3. 验证

- 单测：script-sandbox **373/373**（+带属性提取钉：with-json/无属性/去重/压缩形态/export-from 不提取）；zero-engine lib **3541/3541**（+integrity 提取钉）；zero-webview lib **813/813**（+5 钉：JSON default 导入执行+内联成功无 load / JSON 语法失败元素 error / SRI 命中执行+load / SRI 失配 error+体不执行）；wpt-runner fetcher 钉（.json 默认 MIME/端点 fn+ct/解码/非 JSON 拒/sidecar 覆盖拒）。fmt clean；clippy 六 crate（script-sandbox/engine/webview/renderer/browser/wpt-runner）`-D warnings` clean。
- 定向 FILTER=json-module（对照 s106 基线）：26 行 → 30 行，**9 Fail + 2 Timeout → 13 Pass，零 P→非P**（module.html、valid-content-type ×5、integrity matches+mismatch 双 Pass、charset ×6）；load-error-events 滤页对照零回归（json-module 页面级 Fail→Pass；module/ 族行不变；document-metadata link 页为 filter 域外入围非回归）。
- 全域扫描（无 FILTER，7443 行）交叉核对 integrity 四页：module/integrity **1P+2F → 3P**（+2P）；json-module/integrity **2P**；css-module/text-module mismatch 行维持 Pass（matches 行待 css/text 数据模块，挂账）。
- 全子树终扫（FILTER=scripting-1 口径，与 s106 基线同）：**372P/175F/35T + 15 NotRun**（s106：356P/185F/37T+15N，**+16P**）；行级归一 diff（/tmp/s107-final-scan.log 存档）：10 Fail + 2 Timeout → Pass（json-module 13 行 + module/integrity 2 行 + json-module load-error-events 页面级 1 行），**零 P→非P**（diff 全文仅 `<` Fail/Timeout 与 `>` Pass，无反向）。
- 依赖变更：`crates/webview/Cargo.toml` 增 `sha2 = { workspace = true }`（engine 已用同源 SRI 摘要族）；无新增三方依赖。
- 渲染面核对：本轮纯脚本管线面（fetch/编译/事件派发），无布局/绘制依赖，reftest/product-smoke 未跑。

## 4. 挂账

- css/text 数据模块执行面（css-module/text-module 的 content-type-checking、integrity matches、basic 族 ~30 行）——同一 collect 分派骨架已就位，type=css/text 合成器另切片。
- load-error-events.py 端点 module 路径 3 行（src,200 臂）；parse-error 族 Timeout（parse 失败元素事件模型与 s102 报告面差异）；http-accept-header（Accept 头建模 + 动态 import with 子句）；script-element-json-src（顶模块 MIME 拒绝→元素 error 的 spec 面与 s102 parse→load 模型冲突待梳理）；referrer-policies。
- dynamic import() 的 `with` 子句透传（现被丢弃）；export-from with 属性；classic 外链脚本 SRI；`collect_module_deps_recursive` 账本打包重构（FIXME allow 承接）；hash 列表 options。
- 承袭挂账不变：execution-timing 余 ~42F（scheduler 长尾）、SVG script 运行时面 4 页、charset 族残留、svg-script-fragment-clone 3F、FIXME 组、接口对象基建面、testharness.rs /tmp/zw-hb*.txt 插桩来源待查。
