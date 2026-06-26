# 待消费 clean lever 队列（code-agent hand-off）

> 来源：R678–R709 LAYOUT_DUMP 脉络跨 12 目录采样（7 top-level + 5 CSS2 subdir）。
> 12 目录采样结论：**clean LAYOUT lever 全来自 box-model sizing 目录**（tables/normal-flow/position/grid/margin-padding-clear/floats-clear）；render/flow/dynamic 目录（flexbox/multicol/writing-modes/box-display/linebox/borders）均 structural/render/JS-gated，零 clean lever。
> 本文档汇总 **11 个 clean-ish lever + R678 float-width 簇**，按「effort × yield」排优先级，供 code-agent 消费。每条已 read-only 定位根因 + 代码位置（截至 R709，行号可能随主分支漂移，动手前 `grep` 核验）。
>
> ⚠️ **消费准则**：① 每个修复须 `make test` + scoped `make reftest` 零回归；② 涉及渲染/布局须额外 `make product-smoke`（DC-13 welcome 门禁，diff>20% 退出 2，R541 教训）；③ 用 chromium-Oracle A/B（`make reftest-oracle DIR=<case>`）确证 yield、排查假通过（self-source 同源 REF 会抵消）；④ 代码位置注释见各轮 master.md / archive 详记。

## 优先级总览

| 优先级 | lever | 测试 / 目录 | 发散 | 复杂度 | cluster yield |
|---|---|---|---|---|---|
| **S 快速赢** | R689 | position-static-001 / css-position | 17.53%+3% | 低（一行级） | 2 案（twin） |
| **A 高 cluster** | R678 | font-size-zero-3 / css-fonts（float shrink-to-fit） | 33.53% | 中 | **≥4 案 21-34%** |
| **A 高 cluster** | R696 | inline-replaced-width-009 / CSS2_normal-flow（replaced sizing） | 22.08% | 中 | **≥5 案 12-22%（unified）** |
| **A 高单发散** | R695 | height-percentage-005 / CSS2_normal-flow（%height indefinite-CB） | **88.44%（最高）** | 中 | +潜在多案 |
| B clean | R702 | margin-collapse-101 / margin-padding-clear | 49.38% | 中 | 7 案（101/105/106/155/038/110/111） |
| B clean | R691 | replaced-element-...-002 / css-grid（grid-item %height vs track） | 34.00% | 中 | +replaced-...-001 9% |
| B clean | R679 | empty-table-height / css-tables（table shrink-to-fit） | 30.74% | 中 | table-width 簇 |
| B clean | R699 | block-non-replaced-height-011 / CSS2_normal-flow（§10.5.1 float-height） | 16.12% | 中 | +clearfix 簇 |
| C complex | R705 | margin-collapse-clear-015 / floats-clear（clearance+collapse） | 33.66% | **中高** | 4 案（015/014/012/013） |
| C caveat | R692 | nested-grid-item-block-size-001 / css-grid（aspect-ratio） | 真 64.15% | 中 | **oracle 损坏须先 regen** |
| C caveat | R680 | table-cell-width-0 / css-tables（br 行盒高度） | 20.09% | 中 | §9.2.1.1 blast radius 风险 |

---

## Tier S — 快速赢（最低 effort，建议先做）

### R689 · position:static 元素被应用 inset（应忽略）
- **测试**：`css-position/position-static-001`（17.53%）+ `CSS2/positioning/position-static-001.xht`（3.00%，同 bug twin）
- **根因**：`converter/mod.rs:71` inset **无条件**传 taffy + `:295` `PositionValue::Static => taffy::style::Position::Relative`（Static 映射 Relative，taffy Relative 应用 inset 偏移）→ static 元素被 top/left/right/bottom 偏移。
- **fix scope**：`position==Static` 时 inset 归零（converter/mod.rs:71-74 加 position 条件，Static 不传 top/left/right/bottom）。**一行级条件修复**，spec-correct（CSS：inset 仅适用于 non-static position）。
- **风险**：低（无 spec 测试期望 static 元素被 inset 偏移）。

---

## Tier A — 高 yield（cluster 或最高单发散）

### R678 · float 0-content / width:auto 取满宽非 shrink-to-fit（★cluster≥4 案）
- **测试**：`css-fonts/font-size-zero-3`（33.53%，R678 pin）+ `floats-clear/float-non-replaced-width-007`（21.98%，R704）+ `floats-clear/floats-125`（28.80%，R706）+ `floats-clear/floats-124`（28.24%，推同族）
- **根因**：`float_positioning.rs:16` 注释明言「taffy 将 float 当普通 block（按正常流）」→ float width:auto 被当 in-flow block 拉伸到满宽（784）。ZW `float_positioning.rs` 仅 `shrink_vertical_blocks_to_content`（:47，仅垂直 WM）+ `shrink_inline_blocks_to_content`（:111，仅 inline-block R180），**无水平 WM float shrink 后处理**。
- **fix scope**：加水平 WM float shrink-to-fit 后处理（扩展 `shrink_inline_blocks_to_content` R180 模式到 float，或 taffy 0-content float 测量 clamp）。ZW-side post-process 可行（float 位置由 float_positioning.rs 独立设，与父 height 解耦）。
- **风险**：中（须 A/B 排查 float-heavy 测试，但 spec 上现「满宽」行为错）。

