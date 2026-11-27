# WPT_REV 固定 pin（31597693）三方互斥核账与 DC-3 验收线校准

日期：2026-11-27  
依据：2026-11-27 用户批量拍板（commit `97c619357` 项 ① 选 B：本周期维持 `WPT_REV=315976933870b34d6ea30e3f6643403edae678ba` 不动，剔除上游漂移面并校准 `wptfyi-dom` / `wptfyi-html` DC-3 验收线；收官后统一协调升级）。

## 1. 结论与校准后验收线

三方精确互斥核账（`pin 31597693` 完整 GitHub 树 × `wpt.fyi` 对齐分母 `@a3830cd8f7` × ZeroWeb CI 覆盖产物 `run 45556963936`）修正了此前两处粗估偏差，并给出不混账的 DC-3 校准验收线：

| 流 | 原验收线（11-07 基线 + 净增） | 此前估算漂移 | 实测互斥 `post_pin` 漂移（文件 / 子测试 / Chrome 通过） | Chrome `pin` 内通过上限 | 本周期校准验收线（仅扣真实漂移） | 等效净增（相对 11-07 基线） | 关键边界说明 |
|---|---:|---:|---:|---:|---:|---:|---|
| `wptfyi-dom`（10 目录） | ≥ 99,947（64,947 + 35,000） | ~12,000 | **340 / 5,854 / 5,528** | 99,690（低于原线 257） | **≥ 94,093** | ≥ 29,146 | 占 Chrome `pin` 内通过的 94.39%；其中 2,404 项为 `tentative/` 转正改名失配，ZeroWeb 本地已过 1,359 项但未入快照 |
| `wptfyi-html`（`html/`） | ≥ 152,096（71,096 + 81,000） | 15,429（批十九） | **962 / 3,705 / 2,732** | 151,503（低于原线 593） | **≥ 148,391** | ≥ 77,295 | 批十九将 14,067 项 `pin` 内未覆盖误归入漂移；另单列非单文档载体硬边界 **5,660** 项（Chrome 5,404），对应单文档 Chrome 上限约 **146,099** |

核心判断：

1. **原验收线在维持 `pin 31597693` 下数学不可达**：`wptfyi-dom` 与 `wptfyi-html` 的 Chrome `pin` 内通过上限（`99,690`、`151,503`）均已低于各自原验收线（`99,947`、`152,096`），必须按方案 B 校准。
2. **两项历史漂移估算均需纠偏**：
   - `wptfyi-dom` 真实 `post_pin` 漂移为 **5,854** 项，不是 `~12,000` 项。`quirks/unitless-length`（4,752）与 `quirks/hashless-hex-color`（2,060）文件本体都在 `pin 31597693` 中，属于 `in_pin_uncovered`（iframe 载体与样式写面边界），不是上游漂移。
   - `wptfyi-html` 真实 `post_pin` 漂移为 **3,705** 项，不是批十九报告的 `15,429` 项。批十九点名「整目录 pin 后新增」的七簇（`interaction`、`COOP`、`COEP`、`DIP`、`user-activation`、`anonymous-iframe`、`capability-delegation`）和 `canvas/element/manual/wide-gamut-canvas` 均在 `pin 31597693` 中真实存在，批十九误将「现有脚本未拉取/未覆盖的 `pin` 内目录」混入了漂移桶。
