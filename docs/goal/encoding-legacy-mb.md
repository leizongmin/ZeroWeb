# 遗留多字节编码文档解码面 — encoding/legacy-mb 文档族

**版本**: v1.6
**日期**: 2026-10-29（语料完备性更正：pin 口径语料本已全量在位，「chars 大文件缺/网络卡点」为误记销账——DC-3 收口；唯一剩余 DC-4 coverage 复测）
**状态**: Active——DC-1/DC-2/DC-3 完成；DC-4 待做（47 套件 `--json` 重跑 + `wptfyi_gap.py coverage` 全量复测）；M3 甄别面全部收口
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
- **六通道对账收口（2026-10-28 追记）**：relay 链 base 构建 `cd` 错目录的
  假绿修正后（`--manifest-path` 冷 worktree 构建 + RUSTY_V8_ARCHIVE），
  base 臂（c1482756a）五通道补跑：encoding/web-components/navigation/
  html-semantics case/subtest/状态三层全恒等；dom 368/368 仅
  Range-mutations-* 截断漂移 + **1 处真回归**——深臂 `deep_entry_json`
  属性只发 local 名，foreign context（SVG 片段）丢 `xlink:` 前缀，
  `Node-cloneNode-svg` 克隆属性 namespaceURI 断言 Fail（head 2/2 Fail、
  base 2/2 Pass 定责）。修复：属性名发限定名 `prefix:local`（prefix None
  短路，HTML context 逐字节同旧）。修复后复验：目标案 4/4 Pass；dom 全
  通道重跑零回归（回归翻转消失，余 2 状态差为 Event-timestamp 计时散布，
  base 二进制同机同现于不同子测试）；euc-jp 族 43 case **40,388P/0F
  exit 0** 锚点维持；web-components 全通道对账恒等。细节见 evidence
  「base 臂补跑与六通道对账收口」节。

## M2 进展（2026-10-28 基线；2026-10-29 更正为全量口径，DC-3 收口）

细节见 [evidence/2026-10-28-m2-baseline-classification.md](evidence/2026-10-28-m2-baseline-classification.md)
与 [evidence/2026-10-29-corpus-completeness-dc3.md](evidence/2026-10-29-corpus-completeness-dc3.md)。要点：

- **基线**：385 case / 360,431 子测试 **100% Pass / 0 Fail / 0 超时**
  （decode-wrapper 380 case + decode-errors 5 case，五族全绿，iso-2022-jp
  转义状态机 7,329×2 variant 含内）。
- **语料完备性更正（2026-10-29）**：「chars 大文件缺/网络卡点」为误记——
  上游 pin（315976933）四族清单与本地逐文件对账 **missing = 0**；29 个
  `*-chars*.html`（各 ~852KB）全部在位且为**无脚本 span fixture**（数据经
  wrapper iframe 全量断言，浏览器侧同样 0 子测试）。基线 360,431 即全量
  口径，非部分语料。
- **分母对账（DC-3 依据）**：wpt.fyi 四族分母 1,128,104 = decode 面
  360,431（已 100% 收敛）+ encode 面 767,673（68 文件甄别 not-run 归口，
  见 M3 甄别 evidence）。136 html 普查 = 34 跑 + 29 chars fixture +
  5 errors fixture + 68 encode 面。
- **`.headers` 两形态复核销账**：98/98 sidecar hdr==meta 逐字一致，
  「有 headers 无 meta」「头/meta 不一致」均 0——file 通道不消费 sidecar
  行为中性全量实证。
- **wpt-suites.csv**：planned 行转数据行（360431/360431），note 随更正
  同步（全量口径 + encode 面归口）。

## M3 进展（2026-10-28，甄别面收口）

- **details 裁决态**：细节见
  [evidence/2026-10-28-m3-deep-entry-details-adjudication.md](evidence/2026-10-28-m3-deep-entry-details-adjudication.md)。要点：

  - **甄别结论**：host 解析产物不裁决 details name= 互斥（深 JSON 双 open
    实证），裁决责任在 plain 物化面。M1 挂账的 `_zwMBuildDeepEntry`（template
    contents）实况比理论更有趣：**两面背离**——template content 视图构建后无
    扫描 → 1,1 无裁决；createContextualFragment 渐进填充下写尾 others-prong
    后者关前者 → 0,1 反转（定性）。body innerHTML 面（深臂 + 扫描）1,0,1 一直
    正确。
  - **修复**：builder 改工厂 attrs 直灌（镜像深臂，构造期写尾不再触发）+
    template 视图 / ccf 两面构建后补 `__zwDetailsScan4100(…, true)` parse-face
    裁决。engine lib 3571P/0F（新增两面 1,0 断言测试 + 原 R4100 全绿）；
    通道复验对账零回归，html-semantics 净修复 +7 子测试。

