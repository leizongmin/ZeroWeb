# 遗留多字节编码文档解码面 — encoding/legacy-mb 文档族

**版本**: v1.3
**日期**: 2026-10-28（M1 runner 通道落地 + 通道性能收口，见「M1 进展」节；M0/DC-1 见「M0 结果」节与 evidence/）
**状态**: Active——DC-1/DC-2 完成；M1 余语料补全（网络 degraded 卡点维持）；M2 基线分类待语料
**执行模式**: WPT 驱动（上游 encoding/legacy-mb corpus 为验收标尺）+ runner 管道能力建设
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——文档编码面）

## coverage 对账（coverage-2026-10-27 基线，47 套件干跑口径，终稿待 `docs/compat/wptfyi/coverage-2026-10-27.md` 复核）

| WPT 面 | 全量分母（子测试） | 覆盖 | 缺口性质 |
|---|---:|---:|---|
| encoding/legacy-mb-japanese | 447,723 | 0/482 文件 | runner 文档解码管道缺失 |
| encoding/legacy-mb-korean | 410,448 | 0/435 文件 | 同上 |
| encoding/legacy-mb-tchinese | 268,916 | 0/283 文件 | 同上 |
| encoding/legacy-mb-schinese | 1,017 | 2/6 文件 | 同上 |

三大多字节族合计 **1,127,087 子测试 = 全量分母（2,264,876）的 50%**。解锁即把
全量对账头条从「基本没测过编码文档面」变为「编码面有数字」——JS API 面
（30 号已收口）已验证同族解码表实现通过率 ~100%，文档面解锁后预期高通过率。

## 执行前预检（2026-10-27，v1.1 修订依据）

立项时「runner 现按 UTF-8 加载文档、M0 可行性未验证」的前提**已被后续盘点
部分推翻**——文档解码管道已存在，缺的是 testharness 通道接入：

- **解码管道已在 zero-dom**：R5003/html-syntax-compat P5（b2fde4965，
  2026-10-19）落地 `zero_dom::decode_document_bytes`——BOM 优先 +
  `<meta charset>`/http-equiv 预扫描（spec prescan 近似：注释跳过、引号
  语义、无闭引号 failure 继续扫描）+ encoding_rs 全标签表解码；
  `document.characterSet` 嗅探管线同批落地（P5 通道 61227/61333 = 99.83%）。
  encoding_rs 0.8 为 workspace 依赖（dom/engine/net 三处），30 号
  TextDecoder/Encoder JS API 面（99.96%）同库同表。
- **消费现状**：`decode_document_bytes` 仅被 runner reftest 文件载入链
  （`tests/wpt-runner/src/wpt_file_loader.rs`）消费；**testharness 各通道
  语料读取全是 `std::fs::read_to_string`**（testharness.rs 等 67 处）——
  非 UTF-8 文件读入直接 `Err` 被 skip，这才是 legacy-mb 文档族全灭的机制
  （不是解码错误，是根本读不进来）。
- **`.headers` sidecar 实证**：euc-jp 抽样（14 个带 `.headers` 的案）本体
  均带 `<meta charset>` 声明——file 通道不消费 sidecar 时这些案按 prescan
  解码即可正确。sidecar 消费维持挂账不动（file 通道无 HTTP 头语义，
  html-syntax `content='http'` 排除先例同口径），全量语料到位后 M2 基线
  分簇时复核「有 headers 无 meta」与「头/meta 声明不一致」两形态占比。
- **结论**：M0 从「可行性切片」重定性为「通道接入切片」，不可行分支删除；
  剩余风险仅在实跑验证（iso-2022-jp 状态机、label 识别边界），最小验证集
  覆盖（DC-1）。

## M0 结果（2026-10-28，DC-1 收口）

细节与全量数据见 [evidence/2026-10-28-m0-corpus-read-decode-pipeline.md](evidence/2026-10-28-m0-corpus-read-decode-pipeline.md)。要点：

- **机制实证成立**：testharness 各通道语料读取原为 `std::fs::read_to_string`
  88 调用点，非 UTF-8 文档整案 skip——「读不进来」而非「解码错误」。
- **DC-1 前提修正（预检未记录的险点）**：JS 语料内嵌 `<meta charset>` 字面量
  （FileAPI/url/url-charset.window.js 等），文档 prescan 对脚本形态会误触发
  ——统一入口按扩展名分流：html/htm/xht 走 `decode_document_bytes`，其余走
  BOM > UTF-8 lossy（资源序）。「prescan 对 UTF-8 语料无感」修正为「对 UTF-8
  **文档**语料无感；JS 语料必须类型分流」。script 专用解码面
  （`decode_classic_script_bytes`）不动。