3. **严禁把载体/兼容性缺口伪装成漂移核销**：`wptfyi-html` 若套用批十九误报的 `15,429` 直接降标至 `136,667`，等于把 11,724 项 `pin` 内未覆盖缺口按统计漂移一笔勾销。因此 `wptfyi-html` 校准线严格只扣真实漂移 `3,705`（得 `≥ 148,391`），同时诚实单列 `5,660` 项跨载体硬边界（`worker/shadowrealm`、`manual/`、多源隔离 `COOP/COEP/DIP/anonymous-iframe/capability-delegation`）；是否对载体边界另作豁免，待全量快照解冻后按实测进展由用户复议。
4. **快照解冻前置（`test262-builtins` 6 GiB 内存帽击杀）已核实修复**：`wptfyi-dom` 已在 `70f04eb1b` 根治（`part05.js` `_zwAsciiLower` 与 `part12-trusted-types.js` `_zwTtAsciiLower` 改 charCode 循环，避免撞 `RegExp.prototype.exec` 覆写进入无限分配），全量 `test/built-ins` 验证 `23,458` 案、`23,065P / 393F`、峰值 RSS `648 MB`（`4974ee5df`，证据见 [2026-11-27-s10k51-t262-balloon-root-cause.md](2026-11-27-s10k51-t262-balloon-root-cause.md)）；`wptfyi-html` 十九演首次穿越 18 连贴帽带（150 分钟零内存击杀，`58cb7f51c`），墙钟放宽至 `16200s` 后的二十演已在 3h29min 完跑全量 `53,162` 页（`51,913P / 1,190F / 59TO`，峰值 RSS `2.59 GB`，`79d29e4d7`），正待二十一演二证窗确认 CLEAN 后发射批四十九全量快照。

---

## 2. 核账方法与口径桥接

- **对齐 `wptfyi_gap.py` 1:1 计分逻辑**：完整复现 `tests/wpt-runner/scripts/wptfyi_gap.py` 的 `build_universe`、`build_sliced_totals`、`merge_zw_results`（过滤 `zw-static-corpus`）与 `resolve_to_universe` 路径归并规则。核账结果与官方快照逐子测试零误差：
  - `DOM10`：`den = 110,536`、`zwP = 76,670`、`chromeP = 105,218`。
  - `html`：`den = 160,124`、`zwP = 109,414`、`chromeP = 154,235`。
- **`pin 31597693` 真树对照**：通过 GitHub Git Trees API（`recursive=1`）拉取 `WPT_REV=315976933870b34d6ea30e3f6643403edae678ba` 下 11 个一级目录（DOM 10 目录 + `html`，共 16,355 个子树 blob）的完整路径集合，全部返回 `truncated=false` 零截断。对每个分母路径剥离 query 参数后，匹配直接文件或 `.any.js` / `.window.js` / `.worker.js` / `.serviceworker.js` / `.sharedworker.js` 源文件：命中即归 `in_pin`，未命中归 `post_pin`，两者严格互斥（`in_pin + post_pin == den`）。
- **与 `coverage-2026-11-25.json`（`@056c528ccb`）的桥接**：本核账逐文件明细基于与 `coverage-2026-11-22` 同批的 `wpt.fyi` 对齐缓存 `@a3830cd8f7` 叠加 `run 45556963936`（即 `coverage-2026-11-25` 的全部 167 份 CI 分片产物）。`@a3830cd8f7 → @056c528ccb` 四天间上游分母仅微增：`DOM10` `+13` 项（`110,536 → 110,549`，+4 文件）、`html` `+103` 项（`160,124 → 160,227`，+20 文件），ZeroWeb 通过数逐字一致（`76,670` / `109,414`），不影响校准线量级与结论。

---

## 3. `wptfyi-dom`（DOM 10 目录）精确三方核账

### 3.1 逐目录互斥分解（`@a3830cd8f7` × `pin 31597693` × `run 45556963936`）

