# M1 / DC-2：runner 通道 + 桥建树 O(结果×树) 三处收敛

**日期**: 2026-10-28
**性质**: DC-2 通道落地 + 通道首跑暴露的共享路径性能修复 + dataset 缺失修复
**验证二进制**: release（清理插桩后重建，与提交树一致）

## 通道构成（DC-2）

- **子命令**：`zero-wpt-runner testharness-encoding-legacy-mb`（main.rs CLI +
  `testharness.rs` `run_encoding_legacy_mb_cases` →
  `run_any_js_corpus_subdirs_with_helpers_variants`，variant 展开照 navigation
  S4K 先例，iframe 管道照 html-syntax xmldecl R115 先例）。
- **语料叶目录**（fetch 脚本 `goals/31-encoding-legacy-mb.sh` DIRS 同域）：
  euc-jp / iso-2022-jp / shift_jis / euc-kr / gb18030 / gbk / big5 七目录。
- **per-dir skip 规则**：只跑 `*-decode.html` / `*-decode-<label>.html` wrapper
  与 `*-decode-errors.html`；`*-chars*.html`（iframe 子文档）、`*_errors.html`、
  encode-form/encode-href（goal 划界）、helper JS、`.headers`、`.any.js` 全 skip。
- **Makefile target**：`testharness-encoding-legacy-mb`（test-guard 包裹
  --per-proc-mem 4 --total-mem 8 --time-limit 5400；`ZW_SCRIPT_TIMEOUT_MS=600000`
  仅本通道放宽 V8 看门狗，缺省其他通道维持 90s 零变化）。
- **serve 侧解码修复**（fetch-handler）：合法 UTF-8 字节 + 非 UTF-8 声明的语料
  （iso-2022-jp 7-bit clean 形态）按声明走 `decode_document_bytes` 解码——plain
  wire 侧 shim 嗅探只落 characterSet 元数据、正文不重解码（change-encoding-and-
  restart 步缺失），转义序列全败。replacement 声明例外不解（恒 U+FFFD 毁页，
  html-syntax 2026-10-20 既定行为）；UTF-8/无声明恒等零变化。

## 首跑暴露的三处 O(结果数 × 树) 共享路径成本（全部修复）

### 根因 #1：浅臂建树每元素两次全文档重解析

`_zwMBuildBodyTree` 旧臂对每个元素回调 `__zw_parse_html_query` +
`__zw_parse_html_child_nodes`，各自全文档重解析——O(元素数 × 文档大小)。

- **证据**：chars 文档 250→4000 span，单 span 建树 12.7→85ms 陡增（二次曲线）；
  13k span 单次 iframe 装载 ≥90s 超 V8 看门狗。
- **修复**：host 侧新回调 `__zw_parse_html_subtree`（`parse_html_subtree_deep_json`
  + `deep_entry_json` template contents 内联）单次解析整子树深 JSON 一次下发；
  shim 深臂 `_zwMBuildDeepTreeNode` O(n) 本地建树。解析失败落旧臂零行为收敛。
- **门禁拦截的回归与修复（R4100）**：深臂初版用 `setAttribute` 逐个灌属性，
  attribute 写尾（R4100 details open 互斥 others-prong）在构造期逐 attr 触发、
  后者关前者，破坏解析产物的 host 裁决态（`make test`
  `test_details_name_exclusivity_r4100` T1：实测 `0,1,1` 期望 `1,0,1`——innerHTML
  产物应「树序第一 open 胜出」）。修复：属性改工厂 attrs 映射直灌（镜像浅臂
  `_zwMEl(snap)` 消费形态，id/cls 同浅快照喂 IDL 反射字段），null 原型映射承载
  `__proto__` 属性名（host JSON.parse 产物为自有属性，普通对象赋值走原型 setter
  会丢）。既有 `_zwMBuildDeepEntry`（WC-M2 template contents 路径，早于本 goal）
  同为 setAttribute 灌法，template contents 的 details open 互斥理论同暴露——
  既有行为不动，挂账 M3 甄别。

