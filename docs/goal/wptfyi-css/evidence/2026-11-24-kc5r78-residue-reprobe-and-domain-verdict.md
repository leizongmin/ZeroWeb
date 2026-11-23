# r78 残面重探定谳——刀 COLS 立项（columns 简写族 246F 三站）＋allow-discrete 出局

日期：2026-11-24。前序：[r77 interpolate-size 判死](2026-11-24-kc5r77-interpolate-size-probe-verdict-dead.md)
（commit 9bded3ca1）。运行档：[NOTES-kc5r78](../../../.acceptance/wptfyi-css/kc5r78/NOTES-kc5r78.md)。

## 0. 结论

- **候选 1（columns 简写族）定谳成立→r79 刀 COLS 立项**：当树实账 246F 三站
  分解（apply slash 222＋specified canonical 14＋gCS computed 形 10）。
- **候选 2（allow-discrete ~224F）出局**：当树已收清（transition-behavior
  主账 P=28 F=0，残余 7F 零散）。
- **新池：properties-value 系 260F wholesale**（transition 简写串化通道缺口）
  ＝r80 候选勘察。
- 探针基建：r75 bin（253ed910）单臂＋systemd unit（MemoryMax 24G/
  RuntimeMaxSec 2400）跑 css-multicol（91 页 P=1021 F=592）＋css-transitions
  （118 页 P=1049 F=438 timeout=19）双语料，RC=0 双绿。

## 1. 刀 COLS 三站（当树键级定谳）

### A 站：apply 臂 slash 形不收（222F）

- columns-interpolation.html（css-multicol/animation/）F=232 中 222＝supports
  断言（'from' 94＋'to' 128 "should be supported"）。
- 根因：apply_advanced.rs:2196 columns 臂 `split_top_level_whitespace` 把
  `100px / 200px` 切三段（'/' 独立 token）→ len==3 无分支命中 → apply false
  → `CSS.supports('columns', slash 形)` false。
- testcommon :396-469 每块首组断言即 `assert_true(CSS.supports(...))`，fail
  不 early-return（同 test 内 comparisonFunction 照常执行）＝修复后 222F
  **纯清无 skip 显影面**。
- 插值臂 `parse_columns_endpoint`（animation.rs:2767）slash 已接线
  （:5962 slash 形钉测在）＝A 站为 apply 单站缺口。

### C 站：specified canonical 串化（14F）

- columns-valid.html F=14（`e.style.columns = X` set→serialization should be
  canonical 断言）：width 先 count 后（`2 10px`→`10px 2`）＋auto 槽省略
  （`auto 3`→`3`；`auto auto`→`auto`）＋width 0→`0px`（`0 1`→`0px 1`）＋
  height 槽非 auto 才带 slash（`10px 2 / auto`→`10px 2`；`2 10px / 100px`→
  `10px 2 / 100px`）＝css-multicol-1/-2 §columns serialisation。
- 刀 Q（shape-outside specified canonical）同通道（fold_decl_specified
  白名单＋canonicalize 函数）。

### B 站：gCS computed 串化形（10F）

- interpolation 页 10 F＝全 `from [inherit] to [auto]` 形（FIXME
  (k-discrete-inherit) 同族）：期望通道 width-only 形（`50px`/`auto`），
  引擎 `columns_shorthand_to_css`（computed_style.rs:1026）双槽全串
  （`50px / 100px`/`200px / 100px`）＝count=auto 省略形缺失。
- 扰动普查：multicol 91 页消费 columns 串化形的仅 columns-valid 2F 面＋
  interpolation 10F 面；B 站改动经全语料 A/B 兜底。

## 2. 候选 2 出局与新池

- **allow-discrete 出局**：11-17 旧账 ~224F 当树已收清——transition-behavior
  P=28 F=0（主账绿；残余＝同页另一条目 3F＋custom-property-and-allow-discrete
  2F＋transition-behavior-events 2F 零散非刀面）。Transitions lane 6 键
  （KC-5 挂账）不在本语料口径（sizing 电池 GT 键），随 KC-5 挂账存档。
- **properties-value 系 260F wholesale**（r80 候选）：properties-value-003
  F=122 P=0＋implicit-001 F=60＋inherit-003 F=60＋002 F=18。fail 形
  `assert_not_equals ... got disallowed value ""`（值读回空串）＋census
  `exp='font-size:2s' got=''`/`column-gap:2s`/`column-width:2s` 形＝
  transition 简写/transition-property 值串化通道 wholesale 缺口。support JS
  五件全在（非 corpus 缺口）＝真通道刀位，根因勘察（页源深读＋串化通道
  定位）留 r80。
- 次级池存档：text-shadow composition+interp 24F（WAC 挂账同族）、
  vertical-align 19F、z-index 8F、CSSTransition 生命周期 tentative 簇 ~40F、
  timing-function computed 7F。

## 3. r79 处方（刀 COLS）

1. A 站 apply slash 归一＋C 站 specified canonical 串化为主体（236F）；B 站
   （10F）同轮或次轮视 A/C 落账。
2. 电池＝css-multicol 全语料 91 页 b0/b1 双臂 JSON 对账（刀 K
   verify-*-ab.sh 模式）；哨兵＝css-transitions（当树基线 P=1049/F=438 已由
   本轮锚定，results-r75 JSON 为 b0 参照账）。
3. 预注册门：F2P ≈246±N（N＝串化修复次生显影面）、P2F=0 fail-stop、
   哨兵 SET-IDENTICAL。
