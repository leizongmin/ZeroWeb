# M2 切片 30 — 模态 hit-test/事件派发 inert 排除与 backdrop 命中面（2026-10-23）

## 范围

rendering inert 域第一刀：hit-test 切面。宿主指针命中（runner
resolve_pointer_target 的候选循环）不排除模态 inert 子树——点击模态外
元素照常命中（真浏览器 hit-test 跳过 inert 内容、焦点与事件不出
top-layer），modal-dialog-blocks-mouse-events /
modal-dialog-ancestor-is-inert 失败面。

## 修复（runner 命中谓词 + shim 栈查询/light-dismiss backdrop 臂）

1. **命中候选循环接 inert 排除**（`resolve_pointer_target` 内嵌 JS）：
   面积最小含点候选跳过 `!_zwDialogModalFocusGateOk(e)` 的元素——s29
   门语义（栈顶 open+模态 dialog 的 shadow-including 子树外）与
   hit-test inert 排除完全同源，直接复用；`_zwDialogModalFocusGateOk`
   补 globalThis 导出（页面域可调，shim 内部消费者走词法绑定不变）。
   门缺失（旧 shim）回落原行为。
2. **backdrop 命中回落**（shim `part01 _zwDialogModalTopSel` 新增）：
   候选全被排除（或无候选）且栈顶模态存在 → 事件 target = 栈顶模态
   dialog——::backdrop 几何覆盖视口、命中 backdrop 的派发 target 即其
   dialog；引擎 backdrop 无布局盒，以栈顶模态近似，仅模态存在时生效。
   真 Chrome 仲裁（playwright + chromium-1243，ancestor 语料独立探针）
   实测「点击 inert 区域 target=HTML」——backdrop 被 `display:none`
   时不渲染、点击穿透到根：命中回落读 styleSheets 文本扫
   `::backdrop{...display:none}`，命中则 target=documentElement。
3. **light dismiss backdrop 臂**（`__zw_dialog_light_dismiss_check` 加
   px/py）：backdrop 回落 target=dialog 会被「包含链内」判定压制
   dismiss——真 Chrome 点击 backdrop 照发 close request（s28 实测
   md:dialog 且关闭）。target=栈顶模态 dialog 本身且落点在其 gBCR 外 =
   点击 backdrop → 对 any/closerequest 态 dialog 发 close request；盒内
   （dialog 内容区点击）走常规包含链。px/py 缺省（click 命令前步）保持
   原判定。

## 归因辅助（本轮钉定的相邻事实）

- **modal-dialog-ancestor-is-inert 仍红，红因已换**：引擎 dialog
  showModal 后 UA `position:absolute` 缺失（探针 pos=static，dialog 盒
  (101,201,50x50) 落进父 div 内容区，点击点恰在盒内 → best=dialog 合法
  命中；真 Chrome 盒 (200,200,88x88) 不含点击点 → backdrop → html）。
  归 abspos/layout 挂账（abspos-dialog-layout「showModal 居中」同根）；
  本轮命中面修复为该案铺平（div 已被门正确排除）。
- **inert-svg-hittest 两案 `:active` 断言引擎侧恒 false**：matcher 运行
  时伪类臂无 active/hover（`_ => false`，selector() 名单仅 UA 识别面）。
  案 1 Pass 属假象（assert_false 蹭过）。归交互态伪类账。
- **selection 两案**（inert-node-is-unselectable /
  modal-dialog-selection）：依赖指针拖选产生选区（headless 无真文本选
  择渲染），getSelection 基建在（R3254）但拖选→Selection 链路缺。挂账。
- **dialog-inert**：首断言依赖模态 dialog 居中布局（同 abspos 账）。

## 账本（s29 → 修复树）

dialog 集群 82 files 407 subtest（数据见
[2026-10-23-m2s30-modal-hit-test.json](2026-10-23-m2s30-modal-hit-test.json)）：

- **320P/85F/2T** vs s29 同集 317P/87F/3T。
- 逐 subtest 对 s29 账本 diff：同 key 回归 **0**，同 key 增益 **2**
 （modal-dialog-blocks-mouse-events F→P；dialog-light-dismiss-touch
  F→P 连带）；backdrop-receives-element-events **Timeout→Pass**
 （3T 基线之一收口，backdrop 命中面直接语料；Timeout 案 subtest 名为
  全路径形态、Pass 后为短名，key 形态漂移非同 key 比对项）。
- corner-cases **29P/1F** 维持（唯一 F 为 s28 定性 Chrome 同款
  esc-modeless）；light dismiss 家族（closedby-bounds-clicking 等）
  无回归。
- engine lib 全量 **3427P/0F**；clippy `-D warnings`、fmt clean；零新
  增依赖。常驻回归面 = 语料本体（blocks-mouse-events /
  ancestor-is-inert / corner-cases），无新增 Rust 契约（命中谓词在
  runner 内嵌 JS，Rust 侧无独立单元面）。