### R696 · svg/canvas replaced 元素无 sizing（★unified cluster≥5 案）
- **测试**：`CSS2/normal-flow/inline-replaced-width-009`（22.08%）+ 008/ib-007/ib-008 + `css-tables/percent-height-replaced-in-percent-cell-003`（R683，canvas 满宽）
- **根因**：`tree.rs:165 apply_replaced_element_sizing` line `186 if tag != "img" { return }`——仅 img 走 §10.3.2 sizing；`engine.rs:646` 已把 `img|video|iframe|embed|object|svg|canvas` 全标 `is_replaced=true`，**但 svg/canvas/object/iframe/video early-return 无 sizing**→taffy 当普通元素（满宽 / content 高 0）。
- **fix scope**：扩展 `apply_replaced_element_sizing`（tree.rs:186）超越 img——对 svg/canvas/object/iframe/video：① 读 width/height HTML attr；② 无 intrinsic width 且无 intrinsic ratio 时 used width 默认 **300px**（§10.3.2），无 intrinsic height 默认 **150px**；③ 有 intrinsic 时按 intrinsic+ratio。**code-located 一处 + 簇 yield≥5**。
- **风险**：中（现有 img sizing 不变，新增 svg/canvas 路径，须 A/B）。

### R695 · %height 对 indefinite-CB-height 解析到 CB width（应 compute-to-auto）（★最高单发散 88%）
- **测试**：`CSS2/normal-flow/height-percentage-005`（**88.44%，全 lever 最高**）
- **根因**：CSS §10.5——%height 仅当 CB height **显式 specified** 时解析，否则 **compute to auto**。本案 CB 链 grandparent(0,definite)→parent(auto,**indefinite**)→child(100%,**应 auto**)→img(100%,**应 auto→intrinsic 96**)。ZW 把 %height 传 taffy 作 Percent，taffy 0.7.7 在 indefinite-CB-height 时 fallback 到 CB width(784)；**ZW 无 normal-flow %height 后处理**（`engine.rs:1404 clamp_percentage_max_height` 仅处理 max/min-height Percentage，不处理 height Percentage）。
- **fix scope**：仿 `clamp_percentage_max_height`（engine.rs:1404）加 §10.5 处理：遍历布局树，对 height:Percentage 元素，若其 CB height indefinite（auto / percent-of-indefinite）则把 %height compute-to-auto（→ block 取内容高 / replaced 取 intrinsic；本案 img→96）。须传播 CB-height-definiteness（cascade）；blast radius 须 A/B（definite-CB %height 案如 min-height-094/095 须仍 pass）。
- **风险**：中（非一行，须 CB-definiteness 传播，但 spec 明确、本案 88%→~0%）。

---

## Tier B — clean single / cluster（中 yield）

### R702 · margin collapse-through 丢 inline content height
- **测试**：`CSS2/margin-padding-clear/margin-collapse-101`（49.38%）+ 105（identical）+ 106/155/038/110/111（cluster 7 案）
- **根因**：div.b（块子空 .red **+** inline "B" 文本）被算 h=0——margin collapse-through 嵌套块时 zeroing 父 content height，丢了 inline "B" 匿名块行高（§8.3.1 collapse-through + §9.2.1.1 匿名块）。`margin_collapse.rs:83 collapse_two_margins` + `:33 establishes_bfc` 基础已有，gap 在 collapse-through 路径保 inline 高度。
- **fix scope**：collapse-through 须保留父的 inline/匿名块内容高度（类 R680 §9.2.1.1 同族）。

### R691 · grid-item %height 解析到 grid 容器而非 track
- **测试**：`css-grid/replaced-element-percentage-height-in-grid-nested-in-flex-002`（34.00%）+ replaced-...-001（9.00%）
- **根因**：grid-item `height:100%` 解析到 grid 容器高度（400）而非其 row track（1fr=100）。违反 css-grid §11（grid item %size 应对 its grid area/track 解析）。
- **fix scope**：grid-item 百分比 size 解析须对 item 的 grid track（row height for %height / column width for %width）而非 grid 容器。

### R679 · display:table 0-content 取满宽非 shrink-to-fit
- **测试**：`css-tables/empty-table-height`（30.74%）
- **根因**：`display:table; padding:155px` 空 table 取满宽（784），应 shrink-to-fit（CSS 2.1 §17.5.2：table width = min(CB width, max-content)；空→~padding+border≈312）。table width:auto sizing（table.rs R177 territory）。
- **fix scope**：定位 0-content shrink-to-fit fallback（与 R678 float 同 symptom 簇，可能共享根因或独立 table path）。**注意**：R681/R682/R683/R684 已证 table-width 是多 facet 簇（box/content/horizontal/vertical），修须 writing-mode-aware shrink-to-fit + 子节点约束两层级。