| 目录 | `den` | `zwP` | `chromeP` | `post_pin`（文件） | `post_pin` Chrome | `in_pin` | `in_pin` Chrome | `in_pin` 已覆盖 | `in_pin` 未覆盖 | `in_file_gap` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `dom` | 67,148 | 56,993 | 64,295 | 634（123） | 562 | 66,514 | 63,733 | 60,590 | 5,924 | 3,365 |
| `shadow-dom` | 17,933 | 14,124 | 17,330 | 117（62） | 77 | 17,816 | 17,253 | 17,629 | 187 | 3,189 |
| `custom-elements` | 4,854 | 3,153 | 4,703 | 180（32） | 151 | 4,674 | 4,552 | 4,461 | 213 | 584 |
| `quirks` | 6,984 | 40 | 6,959 | 1（5） | 1 | 6,983 | 6,958 | 110 | 6,873 | 0 |
| `domparsing` | 3,757 | 187 | 3,663 | 3,210（63） | 3,118 | 547 | 545 | 380 | 167 | 65 |
| `domxpath` | 1,172 | 21 | 1,128 | 17（3） | 12 | 1,155 | 1,116 | 1,152 | 3 | 1,033 |
| `sanitizer-api` | 3,834 | 516 | 3,108 | 1,606（31） | 1,535 | 2,228 | 1,573 | 2,227 | 1 | 1,648 |
| `trusted-types` | 3,764 | 1,636 | 3,234 | 33（9） | 30 | 3,731 | 3,204 | 3,124 | 607 | 912 |
| `xml` | 252 | 0 | 45 | 51（11） | 42 | 201 | 3 | 0 | 201 | 0 |
| `webidl` | 838 | 0 | 753 | 5（1） | 0 | 833 | 753 | 0 | 833 | 0 |
| **合计** | **110,536** | **76,670** | **105,218** | **5,854（340）** | **5,528** | **104,682** | **99,690** | **89,673** | **15,009** | **10,796** |

> 恒等校验：`104,682 + 5,854 = 110,536`；`89,673 + 15,009 = 104,682`；`99,690 + 5,528 = 105,218`。

### 3.2 `post_pin`（5,854 项）拆分与「已过未入账」发现

- **`tentative/` 转正改名失配**：**45 文件 / 2,404 项**（Chrome 2,335），其中 `domparsing` 占 2,355 项、`shadow-dom` 占 49 项。典型案例如 `/domparsing/positional-methods.html`（480）、`positional-methods-with-trusted-types.html`（355）、`stream-html-sanitizer.html`（338）、`positional-methods-with-parser-options.html`（284），它们在 `pin 31597693` 中全部位于 `domparsing/tentative/`。
  - 核查 CI 产物的 `zw_only_files` 发现：ZeroWeb 在 `DOM10` 中已有 **46 文件 / 1,446 项、通过 1,359 项**因 `tentative/` 路径改名被 `wptfyi_gap.py` 丢弃未计分（含 `domparsing/tentative/positional-methods.html` 348/348、`stream-html-sanitizer.html` 204/204、`positional-methods-with-parser-options.html` 160/160、`sanitizer-api/sethtml-with-trustedtypes.tentative.html` 151/151 等）。这部分属于**已实现的真实兼容性能力**，收官统一 bump `WPT_REV` 后即可自动回账。
- **`pin` 后纯新增文件**：**295 文件 / 3,450 项**（Chrome 3,193），大头集中在 `sanitizer-api`（1,606）、`domparsing`（855）与 `dom`（634）。

### 3.3 `in_pin_uncovered`（15,009 项）与 `in_file_gap`（10,796 项）

- `in_pin_uncovered` 按类别分解：
  - `quirks` iframe 模式页（`unitless-length` 4,752 + `hashless-hex-color` 2,060）：6 文件 / **6,812** 项（Chrome 6,812）——文件都在 `pin 31597693` 内，依赖 iframe 文档管线与样式协作；
  - `window/any` 主线程变体：95 文件 / **3,069** 项（Chrome 2,834，含 `dom/idlharness.window.html` 1,986）；
  - `xml/xhtml/svg`：59 文件 / **2,167** 项（Chrome 2,152，含 `dom/nodes/ParentNode-querySelector-All-xht.xht` 1,975）；
  - 其他静态 HTML：534 文件 / **1,658** 项（Chrome 1,507）；
  - `worker/shadowrealm` 变体：57 文件 / **1,303** 项（Chrome 1,286，含 `dom/idlharness.any.*worker.html` 660）。
- `in_file_gap`（已覆盖文件中 `raw_den > zw_total`）头部项：`shadow-dom/declarative/gethtml.html`（缺 3,109，含 CI 截断与上游扩测）、`dom/ranges/Range-mutations-dataChange.html`（缺 2,133，停摆截断）、`sanitizer-api/sanitizer-javascript-url.html`（缺 1,292）、`domxpath/xml_xpath_runner.html`（缺 1,023）、`dom/nodes/processing-instruction-attributes.html`（缺 615）、`trusted-types/set-event-handlers-content-attributes.tentative.html`（缺 526）、`Range-mutations-replaceData.html`（缺 469）、`custom-elements/registries/valid-custom-element-names.html`（缺 396）。

