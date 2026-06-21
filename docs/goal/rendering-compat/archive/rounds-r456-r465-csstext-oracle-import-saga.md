# 归档：R456 / R458-R465 css-text 导入 + oracle 工具 saga（2026-06-21）

> 从 master.md 顶部压缩归档（R466 doc-maintenance 轮）。这些轮次是 css-text 第 9 目录的**导入工具链修复 + self/oracle in-flight 观测 saga**。核心数据（self 89.2% R463 / import cluster map R461 / writing-modes 全量 R457 / 当前 oracle in-flight R465）仍保留在 master.md 顶部；本档存被移出的工具 saga / in-flight / pre-confirm 原文。逐轮原文按时间倒序（master.md 原顺序）。

## Saga 时间线指针（一句话）

- **R465**（留顶）: oracle 全量捕获 in-flight（collectTests 修复生效验证，~51min）
- **R464**（归档）: oracle 缺失根因 = 捕获脚本 `capture-oracle-per-dir.mjs` 顶层 `readdirSync` bug（8/1914，同 R458/R459 discover 谱系）→ collectTests 递归修复
- **R463**（留顶）: css-text self 折回 89.2%（1402/1572，聚类反转 white-space/text-align 真失败 vs i18n 假通过）
- **R462**（归档）: self reftest in-flight + 分母修正（1822 runner cases）+ JS 依赖量化（24.5%）
- **R461**（留顶）: css-text 全量导入 COMPLETE（2994 文件/1914 test-ish）+ 聚类映射（oracle 预测 ~30-38%）
- **R460**（归档）: 纠正 R459 — GITHUB_TOKEN gate 移除（10db810 git/trees recursive=1，2 调用/目录）
- **R459**（归档）: 子目录递归 resolved（1568cb0）+ 新 gate GITHUB_TOKEN（后被 R460 移除）
- **R457**（留顶）: css-writing-modes 全量 LANDED（9735eff，8/9 目录，self 78.0% / oracle 5.6%）
- **R458**（归档）: css-text pre-confirm — 嵌套目录（30 子目录），顶层 discover 得 0 对（工具阻塞）
- **R456**（归档）: flexbox probe chromium-Oracle 真影响量化（+7 DC-14 win，已并入基线段 flexbox 50.6%）

---

## 逐轮原文（被移出 master.md 顶部者）

### R464 css-text oracle 缺失根因定位：捕获脚本顶层 readdirSync bug（同 R458/R459 谱系，read-only）

R463 后查 oracle 数据仍缺，定位根因非 chromium/oracle 本身。17:21 试捕 `/tmp/csstext_oracle.log` 只 `css/css-text: 8 ok, 0 fail`（应 1914）。**根因**：`capture-oracle-per-dir.mjs`（HEAD）`readdirSync(dir).filter(...)` 只读 category **顶层**文件；css-text = 9 顶层 .html + 30 子目录（test 散落 white-space/i18n/hyphens/...），实测顶层 test-ish excl-ref = **8**（与 log "8 ok" 精确一致）/ 递归全量 = **1914** → 顶层漏掉 30 子目录全部 test。**同构 bug 谱系**：R458/R459 已发现+修 discover-reftests-authoritative.py 顶层 bug（1568cb0），本轮发现 oracle 捕获脚本**完全相同** bug。**修复在途**：并行 agent working tree（未提交）新增 `collectTests(dir)` 递归收集子目录 test，扁平目录退化行为不变（无回归）；oracle=test 侧单点完整（不需 discover R458 的 ref 路径修复）。8 ok 0 fail = 脚本基础功能正常仅范围 bug。**全量耗时预判**：1914 截图 × ~1.6s（writing-modes 787 oracle≈21min 参考）≈ **51min**；node+puppeteer 脚本不经 test-guard/make reftest，无 1800s time-limit 可长跑。**系统性印证**：css-text/CSS2 嵌套目录触发 discover/import/oracle 三脚本同构顶层 bug；CSS2（43 子目录）下批 oracle 直接受益此修复。零代码变更。详见 `evidence/r464-csstext-oracle-subdir-bug-2026-06-21.txt`。

### R462 css-text self-source reftest IN-FLIGHT 观测 + 分母/JS 依赖量化（read-only）

