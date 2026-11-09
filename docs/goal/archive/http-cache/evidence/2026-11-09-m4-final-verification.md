# M4 终验——全窗 A/B + fetch 头条不回落核对（DC-4 收口）

**日期**: 2026-11-09
**前置**: M3 corpus 批次（@ 30584b1e7，[M3 corpus 证据档](2026-11-08-m3-corpus-import.md)）
+ 主干同步至 252fbb3fd0（46 号切片六 6a，SW fetcher 分表，行为保持声明）→
终测树 70203b7e9（252fbb3fd0 + ci-catchall 一行注册表救济，runner 行为零变化）
→ 全窗 A/B 同树；post-goal CI 对账树 6b3a7d00c（+ weekly 管线容错），
头条聚合含空文件容错 757f4db84。

## 一、结论

- **全窗 A/B 终验通过**：当前主干全 net-api 窗（952 文件，
  **13,992P/1,322F/9T**）对 M2 全窗账（939 文件，13,868P/1,308F/9T）
  **真 P→F = 0、F→P = 0、共享文件 subtest 集合零漂移**，唯一差异面 =
  fetch/http-cache 13 新文件（124P/14F）纯增量；对 v3 终账（767 文件，
  13,275P/1,412F/9T）**真 P→F = 0**，F→P 104 恰为 M2 记账收敛集（族 102 +
  mode-same-origin 2），零意外翻面。DC-4 零回归门（v2/v3 逐 subtest）关死。
- **fetch 头条核对（终验 ✅）**：基线 = v3 复测 **2,266P/17.07%**（2026-11-02
  universe，den 13,274）。post-goal 全量对账（dispatch run 42499865933，
  树 6b3a7d00c，manifest 窗 chrome 157 @ fea36e49dd 2026-11-08）fetch 组 =
  **2,369P/13,274 = 17.85%**——**+103P / +0.85pp，不回落门通过**；其中
  fetch/api 二级目录 2,098→2,202P（+104 = 族 102 翻面 + corpus 124P 入账，
  经 universe 漂移相抵后净 +104）。快照文件因环比护栏（§四）未回写 main，
  本数值由同一 run 的全部分片 artifact 本地同脚本聚合产出（§五）。

## 二、全窗 A/B 方法与账（JSON-vs-JSON，M2 证据档 §三同款）

- **运行**：本地 systemd-run MemoryMax=24G scope + test-guard
  （--per-proc-mem 4 --total-mem 8 --time-limit 3600），runner
  `testharness-net-api --json`（退出码 1 = 存在失败用例，预期），约 9 min；
  产物 `/tmp/zw-results-httpcache-m4/testharness-net-api.json`。
- **对比脚本口径**：逐文件逐 subtest 的 (name, status) 多重集对比；
  跨基线分别算 P→F / F→P / 共享文件内 subtest 增删 / 文件增删。

### vs M2 全窗账（`/tmp/zw-results-httpcache-m2`，2026-11-08 20:48）

| 维度 | 账 |
|---|---|
| 共享文件 | 939（零丢弃） |
| 真 P→F | **0** |
| F→P | **0** |
| 共享文件内 subtest 增/删 | **0 / 0** |
| 新增文件 | **13，全为 fetch/http-cache**（124P/14F = M3 corpus 终跑复现） |
| 总账 | 939→952 文件；13,868P→13,992P（+124，恰为增量面） |

### vs v3 终账（`/tmp/zw-results-m4b`，2026-11-07 03:47）

| 维度 | 账 |
|---|---|
| 共享文件 | 767（v3 面全覆盖，零丢失） |
| 真 P→F | **0** |
| F→P | 104 = request-cache 族 102（no-store 8 + only-if-cached 14 +
  default-conditional 40 + no-cache 4 + reload 12 + force-cache 16 +
  default 8）+ basic/mode-same-origin 2——与 M2 证据档记账逐文件一致 |
| 共享文件内 v3 subtest 丢失 | 0 |
| 新增文件 | 185（M2 批次 172 + 本 goal http-cache 13） |
| 总账 | 13,275P/1,412F/9T → 13,992P/1,322F/9T |

