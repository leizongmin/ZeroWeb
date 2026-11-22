# DC-2 · 批十五：CI 分片三通道语料缺档定谳补档（fetch 目标＋支撑文件闭包）

2026-11-22。控制点 run 44972755575 兑现批十四「待实证」后，对其全分片产物
与作业日志做 DC-3 前置定谳：三类 CI 语料缺档（批十四 infra 腿之外的通道级
余量），无 Rust 码笔，fix commit cde99ad26（5 文件 +60/−1）。

## 控制点兑现

44972755575（批十三码＋缺档语料）快照 coverage-2026-11-22.json
`html/infrastructure zw_passes=82` 精确复现批十四缺档形（批十四证据「待实证」
预期 82 形逐字命中）＝对照数据点落账。控制快照组基线（DC-3 对账用）：
zw_passes 1,914,204/2,276,004（84.1%）；html/dom 59,634 · semantics 8,508 ·
canvas 3,198 · rendering 2,793 · syntax 2,747 · **infrastructure 82** ·
the-xhtml-syntax 25,606 · browsers 217 · **webappapis 3**。

## 三通道缺档定谳

根因类同批十四但发生在**分片 fetch 清单层**（非支撑文件层）：CI fetch 步只拉
`ci-shards.yml` 本分片 `fetch:` 目标（＋bootstrap
`fetch-wpt-html-testharness`），套件经 `zero-wpt-runner testharness-<s>` 直跑，
**Makefile 套件依赖链在 CI 不生效**——goal 脚本不在分片 fetch 行内＝语料零入账。

1. **syntax-dom 分片缺三 fetch 目标**（批四/五/六语料）。run 44972755575
   syntax-dom 作业日志语料清单实测：webappapis 语料 1 文件、browsers-window 与
   semantics 补簇全缺（CI fresh checkout 零带入）——三通道快照挂账
   （webappapis 3 / browsers 217 面内零 / semantics 缺补簇 1,955 面）全是本形。
   补齐：`ci-shards.yml` syntax-dom fetch 行加
   `fetch-wpt-html-webappapis fetch-wpt-html-browsers-window
   fetch-wpt-html-semantics-complement`；预算不变（13,200s 按 93 min 本地
   全语料校准，三语料带入后新增 ~5k 文件拉取属 fetch 步预算、套件墙钟增量
   由批四/五/六本地实测 ~15 min 量级覆盖）。
2. **speculative 通道缺 `/common/utils.js`**（批十二通道，catch-all-4）。
   run 44972755575 catch-all-4 产物实测 126 F / 0 P / 0 TO（本地同码 43 P /
   83 TO）：126 键全灭「token is not defined」——token() 定义源
   `/common/utils.js` 在 63 脚本拉取子树外，同批十四 62 脚本根因类。补齐：
   63 脚本 +1 `fetch_raw "common/utils.js"`。
3. **批四/五/六脚本支撑文件闭包 14 文件**（16 个 `fetch_raw` 位点，57/59
   同文件各自自足）。分片作业只拉本 goal fetch、本地由其他通道带入的共享
   支撑文件，CI 必缺：57 加 cors/support.js（unhandled-promise-rejections 2 页）
   ＋ resources/WebIDLParser.js（event-handler idlharness 形 8 页）；58 加
   common/get-host-info.sub.js（cross-origin-objects function-* 13 页）＋
   common/utils.js＋common/dispatcher/dispatcher.js（agent-clusters
   regression 页）＋ images/green.svg（indexed-browsing-contexts-02）；59 加
   resources/WebIDLParser.js＋idlharness.js（interestfor idlharness 页）＋
   check-layout-th.js＋common/top-layer.js＋close-watcher helpers.js
   （popovers 7 页）＋ fonts/ahem.css＋images/blue.png（interestfor 字体/事件页）
   ＋ media/movie_5.mp4＋sound_5.mp3（command-and-commandfor 音视频 3 页）。
   页面清单为 HTML_SYNTAX_CORPUS_SUBDIRS 计分域实跑页引用闭包扫描产物。

## 非缺口澄清（方法论记录，防误账复演）

朴素「本地语料文件 − CI 结果键」差集会严重超计：初稿对 rendering 差出
556/706 文件、html/dom 差出 259 文件并据此改写 56 脚本深拉——被 CI 作业
日志清单证伪（dom/syntax 通道案数与本地枚举一致＝全量对齐）。真规则是
`testharness_html_rendering.rs` 枚举排除面：跳过 support/resources/reference/
crashtests 目录、-ref/-notref 页、.xhtml、无 testharness 引用页，且只枚举
.html＋.window.js——按枚举规则模拟键域后 rendering **152=152 全对齐**
（4 个「缺失键」实为 runner 不枚举的 .js/.json 资产）。56 脚本误改与测试
拉取标记已回退（`git checkout --`＋删 `.goal-fetched-*`），rendering/html-dom/
html-syntax 三通道**零行动**。runner py-routes（slow.py、speculative
stash.py、cache.py、set-cookie.py）为行为等价模拟，.py 引用非缺档。

## 修复与验证

- 5 文件：`ci-shards.yml`（fetch 行＋批十五注释）＋57/58/59/63 四脚本
  `fetch_raw` 行＋注释块记根因；`bash -n` 全过。
- 本地拉取幂等重跑（GITHUB_TOKEN 认证）：57/58/59/63 FAILED_FETCHES=0
  （fetch_raw 目标全在盘跳过——本地已有、CI 分片缺，正是补档语义）。
- `ci_discover_shards.py` 本地解析（`--runner-src tests/wpt-runner/src/main.rs`）：
  syntax-dom 携全 5 fetch 目标、53 分片总数不变、catch-all-3/4 清单不变。

## DC-3 对账运行

- 44983026495（批十五前码 e2ea19cf7，03:59:18Z 派）**取消**：其账面缺
  四通道（webappapis/browsers/complement/speculative 语料）必为中间态；
  infra 通道输入两 run 完全一致（批十五不动 infra 域），批十四验证力由新
  run 全额承接，且单快照对账避免双 coverage 快照撞车。
- **44985566993**（cde99ad26，04:33:18Z 派，run_wptfyi_coverage=true）在飞，
  预期 ~2.2h。对账预期形：infrastructure 82→~600+（批十四补档兑现，本地
  674/376/138）；webappapis 3→~1,537 face（333 键分母）；browsers 217 面内
  +40（234 键分母）；semantics 8,508→+1,955 face（补簇 428 键）；syntax 内
  speculative 126 F→43 P/83 TO（批十二勘误后首跑）＋template +42；xhtml
  25,606、rendering 2,793、dom 59,634 持位（rendering 渲染流 -62 漂移另账
  不入本流）。

[逐文件 JSON](2026-11-22-dc2-batch15-ci-shard-corpus-channels.json) ·
[批十四证据](2026-11-22-dc2-batch14-ci-corpus-support-files.md) ·
[接入预检证据](2026-11-22-dc3-weekly-integration-preflight.md)
