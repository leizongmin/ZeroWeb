# kc5r79 刀 COLS 电池收账（A+C 站）

日期：2026-11-24。刀 COLS 实施（r75 face 基础上追加 A/C 站）+ 四腿 b0/b1 对照电池（unit `zw-r79-battery2`，锁 `/tmp/zw-r79-battery.lock`，日志 `/tmp/zw-r79-battery.log`）。

## 变更面（face 增量，未提交→本收账提交）

- A 站 apply 语法（`crates/style-system/src/property/apply_advanced.rs`）：旧 `columns` 臂（split len==2/len==1 形）替换为 `columns_shorthand_slots` 分派——slash 前组 width/count 顺序无关（整数≥1→count、长度含 unitless zero→width、`auto` 补空闲槽），slash 段=column-height，槽缺省按 reset 语义归 auto；css-multicol-2 §4.5 文法。
- 共享语法核（`crates/css-parser/src/values/parse_extended.rs`）：新增 `columns_shorthand_slots` + `serialize_columns_canonical`（width 前 count 后、auto 槽省略、双 auto→`auto`、height 非 auto 才带 slash、unitless zero→`0px`、token 逐字保留）。
- C 站串化（`crates/css-parser/src/values/serialize_math.rs`）：`fold_decl_specified` 增 `columns` 臂走上述 canonical 串化；非文法形 None 保留原文零漂移。
- 测试（`crates/css-parser/src/values/tests/columns.rs` 新建，tests/mod.rs 注册）：6 测覆盖槽分派顺序无关/auto 补槽/slash height 形/非法形拒绝/canonical 往返（22 钉面+插值形）/auto 大小写。

## 证据链

- b0 bin：`kc5r75/bins/r75-runner` md5 `253ed910`（r75 归档件，FACE-ANIM bb55d506 与 NOTES §0 锚一致）。
- b1 bin：`kc5r79/bins/r79-runner` md5 `f45097a1`（face 增量后新构建，已归档）。
- 符号门：`nm` `columns_shorthand_slots` b1=1 / b0=0——新代码在执行二进制内成立。
- 四腿全 RC=0：b0 multicol 06:28→06:36:58，b1 multicol →06:37:38，b0 transitions →06:45:38，b1 transitions →06:53:31（R79-BATTERY-DONE 06:53:31）。

## 主判定：multicol F2P=230 / P2F=0

| 腿 | P | F | T |
|---|---|---|---|
| b0 css-multicol | 1021 | 592 | 0 |
| b1 css-multicol | 1251 | 362 | 0 |

- b0 census 与 r78 锚（P=1021 F=592）逐位相等——基线零漂移。
- F2P=230（门 220–280 PASS）：columns-interpolation 198（A 站 slash+height apply 文法）+ columns-computed 18（gCS 锚页）+ columns-valid 14（C 站 14→0 清空）。
- P2F=0：fail-stop 通过，零回归。
- b1 残余 F=362 分解：columns-interpolation 页 P=350 F=34（离散 auto/inherit 端点家族，见下节归因修正）；其余 F 为既有基线池（discrete-no-interpolation 42、column-count/column-width-interpolation 38/37、column-rule-* 41、columns-invalid 17、getclientrects 35 等），三站未触碰、b0 同败。

## 哨兵：transitions 分解形通过（flip=0）

| 腿 | P | F | T |
|---|---|---|---|
| b0 css-transitions | 1049 | 443 | 19 |
| b1 css-transitions | 1049 | 446 | 18 |

- P 双腿 1049 与 r78 锚逐位相等。
- key-diff only-b0=1 only-b1=3 flip=0：严格键集形 SET-IDENTICAL 不等，但 **flip=0 零 pass↔fail 翻转**是回归信号位。
- 4 键全部落单页 `animate-with-color-mix.html`：b0 形=整页 90s TIMEOUT（CASE_WALL_S，子测缺席）；b1 形=页跑完 3 子测 FAIL。r78 窗同页形=4F+T（含 "Transition from srgb to color-mix" FAIL）。三窗分解：r78 4F+T → b0 T → b1 3F——color-mix 全败家族既有失败，90s 墙钟非确定翻转决定报告面，与 columns face 零交集，双向皆不入账。
- 跨窗 F 漂移（438→443→446）全部为时序页 presence 噪声：b0 净 +5＝event-dispatch.tentative.html 6 键新增（Active→After/Before/Idle×3、After→Before，时序域）− color-mix 1 键墙钟缺席；b1 净 +3＝color-mix 2 键墙钟补录 + event-dispatch 同 6 键复现（b0 bin 与基线 bin 同 md5，双侧对称不入账），零真翻转。
- 墙钟假说实证收口（CASE_WALL_S=180 高墙 solo 复跑，`sentinel-hw-rerun.sh`，results-b0-hw/results-b1-hw）：问题页 animate-with-color-mix 双臂各录 4 键同形（3 fail+1 timeout，零不对称）——主跑哨兵不等确证为 90s 截断伪影非行为差异；结合主跑全语料 flip=0，刀体对 transitions 语料零行为位移。

## NOTES 归因修正（B 站）

r78 NOTES §2 预判 B 站（`columns_shorthand_to_css` count=auto 省略缺失）贡献 10F——实测该省略已在 r75 face 内。columns-interpolation 34F 残余实为离散 `auto`/`inherit` 端点动画家族（CSS Animations/Transitions/Web Animations from [inherit]/[auto] 形），既有池，r80 候选，不入 A/C 范围。

## 判定脚本

`.acceptance/wptfyi-css/kc5r79/judge-r79.py`（本地）：multicol census/F2P/P2F + GATE + transitions 哨兵 + 键差分解。已知缺陷：transitions JSON 缺失时早退不打印 multicol 段（本次全量跑未触发）；presence 键归因需另行 inline 分解（本次已做）。

## 下轮候选（r80）

1. properties-value 260F 实施（根因已定谳：clone fixture 后代 gCS by-handle 通道失联，修复站 a/b/c + 预注册门见 evidence/2026-11-24-kc5r79-clone-gcs-root-cause-verdict.md §4）。
2. 离散 auto/inherit 端点动画家族 34F（columns-interpolation 残余）。
3. Q4 文档级复杂选择器对 clone 后代 miss（次要面）。