- **勘误**：控制面/43 号档案所记「v3 终账 13,256P」为笔误，JSON 实测
  **13,275P**/1,412F/9T（M2 证据档 §一数字正确）；本档按 JSON 实测口径。

## 三、fetch 头条不回落——cron 快照面核对

- **基线锚**：v3 复测 2,266P/17.07%（wpt.fyi universe 2026-11-02 快照分母
  13,274；[43 号档案](../../net-api-compat-v3.md) coverage
  对账表）。
- **coverage-2026-11-07 快照**（weekly cron run 42389581601，所测树
  56f1febb7）：fetch 组聚合 **2,265P/13,324 = 17.00%**。
  - 该树含 v3 全部面孔、**不含本 goal 任何提交**（M1 首提交 f1fb5956b 于
    11-08 18:50+08 落地，晚于快照 17 小时）——它证明的是 post-v3 态在 CI
    面的稳定性：−1P / −0.07pp = 跨环境抖动 + universe 漂移（分母
    13,274→13,324，+50 系上游窗口漂移），无引擎归因面。
  - 快照轨迹：11-05 = 1,894P/14.25%（pre-v3 树）→ 11-07 = 2,265P/17.00%
    （v3 面入 CI 窗 +371）。
- **post-#47 头条（实测回填）**：dispatch run **42499865933**（树 6b3a7d00c）
  36 分片全部跑完（仅 html-sem 红，见 §四），fetch 组 **2,369P/13,274 =
  17.85%**。对 11-07 快照同表行（2,265P/13,324 = 17.00%，manifest 窗
  chrome 156 @ ffc4686d11）**+104P / +0.85pp**；对 DC-4 基线（v3 复测
  2,266P）**+103P，不回落达成**。理论抬升 ≈ +226P（族 102 + corpus 124）
  与实测 +104 的差 = universe 漂移相抵（manifest 窗 10-27→11-08 切换，
  分母 13,324→13,274，639 条 ZW 路径上游改名/删除面正常波动）。
  - **快照文件未回写 main**：聚合产出被环比护栏拦下（web-components 面
    2819→0、shadow-dom 面 5111→0，−100% 触发「本次快照不回写」，退出码 3）
    ——护栏行为正确，塌陷面是 §四 的引擎回归（ddb1e73cf）真实后果，非
    语料/管线事故；待回归修复后下一 weekly cron 自然回落正常值。本档 fetch
    头条由同一 run 的 artifact 本地聚合（§五），数值与 CI 聚合同源同脚本。

## 四、过程事件——catch-all-7 语料链断裂三轮归因与 weekly 快照管线容错修复

- **首轮 dispatch run 42496776344**（树 252fbb3fd0）catch-all-7
  「Fetch suite corpora」确定性失败：fetch 列表含 `fetch-wpt-shadow-dom`，
  Makefile 无此目标（git log -S 全史零命中）→ results artifact 缺失 →
  矩阵判败 → aggregate（无 if 容错）跳过，快照不落盘。已取消。
  归因：46 号切片六 6a（252fbb3fd0）注册表化使
  `testharness-shadow-dom` 套件臂对 ci_discover_shards.py 变为可见
  （56f1febb7 树 grep 零命中）→ catch-all 打包重排（11-07: catch-all-7 =
  storage+SRI；现在: shadow-dom+storage）→ 缺省约定派生不存在目标。
  discoverer 脚本与 ci-catchall.tsv 两树零变化——触发面为组合态效应。
- **二轮 dispatch run 42498433483**（树 70203b7e9）：先按注册表头注释
  救济协议补行 `shadow-dom → fetch-wpt-shadow-render-pool`，仍失败——
  **异因**：该目标走 45 号渲染 goal 池脚本（goals/45-shadow-dom-rendering.sh），
  内置共享依赖断言（css/fonts 语料清单的 ahem.css +
  css/reference/ref-filled-green-100px-square.xht，「missing=0 断言」）在
  CI 干净工作区必炸（本地绿是因 css 语料恰在盘）；且 runner 套件 shadow-dom
  的实跑面（10 文件 = 45 号 SUPPLEMENT_POOL，testharness.rs §3486 两侧同源）
  语料只能由该脚本预置——fetch-web-components-subset.sh 零覆盖（grep 实测
  202 处 shadow-dom/ 引用不含此 10 文件）。该面完整接线（css/fonts 共享
  依赖 + 池脚本）体量与收益不成比例（10 文件面 15P/86F），挂账不接。
  已取消。
