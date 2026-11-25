# kc5r132 verdict——selector-validity 刀：::part()/::slotted() 后缀文法状态机＋Query/Supports 双模式＋:host 族＋影子域三面校验门（2026-11-26）

## 结论

- **刀域**（三面协同；判据页 css/css-shadow 五页文法池 140 键——slotted-parsing.html
  36＋part/pseudo-classes-after-part.html 67＋part/pseudo-elements-after-part.html
  23＋host-parsing.html 7＋host-context-parsing.html 7；其中 133 翻绿、7 键
  b0 原已 Pass。全灭机制＝dom 校验器无影子域文法臂＋supports/insertRule 通道
  permissive）：
  1. **Face A（dom/query.rs 后缀状态机）**：`ShadowSuffix {Free, AfterPart,
     AfterSlotted, Sealed}` ＋ `ValidMode {Query, Supports}` 双枚举——
     - `parse_simple_selector_ctx` 状态机版（pub `parse_simple_selector`
       签名不变包 Query+Free；`'#'/./[` 成分 AfterPart/AfterSlotted 态拒）；
     - `parse_shadow_pseudo_element_ctx`：Free 态 `::part(<ident-list>)`（闭合
       非空、形状门）→AfterPart，`::slotted(<compound>)`（裸形/空实参/组合器
       链拒；未闭合保留 R159 宽容）→AfterSlotted；AfterPart 伪元素表 21 名
       （tree-abiding+page 列表含 highlight()/view-transition-*() 函数形）→Sealed；
       AfterSlotted tree-abiding 表 10 名＋`::picker()` 函数形（裸形拒）→Sealed；
       Sealed 拒一切（`::part` 链/`::slotted` 链判非法）；
     - `parse_pseudo_class_ctx`：AfterPart 禁表 18 名（@page left/right/first＋
       结构 11＋树语境 host/host-context/has/scope），`:not` 恒严格递归、
       `:is()`/`:where()` Query 宽容（forgiving_selector_list 失败项丢弃）/
       Supports 非宽容（strict_selector_list）分模式；AfterSlotted 仅 Query
       宽容 `:is()`/`:where()`；表外伪类开放放行 `PseudoClass::Other`（宽纳面
       原文形记账恒不匹配）；
     - `:host(<compound>)`/`:host-context(<compound>)` 新变体（Host/HostContext，
       匹配恒 false——空实参/组合器实参/列表实参拒；**裸 `:host-context` 非法**
       判据页 driving、裸 `:host` 合法）；
     - `parse_selector_chain_mode` 链级门（影子伪元素复合后接组合器判非法——
       `::slotted(*) span` 等）；
     - `selector_is_supports_valid` 新 pub fn（Supports 通道）；
     - 双冒号实参抽取深度感知 `find_matching_paren`（嵌套 `::slotted(:not(.a))`
       不被内层闭括号截断；单冒号老路径裸 find 零漂移——`:not(:is())` 保守拒
       形钉护；FIXME：深度扫描无引号感知）。
  2. **Face B（engine js_dom_bridge.rs supports Selector 臂）**：permissive-true
     改 scoped 门——选择器含影子域标记（`::part`/`::slotted`/`:host`，ASCII
     小写探测）→ `selector_is_supports_valid` 非宽容判定，域外零漂移。
  3. **Face D（shim part06.js _ruleFromText StyleRule 臂）**：insertRule 校验门
     同影子域三族标记——非法形抛 SyntaxError DOMException（cssom-1 §insert-rule；
     `__zw_selector_valid` Query 模式复用 Face A 新文法）。
  4. **并行轮协作账**：find_matching_paren 深度配对＋`::slotted(:not(.a))` 双
     模式钉＋单冒号保守拒护栏由前驱 zombie 复活轮贡献（NOTES 仲裁件
     /tmp/zw-k132/NOTES.md 让位协议；pristine-run1/run2-gen29 代际保全）。
- **指纹**：`ZW-KC5R132-SELECTOR-VALIDITY`（css_wire.rs `#[used]` 28B）＋
  CARRY-K131 `ZW-KC5R131-SHADOW-DEFS`＋CARRY-K130 双线承独立重核＝1/1。