### R699 · §10.5.1 非-BFC 块父 height 计入 float 子（应排除）
- **测试**：`CSS2/normal-flow/block-non-replaced-height-011`（16.12%）
- **根因**：`float_positioning.rs:16`（taffy 把 float 当 in-flow block，**计入父 content height**）→ 非-BFC 父（overflow:visible）继承 float-inclusive height。CSS §10.5.1：block + overflow:visible + height:auto → height = in-flow 子顶到底距离，**floating box 显式 ignored**。
- **fix scope**：加 ZW parent-height post-process（类 `clamp_percentage_max_height` / R695 §10.5）：对非-BFC 块父（`establishes_bfc` margin_collapse.rs:33 / `is_flow_root` engine.rs:676），重算 height = 仅 in-flow 子的 bottom（排除 float 子）。**次级 issue**：`is_flow_root` 仅查 FlowRoot|InlineBlock，未含 overflow:hidden/clip/auto/scroll BFC——若一并修须 expand BFC 检测。与 R108b（float flow_bottom margin）/R145（flex item float 归零）正交（那些是 float 子**位置**，R699 是**父 height**）。

---

## Tier C — complex 或 caveated（高 effort / 有前置）

### R705 · clearance + margin-collapse 算术（COMPLEX）
- **测试**：`CSS2/floats-clear/margin-collapse-clear-015`（33.66%）+ 014/012/013（cluster 4 案）
- **根因**：`#clear-left{clear:left} > div{margin-top:140}`——ZW 做 clear（clear-left 在 float 下）+ collapse（父子同位），但 **clearance + collapse-through-clearance 算术多出 ~110px**（#next-yellow 低 352 vs CHR ~240），子 mt=140 未完全 collapse 进 clearance（§8.3.1 + §9.5.1）。
- **fix scope**：修 clearance + collapse-through-clearance 算术。**medium-high 复杂**（clearance 多步算术），与 R702 margin-collapse 同 area 可一并评估。

### R692 · replaced img CSS aspect-ratio 被固有比覆盖（★oracle 损坏，须先 regen）
- **测试**：`css-grid/nested-grid-item-block-size-001`（committed oracle 20.41% / **真 64.15%**）
- **根因**：`tree.rs` replaced-sizing 分支（~270-285）用 `Length(ch * w/h)` 固有比推 auto 侧，**忽略 `computed.aspect_ratio`**——CSS aspect-ratio 应胜过固有比（本案 width 应 height×2=880，非固有 height×0.5=220）。
- **⚠️ 前置**：committed chromium oracle 损坏（blank broken-img placeholder，抓取 race，R388/R692 class）——**须先 regenerate 本案 oracle**（puppeteer 重抓 + `img.complete`/`naturalWidth` 等待），或建议 `chromium-oracle-shot.mjs` 加 img-load 等待防 race 复发，再 A/B 修复。

### R680 · br-between-block-siblings 匿名块行盒高度（§9.2.1.1，blast radius 风险）
- **测试**：`css-tables/table-cell-width-0`（20.09%）
- **根因**：`<br>` 作 block 容器直系子、与 block 兄弟混排时应被匿名块盒包裹（§9.2.1.1），匿名块 IFC 为 br 生成 line-height 行盒（~19px）；ZW br h=0（`inline/mod.rs:762` br→InlineItem::Br IFC 内正确，但 br-between-block-siblings 匿名块 strut 缺失）。
- **fix scope**：补 br-between-block-siblings 匿名块行盒高度。**R109 territory，须评 blast radius**（br-between-blocks 模式常见，修后或惠及多 case 或致位移；依赖当前 h=0 br 行为的 case 可能受影响）。

---

## 已识 structural-gated 区（非 clean lever，多会话架构，勿单点重试）

- **css-flexbox**（inline-as-IFC，R109/R255）：inline-box-blockification（converter inline→taffy::Block）。
- **css-multicol**（column-flow，Phase A）：无 column distribution（内容不跨列），column-span 不 fragment。
- **css-writing-modes**（axis-swap，R114）：vertical/sideways 模式轴旋转未实现（5.9% pass）。
- **css-grid** taffy track-sizing（R304 deferred）：隐式 auto-column 不扩展、grid intrinsic sizing。
- **原生 form-control 渲染**（R688）：platform-specific，out-of-scope。
- **box-display insert 簇**：dynamic-JS DOM mutation，须 JS/DOM-bridge 成熟。
- **linebox IFC**（R109 Phase-A deadlock，R247/R125）：line-box/leading/baseline/vertical-align。

## 工具链（read-only 复现 / A/B）

- `make reftest-oracle DIR=<dir|case>` — chromium-Oracle 真一致率 + top-15（DC-14 anti-false-pass）。
- `LAYOUT_DUMP=1 make reftest-oracle DIR=<case>` — 布局树 abs_y/height/margin/padding dump（裁决 clean vs structural）。
- `REFTEST_DUMP=1 make reftest-oracle DIR=<case>` — ZW vs self-source PNG（注：self-source 同源 REF 会抵消，须 chromium-Oracle 证）。
- PIL（`python3 + PIL`，无 numpy）— oracle validity / 像素 bbox / 颜色定位。
- `make product-smoke` — DC-13 welcome 回归门禁（渲染/布局变更必跑）。