- **根修（@ 6b3a7d00c，三件容错 + 一笔回退；空文件级后继容错另落
  757f4db84 @ 下文四b。原记 4b460d3bf，11-08 晚 pull --rebase 改写，
  ①②③④ 与该 commit diff 逐字吻合）**：真正缺陷是 discoverer
  头注释既定契约「跑不动的面该面记 0（与未跑同态）」在 weekly.yml 管线
  面未被兑现——fetch 循环 bash -e 首败连带兄弟套件、run 步 exit 1 连带、
  aggregate 无 if 容错，一个套件的语料缺失升级成快照不落盘。修复：
  ① fetch 步 per-suite 失败降级 warning（共享 harness 引导保持致命）；
  ② run 步无 JSON 仍红分片（rc 记账）但循环继续；③ aggregate if 加
  `success() || failure()`（取消仍跳过）；④ 回退 ci-catchall.tsv
  shadow-dom 行（70203b7e9 所指目标 CI 确定性失败，短期该面记 0，
  与既有各快照同态）。46 号切片六后续（6b/6c 在飞）再暴露新套件时，
  同类缺口降级为单面记 0，不再威胁 weekly 快照管线。
- **三轮 dispatch run 42499865933**（树 6b3a7d00c）= 头条终验数据源。
  本轮 html-sem 分片仍红——**新异因：`testharness-web-components` runner
  栈溢出**（套件启动后 ~50s `thread 'main' has overflowed its stack`，无
  JSON 产出；分片容错按设计只红自己，但 shell 重定向留下的 0 字节
  testharness-web-components.json 随 artifact 上传，聚合读到空文件抛
  `Expecting value: line 1 column 1`，快照二次被吞）。

## 四b、空文件容错落地 + 引擎回归定谳（2026-11-09 第二层）

- **空文件容错 @ 757f4db84**：`load_zw_results` 空文件（套件崩在产出任何
  JSON 前）按未跑同态返回空集+告警，非空畸形仍硬错（self-test 钉两面）；
  `gh run rerun --failed` 后聚合读到空文件正确跳过（警告可见）。
- **环比护栏二次拦截（行为正确）**：修完空文件后聚合推进到环比比较，
  `custom-elements: 2819→0（−100%）、shadow-dom: 5111→0（−100%）` 触发
  「本次快照不回写 main」（退出码 3，提示 `--no-guard` 需人工确认）。
  塌陷面 = 下述引擎回归的真实后果，护栏拒绝把塌陷写成正式快照是设计内
  行为；正式快照待回归修复后由下一 weekly cron 落盘。
- **引擎回归定谳（非本 goal 面，归因移交往决项）**：本地单案 bisect
  （worktree @ 56f1febb7 基线单案 `build-deep-detached-shadow-then-
  append-text.html` rc=0 全绿 → 当前 main 栈溢出；git bisect 收敛）——
  **首个坏提交 `ddb1e73cf`**（shadow M3 命令式面三修复，js_dom_bridge
  AttachShadowOnHandle/AppendChildByHandleSel + shadow.rs + part04.js
  appendChild trap）。案源：上游 WPT 回归测试（servo#43998 同型）——999
  层嵌套 attachShadow 后向最深层 append 文本，传播链逐层递归溢主线程栈。
  影响面：web-components 套件启动 ~50s 即崩（custom-elements/shadow-dom/
  template 三面全废）、CI html-sem 分片常红、weekly 快照该两面记 0。
  11-07 快照（56f1febb7）该案通过，回归窗 = ddb1e73cf（11-08 02:20 落地）
  起。初步归 shadow 域（45 号已归档）；同日收尖归因见下条，本轮不越界
  修——已记控制面待决项。
