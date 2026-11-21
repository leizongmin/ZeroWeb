# 刀 BASPGLWJ-002-015 证据（2026-11-21 16:25 收账）

## 靶面与净增

baspglwj-002..015 池 13 页 × 2 失败子测 = **净 +26**（css-text 语料），
RECONCILE gained=26 lost=0，预注册精确兑现零缺额零 P2F，无 ASYM。

- 002（U+200B ZERO WIDTH SPACE，ZW 类）：normal + pre-line 两臂 P 2→4。
- 003..012/014/015（U+2002 EN/2003 EM/2004 THREE-PER-EM/2005 FOUR-PER-EM/
  2006 SIX-PER-EM/2000 EN QUAD/2001 EM QUAD/2008 PUNCTUATION/2009 THIN/
  200A HAIR/1680 OGHAM/205F MEDIUM MATHEMATICAL SPACE，LineBreak BA 类
  ∩ White_Space 共 12 页）：pre-wrap + break-spaces 两臂各 P 2→4。
- 哨兵 values/box/sizing/flexbox 四语料 SET-IDENTICAL（gained=0 lost=0）。
- 013 页不在语料（wpt-data 无文件）。

## 机制（实证基线推翻压缩期旧假设）

失败形态逐臂核（results-knife-uax14ba/b1/css-text.json 新鲜基线）：
**仅 002 败 normal/pre-line；003..015 败 pre-wrap/break-spaces**（normal/
pre-line 因可折叠词分隔既有词间机会已过）——「002-015 全败 normal/pre-line」
旧假设作废。

- preserved 臂（pre/pre-wrap/break-spaces）`split_into_words_with_ws` 仅
  `' '`/`\t` 切词：BA 类空白分隔符落 `current_word.push(ch)`＝词内胶水，
  无断点 → 50ch 盒 39a+CH+30b 单行溢出 offsetHeight=30≤35。
- collapse 臂（normal/pre-line）U+200B 非空白（White_Space 属性假）非
  表内字符，滞留词内（has_cjk/has_ba 均 false）→ 单词不可断。

## face（layout-engine inline 两文件，文件级独占）

1. `inline_types.rs`：`is_uax14_break_after` 表增 BA∩White_Space 空白
   分隔符系 12 员（U+1680/2000..2006/2008..200A/205F）。明确不入：U+2007
   （GL，GLBREAK 域）、U+200B（ZW，LB8 语义不同且 preserved 臂 BA 臂先于
   R1449 弃字臂——入表会改写已通过的 002 preserved 双臂）、U+3000（ID，
   R4372 保留语义）。preserved 臂零代码改动——表扩展后 R646 BA 臂自动
   建立后断机会（字符附着前词尾 flush，advance 保留）；collapse 臂零扰：
   这些字符 step-1 即被 `split(is_collapsible_ws)` 切为词分隔符。
   新增 `#[used]` BASPGLWJ_FACE_MARKER（`"BASPGLWJ ba-space zw-collapse"`，
   与 slice1 UAX14BA 串分立防 FM 门恒真形）。
2. `mod.rs` collapse 子分词器：`has_zw` 门（`uax14_ba_break() && cjk_contiguous
   && contains ZW`，镜像 has_ba 精确形——子分词器内 BA 臂不复查 flag，
   独立入口不挂同款门则 `ZW_UAX14_BA_BREAK=0` 回退态泄漏 BA 臂行为）＋
   词内 U+200B 弃字 flush 臂（UAX14 LB8「ZW ÷」；LB8 的 ZW SP* ÷ 后继
   空白由 step-1 split 天然承载；R1449 preserved 臂零宽先例，flush 不注
   空格 advance 零注入）。
3. 钉测：`uax14_ba_chars_are_break_after` 扩员——+2002/2006/200A/1680/205F，
   −2007（GL）/−200B（ZW）/−3000（ID）；1 passed。

## 电池（zw-knife-baspglwj-battery-1609，test-guard 包裹）

- 16:09 发射，序列化门（urlsmod proc 探测＋busy-units 三层＋裸 bash 探测
  排除自家链）即时放行；预门 cargo check → b0 strip → b0 build 56s →
  b1 apply → b1 build → 十腿 → RECONCILE → 16:25:11 GATE-DONE rc=0。
- 判别门：FM b0=0/b1=1；M2 异 md5（b0 4dfa2ee6364d087bf63a8c378dd1f608 /
  b1 cca02c7cfbf3b9147dbf7097ad4aa8c6）；S1/S3 ride 窗内零扰动。
- 产物：`.acceptance/wptfyi-css/results-knife-baspglwj-002-015/{b0,b1}/`
  五语料 JSON＋bin 双 md5；日志 `knife-baspglwj-002-015-battery-1609.log`。

## 规范链接

- https://drafts.csswg.org/css-text-3/#line-break-details（§5.5）
- https://drafts.csswg.org/css-text-3/#white-space（§3 处理模型）
- https://www.unicode.org/reports/tr14/#BA（BA 类后断）
- https://www.unicode.org/reports/tr14/#LB8（ZW ÷ 后断）

## 残面挂账（R646-slice2 余域）

B2/WJ/SY 类域＋LB13 完整类（CL/CP/EX/SY 全类＋可折叠空白路径断点过滤）
＝FIXME 已注 inline_types.rs；baspglwj-016..118 族 B2/WJ/SY 页维持挂账。
