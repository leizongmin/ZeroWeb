# M1：可导入集入通道（DC-2 收口）

日期：2026-10-31。结论先行：**big5/iso-2022-jp-encoder 根面 2 文件经 36 号
form-url 通道入账（covered>0 达成）——big5-encoder 13/13 全 P（big5 编码器
表 URL 序列化面新证，无引擎缺口）；iso-2022-jp-encoder 2P/10F，10F 为
per-scalar fresh 编码模型与规范流式状态机的结构性分歧簇（挂账，模型不动）；
legacy 七目录扫描代码与谓词恒等，schinese 恒等干跑 303/303，30/31 号通道
零改动。**

## 通道接线（三处，diff 可追溯）

1. **runner**（tests/wpt-runner/src/testharness.rs）：新增
   `ENCODING_FORM_URL_CORPUS_SUBDIRS`（31/30 号共用
   `ENCODING_LEGACY_MB_CORPUS_SUBDIRS` 不动 + 追加 `encoding` 根）；
   `run_form_url_encoding_cases` 换用新常量；`form_url_encoding_case_skipped`
   谓词零改动（`-encoder.html` 放行规则既有，根面仅 big5/iso-2022-jp-encoder
   匹配——根面 16 文件逐一核对，bom/sniffing/utf-32/eof/ref/.https 均不
   匹配放行规则）。
2. **Makefile**：新增 `fetch-wpt-encoding-compat-v2`（37 号 fetch 脚本，
   .PHONY 同步）；`testharness-form-url-encoding` 前置追加该目标（31 号
   脚本保留，双拉幂等）。
3. **weekly.yml**：form-encode-iso2022 / form-encode-big5 两分片 filter
   去尾斜杠（`iso-2022-jp/`→`iso-2022-jp`、`big5/`→`big5`）自然并入根面
   文件（互斥保持、无重复计账——`big5` 子串命中集 = 原 big5/ 目录 +
   big5-encoder.html，无第三处）；fetch 补 `fetch-wpt-encoding-compat-v2`
   拉根面语料；条目分布注释 817→819。YAML 解析校验过。

## 干跑（新面 + legacy 恒等对照，JSON 形状）

命令：`ZW_SCRIPT_TIMEOUT_MS=600000 ./target/test-guard --per-proc-mem 4
--total-mem 8 --time-limit 900 -- ./target/release/zero-wpt-runner
testharness-form-url-encoding encoder.html --json`（release 二进制为本轮
接线后重建，a868e7797 态；exit 1 = 有非 Pass 设计内）。

| 案 | 结果 | 判读 |
|---|---|---|
| `encoding/big5-encoder.html` | **13P** | 新证面：big5 非ASCII 编码器表经 `<meta charset=big5>` + inline `a.href` URL 查询序列化全过（`%A1@`/`%FE%FE` 指针案、重复映射取高指针案、astral NCR 案）——big5 目录无同名案（`big5-enc-ascii.html` 纯 ASCII），此为本 goal 新增覆盖 |
| `legacy-mb-schinese/gbk/gbk-encoder.html` | 49P | legacy 恒等对照 ✓ |
| `legacy-mb-schinese/gb18030/gb18030-encoder.html` | 254P | 同上 ✓（两案 303/303 与 36 号在册账恒等） |
| `encoding/iso-2022-jp-encoder.html` | 2P/10F | 见下节归因 |

合计 328 子测试：315P / 10F（+3 escape 形态差记 F 内）。

## iso-2022-jp 10F 簇归因（挂账，模型不动）

三子簇，同根因：

1. **流尾回切缺失**（6 案）：期望末尾 `%1B(B`（流结束回 ASCII 模式），
   fresh 模型逐 scalar 剥除 `ESC ( B` 伪迹后无人补回。
2. **模式冗余重入**（1 案）：`‾` 已在 roman 态仍重发 `ESC(J`；
   `ESC$B` 在相邻 jis0208 码点间重发。
3. **裸 `%` 字节**（`basics` 案内）：jis0208 输出字节 0x25 按特殊查询集
   应裸出（Chrome 真值），ZW 查询 set 显式含 `%` → `%25`。

根因：`text_encode_percent`（crates/engine/src/js_dom_bridge/text_encoding.rs）
的**逐 scalar fresh 编码模型**——每码点独立编码、状态不跨码点、流尾
`ESC ( B` 剥除。该模型是 encode-form 面（条目 = 单码点、harness 期望
`%1B%28%42$` 尾剥）与 encode-href 面（逐码点单测、`expect()` 自带
`replace(/%1B%28%42$/, "")` 尾剥）钉死的形状——36 号 68 文件 766,959
子测试在该模型下 99.9999%。根面 iso-2022-jp-encoder.html 是**唯一多
scalar 流式真值面**（12 案全为混合输入硬编码期望）。

**决策：挂账不动模型**——改流式状态机需重钉 766,959 在册账的期望构造链
（harness 期望本质 per-scalar 形状），以 10 子测试换最大在册账的回归风险
不成立。2P 照记（covered>0 达成），10F 为常驻回归传感器：未来若做流式
模型改造（挂账清单：iso-2022-jp encoder 流式状态机 + 查询集裸 `%` 语义），
本面 12 案即验收靶。

## 零回归论证（DC-2 门）

- **36 号**：legacy 七目录扫描代码与谓词逐字恒等（仅追加根目录常量项）；
  schinese 两案 303/303 恒等干跑 ✓；根面为纯新增扫描域（原 36 号扫描域
  不含 `encoding` 根，无既有行为变化）。全量常驻回归 = weekly 六分片
  （预算 2167d9ca6/12deb8557 口径）+ 本轮新增两分片根面条目。
- **30 号 / 31 号**：通道、谓词、语料常量零改动（`ENCODING_CORPUS_SUBDIRS`
  与 `ENCODING_LEGACY_MB_CORPUS_SUBDIRS` 原样）；M0 双臂基线
  （12,072P/3F）继续有效。
- Makefile `make -n` 干跑核验（fetch 前置 + FILTER 透传 + test-guard
  包裹齐全）。

## DC-4 数字更新（相对 M0）

- 挂账池 12,879 → **12,889**（+iso-2022-jp 流式模型簇 10）。
- 全可收口天花板 1,139,486 → **1,139,476（98.86%）**；数值门 ≥98.8%
  （≥1,138,566）不变，仍低于天花板。
- 本轮新增通过：**+15P**（big5 13 + iso-2022-jp 2），encoding 根面
  30 号通道账 12,072P 之外独立计。

## 遗留与下一步（M4）

- coverage 复测（weekly cron 或本地聚合）读数对 DC-4 门；wpt-suites.csv
  37 号行回填（planned → data，36 号行 encode-face 计数口径 +2 文件）。
- 本地全通道跑带 `TIME_LIMIT=2400`（M0 envelope 注记沿用）。