承接 R461。并行 agent 触发的 css-text self-source reftest（`reftest-upstream css-text`）仍在运行（无实测 self/oracle 数据）。**分母修正**：runner 实测 **1822 case**（filter=`case.id.contains("css-text")` main.rs:557 确认仅 css-text 范围；权威 `link rel=match` 对，兑现 R461「<1914 待 runner 跑出」预留；非 R461 的 1914 test-ish 文件数）= **迄今最大分母**（flexbox 496 / writing-modes 786 / **css-text 1822**，2.3× writing-modes）。**健康运行非卡死**：PID 62124 elapsed 15:13（913s 墙钟）/ CPU 1295% 满载 / 5GB 内存（<6GB per-proc 限）/ 日志持续刷新；预计 ~22-23min 完成（writing-modes 0.74s/case 外推 1822×0.74≈1348s），time-limit 1800s 有 ~450s 余量，但 i18n(BiDi 16 脚本)/hyphens(大词典) 尾部 case 有超时风险。**JS 依赖量化（R461 预测显性修正因子）**：css-text 469/1917=**24.5%** test 含 `<script>`，runner 报 102 次 `[reftest JS] unknown error`（~5.6% cases）→ 双向影响（self 假通过 / oracle 拉低），符合文字类目录 self 99% / oracle ~30% 分化（fonts 99.3%/34.8%、text-decor 99.2%/28.9%）→ 不改 R461 30-38% oracle 区间，但提示 self↔oracle gap 可能 >60pp，下一轮须以 **oracle chr<1%** 为唯一可信指标。missing-ref 4 个 hanging-whitespace（/css/reference/ 跨目录共享 ref 未导入）计 fail，0.2% 可忽略。详见 `evidence/r462-csstext-reftest-inflight-js-dep-2026-06-21.txt`。

### R460 ⚠️ 纠正 R459：GITHUB_TOKEN gate REMOVED（10db810 git/trees recursive=1）

承接 R459「css-text code-ready，credential 层待 GITHUB_TOKEN」。并行 agent commit **10db810** 把 `collect_test_paths` 从逐子目录 `gh_api(contents)` 递归（css-text ~30/CSS2 ~43 subdir 各 1 调用 = 30-75 次，超 60/hr）改为**单次 `git/trees/{sha}?recursive=1`**（contents(parent) 取 tree SHA + 递归取整棵子树 = **2 调用/目录**），+ 截断守卫（>100k 条目拒绝静默部分导入，保 DC-14 分母完整）。**gate 移除实证**（commit msg）：「css-text = 2994 blobs / 1914 test files，not truncated；clears the rate-limit blocker without a GITHUB_TOKEN」。**本轮独立复验**：dry-run 因本 shell 核心 budget **0/60**（上轮 R459 耗尽，38min 后 reset）首次 gh_api 即 403——**非 gate 问题**（2 调用设计 << 60，是本 rollover shell 自身 budget 耗尽）；fresh budget 下秒完成（并行 agent commit msg 已实测）。**纠正 R459**：css-text 全量导入**无 credential gate**，**完全 unblocked**（R459「子目录递归 RESOLVED」仍成立，10db810 是其 perf 优化非推翻）。**css-text 权威规模**：1914 test 文件（2994 blobs），权威对 ≤1914，精确对数待 fresh-budget dry-run。详见 `evidence/r460-github-token-gate-removed-2026-06-21.txt`。

### R459 css-text 全量导入工具阻塞 RESOLVED（1568cb0 子目录递归）+ 新 gate=GITHUB_TOKEN

**⚠️ gate 部分已于 R460 纠正：10db810 git/trees recursive=1 移除 token gate，css-text 完全 unblocked，见 R460**。承接 R458「css-text 30 子目录 / CSS2 43 子目录，顶层 discover 得 0 对 = 工具阻塞」。并行 agent commit **1568cb0** 落地 R458 可执行清单三项（`collect_test_paths()` 递归每 subdir 1 次 gh_api + ref 路径相对 test 文件目录解析 + 可选 GITHUB_TOKEN 60→5000/hr）。**本轮独立实测（只读 dry-run probe）**：css-text dry-run **递归已生效**——脚本能深入子目录（white-space/segment-break/...）收集 test 文件，**R458「0 对」阻塞在代码层 RESOLVED**；但**全量 pair 计数未取得**——递归中途触发 403 rate limit（~31+ subdir gh_api 超 60/hr 未认证预算），与 1568cb0 commit msg「hit 60/hr mid-way」一致。**结论**：css-text 全量导入（第 9 目录）**code-ready，credential 层待 GITHUB_TOKEN**（或等 60/hr 窗口/缓存 gh_api）= 并行 agent 下一导入批次前向；**非 BLOCK**（credential/env 依赖非平台根本性不可用）。**stale 文件对账**：`evidence/r459-writing-modes-oracle-crossvalidate-2026-06-21.txt`（untracked，上轮 429 中断残留）= writing-modes oracle cross-validate，**结论已完整并入 R457 header**（44/786=5.6% / 8-dir 26.8% / 分布 / 0 lever），**非本轮 R459 内容**（主题撞车）；本轮不 commit 非 own 文件，保留 untracked 待上轮 owner 处置。详见 `evidence/r459-csstext-subdir-recursion-resolved-2026-06-21.txt`。

