# M2 切片 377（s377）——dialog-overlay 2F 施工翻绿（overlay 属性全链 + modal gCS 通道双根因修复）+ select-dialog-mode-focus 2F 定性

日期：2026-11-10（承 s376 挂账：流内剩余候选 select-dialog-mode-focus 2F / dialog-overlay 2F）
域：html/semantics/interactive-elements/the-dialog-element（dialog-overlay）
性质：**施工轮——dialog-overlay 2F 翻绿（interactive-elements 真案 18F→16F）+ select-dialog-mode-focus 2F 定性 customizable-select A2 渲染流挂账簇 + modal gCS 通道 stale/串台双根因修复**

## 案一勘根定性：select-dialog-mode-focus 2F（customizable select dialog mode）

**案面**：`forms/the-select-element/customizable-select/select-dialog-mode-focus.optional.html`（不在 interactive-elements 域内——s376 挂账口径为全语义面），dialog mode 键盘焦点链 2F。

**三证定性零施工**：customizable select dialog mode 特性未建面——①`parse_appearance` 名单无 `base-select`（css-parser parse_extended_visual.rs）；②`::picker(select)` 仅选择器解析放行（R5359 判据面），无 open 态渲染面；③part06 select 键盘面只有经典 closed-select 语义（ArrowUp/Down/Home/End/type-ahead），且 `is_open_state_element` 名单无 select；dom document/mod.rs 架构注释「待渲染流 A2 appearance:base-select 交付后镜像」。**改挂 customizable-select A2 渲染流挂账簇**（域 225 文件，与 select picker open 态注入通道同簇）。

## 案二施工：dialog-overlay 2F 翻绿

**案面**：`dialog-overlay.html` 2 子测——modal dialog 打开时 computed `overlay` 应为 `auto`（top layer 成员），关闭回落 `none`。

**勘根**：CSS Position 4 §overlay 属性（none|auto 两关键字、非继承、initial none）全链缺席 + UA 注入缺席。UA 规则 `dialog:modal { overlay: auto }`（Chromium UA sheet 同形）；`::backdrop` 不注入（恒 none）。

**施工（overlay 属性八处全链，isolation 属性 R5482 同模板）**：
1. css-parser `values/parse_layout.rs`：`OverlayValue` 枚举 + `parse_overlay`（none|auto 大小写归一；parse_basic.rs 重复定义死文件不在 values/mod.rs 列表，生效 parse_layout 版本）
2. style-system `types.rs`：镜像枚举 + `PropertyValue::Overlay` 变体
3. `computed_style.rs`：`overlay` 字段；`default_impl.rs`：initial `None`
4. `registry.rs`：initial_value 臂 + known_properties 名单；`inherit.rs`：reset 臂（非继承免 inherited 臂）
5. `apply_advanced.rs`：apply 臂（parse→映射 return true）
6. engine `computed_style.rs` serialize 输出臂（none/auto 关键字）
7. style-system `lib.rs` s373 块：UA 注入——modal 集 dialog push `("overlay","auto")`（`::backdrop` 不注入由 `pseudo.is_none()` 门保证）
8. shim `part01.js`：gCS 入口 `isBackdropPseudo` 判定——`::backdrop` + `overlay` 查询短路返回 `none`（此前 ::backdrop 通道被忽略回落本体 FIXME 近似；其余属性回落本体维持近似+null 参零漂移）
9. 钉测 `style-system/tests/ua_dialog_overlay.rs`：镜像 ua_dialog_visibility 全量通道两态断言（空 sheets overlay None / set_modal_dialogs 含 dialog → Auto）

## 过程双根因（施工中暴露+修复）

**根因一：modal 态 styles stale**。shim showModal/close 的 open 属性 SetAttr 非 style 类 mutation → gCS 增量换代 changed=[] 零重算 → `apply_runner_modal_state` 注入后 styles 停在注入前旧态。此前 visibility-hidden 案绿是恰有作者 style mutation 驱动 changed={dialog} 重算的偶然覆盖。**修复**：dom `Document::modal_dialogs()` getter + `apply_runner_modal_state` 内比对新旧 modal 集，变化时定向重算（旧∪新 modal 集∪`collect_descendants` 后代；shadow scopes 在场走 `compute_styles_with_shadow` 全量兜底）。

