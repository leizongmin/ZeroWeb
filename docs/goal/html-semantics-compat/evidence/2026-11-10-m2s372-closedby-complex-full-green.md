# M2 切片 372（s372）——modal light-dismiss 四根因修复（closedby-complex 9F→0F 整文件翻绿）

日期：2026-11-10（承 s371 挂账首位：残余 9F modal light-dismiss 分类面）
域：html/semantics/interactive-elements/the-dialog-element（dialog-popover-closedby-complex）
性质：**9F→0F（40P/0F，s173 LOST 基线首次整文件翻绿；normal 段 20 case 全程保持）**

## 承接与勘根（s371 残余 9F = over-dismiss 4 + under-dismiss 5）

**二分定谳**：探针页手动 light-dismiss（绕过指针管线直调 check/up）3/3 终态正确 = 分类机器（zwsh 解析/flat 包含/popover pass 反走/dialog pass R5263 两腿）无恙；真实 clickOn 3/3 红 = 投递链错。勘根收敛到「点击几何 → 命中目标 → 分类输入」链。

**探针证据链**（zz_ld_probe_s372.html v1→v3，gitignore 语料区，勘根毕保留）：

1. **v2 钩子捕获**：`globalThis.__zw_dialog_light_dismiss_check/up` 替换 logging wrapper——td_debug `[ptr-resolve] out=` 打印 host2 裸串、down 钩实收 `zwsh50:...host2 三段串...|div.popoverA`。**坑：`[ptr-resolve]` 日志（testharness.rs:11601）打印的是 s176 element_from_point 覆盖臂（11623-11629）之前的 best-scan 结果，非最终投递串**——覆盖臂对 zwsh 前缀 engine 命中直接换 hit。
2. **v3 normal/shadow 双段同页**：normal 段 clickOn(dialogA) 投递 `dialog.dialogA` @ (400,109) 终态正确（该段全绿根因 = engine 的 dialog 盒高只含文本不含 light 子元素（[100,118] h=18），popoverA in-flow 盒 [118,136] 恰不与其中心相交——**偶然几何**）；shadow 段 slot 摊平内容计入盒高（dialogA h=36 [100,136]），中心 (45,118) 落 popoverA 盒内 → under-dismiss。同构嵌套两段几何分叉排除「UA 规则既有」假说：style-system 全文 grep 无任何 popover 处理。
3. **进程内 layout dump 钉**（s372_popover_ua_fixed_tests 首版）：UA fixed 注入后坐标全对（y 相对父链叠加 = 绝对 100/150/200/250）、**含元素子的 fixed 盒宽全塌 0**（叶盒 popoverB 纯文本 62/74 正常）→ 定位 R3929 补测 pass 门缺口。

## 四根因定谳

| 根因 | 面 | 形态 |
|---|---|---|
| E | `[popover]` 无 UA `position: fixed`（style-system 零 popover 条目）→ in-flow 布局 | shadow 段 popover 盒叠 dialog 中心 → elementFromPoint 命中 popover → s176 覆盖臂采信 → light-dismiss target=popover 自含 break → under-dismiss 5 |
| F | `_zwDialogModalFocusGateOk`（s28）包含判定 `_zwDialogShadowInclContains` 沿 **light parentNode 链上行**——slotted light 子的 engine 父链锚宿主（host3→host2→body），永不经过住宿主 SR 内的模态 dialog | best-scan 全跳 → best=null → msel 兜底 → over-dismiss 4（Chromium inert/hit-test 用 flat tree，s330 close-the-dialog 同面） |
| G | R3929 `apply_abspos_shrink_to_fit_width` 门 `if !b.is_absolute … continue`——LayoutBox `is_fixed` 独立旗标，**fixed 盒全程无塌 0 补测** | UA fixed 注入暴露：含元素子 fixed branch 盒 taffy content_size=流内子 max（OOF 全剔→0）→ 宽塌 0 → click 中心落空（首轮 6 案 normal 新红即此） |
| H | backdrop-through 缺失——modal dialogB 的 ::backdrop 覆盖视口，几何命中位于 top-layer 序其下的元素（dialogA）时应 retarget 栈顶模态 | modalB 组合 clickOn(dialogA) 期望 `dA open + pA open（pA flat 含 dB）+ dB 关（backdrop 'out'）`，实得 pA 被隐 |

