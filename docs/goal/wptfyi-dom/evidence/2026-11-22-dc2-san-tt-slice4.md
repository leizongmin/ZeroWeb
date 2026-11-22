# DC-2 · 切片 4（sanitizer/trusted-types 语料+通道三缺面，2026-11-22）

日期：2026-11-22。基线：切片 3（`5e217bdba`）。目标：DC-1 优先序第三簇落地——sanitizer-api（3,834st）/ trusted-types（3,764st）的「语料+通道+API」三缺中先补前两缺，API 刀位按基线失败面 census 设计。

## 语料（WPT_REV pin sparse 克隆）

`tests/wpt-runner/scripts/fetch-sanitizer-tt-subset.sh`（新）：与 fetch-gap-batch1-subset.sh 同一 pin `315976933870b34d6ea30e3f6643403edae678ba`，sparse blob:none 浅克隆（逐文件 raw + contents API 在百文件量级超时/限额——batch1 先例）；覆盖 `sanitizer-api/**` + `trusted-types/**` + 共享 harness 资产（resources/testharness.js、testharnessreport.js、common/dispatcher、common/security-features）。落盘实测：

- `wpt-data/sanitizer-api/`：28 页 .html 全顶层 + support/（helper 资产）。
- `wpt-data/trusted-types/`：225 页 .html 全顶层 + support/（9 .html 全是 iframe/worker 载体页与 helper 脚本，含核心断言域 `helper.sub.js`）。

哨兵教训：初版脚本哨兵按上游后期布局猜（`sanitizer-api/sanitizer-api.html`、`trusted-types/dedicated-tests/...`），本 pin 实测均不存在——dedicated-tests/ 是上游后来的布局；修正为实测文件 `sanitizer-basic-filtering.html` + `DOMParser-parseFromString.html`。**fetch 脚本哨兵必须 ls 实测后落笔，不得按记忆猜路径。**

## 通道（runner + Makefile）

- `testharness.rs`：`SANITIZER_SUBDIRS = &["sanitizer-api"]`、`TRUSTED_TYPES_SUBDIRS = &["trusted-types"]`（support/ 不入清单——wai-aria manual/ 先例）+ `run_sanitizer_api_cases` / `run_trusted_types_cases`（共用 `run_corpus_subdirs` + `gap_batch1_case_skipped` 谓词，worker/iframe/非 testharness 形态按内容规则排除）。
- `main.rs`：两 dispatch 臂 + `cmd_testharness_sanitizer_api` / `cmd_testharness_trusted_types` + help 文本。
- `Makefile`：`fetch-wpt-sanitizer-api`（fetch-wpt-trusted-types 同目标体）+ `testharness-sanitizer-api` / `testharness-trusted-types`（target-disk-guard → fetch → test-guard → release runner，通道模板五件套；`FILTER=` 路径子串透传）。

## 基线腿（守卫入口）

锚定：HEAD `5e217bdbafcf` + 树内兄弟流 CSS INSETS-V4 WIP（layout-engine 4 文件）恒在于 A/B 两侧、差量归因内部抵消（DC-1 碰撞记录先例；禁 stash 他流字节）。构建链本次全量重链 layout-engine→net→engine→page-runtime→webview→zero-wpt-runner（兄弟 WIP 字节演化所致），二进制 md5 见腿记录。

- sanitizer-api 腿（`make testharness-sanitizer-api TIME_LIMIT=3600`，release 增量 55.09s，runner md5 `0805cfb79f`）：**39P/363F/1T**，27/28 文件执行（`sethtml-xml-document.html` 正文含字面 `<iframe>` 元素被 `gap_batch1_case_skipped` 内容规则排除，谓词按政策执行记账不改）。
- trusted-types 腿（`make testharness-trusted-types TIME_LIMIT=3600`，同二进制 `0805cfb79f` 两腿字节绑定，约 7 分钟）：**31P/304F/30T**，179/225 文件执行（54 个未执行全为 DedicatedWorker-* / iframe 载体 / CSP-adoption worker 形态——`gap_batch1_case_skipped` 按设计排除）。

失败面形态（san 腿）：整面 `Sanitizer is not defined` 型 wholesale（全 log 命中 104 处，含 4 处 `trustedTypes is not defined` 交叉依赖）；F 大头 sethtml-tree-construction 81F、sanitizer-config 71F、sanitizer-javascript-url 42F、sanitizer-parseHTML 25F；真 Pass 面 39P——html5lib-basics 11P / sethtml-safety 12P / processing-instructions 2P 等不触 API 的解析行为面。执行子测 403 ＜ DC-1 静态估计 3,834st：DC-1 计 idlharness 变体与静态全集，runner 实跑窗口口径以本腿为准。

失败面形态（tt 腿）：整面 `trustedTypes is not defined` 型 wholesale（149 处，另有 TrustedTypePolicyFactory 2 处 / TrustedHTML 1 处）；F 大头 TrustedTypePolicyFactory-getPropertyType.tentative 28F、createPolicy-createXYZTests 28F、trusted-types-createHTMLDocument 12F、Document-write 8F、block-string-assignment-to-* 簇（outerHTML/insertAdjacentHTML/setHTMLUnsafe/parseHTMLUnsafe/DOMParser/createContextualFragment 各 4–7F）；真 Pass 面 31P——trusted-types-event-handlers 5P / no-require-trusted-types-for 3P 等不触 factory 的面。30T 全为 navigation/reporting 页（trusted-types-navigation.html 按 query 分片 ?21-25/?26-30/?31-35/?36-last 等）——真实导航/iframe 管道域（runner 无 iframe 文档管道，DC-1 已记账），非 API 面。

判读注：本腿日志复现 grep 静默失效（`grep -c 'not defined'` 对含 Fail 行提取返回 0，而 `grep -aoE` 同文件命中 149）——计数与对账全走 `LC_ALL=C awk` 口径（切片 3 沉淀先例）。

## API 刀位（按 census 设计，切片 5 入场序）

1. **TrustedTypePolicyFactory 核心**（`window.trustedTypes`：createPolicy / is* / getAttributeType / getPropertyType / emptyHTML / emptyScript / defaultPolicy + TrustedHTML/Script/ScriptURL 类型对象）——破 tt 腿 wholesale 面（149 处 not defined + block-string-assignment-* 簇 30+F），policy 行为面（回调变换、异常序、名字规则）在无 CSP enforcement 时可大量转 P。
2. **Sanitizer 构造面**（`new Sanitizer(config)` + Element.setHTML/setHTMLUnsafe 新方法挂接）——破 san 腿 wholesale 面（104 处 + sethtml-* 簇 ~140F）；config 校验/合并语义按 spec 配链接。
3. **sink enforcement（CSP require-trusted-types-for 门）**：需 CSP 管道联动（跨 crate 面），最后做。
4. 30T navigation/reporting 页：runner 导航/iframe 管道域，非 API 面，不入刀序（记账同 DC-1）。

净增记账：本片通道落地即 +70P（san 39 + tt 31）进覆盖面；F 池 667（363+304）为切片 5 转化目标上限。

## 剩余挂账

- API 面（TrustedTypePolicyFactory/Policy/TrustedHTML·Script·ScriptURL + sink enforcement CSP 门）：切片 5 候选，按本片 census 定刀序。
- 页预算截断带、RT 深语义 41F、accname testdriver 通道：切片 2/3 遗留泊位不动。