- **电池代**：b1＝gen31 `r132-selector-validity-runner`（md5
  3aecc612f40e52f47f6f468fab8ac25f，两次追加修复收束后终代）；b0＝k131 归档
  bin `r131-shadow-defs-runner`（md5 f09de3e3a99d7e2da6a908b5038751c8）。
  中间代考古：gen28 b911a21e（+123，残 `::slotted(:not(.a))` 深度配对 1 键）→
  gen29 ce1fa25e（zombie 修复，+124，残 host 9 键）→gen30 ab768ec5（Face B
  `:host` 门，+124 同残——失败通道 insertRule 非 supports）→gen31（Face D
  `:host` 门，+133 全清）。
- **三语料终判**（KEY 级差分程序化生成，diff-all-k132.py 含 bare() 键名剥离
  ＋判据池交叉核；净恒等式逐 suite 验算全过）：
  - **css-shadow（主靶）P 95 → 228 = +133**：F2P=133 P2F=0 GONE=NEW=0，
    判据池 140 文法键 not-Pass-after=0（F2P 分页：part 两页 90＝67+23＋
    slotted-parsing 29（池 36，7 键 b0 原已 Pass）＋host 两页 14 全翻）。
  - **css-mixins（主判据红线）P 485 → 485 = +0**：F2P=0 P2F=0 GONE=NEW=0
    （红线 PASS）。
  - **css-values（共享通道 tripwire）P 6841 → 6841 = +0**：F2P=0 P2F=0
    GONE=NEW=0。
- **钉测**：dom 常驻钉 7（query.rs tests）＝part 文法矩阵（合法基线 11＋禁表
  13＋形态门 5）＋part :is/:where 宽容与 ：not 严格双模式＋part 后缀伪元素表
  与链级门＋slotted 全矩阵（合法 9＋拒 16＋R159 未闭合成保留＋深度配对双模式
  ＋单冒号保守拒护栏）＋slotted Supports 严格＋host 族文法/判据 invalid 臂 12
  形/supports 合法形＋零匹配语义＋Free 态回归护栏（域外 15 形不变＋裸
  `::slotted` 翻非法翻转钉）。
- **门禁**：zero-dom 全量绿（含 138 query 模块）；css-parser/style-system lib
  绿（3419+2731）；engine lib 4321P/3F＝既有红名单三钉（dialog_focus_s330/
  expando_r3046/p99s_inline_custom_property）对账合，零新红；fmt/clippy 三
  crate 绿（style-system custom_function.rs let_and_return clippy 1.99 现版
  红一行顺手门禁修复）。
- **挂账**：单冒号 `:host(...)` 实参嵌套 `:not` 深度扫描未接（老路径保守拒，
  FIXME(selectors-4) 已注记）；深度扫描无引号感知（`[title=")"]` 值内括号
  误配面，语料未见）；::part 实参 ident 语法级校验从宽（形状门拒
  逗号/括号/属性符，未做 css-syntax ident 全文法）。
- 判据页 <https://drafts.csswg.org/css-shadow-1/#part-pseudo>／
  <https://drafts.csswg.org/css-shadow-1/#slotted-pseudo>／
  <https://drafts.csswg.org/css-shadow-1/#host-pseudo>／
  <https://drafts.csswg.org/css-conditional-3/#supports-general>。

## 证据链

- 电池：chain2 六腿（04:38:27→04:46:49）gen28 全 RC=0；chain3 六腿
  （04:47:36→04:55:45）gen29 全 RC=0；gen31 b1 三腿（05:05→05:10）全 RC=0。
  ARM-b1 BIN md5=3aecc612 FACE=1（battery-k132.log）。
- 差分：/tmp/zw-k132/diff-all-k132.py（KEY 级＋bare() 键名剥离＋判据池交叉核
  ＋净恒等式验算）对 results-k132-b{0,1}-* 现算；中间代 pristine 保全
  /tmp/zw-k132/pristine-run1/（gen28）＋pristine-run2-gen29/。
- 并发环境：姊妹 clone ZeroWeb-3 trusted-types 流全程在跑（cwd 门天然隔离，
  零碰撞）；k132 电池锁键 /tmp/zw-k132-battery.lock 双 census 门全程生效
  （battery-arm 让位退出 6 一次＝协议正确）。
- 工件：/tmp/zw-k132/（chain/battery/battery-arm 三脚本＋build-out＋legs
  ＋transcripts＋diff/f2p 脚本＋NOTES.md 仲裁件）；归档 bin
  .acceptance/wptfyi-css/k132/bins/r132-selector-validity-runner（gen31）；
  电池结果 .acceptance/wptfyi-css/k132/results-k132-b{0,1}-*。