- **form-encode / encode-href 可跑性甄别**：细节见
  [evidence/2026-10-28-m3-encode-form-href-runnability.md](evidence/2026-10-28-m3-encode-form-href-runnability.md)。要点：

  - 本地语料 encode 面合计 68 文件（form 46 + href 19 + `*-encoder.html` 2
    + `big5-enc-ascii` 1；初记 65 漏 3 个非 `encode-*` 命名同面文件，
    2026-10-29 更正）——通道 skip 规则排除的主体，本轮把「未甄别」升格为
    已甄别。
  - **encode-form 不可跑**（三层缺面）：`_zwFormSubmitNavigate` `_encPair`
    硬编码 UTF-8 `encodeURIComponent`（acceptCharset 零消费——核心被测
    语义不可达）；form.target → iframe 嵌套导航缺面（`iframe.onload` 断言链
    不可达，卡的不是 blank.html 文件是嵌套导航）；单文件 ~7,000 async_test
    规模面。归口 form 提交序列化面。
  - **encode-href 可跑但面缺**：同步 `a.href` 面无基础设施依赖，但引擎 URL
    面全 UTF-8（shim + rust-url），无「encoding-parse a URL」文档编码
    override——实跑预期近全量 Fail，无判据意义，skip 理由升格。归口
    URL/HTML 集成面。
  - **口径影响**：DC-3 分母对账把 encode 面单列 not-run（767,673 子测试），
    与 decode 面分开计——解锁面通过率不被未解锁面稀释。

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
  管道能力就绪后在 M3 一并甄别可跑性，不强收）。**甄别已完成**（2026-10-28，
  见 evidence「M3：form-encode / encode-href 面可跑性甄别」）：form 面
  不可跑（UTF-8 序列化硬编码 + 嵌套导航缺面），href 面可跑但缺
  encoding-parse a URL——两面均归口 runner form/URL-compat 能力缺口，
  skip 维持且理由升格。
- `.py` 服务器端点不拉（lib.sh 不含）；`.headers` sidecar 全拉（HTTP charset
  头优先语义是用例依赖，DC-1 甄别 runner 是否消费）。预检实证（见上节）：
  带 sidecar 的案本体均带 meta 声明，sidecar 消费维持挂账不动。
- 单字节遗留编码（windows-125x/iso-8859-x 等 `encoding/` 根下 single-byte 面）
  属 30 号 JS API 范畴的文档形态延伸，不在此 goal；如 DC-1 管道就绪可作为
  顺带受益面在 M3 记账。

## 里程碑

- **M0** ✅（2026-10-28）：通道接入切片（DC-1）——testharness 通道读取统一
  接解码管道 + 最小验证集 + 零回归对账（见「M0 结果」节）。
- **M1** ✅（2026-10-28）：runner 通道 + Makefile（DC-2）+ 通道性能收口 +
  六通道对账收口 + 深臂限定名回归修复（见「M1 进展」节）；语料补全卡点
  移交 M2。
- **M2** ✅（2026-10-28 基线；2026-10-29 更正为全量口径，DC-3 收口）：
  基线分类 + wpt-suites.csv 回填 + `.headers` 复核 + 分母对账（见「M2
  进展」节与 2026-10-29 evidence）。
- **M3** 进行中：`_zwMBuildDeepEntry` details 裁决态甄别 ✅（2026-10-28，
  两面背离修复）+ form-encode/href 可跑性甄别 ✅（2026-10-28，68 文件
  归口非 goal 面，见「M3 进展」节）；**DC-4 批量驱动已启动**（2026-10-29
  夜，51 通道 `--json` 顺序跑，结果目录
  `/lzcapp/document/work/zw-coverage-dc4/results/`，status.tsv 逐通道 rc；
  数小时量级——下轮收齐跑 `wptfyi_gap.py coverage` 出新日期全量报告）。

语料进度（2026-10-29 更正）：七族 253 文件**已全量在位**（30 号预置即齐，
上游 pin 315976933 对账 missing=0）——此前多轮「网络 degraded 语料卡点」
为误记：curl 报错是真实的，但重拉的是已在位文件（FORCE=1 全量重下语义），
没有任何文件缺失过。fetch 基建经代理可用（直连 raw 持续超时，代理 10801
正常），日常续拉用 `FORCE=1` + 代理环境变量。

## 依赖约束（run-rules §9）

- 文档解码管道变化核对 `zero-engine`/`zero-webview` 边界（文档加载入口归属），
  不在 runner 侧引入编码转换旁路。
- 与 40 号 html-syntax-compat 的挂账关系：文档编码嗅探面原挂 html-syntax M4，
  本 goal 立项即承接该面；40 号归档文档相应条目在本 goal DC-1 落地时销账。