### R458 css-text 全量前 pre-confirm（只读，第 9 目录备基线）— 嵌套目录，顶层 discover 得 0 对（工具阻塞）

**⚠️ 已于 R459 RESOLVED：1568cb0 子目录递归；R460 进一步移除 token gate（10db810），css-text 完全 unblocked，见 R460**。css-text wpt-data 仍 0 文件（未导入）。**关键结构性差异（区别前 8 个扁平目录）**：`css/css-text/` 顶层 = **9 html + 30 子目录**（bidi/hyphens/line-breaking/overflow-wrap/text-align/text-indent/text-justify/hanging-punctuation/tab-size/letter-spacing/line-break/text-autospace/text-spacing-trim/text-wrap-style/…）；reftest 全在**子目录**内（spot-check：`line-breaking/` 187 文件/56 ref≈131 test、`hyphens/` 51/3）。`discover-reftests-authoritative.py`（58d8aa8 后版本）仅读**顶层** → 对 css-text **得 0 权威对**（顶层 9 html 无 `link rel=match`）。**工具阻塞**：导入 css-text 须先让 discover/import **递归子目录**（或逐子目录调用），否则 0 导入；**CSS2 同构已核（更甚）**：`css/CSS2/` 顶层 18 html + **43 子目录**（abspos/backgrounds/borders/box/floats/normal-flow/positioning/linebox/…），顶层少数 html 有 `link rel=match`→`reference/`（discover 能得若干对，区别 css-text 的 0）但主体在 43 子目录同样被顶层 discover 漏掉；另 **`css/css2/` 上游 404 不存在，真实目录仅 `css/CSS2/`（大写）**。**实现核查**（read-only grep）：核心换行属性已注册+解析+apply（white-space✅/word-break✅ 含 break-all·keep-all·break-word·normal 全值/overflow-wrap✅/hyphens✅ default None/tab-size✅ default 8，与 M5「CJK 换行+justify 修复」声称一致）；缺口预计在 layout 消费精度 + 较新属性（text-autospace/text-spacing-trim/text-wrap-style/text-justify/line-break 禁则）。**预测**：权威对数须待递归导入后量化（30 子目录估数百 reftest）；oracle 预测**暂不给出**（工具未就绪前过早）——line-breaking 核心 ZW 已有 CJK 换行故或高于 text-decor 28.9%，但 hyphens/justify/autospace 精度=字体/度量谱系有下拉风险。

### R456 probe chromium-Oracle 真影响量化（验证为真 DC-14 win，非 selfsource 噪声）

REFTEST_DUMP + cross-validate vs R425 chromium oracle（chromium 不随 probe 变，复用 502 截图）实测 css-flexbox **chromium-Oracle chr<1% 244→251/496 = 49.2%→50.6%（+7, +1.4pp）**。selfsource 净 +14 中仅 +7 是真匹配 chromium（差值=同源假通过/假阴性吸收）；103 case self-fail 但 chr<1%（同源假阴性：ZW-test≈chromium 仅 ZW-ref 发散）。**结论**：probe（commit 574db50）是经独立 Oracle 验证的真 DC-14 win，当前基线 flexbox oracle 更新 49.2%→50.6%、7 目录聚合 622→629/1726=36.0%→36.4%（R425/R426 round-record 保留历史 pre-probe 49.2%）。R429（88e11d2）oracle 隔离实测 **+1 DC-14 win**（probe-only 250→probe+R429 251/496；selfsource 同源抵消 net-0 但 oracle 揭示真匹配 chromium，纠正 R455「非 pass-rate lever」结论）。css-flexbox oracle 总 delta = **+6（probe/R428，244→250）+1（R429，250→251）= +7** vs R425 244（R456b 隔离 probe-only 250，校正 R456「probe +7」未扣除 R429 的双计）。详见 `evidence/r456-probe-oracle-impact-2026-06-21.txt`。
