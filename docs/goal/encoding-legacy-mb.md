# 遗留多字节编码文档解码面 — encoding/legacy-mb 文档族

**版本**: v1.0
**日期**: 2026-10-27（立项，coverage 对账驱动）
**状态**: Active——M0 可行性未启动
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

## Mission

让 runner 文档加载管道按文档编码声明（`<meta charset>` / HTTP Content-Type
charset / BOM）解码字节流，使 legacy-mb `*_chars-<label>.html` 族可运行并收敛。

用例形态（已 fetch 语料实证）：文档头部 `<meta charset="cseucpkdfmtjapanese">`
+ DOM span 携带 `data-cp`（期望码点）与 `data-bytes`（原始字节）——文档字节流
按声明编码解码正确时，脚本断言全部通过；解码管道忽略声明则整页乱码全败。

## Done Criteria

- **DC-1（M0 可行性切片）**：runner 以指定编码加载一条最小文档（eucjp_chars-
  cseucpkdfmtjapanese.html）并断言语义通过；**不可行**（管道改造超出合理切片）
  时输出不可行证据与重估建议，按挂账处理，不强行推进。
- **DC-2（runner 通道）**：`zero-wpt-runner` 新增 testharness 子命令 + per-dir
  skip 规则 + Makefile target（test-guard 包裹，照 testharness-fs 先例）。
- **DC-3（基线）**：全量语料分类通过率（文本 + JSON）落
  `docs/goal/encoding-legacy-mb/evidence/`；`docs/compat/trends/wpt-suites.csv`
  planned 行转数据行。
- **DC-4（coverage 复测）**：重跑 coverage 对账，legacy-mb 未覆盖子测试清零
  路径成立（分母计入、通过率如实）。

## 划界（沿 30 号收口约定）

- form 提交编码面（`*-encode-form-*.html`）与 big5/iso-2022-jp encoder 的
  `<a href>` URL 序列化面不属本 goal（document 编码耦合的另一形态，随 DC-1
  管道能力就绪后在 M3 一并甄别可跑性，不强收）。
- `.py` 服务器端点不拉（lib.sh 不含）；`.headers` sidecar 全拉（HTTP charset
  头优先语义是用例依赖，DC-1 甄别 runner 是否消费）。
- 单字节遗留编码（windows-125x/iso-8859-x 等 `encoding/` 根下 single-byte 面）
  属 30 号 JS API 范畴的文档形态延伸，不在此 goal；如 DC-1 管道就绪可作为
  顺带受益面在 M3 记账。

## 里程碑

- **M0**：可行性切片（DC-1）——解码管道现状盘点（runner 文档加载入口、现有
  charset 处理点）+ 最小通道落地或不可行结论。**此关未过不进入后续里程碑。**
- **M1**：语料补全（`FORCE=1` 跑 31 号脚本至全量）+ runner 通道 + Makefile。
- **M2**：基线分类（按编码族/用例形态分簇）+ wpt-suites.csv 回填。
- **M3**：失败簇收敛或归口（form-encode 面甄别在此）+ coverage 复测收口。

## 依赖约束（run-rules §9）

- 文档解码管道变化核对 `zero-engine`/`zero-webview` 边界（文档加载入口归属），
  不在 runner 侧引入编码转换旁路。
- 与 40 号 html-syntax-compat 的挂账关系：文档编码嗅探面原挂 html-syntax M4，
  本 goal 立项即承接该面；40 号归档文档相应条目在本 goal DC-1 落地时销账。
