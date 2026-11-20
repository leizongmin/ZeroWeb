# 刀 FONT-1b 证据（2026-11-20，R3-LAUNCH 发射 + 续开轮收账）

## 刀面

@font-face local() 源链 + oblique ±90deg 界（5 Rust 文件 + 1 钉测文件）：

1. `crates/css-parser/src/parser/helpers.rs` — 新增 `extract_local_sources_from_src`（css-fonts-4 §src-desc `local(<family-name>)` 源形提取，`local(` 前缀判别约定）
2. `crates/css-parser/src/parser/at_rules.rs` — `consume_font_face_rule` src 臂 local 提取分支；此前只认 url() 载荷，local-only face 整条丢弃
3. `crates/engine/src/js_dom_bridge/css_wire.rs` — sources 串化 local 分支（verbatim 回拼，载荷经声明 tokenizer 规范为 canonical 双引号形）+ wire 双钉
4. `crates/css-parser/src/values/parse_misc.rs` — `parse_font_style` oblique 臂 ±90deg 界（**生效份**：values/mod.rs:54 `pub use parse_misc::*`，apply.rs:741 调用唯此路径）
5. `crates/css-parser/src/values/parse_basic.rs` — 同款界（重复份，非公开导出路径，双份保持一致）
6. `crates/css-parser/src/values/color.rs` — 钉测 `test_parse_font_style_oblique_angle_bound`（90/-90 界内合法、91/-91 拒）

根因链：主靶 at-font-face-descriptors.html 静态 `<style>` local-only src face → parser None → 整条丢弃 → 页面 88F wholesale 形（FONT-1 后仍残）。

## 离线钉面（发射前全绿 + 续开轮复验）

- css-parser `test_parse_font_style*` 8/8（含新钉 `test_parse_font_style_oblique_angle_bound`）——续开轮 17:24 复验 8/8 全过
- css-parser corner 面定向 6/6（树态整编译绿副证）
- engine `wire_font_face*` 2/2（`wire_font_face_rule_entry` url 形 + `wire_font_face_local_source_entry` local 形，期望按实际 canonical 双引号形修正）

首跑修证：parse_font_style 双定义定位（parse_basic.rs:418 / parse_misc.rs:871 结构重复为既有形态，只提及不合并；`pub use parse_misc::*` 实证生效份）；wire local pin 期望值首版单引号错，按 tokenizer 规范形修正（行为正确，pin 期望错）。

## 电池（knife-font1b-battery.sh，zw-font1b-battery.service）

- 发射 13:49:12，GATE 14:30:33，rc=0，TRAP-NOOP（face 在树=发射态）
- 判别三件全过：FACE-MARKER `extract_local_sources_from_src` b0=0/b1=1（Rust 函数名符号法，CS-1 salvage 同法先例）+ M2 md5 异 + S1/S2/S3 ride census 零扰动（RIDE 13 文件）
- 14 腿（7 corpus × b0/b1），CASE_WALL_S=60，test-guard 包裹

## A/B 账

| 页 | b0 | b1 | Δ |
|---|----|----|---|
| css-fonts/variations/at-font-face-descriptors.html | P 0/88 | P 30/88 | **+30** |
| css-fonts/variations/font-parse-numeric-stretch-style-weight.html | P 78/81 | P 80/81 | **+2** |
| css-fonts/variations/font-style-parsing.html | P 31/43 | P 33/43 | **+2** |

TOTAL: gained=34 lost=0 regressed=0（-16 见下节定谳 flake 不入账）。三 gained 页两侧 subtest total 相等（88/81/43 对 88/81/43）——无 phantom gain。JSON 独立复算（续开轮 verify_ab.py）与电池 RECONCILE 一致。

- +30：页面 wholesale 解除（local face 存活 → cssRules/style 面恢复）；残 58F 全形 `assert_not_equals ... got disallowed value ""` = FontFace 构造器 descriptor 语义域（weight keyword/number/calc、stretch、style 等回读空串）→ FONT-2 候选
- +2/+2：oblique 91deg/-91deg 拒收兑现（预判 2F 精确命中）；font-style-parsing 页 +2 为同界额外翻面（同根因，合法收益）

## css-animations -16 定谳 flake（收账门，续开轮 solo 探针闭合）

FONT-1b reconcile 曾计 `Element-getAnimations.tentative.html` P 17→1（-16）。四象限矩阵定谳：

| 跑 | b0（face 无） | b1（face 有） |
|---|---|---|
| 电池 13:49-14:30 | 17/21 | 1/21 |
| solo 17:23-17:29 | **1/21** | **17/21** |

两格互换、两侧二进制同代（solo 复用电池 bin 拷贝件，md5 未变）——同一二进制两侧均现 17↔1 双向翻转，页面本身高 run-to-run 方差（Timeout 条目两侧均有、b1 失败形含 `Cannot read properties of undefined (reading 'playState')` 整页崩在 b0 solo 同现）。face delta 无因果路径（页面无 @font-face/font-style，grep 零命中）。按 flake 域规则双向皆不入账：**lost=0 regressed=0 维持，-16 摘除**。

## 挂起/移交

- FONT-1 的 event-dispatch +2 挂起：本电池 reconcile 未再现（该页两侧一致未出现在 diff）——挂起项独立维持原窗规则，不因本电池消失而转正
- font-stretch calc(1+10%) 混纲拒收：calc 类型代数刀（apply.rs:2985 math 一刀放行点）
- FONT-2 候选：FontFace descriptor 语义核（at-font-face-descriptors 残 58F + 构造器面）
