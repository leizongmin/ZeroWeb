# M2 切片 354：runner report 元素装配保真——execution-timing 056/063 双案 F→P

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/execution-timing/056.html`（s353 基线 13F 首案勘根定谳 = runner 装配保真缺失，非引擎语义）
**结果**: 056 F→P + 063 F→P（同因连带，063 兼 A 面 Fail+Timeout 双态对消解 → T 8→7）；051/052 P→F（装配保真修正揭出既有引擎面，非语义回归）；域 join det F→P=2 / P→F=2 / T→P=1 / 同 key+subtest 双态键 A/B 均 0；语料口径 **13F→13F，1109P→1109P**（distinct-pair 1130→1129，319 键两侧）

## 勘根（runner 装配保真）

[HTML prepare the script element](https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element)：script 元素在文档中恒在；空源 script 的 prepare 在空性检查早退，**不置 already-started**——后续追加文本仍可触发执行。056：head 第 4 个 script 为空元素（浏览器真值索引 [3]），body inline#1 把代码文本 appendChild 到 `[3]` → 注入代码同步执行，期望 `['inline script #1', 'injected script code', 'end script #1', 'inline script #2']`。runner 装配（`prepare_harness_html`）把 `/resources/testharnessreport.js` **整标签移除**（`replace_script_source(html, marker, "")` 空串替换 = 标签删除）→ 装配文档少一个 script 元素、集合序前移一位：056 的 `[3]` 落到 inline#1（已启动 no-op）→ 注入代码永不执行（红态 3 元素缺 'injected script code'，s354-solo-056-red.log）。063 同因异形：head 第 4 个 inline script 取 `getElementsByTagName('script')[3]` **自身** cloneNode(true) 追加（期望 clone 不重执行，eventOrder `['HEAD script', 'inline script #2']`）——修前集合长度 3 → `[3]` 为 undefined → TypeError → Fail+Timeout 双态。

引擎面 5 轮 probe 排除（probe14 迭代实证）：s349 预盖章不含空源（extract 交付集排除空源，stamp 不触）；`_zwScriptMaybeStartDynamic` 门链完好（空体不置记账、补文本可触发——手动调用 eval 成功）；s95 trap 臂（pre 空+post 非空=执行；pre 非空=已启动 no-op）对白空格追加自然防护——057/emptyish 白空格形状不依赖交付盖章；clone 族 cloneSel=null 全新手柄不重执行（before=1 after=1）。

## 施工（runner 单面，testharness.rs 两处）

1. `prepare_harness_html`：report 标签改保**空元素** `<script></script>`（原空串整删）。空元素不进交付集（extract 空源 bail）、不盖 `_zwParserDelivered`、write/fetch 通道（testharness_py_routes report 服务）本就保元素——两通道装配 DOM 形态自此一致。
2. `csp_script_positions_prepared`：REMOVED_SRC_MARKERS 3→2（剔除 report 标记）。序号空间核验：`script_index` = 全 script 元素文档序（s349 注释「全 script 元素文档序，extract_page_scripts_sched 透传」）；`script_csp_info` = prepared DOM 全量（`get_elements_by_tag_name` 无跳过，空元素有位）；positions = 原文全量 − removed markers（report 保留在表）——三表同空间，`get(script_index)` 对齐。

## Blast radius 普查

- 位置访问 `getElementsByTagName('script')[N]`：全语料 18 文件全在 the-script-element；`querySelectorAll('script')[N]` / `document.scripts` / head.children 结构访问全语料 0。
- 计数访问 `.length`：仅 execution-timing/scripts/count-script-tags.js（051/052）+ testlib.js。
- 白空格形状：057 + emptyish-script-elements 7 子测全 Pass（s95 trap 臂 + 空体不置记账双保险，battery 实证）。
- clone 族 061-067 全 Pass（probe14 机制实证 + B 面逐键核对）。

## 暴露面（装配保真修正揭出既有引擎面）

- **051**（动态外链 classic src 装载序）：引擎动态插入 src 脚本的取回+求值 setTimeout(0) FIFO 落在交付单元之间（count '6' 在 inline#2 前），浏览器 fetch 任务在 parser 停车链（inline#2）之后——上游 assert_any **双真值已核**：053 alt1/alt3 分别 sanction 动态先行/让后两序（alt3 = 完整让后序），051 altB = 让后序 + count 6（批模型可达）。修法 = webview↔engine「交付排水门」（交付窗内插入的 classic src 挂起、排水毕 flush，webview run_page_scripts_impl 循环前后置旗标）——s355 首选候选；动态 src 族全量回归（050/053/execorder 族）随行。
- **052**（批解析可见性）：count-script-tags.js 在静态外链求值时数集合长度——浏览器流式解析 inline#2 未物化 → 4；引擎批模型全文档已物化 → 恒 5（修前因 report 缺位巧合 4）。流式物化架构面长挂，非调度可修。

## 验证

- **红态**：修前 solo 056 `lengths differ, expected 4 got 3`（evidence/logs/s354-solo-056-red.log）。
- **solo battery**（FILTER=execution-timing/05）：050/053/056/057/058/059 P + 051/052 红态（051 双 alternative 均不配：count 6 对 alt A 的 5 / alt B 的让后序；052 count 5 对 4）（evidence/logs/s354-solo-battery.log）。
- **域 join**（LC_ALL=C + (key,state) sort -u + 同 key+subtest 双态 A/B 均 0）：A=s353-b **1109P/13F/8T** vs B=s354-b **1109P/13F/7T**——F→P=2（056 目标 + 063 连带）；P→F=2（051/052）；T→P=1（063 Timeout 对消解）；残 11F 双侧逐键相同；key 集合两侧全同（319 键）。
- **clippy** zero-wpt-runner `--all-targets -D warnings` 净 + `cargo fmt --all -- --check` 净（纯 runner 改动，无 shim/引擎变更）。
- 全树附带面（非对照基线）：无 FILTER 全树 1182 键跑存 /tmp（style-element T 族等他子树红态为 the-script-element 口径外既有未基线面，不入账）。

## 挂账 / 下一步

- **051 交付排水门**（s355 首选）：见暴露面。
- 052 批解析可见性：流式物化架构面长挂。
- 143 upstream 真值复核（s352 挂账延续）；fetch-src/alpha/base、json-module parse-error-location、custom-element-exception、remove-src-attr、script-not-found 逐案勘根；import.defer 三案独立切片；probe12/probe13 自建单列；T=7 长尾在册。

## 日志

- evidence/logs/s354-solo-056-red.log（修前红态）
- evidence/logs/s354-solo-battery.log（05x battery）
- evidence/logs/s354-b-script-element.log（域跑 B 面）
- s353 基线 A 面：evidence/logs/s353-b-script-element.log（既有）
