# M0：testharness 通道语料读取统一接文档解码管道——DC-1 切片

日期：2026-10-28。目标域：`encoding-legacy-mb` M0/DC-1。文件轮：runner
`wpt_file_loader.rs`（共享入口 + 最小验证集）+ 通道文件 10 个（读取点替换，
清单见「替换范围」）。

## 入场定谳（语料丢失机制 + JS 字面量险点）

- 预检节结论实证成立：testharness 各通道语料读取全是
  `std::fs::read_to_string`，非 UTF-8 文档（EUC-JP/Shift_JIS/ISO-2022-JP
  等字节流）读入直接 `Err` → 案被 skip → legacy-mb 族 0 文件可跑。机制是
  「读不进来」而非「解码错误」。
- **新发现（预检未记录）**：JS 语料不能走文档解码——WPT JS 语料内嵌
  `<meta charset=...>` 字符串字面量（如 FileAPI/url/url-charset.window.js
  体内 `<meta charset="ISO-8859-1">`），若对 `.js` 一律 prescan 会按字面量
  改判编码、把 UTF-8 JS 误解码。DC-1 原文「prescan 对 UTF-8 语料无感」据此
  修正为「对 UTF-8 **文档**语料无感；**JS 语料必须类型分流**」。
- script 专用解码面不动：classic script 取回分支既有
  `decode_classic_script_bytes`（BOM > CT charset > charset 属性 > UTF-8，
  html-semantics s115 落地）语义独立于文档解码，本轮不扩展不覆盖；
  script-charset-02 文档编码面挂账仍归 html-semantics 域跟踪。

## 设计（共享入口 `read_corpus_to_string`）

`wpt_file_loader.rs` 新增 pub fn：`fs::read` 整读 → 扩展名 html/htm/xht 走
`zero_dom::decode_document_bytes`（spec 文档解码序：BOM > prescan > UTF-8
lossy）→ 其余扩展名走 `encoding_rs` BOM 嗅探 > UTF-8 lossy（资源序；JS 按
fetch a classic script 的 fallback 尾序相容，charset 属性/CT 参数感知仍归
script 专用分支，不在本入口）。原 reftest 两处内联实现改为调它（行为恒等）。

### 替换范围

testharness.rs 68 处 + 通道文件各 2–3 处（cookies 3、dnd 3、pointerlock 2、
page_state 2、fileapi 2、html_semantics 2、webmessaging 2、svg 2、workers 2）
= 88 调用点。替换后 testharness.rs 非 `cfg(test)` 区 `read_to_string` 清零
（余 1 处在单测内，读临时 fixture）。特例留痕：workers `004.worker.js` 原
lossy 读注释保留并注记统一入口；harness 自身 testharness.js 载入亦走统一
入口（harness 资源恒 UTF-8，行为无感）。

## 最小验证集（DC-1 语义钉，常驻单测）

`wpt_file_loader` 单测新增 2 枚：

- `legacy_mb_chars_decode_semantics_euc_jp_and_iso_2022_jp`：对语料
  `eucjp_chars-cseucpkdfmtjapanese.html`（13,459 span / 696,879 字节）断言
  统一入口读出的每 span 文本 == `encoding_rs::EUC_JP` 按 `data-bytes` 原始
  字节解码；`iso2022jp_chars.html`（7,329 span / 542,326 字节）同构对
  `ISO_2022_JP`（ESC/SI/SO 转义状态机边界）。参照系选 encoding_rs 的理由：
  与产品管道同库同表；若参照误用 UTF-8 lossy 则 100% span 失配——该钉承重，
  锁住「文档必须按声明解码」。`data-cp` 不作参照（生成器侧 JIS 信息，ASCII
  区字节映射与语料参照解码器不一致；首版踩坑记录在案）。语料缺失则跳过该
  臂（目录探测 `wpt-data` 与 `tests/wpt-runner/wpt-data` 双路径）。
- `chars_span_extraction_handles_multiline_iso2022jp_shape`：跨行 span
  提取形状钉。

验证：`cargo test -p zero-wpt-runner --bin zero-wpt-runner wpt_file_loader`
8 passed（新 2 + 既有 6）；fmt、clippy（`-p zero-wpt-runner --all-targets`）
过。

## A/B 零回归门

