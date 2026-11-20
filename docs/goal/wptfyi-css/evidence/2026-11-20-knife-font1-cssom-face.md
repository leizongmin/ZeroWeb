# FONT-1 CSSOM 对象面电池证据（2026-11-20 fork=R3-LAUNCH）

face 落库 `74f08526e`（css_wire.rs @font-face wire 臂 + wire pin 钉测同提交；
part06.js descriptor 文法核/CSSFontFaceRule 物化/FontFace accessor throw/src
format 校验）。guard：lei-pre-commit-guard 隔离子 Agent 扫描 PASS（2 文件
339 行，0 发现 0 提醒）。

## 刀面

1. css_wire.rs `style_rules_text` wire 循环加 `Rule::FontFace` 臂——`@font-face`
   此前随 @-rule defer，`sheet.cssRules[0]` undefined → at-font-face-descriptors
   全页 `.style` 崩（88F wholesale 形）。selectorText 字段 = `@font-face` 标记，
   shim 侧据以物化 CSSFontFaceRule（cssom-1 CSSFontFaceRule type 5）。
2. part06.js getRules 循环加 @font-face 臂（@font-feature-values 臂后）。
3. part06.js descriptor 文法核 `_zwParseFontFaceDesc`：weight/stretch range 二
   token + style oblique ±90 字面拒/0deg→normal/等界折叠（含 20deg
   former-default 保形例外）+ calc shunting-yard `_zwCalcEval` 单位跟踪。
4. part06.js CSSFontFaceRule 全局 + `_makeFontFaceStyle`（getter/setter 静默拒
   + '' 清除）+ `_makeFontFaceRule`（family 提取 + style face）。
5. part06.js FontFace `_desc` 槽 + prototype weight/style/stretch accessor
   （throw DOMException SyntaxError）+ src format() 8 关键词校验（load()
   reject SyntaxError）+ isItalic 改 oblique 前缀判。
6. 离线钉面：grammar 140/140 + style-face 16/16 + 全 shim 拼接语法 OK
   （harness 在 /tmp，不入库）。

## 电池

- 配方 `knife-font1-battery.sh`（knife-sha1-battery.sh 逐字配方；FACE_FILES=
  css_wire.rs+part06.js；FACE-MARKER=`_zwParseFontFaceDesc` ASCII 判别；
  corpus=fonts/masking/shapes/transitions/animations/properties-values-api/
  typed-om 七份 × b0/b1 双臂 14 腿，CASE_WALL_S=60）。
- census：ride 14 文件 S1/S2/S3 全零扰动；FACE-MARKER b0=0 b1=5；M2 过
  （b0 `420573c5de5dcdae22b6b9e6b7e3c863` / b1 `238d288082364fe1acb0be66930c00e4`）。
- 时间线：一发 12:17 unit 自拒（脚本 systemctl 自查匹配自身 unit 名——flock
  已兜底，删自查重发）；二发 12:18 FACE-MARKER 门误杀（bash 惯用法
  `grep -c || echo 0` 零匹配时输出 "0\n0"，整数比较炸——面无损，M2 与标记
  实证全对，修惯用法重发）；三发 12:21:49 全程过，legs 12:22-12:58，
  GATE-DONE 12:58:58 rc=0。

## A/B 账

```
css/css-fonts/format-specifiers-variations.html:                       P 8->48 (+40)
css/css-fonts/variations/font-parse-numeric-stretch-style-weight.html: P 49->78 (+29)
css/css-animations/event-dispatch.tentative.html:                      P 0->2  (+2)〔挂起〕
== TOTAL: gained=71 lost=0 regressed=0 pages_gain=3 pages_loss=0
```

- format-specifiers +40：两侧子测试总数 48 恒等，全 F2P 真翻（src format
  校验面全兑现）。
- font-parse-numeric +29：两侧总数 81=81 恒等真翻。残 3F（FONT-1b shim 补门）：
  ① `calc(1 + 10%)` stretch 应拒未拒——`_zwCalcEval` +- 混纲空档（无量纲+%
  类型不兼容未判）；
  ②③ `oblique -91deg` / `oblique 91deg` 应拒未拒——F3 构造器路径走
  `_zwFfDescStore` 宽容归一，未过 `_zwParseFontFaceDesc` ±90 字面门（F1
  style face 有门、构造器漏装——单源化修）。
- **event-dispatch.tentative +2 挂起不入账**：b0 子测试总数 13 ≠ b1 总数 19
  （report-completeness-asymmetry 纪律——gained 页两侧断言数不等即挂起）。
  页面无 FontFace 引用（grep 空），差集指向异步 setup 装载深度漂移
  （animationstart 等待型页）。FONT-1b 电池随 css-animations 哨兵复采，
  再现不对称即 solo 长墙定谳。
- **净收账 = +69**（71 − 挂起 2）。

## 主靶缺页与 FONT-1b 认领

at-font-face-descriptors.html 88F 未翻（b0/b1 恒 88/88F，`cssRules[0]`
undefined）。根因在 css-parser：`consume_font_face_rule`（at_rules.rs src 臂
`extract_urls_from_src` 只认 `url()` 形 + `family.is_empty() ||
sources.is_empty()` 整条丢弃门）——`src: local('Courier New'), local('Courier')`
sources 空 → face 丢弃（mod.rs font-face 臂 None 分支）。css-fonts-4 §src-desc
local() 为合法源形；该页静态 `<style>` 即 local-only src。FONT-1b 认领：
parser src local() 源臂（sources 存 `local(…)` 原样形）+ css_wire 串化 local
分支 + shim 残面三门（calc 混纲 + 构造器 oblique ±90 单源化严格核）。

## 过程注记

- 发射窗：SH-A1 电池 GATE 11:35:59 后机器空转 40min（rally-turn 11:49
  css_wire.rs 断括号接管预告因 11:45 括号修 + cargo check PASS 自动作废），
  12:17 发射；其时 SH-A1 face 未提交、rally-turn b1' 无在跑痕迹。
- rebase 撞树（后发事件，账不受染）：SH-A1 lane 12:5x 提交 face
  （e791760c5→rebase 后 `5fd4bda01` push）；rally-cont 12:59（本刀 GATE 后
  1min，等待合规）stash peer WIP（含本刀 face 两文件，stash@{0}）并 rebase
  onto d516f4e29——bins 冻结于 build 窗、S1/S3 census 后树才被动。face 恢复
  双源：stash pop（rally-cont 序列 13:09 完成）与电池 patch
  `knife-font1-face.patch`（12:21 落盘，4 hunk，`git apply --check -R`
  树态字节级等价复核）。
- 提交链：859db7316（rebase 前）→ `74f08526e`（onto 94b31d51d 重放落库），
  HEAD blob marker 核（wire 臂 1 / shim 文法核 5）前后一致；push rebase 显式
  stash `peers-wip during font1 push rebase (R3-LAUNCH)` + pop 干净（他流
  WIP 13 文件复原）。
