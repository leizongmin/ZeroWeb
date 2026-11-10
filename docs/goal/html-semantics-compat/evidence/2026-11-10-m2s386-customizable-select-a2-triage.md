# M2 切片 386（s386）——customizable-select A2 簇分诊轮（零源码改动）：特征门簇 53 文件无流内施工入口立项征询 + 无门散单 6 文件 17 行三分

日期：2026-11-10（承 s385 下一步候选：A2 簇分诊——s377 挂账 customizable-select A2 渲染流簇定性收窄）
域：html/semantics/forms/the-select-element（域 corpus 红案 152F+2T=非探针 154 行，全语义面最大余红块）
性质：**分诊轮（零代码改动）——红案文件 × 源码特征标记全量扫描定谳簇结构**

## 方法

s385 全树 sweep（8198P/270F/37T）红案清单 → the-select-element 域非探针 fail 文件 59 个（域 154 行=s325 探针 22 单列外）→ 逐文件源码特征标记扫描（`base-select` / `::picker` / `selectedcontent` / `<button` 四标记 grep 计数）→ 无门文件逐案读源定性（失败签名 + 断言依赖链）。

## 分型结果

### 簇 A：特征门簇——53 文件 137 行（135F+2T），流内无施工入口

53/59 文件源码引用 `appearance: base-select`（customizable-select/ 主体 + customizable-select-listbox/ 全部 + customizable-select-multiple-popup/ 全部 + filterable-select/ 全部 + 域根 select-required-attribute / select-option-focusable / select-many-options / select-popover-position-with-zoom / customizable-select 下 selectedcontent 三姊妹）。ZW 现状（s377 定性复核维持）：`parse_appearance` 无 base-select 值→appearance 声明非法被弃→picker 永不物化→开态机/UA 规则/交互语义全断。**簇内断言全部以 picker 存在为前提（全有全无门）——不存在特征门下的增量施工入口**（逐簇抽验：switch-picker-appearance/select-base-appearance-computed-style/select-keyboard-behavior 等断言链首环即 picker 物化/开态）。

涉及面清单（立项规模参考，六子面）：①parser——select content model（button/selectedcontent 及泛化子元素，whatwg/html#9799）；②css-parser——appearance base-select 值 + ::picker/::picker-item/::checkmark/::picker-icon 伪元素族；③style-system/UA sheet——base-select UA 规则族 + option display 面；④engine——open/close 态机 + picker 元素物化 + light-dismiss；⑤渲染——picker 定位/zoom/top-layer 锚定；⑥交互——键盘/鼠标/type-ahead/testdriver 注入面。上游态：whatwg/html#3748 系 + CSS tentative，语料 .tentative/.optional 标记 53 文件中 29 个。

### 簇 B：selectedcontent 族——gate 桶内 3 文件 4 行随簇 A（selectedcontent.html / -listbox / -multiple 均 base-select≥1）

### 簇 C：无门散单——6 文件 17 行，三分归属

| 文件 | 行数 | 定性 | 归属 |
|------|------|------|------|
| select-option-display-inherit.tentative | 3F | UA sheet option display 面：`display:inherit`→computed "contents" + closed 单选 option 零 gBCR（whatwg/html#3748 决议，option 不设 UA display 或 UA display:contents；ZW 现值待勘根） | **流内候选**（style-system UA sheet + gCS 链勘根后施工） |
| select-multiple-resize-does-not-autoscroll | 1F | multiple select resize 时 option list autoscroll 面 | 待勘根（疑渲染/交互） |
| select-listbox-touch.optional | 1F | touch 注入通道——runner 无 touch 事件注入能力（"Using touch input" 子测直败） | 通道能力上限面（s64 link/style 事件通道上限同形），不施工 |
| select-input-parsing.tentative | 2F | select 内 `<button>` 子元素解析（content model） | 簇 A 立项 scope（parser 面） |
| selectedcontent-mutations | 8F | `<selectedcontent>` 元素 + clone 语义 + 泛化子元素 content model（whatwg/html#9799） | 簇 A 立项 scope（parser 面） |
| select-picker-popover-open-invalid.tentative | 2F | ::picker 引用在场，断言依赖 picker 元素存在性 | 簇 A 特征邻位（随簇） |

### probe 单列（不计 corpus 口径）

s325-probe-options-move 13F + s325-probe2-corpus-replay 9F——自建诊断探针恒红记账面（s343 方法论），翻绿前提同簇 A 特征落地。

## 定谳

1. **A2 簇主体（53 文件 137 行）= customizable-select 特征级深结构面**：parser content model 为深结构改动（Mission 约束 3），跨六子面，**流内无轻量施工入口——转立项征询（用户决策）**；Chrome 上游亦分阶段交付、语料 tentative 标记面广，立项时建议按子面分期（parser→css-parser/UA→态机/渲染→交互）。
2. **流内近期候选**：select-option-display-inherit 勘根切片（3F，唯一明确 style 面无门案；先核 whatwg/html#3748 决议原文与 Chromium html.css 原件定 UA display 期望值与 ZW 现值差）+ select-multiple-resize 勘根（1F）。
3. the-select-element 域 corpus 口径定格 **152F+2T = 簇 A 135F+2T（立项挂账）+ 簇 C 17F（流内候选 4 / 通道上限 1 / 立项 scope 12）**；s325 探针 22 单列维持。

## 验证

- 标记扫描全量 59 文件 grep 计数（/tmp/s385/sel-markers.tsv）；无门 6 文件逐案读源 + 失败签名核对（s385 sweep 日志）；零代码改动 → lib/fmt/clippy/product-smoke 不适用。
- 日志与中间产物 /tmp/s385/（sweep-full.log、sel-fail-files.txt、sel-markers.tsv、bucket-gate.txt、bucket-nogate.txt）。
