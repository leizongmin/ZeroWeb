# M2 s243 — dialog 拖选 caret 文本原点补面（text-align 内缩 + 级联字体回落 + user-select:none 域闸；s236 起四轮挂账销案）

日期：2026-11-02。切片号 s243（承 s242）。根：s236 起挂账的
modal-dialog-selection 1F（『By default, text inside a modal dialog can be
selected』expected "345678" got ""，pointer 拖拽面存量——s30/s48/s57 三轮
归因「渲染流面/深车道」挂起）。

## 入场定刀位

s242 下一步候选：modal-dialog-selection 勘根轮（pointer 拖拽面存量 1F，
先勘根定拖拽语义缺口归因）。

## 勘根（四轮探针 zz-s243-probe.html 验后删；单行化教训在账）

- **既有机器盘点**：拖选双腿**已存在**（selection-compat M2 拖选切片，非本轮
  新建）——down 腿 = mousedown 默认动作 caret 落点（part06
  `__zw_pointer_down_sequence` 尾臂，`_zwCaretFromPointDoc(x,y)` →
  `_getSelection().collapse`），move 腿 = 拖拽中 mousemove 默认动作选区
  extend（part06 `__zw_pointer_move` 尾臂，`st.buttons & 1` 门）。runner 侧
  Actions 指针步注入（uievents M3）逐步重放 pointer_move/down/up 宿主命令。
- **probe1 分层**：runner 宿主几何解析 dialog@370,13（事件 target=dialog，
  origin 中心+offset 解析正确）；JS 侧 gBCR 测量在 promise_test 体测=全零
  （RAF 帧驱动布局未 settle）；选区 rc=1 anc=foc=#text@10（双腿都落末位）。
- **probe2 定谳 mut 序**：patch Selection 单例 collapse/extend 记参——
  `collapse(#text@10,10) | extend(#text@10,10)`：双腿 caret 都解析到 offset
  10 = 文本长度（clamp 末位）；efp/cftp370/430 全 null（elementFromPoint 对
  dialog 自身盒返 null = s57 已知族；caretPositionFromPoint 注册表域无此
  hit——注册表 0 基归一化坐标，与视口域无关，不动）。
- **probe3 拖时点地面真值**（pointerdown listener 内测）：dialog gBCR =
  **[8,8,784,10]、pos=static、margin=0px/0px、w=auto h=auto、ta=center**——
  **UA modal 居中 sheet（position:absolute/inset:0/margin:auto/fit-content）
  在 JS computed style/几何面整体缺位**（同族钉面：multiple-centered-dialogs
  expected 300 got 0、abspos-dialog-layout 6F、modal-dialog-in-visibility-
  hidden visibility:visible 缺——渲染流车道，s48/s57 挂账维持）。
- **根因链**（双层，均在 shim 拖选 caret 面）：盒=全宽块 [8..792]，100px
  文本经 text-align:center 居中在 350..450；`_zwCaretFromPointDoc` 的
  `lx = x - gr.left` **把文本起点当作盒左缘**（无视 text-align 内缩）→
  lx=362/422 → 全部字形中点 < lx → 双腿 clamp 末位 offset 10 → 选区
  (10,10) → toString ""。次层：`_zwTextElStyle` 只读 inline style，样式表
  `font: 10px/1 Ahem` 钉面在 default 字体度量下 advance 失真（内缩与字形
  路由全偏）。
- **规范面**：css-ui-4 §content-selection（test 链接锚）——文本默认可选、
  user-select:none 阻止选择；subtest 3 钉面 = user-select:none 子树拖选
  不建选区（s236 起该面 P 是「选区恒空」假绿，subtest 1 修复后必须补真门
  防 P→F——本轮一并入闸）。

## 桥力学

