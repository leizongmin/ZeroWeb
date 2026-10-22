# M2 切片 31 — dialog `:modal` 状态通道与 UA 定位注入实验定性（2026-10-23）

## 范围

dialog showModal 布局域（abspos-dialog-layout ~10F 与
ancestor-is-inert/dialog-inert 同根假设）收口第一刀：定位「UA 定位规则
缺失」红因、落地 `:modal` 运行时状态通道、用注入实验矩阵给布局缺口
定量归因。

## 谜题破案：gBCR 零盒是探针伪象

- s30 遗留谜题「dialog[open] computed display=block 但 gBCR 全零」**不
  成立**：`getBoundingClientRect` 不是同步布局查询——读 renderer 上一帧
  render 后填充的 rect 快照（`rect_bridge.rs fill_layout_rect_snapshot`），
  `setAttribute('open')` 同 tick 读拿到 open 前旧布局（display:none 无盒
  → 全零）。等帧（setTimeout 120ms）后四定位变体全部有盒。
- runner 探测环刷新顺序（take_probe：flush → fire timers → mutation 门
  控刷新）钉死 stale 窗口：turn K 测试体读 gBCR = turn K-1 末快照，
  showModal 同 turn 测量必 stale 一拍。abspos-dialog-layout 9 案中 6 案
  红因 = **双侧 stale**（expected 300 = offsetHeight stale 0；got 0 =
  gBCR stale 顶），UA 规则修复不解决。
- **挂账 A（本流，engine/webview）**：强制同步布局冲刷——布局查询 API
  （gBCR/offset*）读前冲刷 pending mutation + 重排（CSSOM view 规范语
  义）。abspos-dialog-layout / non-modal-dialog-layout 的同 turn 断言面
  等此切片。

## `:modal` 状态通道（本切片落地）

模态性只在 shim JS 栈（`_zwDialogModal`），DOM 序列化不携带 → render
每帧 fresh-parse 的新 Document 不可见。镜像 `:focus` 注入通道（R4241
`inject_focus_state`）建全链：

1. **dom**：`Document.modal_dialogs` + `set_modal_dialogs` +
   `modal.rs is_modal_dialog`（镜像 focus.rs 拆分模式）。
2. **pipeline**：`modal_dialog_selectors` + `set_modal_dialog_selectors`
   + `inject_modal_state`（parse 后注入，两个 render 路径同插）。
3. **webview**：`set_modal_dialog_selectors` 转发（doc(hidden)，镜像
   set_focused_selector）。
4. **runner**：take_probe 泵后轮询 shim `_zwDialogModalAllSels()` →
   管线注入（先于 mutation 刷新——刷新的 render 带上 `:modal` 态）。
5. **shim part01**：`_zwDialogModalAllSels()` 全量模态 selector 导出
   （栈序、stale 栈项跳过，嵌套模态全量——`:modal` 语义为 top layer
   模态皆匹配，区别于 s30 的仅栈顶 `_zwDialogModalTopSel`）。
6. **style-system**：matcher `:modal` 臂（委派 `is_modal_dialog`）+
   SUPPORTED_SIMPLE_PSEUDO_CLASSES（css-parser 已识别 `:modal`，matcher
   原 `_ => false`）。

作者样式 `dialog:modal { ... }` 自此真实匹配（render 路径；getComputedStyle
路径 `compute_document_styles` fresh-parse 未接注入——挂账 D，验收面不
依赖）。DOM `querySelector(':modal')` 维持静态不匹配（与 `:focus` 同款
两阶段模型既有语义）。

## UA 定位注入实验矩阵（回归 0 纪律下的定性）

规范 UA（rendering.html#the-dialog-element-2）：`dialog { position:
absolute; inset-inline: 0; margin: auto }` + `dialog:modal { position:
fixed; inset-block: 0 }`。三种形态全量集群实测（dialog 集群 82 files
407 subtest，逐 subtest diff s30 账本 320P/85F/2T）：

| 形态 | 结果 | 机制 |
|------|------|------|
| A 规范照抄（inset 0 + margin auto） | **9 回归** 3 增益 | 布局对「inset 指定 + size auto」走满视口**拉伸**（CSS2 §10.3.7 应
shrink-to-fit），dialog 800×600 覆盖全屏 → light-dismiss「盒外点击」全族失效（closedby 7 +
toggle-source + blocks-mouse-events 打回） |
| D inset auto 近似（静态位 + 收缩盒） | **3 回归** 3 增益 | dialog 出流、兄弟上滑重叠静态位盒 → selection/editing 点击几何漂移（inert-node-is-uneditable /
modal-dialog-selection ×2 / simulated-click-inert） |
| C 不注入（本切片终态） | **0 回归 0 增益**（320P/85F/2T 与 s30 打平） | 惰性通道零行为差 |

增益侧（A/D 共有）：ancestor-is-inert 翻绿（dialog 盒不再含点击点 →
门排除 → backdrop 扫描命中 html）、closedby-bounds-clicking 盒内不关
2 案——均依赖「dialog 不挡在内容区」，Chrome 几何（fixed 居中小盒）
同理满足；C 形态随注入一并撤回。

**结论**：UA 定位规则的 Chrome 几何（居中小盒）依赖两个布局能力——
abspos 指定偏移 shrink-to-fit、fit-content 求值（inset 语境现状 8×0 碎
盒）。**挂账 B（渲染流，layout-engine）**：abspos 收缩/fit-content 语
义补齐，UA 规则随该切片按规范回填（本切片的通道与注入架构直接复用，
A 形态实验已端到端验证 runner 轮询链：探针 modal 臂几何生效即该链产
出）。s30 证据「ancestor-is-inert 红因 = UA position:absolute 缺失」修
正为「注入受布局收缩缺口阻断」。

## 账本

- dialog 集群 82 files **320P/85F/2T**（数据
  [2026-10-23-m2s31-dialog-modal-channel.json](2026-10-23-m2s31-dialog-modal-channel.json)），
  逐 subtest 对 s30 同集 diff：同 key 回归 **0**、增益 **0**（账本逐位
  打平）。
- gates：zero-dom lib **863P/0F**、zero-engine lib **3428P/0F**（+1
  `test_modal_pseudo_injected_state_matches_author_rule`）、
  zero-style-system lib **2446P/0F**（+1
  `test_modal_pseudo_document_state`）；clippy `-D warnings` clean、fmt
  clean；零新增依赖。渲染/UA 样式净变化为零（lib.rs 仅注释），免
  product-smoke。
- 零盒探针语料（zz-probe-s31/s31b）已删，集群干净复跑。

## 挂账（本轮钉定）

- **A 强制同步布局冲刷**（本流）：gBCR/offset* 读前冲刷 + 重排；
  abspos-dialog-layout 6 案、non-modal-dialog-layout 同 turn 断言面的
  共同前置。
- **B abspos 收缩/fit-content**（渲染流）：UA 定位规则回填前置；
  ancestor-is-inert / dialog-inert / light-dismiss 居中几何依赖。
- **D getComputedStyle 路径 :modal 注入**（本流，低优）：`compute_document_styles`
  代际缓存接模态态；验收面不依赖。
- selection 拖选→Selection 链路、`:active` 交互态伪类（s30 挂账维持）。