### 根因 #2a：`_zwMFindRealNode` 512 上限悬崖

`if (idx.size > 512) idx.clear()` 清空后仍回存 → 索引恒空、每次查找全量重建。

- **证据**：decode-common 单页 13.5k span，qsa 千元命中 = 1000 次 × ~14ms 重建
  ≈ 13.9s；全量页外层串分配 4GB 堆 OOM（修复前 176s 后 V8 OOM 硬终止）。
- **修复**：上限 512→200000（13.5k 条键 Map ≈ 1MB 级；失效语义不变，仍走各写点
  `_zwNodeIdx = null`）。

### 根因 #2b：元素级 wrap 桥臂 eager `_zwRootHtml = _zwMOuterHtml(root)`

每个查询结果全树序列化（50KB 级）——O(结果数 × 根树)。

- **证据**：n250 档 683ms 同源实证（250 wrap × ~2.7ms）。
- **修复**：`_zwRootHtmlLazy` 惰性 accessor（非枚举 defineProperty getter，读时
  才调生产者）。唯一读者 `_zwParseEl.prototype.matches`（part02）不在真节点
  （`_zwMEl` 产物）原型链上——真节点消费面（Element.prototype.matches 走真树）
  不读它，零开销。cached 臂（`_zwParseEl` 本体）维持 eager 重挂语义不变。

### 根因 #2c：doc 级 `_zwWrapCached` 桥臂 eager `detHtml()`

同款：每结果全文档序列化。

- **证据**：`Date.now()` 插桩定位（shim `performance.now()` 是虚拟时钟，插桩
  显示 16ms 而页面墙钟 12.87s——死点不在 walk/loop）：999 × ~12.8ms ≈ 12.8s，
  与 P1 墙钟 12.87s 精确吻合；第二查询再烧一轮致整案超时丢 P2/P3。
- **修复**：同款惰性化 + cached 臂 `instanceof _zwParseEl` 守卫。

### 修复效果

qsa 千元命中（13.5k span chars 文档）：**12,872ms → 74ms（172×）**。

## dataset 缺失修复（根因 #3）

`_zwMEl` 工厂节点（iframe/innerHTML 解析产物）缺 `dataset`——decode-common.js
showNodes 读 `nodes[i].dataset.cp` 直接 TypeError，解码正确也整链断（实测
P3-ds ds=A5 钉）。

- **修复**：`_zwMEl` 本地 attrs 版 dataset Proxy（camelCase ↔ data-kebab、
  missing → undefined、delete 真移除、ownKeys 走 getAttributeNames 过滤），
  语义对齐 part05 `_datasetProxy` 但直读本地 attrs，无 sel/handle 桥路径。
  spec：https://html.spec.whatwg.org/multipage/dom.html#dom-dataset

## 端到端结果

`eucjp-decode-x-euc-jp`（euc-jp 族 wrapper，14 variant 全展开）：

| | 修复前 | 修复后 |
|---|---|---|
| 结果 | ~176s 后 V8 4GB 堆 OOM 硬终止（qsa 13.9s × 多查询 + dataset TypeError 整链断） | **13,459 Pass / 0 Fail**，无 OOM 无超时 |

rebase 后全树复核（origin/main 含并行流 s124–s126 engine 变更合入，fmt/
clippy 过后重跑）：`make testharness-encoding-legacy-mb FILTER=euc-jp/`——
euc-jp 族 4 wrapper（`eucjp-decode{,-cseucpkdfmtjapanese,-x-euc-jp}.html` 三
chars 档 ×14 variant + `-errors`）**40,388 Pass / 0 Fail**，exit 0。

最终构建（R4100 工厂直灌修复 + null 原型映射后）复验一致：13,459P/0F、
14 case、EXIT=0（`testharness-encoding-legacy-mb eucjp-decode-x-euc-jp --json`）。

## 质量门禁记录（2026-10-28）

