# ⑫ 轮证据：③ 载体形/prop-carry（bg-merge + carry-clone + 残余门禁）

日期：2026-11-05　基线：fix10 五语料 + fix10-mt-part-1..10　运行目录：/tmp/qv-fix4

## 实现（crates/engine/src/js_dom_shim/part01d.js，+136/-9）

1. **Arm 1 bg-merge**（`_zwVaSetSelectionValue` nodes pass，:1599 面）：bg 命令
   （hilitecolor/backcolor）对全含样式载体**并值不外裹**——条件 = 元素节点 +
   `_zwVaSpecifiedValue(node, command) === null` + `_zwVaCarrierHasOwnStyling`
   （style ≥1 声明或 legacy font size/color/face 属性）+ 单成员 run（nodes 内
   无前/后兄弟成员）。Chrome 录形 `<font size=6>[foo]</font>` / xx-large span
   全选单成员并值到载体（hilitecolor 语料 :353-372）；裸 span 不并
   （backcolor L18 外裹形）；多成员 run 不并（:395-412 单外裹）。
2. **Arm 2 carry-clone**（`_zwVaPeelOne` end 侧，:1095 面）：移出片段包载体克隆
   （属性克隆 + style 归一），受 `_zwVaCarrierResidualStyling` 门控（见 3）。
   克隆经 clearValue 摘冲突 prop 后残余声明存活，forceValue 把新包裹收进克隆内
   （fontsize teal `[a]bc`/`a[b]c` 期望 bg-carry 形，语料 :843/:849）。
3. **残余门禁**（`_zwVaCarrierResidualStyling`，:1284）：载体去掉命令等价面
   （fontsize↔size/font-size、fontname↔face/font-family、forecolor↔color/color、
   bg↔background-color）后仍持其他样式信号才克隆；限五个 font 簇命令。
   **第一版无条件克隆产生 18 P2F**（fontsize +12/fontname +2/forecolor +4，
   fix12 门禁实证）：单等价面载体期望裸移并拢 `ob` 合一形（join family，
   `fo[o<font size=2>b]ar</font>baz` 期望 `fo<font size=4>ob</font>…`），
   克隆隔断 adjacency 后 wrap merge-only 不并拢。残余门禁恢复裸移路径后归零。
4. **join('; ')**（`_zwVaSetStyleProp`，:309）：手工合并序列化改 Chrome CSSOM
   `k:v; k2:v2` 形（style.setProperty 不回写 attr，zz-probe-nstyle2 实证）；
   单声明 join 无分隔不变，多声明形 ③ 并值载体需要分隔（hilitecolor :367）。

## 判读

- **探针 zz-probe-carrier.html**（6 目标 + 10 守卫 G1-G5/J1-J5）：r3 22 Pass/0 Fail
  （11 例，J1-J5 增补前）；r4 32 Pass/0 Fail（16 例全量，含 join 判据守卫，
  残门定稿版 gated 构建），exit=0。探针页随 4d35466dd `git add -f` 入库
  （run/ gitignored，11 zz-probe 先例）。
- **五语料 fix12b**（残门定稿版，12:16:33 起 systemd zw-fix12b-gate）：
  judge-fix12.py 对 fix10 基线 **F2P 5 / P2F 0 / newF 0 / lost 0**。
  - hilitecolor 4F→0F（F2P 4：`<font size=6>[foo]</font>` 与 xx-large span
    双模式全数命中——③ 主目标）。
  - fontsize 7F→6F（F2P 1：teal `a[b]c` sw=false compare，carry+wrap-in 形）。
  - fontname 3F→3F、forecolor 17F→17F、backcolor 0F→0F 零漂移。
- **multitest 10 片**（fix12b 全分母 10,092 行，judge-fix12b-mt.py 行级元组）：
  **F2P 7 / P2F 0 / newF 0 / lost 16**。逐片 fail 数对 fix10-mt-part 基线零漂移
  （30/24/34/24/50/49/59/47/14 全等）；片 10（?9001-last）84F vs 基线 92F，
  差额 = 尾部截断带伪影（`testharness page timeout … 1 test(s) unreported`
  Timeout 一枚，lost 16 行全落该尾测子测面——⑪ 审计既有截断带口径）。
  键级复核（判读后补）：真 P2F 0——行级判读唯一 P2F 键 `multitest.html?9001-last`
  为 base 缺席键（300s 杀页残行，post#1076），非真实测行；F2P 7 逐行位置对齐
  （base#730-768 `abc[]ef` caret 面）。
  F2P 7 = 额外红利：`abc[]ef` caret 面 backColor/fontName 组合 insertText
  载体形（Arm1 bg-merge 单成员 run 并值在 caret 场景的延伸命中，基线 7 红
  全数转绿）。
- enginelib：fix12b 腿死于 test-guard 自 exec 竞态 Error 127（本会话并发探针
  make 所致，非测试红）；fix12c 复跑 **3727P/5F 五红名逐一对齐 fix10 基线**
  （p56_adopted/p99s_custom_property/r3276/r2810/r2809），零新红。
- 期中运维事故（已恢复）：本会话 TEMP-DEBUG `if (true || …)` 曾落盘约 4 分钟，
  按片时间线核 fix12b 全部 10 片逐值对齐 fix10 基线（含 TEMP 窗内片 7-8），
  无污染证据，免复跑。TEMP 已撤销，终树仅残门定稿版（乙线并行仅在 part01c
  追加临时 trace 钩子后自撤 + part01d 两处纯格式化，语义零异）。

## 未决（下轮候补）

- **teal `[a]bc` 语料上下文谜团**：该行在语料 ?2001-last 恒红且 got 随版本
  变形（fix10 裸移 `<font size=5>a</font>` 无 carry；fix12b 空克隆
  `<span bg></span>` "a" 消失）；同 fixture+commands 的独立探针（含语料序
  6 行窗、无条件 clone 版）全绿。zz-bisect.html（真 runConformanceTest 页）
  5 行窗即复现红；zz-bisect-nocss/noflip 排除 reset.css 与 styleWithCSS=true
  页 hack。剩余差异面 = testharness 机制（post-exec `documentElement.
  cloneNode(true)` + detached 树 querySelector——探针注入同款 clone 时引擎
  抛 `querySelector("[contenteditable]") null`，克隆树缺 CE div）。该行基线
  即红，不阻塞本轮 F2P/P2F 判读；机制下轮单开。
- forecolor compare 7 行挂账（purple=样式表有效值、ghost br 机制，异质）不变。

## 独跑复核定谳（提交后补账，12:49-12:53）

multitest 片 10 独跑复核（fix12d-mt-9001-last.log，test-guard 包裹）：
**rows=1092 满额（首轮 1077 缺 15 行为瞬态）、Timeout=0、fails=85**——
首轮 `testharness page timeout` 楔死为瞬态（typing 簇 delete/forwarddelete
行楔死致整页 script timeout，后续 16 行被吞；typing 命令不经过本轮三处
改动路径）。行级元组（judge 对 fix10-mt-part-9001-last）：
**F2P 7 / P2F 0 / lost 0**。首轮多出的第 8 行 F2P（`abc[]` caret
forwarddelete/inserttext compare，typing 簇基线红）在复核回红，证一过性
翻转不计入。全分母 multitest 定谳 **F2P 7 / P2F 0 / newF 0 / lost 0**，
逐片与 fix10 基线零位移。pristine 已恢复（variant metas=10）。
