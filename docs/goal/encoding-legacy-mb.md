# 遗留多字节编码文档解码面 — encoding/legacy-mb 文档族

**版本**: v1.1
**日期**: 2026-10-27（立项，coverage 对账驱动；同日执行前预检修订，见下节）
**状态**: Active——M0 未启动（管道可行性已预检实证，M0 重定性为通道接入切片）
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

- **M0**：通道接入切片（DC-1）——testharness 通道读取统一接解码管道 +
  最小验证集 + 零回归对账。**此关未过不进入后续里程碑。**（管道现状盘点
  已由预检节完成，不再占 M0。）
- **M1**：语料补全（`FORCE=1` 跑 31 号脚本至全量）+ runner 通道 + Makefile。
- **M2**：基线分类（按编码族/用例形态分簇）+ wpt-suites.csv 回填。
- **M3**：失败簇收敛或归口（form-encode 面甄别在此）+ coverage 复测收口。

语料预置进度：七族 253 文件已到位（30 号预置）；2026-10-27 已起 `FORCE=1`
全量续拉（~1,200 文件，幂等补缺），M1 开工时以 `goals_inventory` 输出核对。

## 依赖约束（run-rules §9）

- 文档解码管道变化核对 `zero-engine`/`zero-webview` 边界（文档加载入口归属），
  不在 runner 侧引入编码转换旁路。
- 与 40 号 html-syntax-compat 的挂账关系：文档编码嗅探面原挂 html-syntax M4，
  本 goal 立项即承接该面；40 号归档文档相应条目在本 goal DC-1 落地时销账。