- fmt `--all -- --check`、clippy workspace `-D warnings` 零 warning。
- `make test` 六次尝试：①engine lib R4100 拦截（本 goal 深臂回归，已修复复绿）；
  ②integration `image_batch_e2e` 报 zero-browser 夹具缺失（环境工件，`cargo
  build -p zero-browser` 补齐后消失）；③④⑤ image_batch 41≠36 / network peak
  7>6 负载下散布红——R5295（同案 37≠36 干净 HEAD 逐字节复现）/ R5300（同两案
  隔离复跑 2/2 绿）已记账 flake 家族，本日两案隔离复跑各 2/2 绿。
- 腿补全（配方逐条，flake 两案腿 1 跳过、隔离补绿）：workspace 腿（--skip 两
  flake 案）、renderer v8 bin/lib、browser bin/tests（xvfb 前缀）、GPU adapter
  双腿、quickjs clippy + workspace crates + renderer bin——全绿。
- `quickjs renderer lib` 腿 `mutation_arm_does_not_apply_across_document_reset_window`
  红：**stash A/B 定责 HEAD 同样 0.63s 复现同断言**——既有问题非本改动（R5295
  跨流散布红清单「renderer page_scripts」同族），按车道规则记录不修。
- 附带发现（与本 goal 无关，只记录）：zero-browser feature 统一下
  `crates/script-sandbox` 冒出 3 个既有 dead-code rustc warning
  （HOST_CALL_TRACE 族，v8 workspace clippy 门不覆盖该 feature 组合）。

## 共享面影响与回归对账

本改动触面（`_zwMFindRealNode`/`_zwMWrapCached`/`_zwWrapCached`/`_zwMEl`）是
DOM 桥共享路径。六通道 lane（make test-guard 守卫入口，dom TIME_LIMIT=1800
对齐 M0 envelope）前两通道结果与归因：

| 通道 | 结果 | 归因 |
|---|---|---|
| testharness-dom | `window-named-properties.html` Dynamic/Ghost name 2F | **HEAD（stash 去本改动）二进制同现 2F**——基面属性 |
| testharness-html-semantics | 839F/4381P（431 文件；relevant-mutations.html 46F/67P 等；语料 mtime 未变） | **HEAD 二进制逐字节同现**（46F/67P 精确一致）——基面属性 |
| navigation | 中途停止（为 A/B 让路） | 改由六通道 `--json` 集合恒等对账覆盖 |
| encoding / legacy-mb / web-components | 未起跑 | 同上；legacy-mb 整跑 JSON 由对账链产出，兼作 M2 基线数据源 |

**基面位移说明**：本轮 mid-flight 并行流 s124–s126 engine 提交合入
（rebase），M0 全绿基线（html-semantics 5306/5306 等）随基陈旧——上述失败
为新基属性，与本改动无关（stash A/B 证据：本改动前同基二进制已存
`/tmp/zero-wpt-runner-head-nodc2`，供对账链作 before 侧）。零回归结论改由
六通道 `--json` 集合恒等对账承担（本改动二进制 vs head-nodc2，同基两侧）。

rebase 后定向复验（新基 × 本改动）：R4100 单测复绿（1 passed）；fmt/clippy
复跑绿；euc-jp 全族 `make testharness-encoding-legacy-mb FILTER=euc-jp/`
**40,388P/0F、exit 0**（4 wrapper：decode ×3 chars 档 ×14 variant + errors）。

语料续拉复试（FORCE=1 + GITHUB_TOKEN + 代理）：网络仍 degraded（curl
timeout），盘点维持 136 html——M1 语料补全卡点不变。

## 方法论备忘

- shim 内 `performance.now()` 是虚拟时钟——桥路径耗时插桩必须 `Date.now()`。
- `ulimit -v` 会触发 V8 `Oilpan: CagedHeap reservation` 硬失败（8G/16G 同），
  runner 无人值守内存封顶用外部 RSS 看门狗（阈值杀进程树）。
- runner CLI：filter 是位置参数；`--json` 须在子命令之后。
- 诊断桥 `__zw_dbg_log`（`ZW_DBG_LOG=1` 输出 stderr）保留为长尾案定位工具。