**根因二：全局槽并行串台**。`RUNNER_MODAL_SELECTORS` 全局 RwLock 在 rayon 8-job 并行下跨 job 串台——旧代码串台不显形（注入不重算 styles），根因一修复后重算使串台 modal 态直接写进他案 styles（首轮 sweep：dialog-overlay 子测 1 初始 got auto、visibility-hidden 非模态 got visible）。**修复**：thread_local 化（`RefCell<Option<Arc<Vec<String>>>>`）——webview `execute_script` 直呼同线程 &mut self，宿主回调 JS 栈内同步触发，模态态天然线程局部；shim reset 推送 `__zw_set_modal_state('')` 清槽覆盖同线程跨案残留。

**验证坑**：探针 visibility 对照断言假绿——无 hidden 父链页面 visibility 恒 visible===visible 不能证明注入执行；改用作者样式 cascade 探针 + style-system 钉测（全量通道）分离定谳。

## 跨流回归归因（closedby 族 19 行 + auto-expand 1 行，非本刀）

**现象**：本刀干净 sweep 446P/37 Fail 行 vs s375b 基线 464P/22 行——join 变红 20 真案行（dialog-popover-closedby-complex 16 + closedby-simple 3 + auto-expand-details-element-fragment 1），翻绿 5 行（dialog-overlay 2 + zz_ld_probe_s372 探针 3 行因文件已删消失）。

**归因链三步定谳**：
1. s375b 日志 4 文件全 Pass（closedby-simple 6P、closedby-complex 40P、auto-expand 1P）——翻红窗口 = s375b→s377；
2. stash 本刀后于 fe4c789ea 组合态（无本刀代码）rebuild release runner，solo closedby-complex 同样 16F——**非本刀回归实证**；
3. 窗口内唯一代码变更 = 1bcfaac6d（渲染流 R5491 layout-engine inline fill 上溯；s376 为纯文档提交；closedby-simple 3 行 + auto-expand 1 行与 closedby-complex 同窗口同案族，closedby-complex 已实证）。

**定谳**：20 行归渲染流 1bcfaac6d 波及面（closedby 族 click 分类依赖盒几何）。按 run-rules #9/#10 记账不修，归因写入本流控制面，渲染流会话经由飞书通知可见。s376 轮只 solo 未跑全域 sweep 故该漂移未被及时暴露（教训：全域口径基线 join 是渲染流大改动后本流的必要验收）。

## 域口径变化

- interactive-elements 真案：**18F→16F**（dialog-overlay 2F 翻绿；R-s31 簇 13 + showModal 1 + form-submission 1 + corner-cases 1 上游单列）
- 跨流既有面 20 行单列（不读本流可施工口径）；closed-details 出域 1 行维持
- 域 Fail 行 22→37（+20 跨流 −2 翻绿 −3 探针删）；sweep 464P→446P（−18=变红 20−翻绿 2）
- 全语义面挂账口径：select-dialog-mode-focus 2F 移入 customizable-select A2 渲染流簇

## 挂账（移交下一切片）

1. **R-s31 UA dialog 定位簇**（渲染流）：维持（abspos 6F、bounds 2F、multiple-centered、non-modal、scroll-height、top-layer-position、ancestor-is-inert、showModal top-most 嫌疑）。
2. **customizable-select A2 渲染流簇**：扩容 select-dialog-mode-focus 2F（+select picker open 态注入通道 39 涉案文件）。
3. **上游已知失败面**（单列）：corner-cases 1F 维持。
4. **跨流回归面**（渲染流 1bcfaac6d 归因）：closedby-complex 16F + closedby-simple 3F + auto-expand 1F——渲染流控制面处理，本流后续 sweep 口径将其单列。
5. 流内剩余：form-submission 亚像素坐标、closed-details-layout-apis（content-visibility 渲染流）、inert hit-testing 穿透近似面、基建面 T=3+probe8/9/10 转正决策承继。

## 验证

- dialog-overlay solo 2/2 全绿（solo → 并行串台修复 → thread_local 后 solo 复跑维持）
- 干净口径 sweep 446P/37 Fail 行（跨流 20 行单列归因非本刀；本流真案 18F→16F 零新增红零改名）
- style-system lib 2505P/0F（含新钉测 ua_dialog_overlay）；dom lib 903P/0F（getter 面）；engine lib 3826P/7F 逐名同构 s375b 基线集零新增红
- shim 22 part 拼装 node --check PASS；fmt 四 crate PASS；clippy 四 crate PASS
- 渲染面零变化论证：overlay 纯标记属性不参与布局/绘制，modal 重算仅 runner-only gCS 通道（runner 独占路径），product-smoke 不适用

日志 /tmp/s377/（本地：engine_lib.log、lib_style.log、lib_dom.log、lib_engine2.log、sweep_dialog.log 串台轮、sweep_dialog2.log thread_local 轮、sweep_dialog3.log 干净轮、f375b.txt/f377.txt join 集）。