- **语义钉**：euc-jp（13,459 span）+ iso-2022-jp（7,329 span，转义状态机）
  常驻单测，参照 encoding_rs（产品同库同表）。
- **零回归门**：基线 a9c1c86c8 vs 本变更 release 二进制，35 通道 `--json`
  集合恒等对账完成——31 通道全恒等；media/selection/html-semantics 各 1 处
  超时表现形态 flake（同二进制重跑定性）；net-api 恒等性外 **1 案预期解锁**
  （`websockets/constructor/016.html`，windows-1252 文档原 read `Err`
  不可见，现可运行如实失败）——正是本 goal 要解锁的形态。dom/navigation/
  uievents/css-variables 通道 envelope 对称超限（预置条件）放宽补跑中，
  timing/security/service-workers 等 15 通道补对账中，行回填同文件。
- **卡点**：全量语料续拉网络持续 degraded（jsdelivr 拒连、raw ~1/3 成功率，
  FORCE=1 + GITHUB_TOKEN + curl shim 六轮重试）——当前 136 html（七族
  wrapper 已齐，chars 大文件缺），不阻 M1 通道先行。

## M1 进展（2026-10-28，DC-2 收口）

细节与数据见 [evidence/2026-10-28-m1-runner-channel-perf.md](evidence/2026-10-28-m1-runner-channel-perf.md)。要点：

- **DC-2 通道落地**：`zero-wpt-runner testharness-encoding-legacy-mb` 子命令 +
  per-dir skip 规则（只跑 decode wrapper/decode-errors，chars/encode 面/helper
  skip）+ Makefile target（test-guard 包裹，5400s envelope + 本通道
  `ZW_SCRIPT_TIMEOUT_MS=600000`）。
- **通道首跑暴露三处 O(结果数 × 树) 共享路径成本，全部修复**：①建树浅臂每元素
  两次全文档重解析 → host 单次解析深 JSON 深臂（`__zw_parse_html_subtree`）；
  ②`_zwMFindRealNode` 512 上限悬崖（13.5k span 页索引恒空、每次查找全量重建）
  → 提额 200000；③查询产物 eager `_zwRootHtml` 全树/全文档序列化 ×2 处 → 惰性
  accessor。qsa 千元命中 12,872ms→74ms（172×）。
- **dataset 缺失修复**：`_zwMEl` 工厂节点补 dataset Proxy（本地 attrs 版，
  part05 `_datasetProxy` 同语义），decode-common.js showNodes 整链 TypeError 解锁。
- **门禁拦截的回归（R4100）**：深臂初版 setAttribute 灌属性触发 details open
  互斥 others-prong 破坏解析产物 host 裁决态——改工厂 attrs 映射直灌（镜像浅臂），
  单测复绿；既有 `_zwMBuildDeepEntry`（template contents）同暴露挂账 M3。
- **端到端**：`eucjp-decode-x-euc-jp` 全 14 variant **13,459P/0F**（修复前
  ~176s 后 V8 4GB 堆 OOM），插桩清理后最终构建复验一致；rebase（并行流
  s124–s126 engine 合入）后全族复核 **40,388P/0F**（4 wrapper × 14 variant
  + errors，exit 0）。
- **回归对账（新基归因）**：六通道 lane 前两通道（dom/html-semantics）出现
  失败——**stash A/B 定责为基面属性，非本改动**：dom
  `window-named-properties` 2F、html-semantics 839F（relevant-mutations 46F/
  67P 等，语料 mtime 未变）在去掉本改动后的 HEAD 二进制逐字节同现。M0 全绿
  基线随 s124–s126 合入陈旧；余四通道零回归改由并行 relay 会话六通道
  `--json` 集合恒等对账承担（before = worktree 冷建 c1482756a base 二进制，
  精确同基去本改动），legacy-mb 整跑 JSON 兼作 M2 基线数据源。
- **门禁**：fmt/clippy workspace 零 warning；make test 腿补全全绿（腿 1 跳过
  R5295/R5300 已记账 image_batch/network 并发窗 flake 两案，隔离复跑 2/2 绿；
  quickjs renderer lib 腿 `mutation_arm_…_reset_window` 红为 HEAD 复现既有问题
  （stash A/B 0.63s 同断言），同 R5295「renderer page_scripts」记账不修）。
  rebase 后定向复验：R4100 单测复绿、fmt/clippy 复跑绿、euc-jp 全族
  40,388P/0F（上条）。