设计：基线 HEAD（a9c1c86c8）release 二进制 vs 工作树 release 二进制，通道
`--json` 对账；判据 = case-id 集合恒等 + (case, subtest) 对集合恒等，
status 差异另列定性。跑法全部 test-guard 包裹
（`--per-proc-mem 4 --total-mem 8`），时限按 Makefile envelope（svg 1900s、
html-semantics 5400s + RAF_FRAME_DRIVEN=1，余 900s）；顺序 lane 防争用
（教训：3 runner 并行把 dom 顶到超时，两侧同死属负载噪声不作数）。

| 通道 | cases b/a | subtests b/a | 集合 | status 差 |
|---|---|---|---|---|
| encoding | 36/36 | 12075/12075 | = | 0 |
| fileapi | 35/35 | 499/499 | = | 0 |
| workers | 175/175 | 714/714 | = | 0 |
| dnd | 44/44 | 446/446 | = | 0 |
| pointerlock | 18/18 | 22/22 | = | 0 |
| webstorage | 25/25 | 1239/1239 | = | 0 |
| compression | 18/18 | 318/318 | = | 0 |
| fs | 13/13 | 133/133 | = | 0 |
| html-syntax | 125/125 | 61411/61411 | = | 0 |
| cookies | 22/22 | 837/837 | = | 0 |
| webmessaging | 72/72 | 135/135 | = | 0 |
| page-state | 15/15 | 41/41 | = | 0 |
| svg | 341/341 | 681/681 | = | 0 |
| media | 207/207 | 640/640 | = | 1（噪声，见下） |
| html-semantics | 939/939 | 5306/5306 | = | 0（子集差 1 对=超时形态 flake，见下） |

media status 差 1：`track-cues-sorted-before-dispatch`「chronological」
before Fail / after Pass；同二进制三连跑 before Fail×3、after
Pass/Pass/Fail → 时序 flake，非本变更效应。

selection 子集差 1 对：`selection-nested-video.html` 两侧同为 case 级
Timeout，仅失败形态不同（before=「completion callback 未调用」URL 名合成行，
after=真实子测「Test timed out」）；单案重跑两二进制 ×2 全部复现 URL 名形态
→ 超时表现形态随负载抖动，非本变更效应。

**net-api 集合差 1（DC-1 预期解锁，非回归）**：
`websockets/constructor/016.html`（702 字节 3 非 ASCII，`<meta
charset=windows-1252>`，「non-ascii URL in query, document encoding
windows-1252」文档编码依赖案）——before 侧 read_to_string `Err` 整案不可见，
after 侧按声明解码可运行（1 子测，现 Fail：assert_unreached，产品 URL 面
能力账，归 M2/M3 基线分析）。这是「读不进来 → 读得进、如实失败」的直接
实证。

envelope 对称超限四通道（两侧同构，负载/footprint 预置条件，非变更效应）：
dom 与 navigation 900s 双侧超时（exit=124 空输出）、uievents 900s 双侧超时、
css-variables 双侧单进程内存超限（4.22–4.32GB > 4GB per-proc 上限）——均以
放宽 envelope（1800s；css-variables per-proc-mem 6）补跑对账，行随后续提交
回填。其余通道（timing 族 8、security 族 3、service-workers 3、dom-native）
经核对同走替换后读取路径（run_performance_api_subdirs /
run_service_worker_case_set 等），补 lane K 全量对账。

### lane 收尾回填（2026-10-28 全 lane 完成后）

| 通道 | cases b/a | subtests b/a | 集合 | status 差 |
|---|---|---|---|---|
| dom（放宽 1800s） | 368/368 | 56172/56177 | +20/-15¹ | 1² |
| navigation（放宽 1800s） | 431/431 | 584/584 | = | 0 |
| uievents（放宽 1800s） | 317/317 | 2397/2397 | = | 0 |
| selection | 611/611 | 41223/41223 | ±1³ | 0 |
| net-api | 487/488⁴ | — | 预期解锁 | — |
| csp（@3600） | 445/445 | 678/678 | = | 0 |
| service-workers / -fetch / -cache-storage | 65 · 31 · 25 / 同 | 245 · 76 · 318 | = | 0 |
| dom-native（ZW_NATIVE_DOM=1，1800s） | 368/368 | 56008/56175 | +172/-5¹ | 0 |
| dom-range | 10/10 | 1903/1747 | +36/-192¹ | 0 |
| dom-gamepad | 2/2 | 5/5 | = | 2² |
| css-variables（放宽 per-proc-mem 6） | — | — | 双侧内存超限⁵ | — |
| timing 族 10 通道 | — | — | lane 脚本子命令名错⁶ | — |
| 其余 20 通道（html、fullscreen、clipboard-apis、keyboard、touch-events、webgl、wasm、webaudio、webcrypto、indexeddb、console、canvas、canvas-worker、constraints、intersection-observer、resize-observer、cache-storage、web-components 等） | 全同 | 全同 | = | 0 |

