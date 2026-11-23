# r78 残面重探定谳——刀 COLS 立项（columns 简写族 246F 三站）＋allow-discrete 出局

日期：2026-11-24。前序：[r77 interpolate-size 判死](2026-11-24-kc5r77-interpolate-size-probe-verdict-dead.md)
（commit 9bded3ca1）。

## 0. 结论

r78 探针（probe-r78.sh，unit zw-r78-probe-1132，r75 bin 253ed910 单臂，语料
css-multicol＋css-transitions）双腿全绿（multicol 48s RC=0／transitions
05:30:12→05:38:14 RC=0，幸存多轮 session 重启——systemd unit 幸存通道再验证）。

- **刀 COLS 立项**（r79 实施）：css-multicol columns 简写族当树实账 **246F
  三站分解**（A apply slash 222／C specified canonical 14／B gCS 10）。
- **候选 2（allow-discrete ~224F）出局**：当树已收清（transition-behavior.html
  P=28 F=0）。
- **新池入册**：css-transitions properties-value 系 **260F wholesale**
  （transition 简写串化缺口）＝r80 候选。

## 1. 刀 COLS 三站定谳（columns-interpolation.html P=152 F=232＋columns-valid.html F=14）

### 1.1 A 站：apply 臂 slash 形不收（222F 主靶）

- supports 断言 222 fail（'from' 94＋'to' 128 "should be supported"）＝
  `CSS.supports('columns', '100px / 200px')` false。
- 根因＝apply_advanced.rs:2196 columns 臂 `split_top_level_whitespace` 把
  `W / H` 切三段（'/' 独立 token）→ len==3 无分支命中 → apply false。
- 对照：插值臂 `parse_columns_endpoint`（animation.rs:2767）slash 已接线
  （:5962 钉测在）＝同族两臂接线不对称。
- 修复＝三段归一两段（W/H 两臂既有解析器复用）。
- **testcommon :396-469 supports 断言 fail 不 early-return**（同 test 内
  comparisonFunction 照常执行）＝修复 222F 纯清无 skip 显影面。

### 1.2 C 站：specified canonical 串化缺口（columns-valid 14F）

刀 Q 同通道（fold_decl_specified 白名单），css-multicol-1/-2 §columns
serialisation 规则：
- width 先 count 后（`2 10px`→`10px 2`）；
- auto 槽省略（`auto 3`→`3`；`auto auto`→`auto`）；
- width 0 → `0px`（`0 1`→`0px 1`）；
- height 槽（css-multicol-2 扩展）非 auto 才带 slash（`10px 2 / auto`→`10px 2`；
  `2 10px / 100px`→`10px 2 / 100px`）。

### 1.3 B 站：gCS computed 串化形（10F）

10 fail 全 `from [inherit] to [auto]` 形（FIXME(k-discrete-inherit) 同族）：
期望通道 width-only 形（`50px`／`auto`），引擎 `columns_shorthand_to_css`
（computed_style.rs:1026）双槽恒串（`50px / 100px`）。修复＝count=auto 时
省略 count 槽。扰动普查：multicol 语料 91 页中消费串化形的仅上述两页。

## 2. css-transitions 当树账（候选 2 判读）

- corpus 118 页 P=1049 F=438（当树基线锚定，r79 刀 COLS 电池哨兵用）。
- allow-discrete 簇：transition-behavior.html **P=28 F=0**（旧 ~224F 已收清，
  ffix2 discrete 刀＋后续收割兑现）；残余 7F（behavior 页 3＋custom-property
  2＋events 2）＝小残面。
- **properties-value 系 260F wholesale**：properties-value-003 F=122 P=0／
  implicit-001 F=60／inherit-003 F=60／002 F=18——fail 形
  `exp='font-size:2s' got=''`／`column-gap:2s`／`column-width:2s`＝transition
  简写/transition-property 值串化通道 wholesale 缺口（页级全灭）＝r80 候选，
  根因勘察待 r79 并行。
- 次级：text-shadow 24F（WAC 挂账同族）／vertical-align 19F／z-index 8F／
  CSSTransition 生命周期 tentative ~40F／timing-function computed 7F。

## 3. r79 处方

1. **刀 COLS 实施**：A＋C 双站主体＋钉测（apply slash 三形＋canonical 七形），
   B 站同轮或次轮；电池＝css-multicol 91 页 b0/b1 双臂 JSON 对账
   （刀 K verify-*-ab.sh 模式）；预注册门 F2P≈246±N／P2F=0 fail-stop／
   css-transitions 哨兵 SET-IDENTICAL（§2 基线锚定）。
2. properties-value 系 260F 根因勘察（页源＋串化通道定位）与刀 COLS 电池窗
   并行，r80 立项决策。
3. face 驻留条款持续（r75 工作基 2574F，REDLINE P2F 85>0）。