## Mission

让 runner 文档加载管道按文档编码声明（`<meta charset>` / HTTP Content-Type
charset / BOM）解码字节流，使 legacy-mb `*_chars-<label>.html` 族可运行并收敛。

用例形态（已 fetch 语料实证）：文档头部 `<meta charset="cseucpkdfmtjapanese">`
+ DOM span 携带 `data-cp`（期望码点）与 `data-bytes`（原始字节）——文档字节流
按声明编码解码正确时，脚本断言全部通过；解码管道忽略声明则整页乱码全败。

管道底座（BOM/prescan/encoding_rs）已就绪（见预检节），本 goal 实际工作是
**把 testharness 通道的语料读取接到既有解码管道**，再建 legacy-mb 通道与基线。

## Done Criteria

- **DC-1（M0 通道接入切片）**：testharness 通道语料读取统一接
  `decode_document_bytes`（`read_to_string` → `fs::read` + 解码，替换范围 =
  全部套件语料读取点，理由：避免「会解码/不会解码」两套读取语义长期并存；
  prescan 对 UTF-8 语料无感，理论零行为变化）；最小验证集 euc-jp +
  iso-2022-jp 各一案语义通过（覆盖状态机编码边界）；受影响既有套件
  `--json` A/B 对账集合恒等（零回归门，perf/wpt-runner-testharness-speed
  判据先例）。
- **DC-2（runner 通道）**：`zero-wpt-runner` 新增 testharness 子命令 + per-dir
  skip 规则 + Makefile target（test-guard 包裹，照 testharness-fs 先例）。
- **DC-3（基线）**：全量语料分类通过率（文本 + JSON）落
  `docs/goal/encoding-legacy-mb/evidence/`；`docs/compat/trends/wpt-suites.csv`
  planned 行转数据行；`.headers` 两形态占比复核结论入 evidence。
- **DC-4（coverage 复测）**：重跑 coverage 对账，legacy-mb 未覆盖子测试清零
  路径成立（分母计入、通过率如实）。

## 划界（沿 30 号收口约定）

- form 提交编码面（`*-encode-form-*.html`）与 big5/iso-2022-jp encoder 的
  `<a href>` URL 序列化面不属本 goal（document 编码耦合的另一形态，随 DC-1
  管道能力就绪后在 M3 一并甄别可跑性，不强收）。
- `.py` 服务器端点不拉（lib.sh 不含）；`.headers` sidecar 全拉（HTTP charset
  头优先语义是用例依赖，DC-1 甄别 runner 是否消费）。预检实证（见上节）：
  带 sidecar 的案本体均带 meta 声明，sidecar 消费维持挂账不动。
- 单字节遗留编码（windows-125x/iso-8859-x 等 `encoding/` 根下 single-byte 面）
  属 30 号 JS API 范畴的文档形态延伸，不在此 goal；如 DC-1 管道就绪可作为
  顺带受益面在 M3 记账。

## 里程碑

- **M0** ✅（2026-10-28）：通道接入切片（DC-1）——testharness 通道读取统一
  接解码管道 + 最小验证集 + 零回归对账（见「M0 结果」节）。
- **M1** 进行中：runner 通道 + Makefile ✅（2026-10-28，DC-2，见「M1 进展」
  节）；语料补全（`FORCE=1` 跑 31 号脚本至全量）卡网络 degraded 维持挂账。
- **M2**：基线分类（按编码族/用例形态分簇）+ wpt-suites.csv 回填。
- **M3**：失败簇收敛或归口（form-encode 面甄别在此）+ coverage 复测收口。

语料预置进度：七族 253 文件已到位（30 号预置）；2026-10-27 起 `FORCE=1`
全量续拉（~1,200 文件，幂等补缺）——**网络持续 degraded（jsdelivr 拒连、
raw ~1/3 成功率带超时），六轮重试后仍 136 html**（七族 wrapper 已齐、chars
大文件缺）。M1 通道以现有语料先行，语料补全随网络恢复在 M1/M2 间补（以
`goals_inventory` 输出核对）。

## 依赖约束（run-rules §9）

- 文档解码管道变化核对 `zero-engine`/`zero-webview` 边界（文档加载入口归属），
  不在 runner 侧引入编码转换旁路。
- 与 40 号 html-syntax-compat 的挂账关系：文档编码嗅探面原挂 html-syntax M4，
  本 goal 立项即承接该面；40 号归档文档相应条目在本 goal DC-1 落地时销账。