### 3.4 `wptfyi-dom` 校准验收线

- 扣减实测互斥 `post_pin` 漂移 `5,854`：`99,947 − 5,854` = **`≥ 94,093`**（相对 11-07 基线 `64,947` 净增 **`≥ 29,146`**）。
- 比例核对：`94,093 / 99,690 = 94.39%`（Chrome `pin` 内通过基准），与 95% 冲刺原目标保持同一严格度。当前 CI 快照 `76,670` 距校准线余 `17,423`，本地推进头数 `84,324`（`f749ae6bc`，含未入快照的 `tentative/` 与近期刀群）距校准线余 `~9,769`。

---

## 4. `wptfyi-html`（`html/`）精确三方核账与批十九勘误

### 4.1 总账互斥分解（`@a3830cd8f7` × `pin 31597693` × `run 45556963936`）

| 桶 | 文件数 | 子测试数 | Chrome 通过 | ZeroWeb 通过 | 说明 |
|---|---:|---:|---:|---:|---|
| `in_pin_covered`（pin 内已覆盖） | 6,535 | 142,352 | 138,204 | 109,414 | 含 `in_file_gap = 13,364`（大头 `html/dom/reflection-*.html` 8 份合计 ~4,446 等） |
| `in_pin_uncovered`（pin 内未覆盖） | 3,482 | 14,067 | 13,299 | 0 | 含未拉取目录、worker、manual、多源隔离与 skip 规则域 |
| **`in_pin` 小计** | **10,017** | **156,419** | **151,503** | **109,414** | **Chrome pin 内上限已低于原验收线 152,096（差 593）** |
| `post_pin` · `tentative/` 与目录迁移 | 78 | 532 | 513 | 0 | `template-for-*` 从 `html/dom/partial-updates/tentative/` 迁出等（`zw_only` 另有 134 通过） |
| `post_pin` · pin 后纯新增文件 | 884 | 3,173 | 2,219 | 0 | `semantics` 1,621、`canvas` 577、`browsers` 253、`DIP` 155 等 |
| **`post_pin` 合计（真实漂移 `D_html`）** | **962** | **3,705** | **2,732** | **0** | **修正批十九 `3,496 文件 / 15,429S` 的高估** |
| **分母总计** | **10,979** | **160,124** | **154,235** | **109,414** | 11-25 快照（`@056c528ccb`）为 `160,227`（+103），`zwP` 不变 |

### 4.2 批十九 `15,429S` 勘误实证

[批十九跑道勘误二](../../wptfyi-html/evidence/2026-11-22-dc3-goal-accounting-and-corpus-runway.md)报告 `上游漂移（pin 315976933 后新增）= 3,496 文件 / 15,429S`，并据此断定「七簇整目录 pin 后新增」「canvas/element wide-gamut 全属漂移桶」。对照 GitHub Trees API 在 `31597693` 的完整树，反证如下：

| 批十九归入「pin 后新增」的目录 | `pin 31597693` 实测 blob 数 | `in_pin_uncovered`（文件 / 子测试） | 真正 `post_pin`（文件 / 子测试） |
|---|---:|---:|---:|
| `html/interaction/` | 217 | 195 / 742 | 14 / 98 |
| `html/cross-origin-opener-policy/` | 192 | 160 / 645 | 4 / 13 |
| `html/cross-origin-embedder-policy/` | 153 | 103 / 576 | 6 / 23 |
| `html/document-isolation-policy/` | 42 | 35 / 171 | 24 / 155 |
| `html/user-activation/` | 29 | 22 / 48 | 50 / 66 |
| `html/anonymous-iframe/` | 25 | 32 / 57 | 0 / 0 |
| `html/capability-delegation/` | 8 | 6 / 16 | 0 / 0 |
| `html/canvas/element/manual/wide-gamut-canvas/` | 102 | 属 `manual` 桶（`pin` 内存在） | — |