- **根因帧定位 + 崩溃阶段收尖（2026-11-09 同日）**：gdb backtrace（须去
  `ulimit -v`——内存墙会饿死 V8 sandbox 预留，首试误报 OOM）定谳溢出帧
  在 **style-system**：`compute_styles_recursive` 逐树层自递归 ~2,000 帧
  打满主栈，进入链 `compute_styles_with_shadow` ← `compute_styles` ←
  `RenderPipeline::render_html` ← `WebView::apply_pending_shared_mutations`
  ← `run_page_scripts_strict`；dom 侧 append_child/resolve_slots 均平铺
  迭代，排除。三支探针（999 层 detached 嵌套 shadow）收尖崩溃阶段：
  probeA 仅构建 **通过**、probeB 挂入文档 **崩**、probeC 挂入 + 深层
  append 文本 **崩**——崩溃点在挂树触发样式计算，非构建、非 append
  传播链。产品面同样暴露：`apps/browser/src/tab_worker.rs` spawn 未设
  stack_size（默认 ~2MB 线程栈），远浅于 999 层的深嵌套页即可打爆。
  修复方向 = style-system 遍历迭代化，属渲染流专属面（run-rules §9）+
  perf 关键路径（改动须 perf-gate）；ddb1e73cf 的内层 shadow 物化本身
  是正确行为不回退。控制面待决项已按此口径收尖（探针文件为一次性诊断
  产物，已自语料删除）。
- **goal 面零漂移复核（2026-11-09，收口后一致性检查）**：current main
  caa08eb45 定向复跑 request-cache 族窗 **106P/0F** + headers 窗
  **286P/0F**，均与 M3/M4 账面全等——他流 cf2e22863 engine 提交落地后
  goal 交付面零漂移，文档与实态一致。
- **corpus 窗归档后复跑（2026-11-09，零漂移复核尾项）**：current main
  4b020ea96 复跑 FILTER=http-cache **124P/14F**，逐文件与 M3 corpus 账
  全等（含 cc-request 1F/credentials 2F/partial 1P·8F/post-patch 2F/
  pragma 1F 失败分布；partial 落绑定跑形态）。过程注记：首跑空结果
  exit 1——工作树语料已不在（gitignored）且 20 号拉取脚本划界排除
  fetch/http-cache，经 47 号脚本带 GITHUB_TOKEN 续拉后复跑全等；语料
  预置脚本保留的决策被此轮验证。

## 五、核实方法

- 全窗：`testharness-net-api --json`（952 文件 JSON），对比脚本
  逐 subtest 多重集（本档 §二口径），基线 JSON 路径见 §二。
- 快照：`docs/compat/wptfyi/coverage-2026-11-07.{json,md}` 组聚合
  （python 标准库求和 fetch* 组）；run 元数据 `gh run view`。
- **post-goal 头条聚合（快照未回写 main 的替代数据源）**：`gh run
  download 42499865933 --pattern "zw-results-*"`（36 分片 134 JSON，
  含 0 字节 web-components 空文件）→ 本地 `wptfyi_gap.py coverage
  --results <flattened>`（含 757f4db84 空文件容错；环比护栏仅存在于
  回写判断，本地聚合照常产出 report+json，不落 main）→ fetch 组行
  2,369P/13,274 = 17.85%（manifest 窗 chrome 157 @ fea36e49dd
  2026-11-08，全量 125,445 文件/2,265,421 子测试，ZW 覆盖 16,302 文件
  1,543,814P = 68.15%）。与 CI 聚合同脚本同输入，仅少一次回写。
- 回归 bisect：worktree @ 56f1febb7（`.cargo/rusty_v8/archive` 需从
  主树复制，CARGO_TARGET_DIR 独立）单案对照 + `git bisect run` 探针
  （build 失败 exit 125 skip；案 filter `build-deep-detached-shadow`
  单案级，栈溢出=bad）。
- CI：`gh workflow run weekly.yml -f run_wptfyi_coverage=true`（两次），
  run 42496776344（取消）/ 42498433483（终验）；discoverer
  `ci_discover_shards.py` 本地两度核验。