## 施工

1. **style-system/src/lib.rs**：`[popover]` UA decl 注入 `position: fixed`（UA 0,0,0 可被作者覆盖）。规范 UA 样式表全规则（inset:0/width/height fit-content/margin:auto/border/padding/overflow/color/background）仅注 position——R-s31 先例（dialog 定位注入实验回退：spec 形满视口拉伸 9 回归/inset auto 出流重叠 3 回归）约束居中语义随布局侧 abspos 收缩/fit-content 缺口一并挂账；display:none-when-closed 不在本刀（R3071 show/hide 为 shim JS 态无 display 通道，:popover-open cascade 面未勘）。
2. **part01.js `_zwDialogModalFocusGateOk`**：包含判定 flat 先行（`_zwLdFlatContains`，s176 slot 宿主改道已证）+ `_zwDialogShadowInclContains` 保底（s29 纯 shadow 内容候选——part04/part06 焦点门调用面不变）。
3. **layout-engine engine/sizing.rs R3929**：`fixed_width_rescue` 窄口扩展——fixed 仅入**宽侧塌 0** 补测臂（`!is_absolute && is_fixed && !is_replaced && width<=0.5`）；`>0.5` 扩臂（R5236 branch-text/R5445 D2）与 height_fix 显式限 `is_absolute` 域不动，replaced 不入（自有 R4015 臂）。abs 域行为逐条件比对不变。
4. **part01.js backdrop-through**：check/up 双腿 R5234 早退后——栈顶模态 M 存在且 `targetSel !== M sel` 且 T 不在 M flat 子树 → `target/targetSel` retarget 为 M；既有 backdropHit 盒内外分流随即生效（M 自身 'out'、flat 含 M 的 popover 保持）。两腿同 retarget 保 sameTarget 同态比对一致。
5. **钉测**：layout-engine `s372_popover_ua_fixed_tests`（语料 normal 段最小复刻：全盒 shrink-to-fit 非 0 + 单行高 + 叶盒文本测量锚 74±3）。

## 验证

- **solo 域跑**：`make testharness-html-semantics FILTER=dialog-popover-closedby-complex` → 修复①+② 32P/8F（shadow 7 翻绿但 normal 6 新红=G 首曝）→ +③ 38P/2F（残 H 形态）→ **+④ 40P/0F**。日志 /tmp/s372/solo-fixed3.log（本地）。
- **A/B 名单级 diff**（HEAD 基线 vs 修复树，FILTER=popover / FILTER=dialog 两域 sweep）：popover **19F→10F**、dialog **34F→25F**（头基线剔除探针自身 3 永红）——diff 纯移除 9 案 closedby-complex（两 FILTER 各捕同文件），**零新增红零改名**。
- **layout 钉测**：`s372_popover_ua_fixed_geometry` 绿（宽塌 0 修复实证：0/0/0/62→289/220/142/74，高 36 折行→18 单行；叶盒锚 74）。
- **engine lib 全量**：见下（sizing 扩展波及全 fixed 塌 0 盒，最高风险面）。
- **webview lib 全量**：见下。
- fmt / clippy：见下。

## 挂账（移交下一切片）

1. **UA popover 全规则**（inset:0/margin:auto/fit-content 居中 + border/padding/color/background + `display:none-when-closed`）随 R-s31 布局缺口挂账渲染流；display:none 需先勘 `:popover-open` cascade 通道与 show/hide display 联动。
2. **塌 0 补测 text walk 计入 OOF 后代文本**（dialogA 测 289 应 ~66 自身文本）——R5300 补测链既有语义（R5444 仅 scoped table 臂收窄），盒垂直不相交不影响分类面；语义修正随渲染流。
3. 既有在册维持：基建面 T=3（s102/s124）、probe8/9/10 转正决策、customizable-select 126F / dialog 域 25F（本轮 sweep 口径，HEAD 同名集）既有未建面。
4. 探针 zz_ld_probe_s372.html 在 gitignore 语料区保留（勘根证据载体，不入库）。