`_zwCaretFromPointDoc` 文本原点内缩：inline 字体空时读 computed
fontSize/fontFamily 合成最小形 "<size> <family>"（规避 computed shorthand
"10px / 10px Ahem" 行高段解析方差，probe4 实证双键可读）；textAlign ∈
{center,right,end} 时预量路由同滤同 geom 的文本总 advance，内缩 = center →
(盒宽-总advance)/2、right/end → 盒宽-总advance（负值钳 0），`lx = x -
gr.left - _taInset`；其余对齐值零内缩（左对齐面位移零=零回归）。probe5
复测：collapse@2 + extend@8 → toStr "345678" 与解析预期逐位吻合（Ahem
10px 每字 10px、盒左 8 + 内缩 342 → lx=20/80 → 字形中点计数 2/8）。

`_zwDocDragSelectAllowed` user-select 域闸：祖先链走查（既有 32 层帽）加
computed `userSelect === 'none'` → false（inline 与级联同覆盖——probe4
实证 inline set → computed "none"；读不到的宿主面按未禁用零回归）。双腿
共用此闸，单点改双达。

## 施工（一处文件两块，零 rust）

- part06 `_zwCaretFromPointDoc`：级联字体回落 + text-origin 内缩（s243 注记
  块 ×2：勘根地面真值 + css-ui-4 锚）。
- part06 `_zwDocDragSelectAllowed`：user-select:none 祖先域闸（s243 注记）。

## 验证

- node --check 全 shim concat（bridge include 序 18 片）干净；
  `make zero-wpt-runner-release` 重建 29.0s。
- 探针 probe5（验后删）：collapse@2/extend@8/toStr "345678" 三点全吻合。
- solo modal-dialog-selection **3P/0F**（目标键 1F→P，subtest 2/3 保持 P——
  subtest 3 经新 user-select 闸保持非假绿）。
- solo selection 族 **857P/0F/0T**（s237 基线 856P/1F——唯一 Fail 即目标键，
  销案后 +1P 零其他位移）；solo the-option-element **108P/0F/0T**（standing
  门零回归）。
- 全树 A/B（evidence/logs/s243-ab.log vs s242-ab.log）：见下节。

## 全树 A/B（evidence/logs/s243-ab.log vs s242-ab.log）

- (key,state) 对 sort -u join：8293/8293 对，join 全闭合；唯一翻转 =
  modal-dialog-selection ::『By default, text inside a modal dialog can be
  selected』**Fail→Pass**（恰目标键）；LOST 1 = 目标键 F 态、GAINED 1 = 目标
  键 P 态，一一对应；行级 7708P/507F/81T → 7709P/506F/81T（ΔP=+1、ΔF=−1、
  Timeout 81 零位移）——**第三个连续完美 A/B**（承 s241/s242）。
- 改动臂三处均无其他通道位移与机理一致：内缩仅 center/right 对齐文本生效
  （语料拖选/click caret 面绝大多数左对齐零内缩）、级联字体回落仅 inline 空
  时生效、user-select 闸仅 none 子树收窄。

## 残余挂账（本切不动）

- **UA modal dialog 居中/visibility sheet 缺位**（渲染流车道）：multiple-
  centered-dialogs 1F、abspos-dialog-layout 6F、modal-dialog-in-visibility-
  hidden 1F、modal-dialog-scroll-height 1F 同族——UA sheet
  position/inset/margin:auto/fit-content/visibility:visible 在 JS computed
  面/几何面整体缺位；s48/s57 挂账渲染流面维持，本轮探针地面真值
  （bcr=[8,8,784,10] pos=static）可供该车道直接引用。
- elementFromPoint 对 dialog 自身盒命中返 null（s57 已知族——本轮 walk
  fallback 已兜住拖选面，hit-test 缺口本身挂渲染流车道）。
- caretPositionFromPoint 注册表域与视口坐标域无关（0 基归一化坐标设计），
  本轮不动。
- s242 存量：snapshot option 邻接 IDL 面（selected/disabled/index/form）；
  s241 存量：同类直写臂未同步清单、`_zwOptionValue` 键派生不一致；s240
  存量：plain detached select.value undefined、option.text null→'' 字面差。

## 下一步候选

snapshot option 邻接 IDL 面（s242 新账，独立勘根轮）；img SVG 密度族
（naturalWidth 22F + current-pixel-density 13F，需独立勘根轮）；
customizable-select 177F（挂起待用户拍板）。