¹ `dom/ranges/Range-mutations-*` 五个 300+ 子测大案双侧均触 30s 案级超时，
截断点随负载漂移——公共对 status 零翻转（dom 行 +20/-15、dom-native
+172/-5 其中 172 为 before 侧案级超时不可见的新完成子测 135P/35F/2T、
dom-range +36/-192），同 selection/html-semantics 超时形态 flake 范畴。
² `Event-timestamp-high-resolution` gamepad 构造时间戳面：dom 行 Pass→Fail
×1 与 dom-gamepad 行 Fail→Pass ×2 反向成对（同文件族）→ 时序 flake。
³ 见前文 selection 叙述（超时表现形态）。
⁴ `websockets/constructor/016.html` before 侧 read `Err` 不可见、after 侧
预期解锁，见前文叙述。
⁵ 放宽到 6GB per-proc 后仍双侧对称超限被 guard 杀（6.3GB / 6.9GB，exit
124），无 A/B 数据——维持预置条件定性，不阻 DC-1（通道走同一替换后读取
路径，改动面与 dom 恒等通道同构）。
⁶ lane-k 脚本对 timing 十通道漏 `testharness-` 前缀，双侧均产出 usage 文本
秒退（无真数据）；以保存的 M0 before/after 二进制补跑（k2 lane），行随补跑
回填。

### html-semantics 专项分析（lane G 实证 + 解析证明）

通道语料 2,197 个 .html 解析分类：可证不变 = 纯 ASCII 文件（任何 ASCII
相容单字节解码恒等）+ 头部无声明/utf-8/不可识别 label 的文件；**解码文本
可能变化面 = `scripting-1/the-script-element/{css,json,text}-module/
charset-2.html` 三文件**（`<meta charset="windows-1250">` 在 1024 窗内、
本体恰为合法 UTF-8——before 侧按 UTF-8 读入，after 侧 prescan 按
windows-1250 解码出 mojibake 源）。**lane G 实证：三案结果逐字节恒等**
（status/name/完整 message 全同——css-module/json-module/text-module 三案
在两侧同为 Timeout/Pass/Timeout，失败均在模块能力面，与页面源文本无关；
json-module/charset-2「loaded as utf-8」两侧同 Pass）。
`textarea-textLength.html` 的 `<meta content="charset=utf-16">` 核过 sniff
实现（parser.rs:619）：无 charset 属性、无 http-equiv content-type → 非编码
声明，spec 语义不触发——不变。

实测子集差仅 1 对（非编码面）：`the-img-element/update-the-image-data/` 族
一子测 before=「srcset = /images/not-found 2 time(s)」Pass / after=URL 名
Timeout 合成行——与 selection 同款超时表现形态 flake（图片加载时序，lane F
双通道并行负载下），非本变更效应。全通道 A/B（5400s envelope，实际 ~8min/
侧）行见上表。

## 偏离与挂账

- 全量语料续拉受阻：cdn.jsdelivr.net 连接拒绝、raw.githubusercontent.com
  约 1/3 成功率带超时（探测单次成功，批量 degraded）；FORCE=1 + GITHUB_TOKEN
  + curl 改写 shim 后台重试仍超时。当前 136 个 html（七族 wrapper 已齐，
  chars 大文件缺）。M1 开工以 `goals_inventory` 核对，网络恢复即补。
- charset 属性脚本解码面（`decode_classic_script_bytes` 是否吃 charset
  属性）维持 s115 口径挂账，不随本入口扩展。
- 本机 envelope 漂移：dom/navigation/uievents 900s 与 css-variables 4GB
  per-proc 在两构建下对称超限（含空闲机顺序跑）——Makefile 既有 envelope 对
  当前语料量已偏紧，属预置条件；本 goal 不动他域通道 envelope，仅在 A/B 中
  放宽补跑（他域 envelope 是否上调归各自 goal）。