原因在于批十九核查范围局限于现有 `fetch-wpt-html*.sh` 拉取清单与 `summary-chrome` 物理键，未对照 `pin 31597693` 的完整 GitHub 树，也没有做 `build_sliced_totals` 与变体归并。

### 4.3 `in_pin_uncovered`（14,067 项）构成与单文档载体硬边界

虽然批十九把未覆盖目录错标成了漂移，但拆解 `in_pin_uncovered`（14,067 项）可见，其中仍有大块超出当前单文档 headless runner 载体能力：

| 类别 | 文件数 | `den` | Chrome 通过 | 属性判断 |
|---|---:|---:|---:|---|
| `other-static-html` | 2,230 | 5,607 | 5,239 | 含批二十甄别出的大量 iframe/多窗口/导航 skip 门与可拉未接项 |
| `worker/shadowrealm` | 123 | 2,443 | 2,397 | **跨载体硬边界**（待引擎 worker/shadowrealm 车道） |
| `manual`（含 display-p3 广色域等） | 342 | 2,144 | 2,024 | **跨载体硬边界**（手工/特殊显示环境） |
| `COOP/COEP/DIP/anonymous-iframe/capability-delegation` | 216 | 1,073 | 983 | **跨载体硬边界**（需多源隔离 HTTP 上下文） |
| `xml/xhtml/svg` | 71 | 1,036 | 1,034 | 解析器/命名空间通道面 |
| `window/any` 主线程变体 | 284 | 975 | 901 | 变体包装面 |
| `interaction/user-activation` | 216 | 789 | 721 | 用户激活/焦点交互面 |
| **合计** | **3,482** | **14,067** | **13,299** | 其中三类硬边界合计 **5,660** 项（Chrome **5,404**） |

### 4.4 `wptfyi-html` 校准验收线与可达性说明

- **严格漂移校准验收线**：仅扣真实 `post_pin` 漂移 `3,705`：`152,096 − 3,705` = **`≥ 148,391`**（相对 11-07 基线 `71,096` 净增 **`≥ 77,295`**，较当前快照 `109,414` 余 `38,977`）。
- **单文档载体可达性警示**：扣除三类硬边界（`worker/shadowrealm` 2,443 + `manual` 2,144 + 多源隔离五簇 1,073 = 5,660 项，Chrome 通过 5,404）后，单文档可执行 `pin` 内 Chrome 上限约 **`146,099`**（仍低于 `148,391` 达 `2,292` 项）。因此本流后续收敛需同步追踪两条线：
  1. 行为面推进（`rendering`、`popovers`、`document.write`、`speculative`、`html/dom/reflection` 缩量等，归引擎/渲染/语义跨流碰头）；
  2. 待批四十九全量快照解冻落地后，结合实际收敛曲线决定是否将 `5,660` 跨载体硬边界另行豁免或移交引擎载体专项，绝不借用错误漂移数暗中降标。

---

## 5. 口径纪律与收官升级衔接

1. **双轨分账**：每轮全量快照对账必须同时列报「固定 `pin 31597693` 校准线净增（真实兼容性提升）」与「`post_pin` 漂移量（统计口径变化）」，不得把漂移变化、重命名回账或 `max(raw_den, zw_total)` 膨胀计为真实兼容性修复。
2. **收官统一 bump `WPT_REV` 衔接**：待 `wptfyi-css` / `wptfyi-dom` / `wptfyi-html` 主力冲刺收官后，选单一跨流窗口升级 `tests/wpt-runner/scripts/lib.sh` 的 `WPT_REV`，同步执行：
   - 切新前固定同一 ZeroWeb commit 跑旧/新 `WPT_REV` 双快照，精确分离 `Δ_corpus`（含 `DOM10` `tentative/` 改名回账的 1,359+ 项）与 `Δ_compat`；
   - 将 DC-3 跨周期对照线切回新 `pin` 全口径；
   - 若新 `pin` 出现大面积分片超时、内存击杀或脚本拉取断裂，单笔 `git revert` 恢复 `WPT_REV` 即可原子回滚。
