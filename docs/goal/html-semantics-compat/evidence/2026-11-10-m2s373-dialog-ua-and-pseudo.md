# M2 切片 373（s373）——dialog 语义面三施工（:open/:closed 伪类 + UA modal visibility/色 + elementFromPoint 近似；dialog 域真案 25F→22F）

日期：2026-11-10（承 s372 挂账：dialog 域 25F 既有未建面 + UA 规则簇首层）
域：html/semantics/interactive-elements/the-dialog-element（dialog-inert / modal-dialog-in-visibility-hidden / default-color）+ dom/style-system 基座 + runner gCS 通道
性质：**dialog 域真案 25F→22F（-3 全按预期翻绿：dialog-inert、modal-dialog-in-visibility-hidden、default-color 案1）**

## 三项施工

1. **施工② `:open`/`:closed` 伪类**（crates/dom/src/query.rs + crates/style-system/src/matcher/mod.rs）：`PseudoClass::Open/Closed` 枚举 + `parse_pseudo` 分支 + `matches_full` 求值 + `pub fn is_open_state_element` helper（style matcher 复用，语义=dialog 有 open 属性 / popover showing 态）+ matcher `PSEUDO_CLASSES` 名单与求值臂 + 钉测 `zz_s373_open_closed_pseudo_parse_and_eval`。规范：HTML `:open`（dialog 非模态 open 态 / popover showing 态）。select-dialog-mode-focus 的 `:open` 子断言经此静默通过（余 2F 为 focus 行为面，见挂账）。
2. **施工③ UA 注入**（crates/style-system/src/lib.rs `ua_decl_inputs`）：`dialog { color: CanvasText; background-color: Canvas }`（tag 判定，UA 0,0,0 作者可覆盖）+ `dialog:modal { visibility: visible }`（tag + `doc.is_modal_dialog` 双判定）。规范 UA 样式表 margin/border/padding 三项改盒几何不注入（R-s31 定位挂账面），色为纯绘制声明零几何影响。规范：https://html.spec.whatwg.org/multipage/rendering.html#the-dialog-element-2 。default-color 案1（"colors react to light mode" 面值断言）翻绿；案2（dark mode）挂账系统色 color-scheme 分支。
3. **施工① elementFromPoint 缺席臂 gBCR 近似**（crates/engine/src/js_dom_shim/part06.js）：`_zwEfpHitSel(x,y)` —— `querySelectorAll('body, body *')` 逆序遍历 + 面积严格最小（同面积文档序靠后者赢 = 更深/topmost）+ inert 祖先链屏蔽 `_zwEfpInertBlocked`（64 层 guard，R4600 模式）+ 无布局证据（hasLayout=false，rect_snapshot 由 render 管线写、同 turn 全零）→ 栈顶模态 dialog 视口命中兜底（s30 backdrop 几何先例）。挂接 elementFromPoint 缺席臂（image-map 处理后）。dialog-inert 翻绿。

## runner 模态态 gCS 通道（施工③支撑工程，勘根五轮）

**根因面**：runner gCS 走 js_dom_bridge 代际槽 Document（`with_cached_document_styles` 持久对象），render 走 pipeline fresh-parse 每帧新 Document——`inject_modal_state` 只喂后者，gCS 面恒无 modal 集。

**五轮排查链**（每轮「仍红→勘根→加一层」）：

1. 建 `RUNNER_MODAL_SELECTORS` 通道（`static RwLock<Option<Arc<Vec<String>>>>`，镜像 QUERY_STYLE_OVERRIDE 先例）+ `apply_runner_modal_state` 注入槽 doc `set_modal_dialogs`——仍红。
2. 勘得 WPT `test()` 是同步函数：showModal 后**同 turn** gCS 断言，探测环拍间轮询集合恒 stale → 改 shim **同步推送**：showModal/close/attrClosed/reset 四点 → `__zw_set_modal_state(sels)` 宿主回调（callbacks.rs 注册）——仍红。
3. 勘得推送点 `AllSels` 的 `hasAttribute('open')` 读桥快照 stale（`_zwSetAttr` 异步入队，推送点读必 false）→ 新增 `_zwDialogModalAllSelsRaw()`：只以 `_zwDialogModal[key]` 内存态为真值（stale selector 交 compute 侧 `query_selector` 容忍，与 pipeline `inject_modal_state` 的 filter_map 同语义；栈序外层先→栈顶取末项）——仍红。
4. `ZZ_DBG_S373` 探针序列揭示**注入时序悖论**：`apply_runner_modal_state` 插 `with_cached_document_styles` 的 f 前来不及——增量 compute（`compute_styles_incremental`）在 slot 更新（`finish_generation_update`）内完成，styles 已按旧 modal 态产出 → 注入点移到 `finish_generation_update` 内 `StyleSystem::new()` 调用**前** + `with_cached_document_styles` f 前快路径兜底——3/3 全绿。
5. 防重注入：applied 代际键 `(Arc::as_ptr, drain_gen, style_version)` 同键短路；模态变化写新 Arc（指针变）或 doc 换代（gen 前进）均失配重注；NodeId 随换代失效故键必含 gen 分量。

**转正钉测教训（单测构造假象）**：`compute_element_style(parent_style=None)` 无继承上下文——单元素通道 visibility 继承面不可观测（plainDiv 的 Hidden 来自 div 规则直接命中非继承），假阳性/假阴性双向失真 → 转正钉测 `crates/style-system/src/tests/ua_dialog_visibility.rs` 走**全文档通道** `compute_styles`（自动递归父链），`dom_with_hidden_parent_dialog` + `hidden_div_sheet`（div{visibility:hidden}）helper，non-modal Hidden / modal Visible 两断言。文件头注明此坑。

**dialog-inert 勘根**：rect_snapshot 布局矩形由 render 管线写——同 turn showModal 后 gBCR 全零（rect=0,0,0,0、body=800x0）→ hasLayout 标志 + 栈顶模态 dialog 兜底臂（`_zwDialogModalAllSelsRaw` 取末项 selector querySelector 代理验证非 inert）。

## 验证

- **visibility-hidden**：hidden→visible→hidden 序 3/3 断言全绿（/tmp/s373/verify-visibility7.log，本地）。
- **default-color**：案1 翻绿、案2（dark mode）维持挂账（verify-color.log）。
- **dialog-inert**：翻绿（verify-inert2.log）。
- **dialog 域全域 A/B**：真案 **25F→22F**（基线 /tmp/s373/baseline-fails.txt 名单级：16 文件 25 子测试；修后 default-color 残 1F=dark mode、dialog-inert 与 modal-dialog-in-visibility-hidden 零 Fail）；`zz_ld_probe_s372` 恒红探针 3 子测试单列口径不入真案数（s343 口径）。
- **波及面定点**：image-map 族 0F、usemap-casing 0F（缺席臂改动无波及）；select-dialog-mode-focus 2F 维持（focus 行为面挂账，:open 子断言经施工②通过）。
- **lib 基线**：zero-dom 903 passed（test-dom.log）；zero-style-system 2504 passed 含转正钉测（test-style.log）；zero-engine 3826P/7F——7 红逐名 img settle/fetch/worker/websocket 既有基线红，stash 修改面重跑单失败样本同败定谳非回归（test-engine.log）。
- **渲染面**：`make reftest` 725 total / failed 0 (0.0%) / 真通过-可信 558 (77.0%)（reftest.log）；`make product-smoke` 6/6 struct-check PASS（smoke.log，UA dialog 色对默认页零影响）。
- fmt / clippy（dom/style-system/engine/wpt-runner 四 crate）PASS。shim 校验须按 13 文件拼装序 node --check（js_dom_bridge.rs 拼装序；单文件 check 假报 Unexpected EOF，基线同样）。

## 挂账（移交下一切片）

1. **default-color 案2**（dark mode）——系统色需按 color-scheme light/dark 分派 Canvas/CanvasText。
2. **select-dialog-mode-focus 2F**——showModal 焦点链行为面（首个 focusable 元素定位 + Tab 不关闭 picker）。
3. **select picker open 态注入通道**（customizable-select 39 涉案文件）。
4. **UA popover/dialog 定位全规则簇**（inset:0/margin:auto/fit-content 居中 + border/padding + display:none-when-closed）随 R-s31 渲染流（s372 挂账承继）。
5. **overlay CSS 属性 + top-layer 强制定位**（dialog-overlay 2F / top-layer-position、abspos-dialog-layout 6F 面）。
6. **inert 编辑屏蔽**（inert-node-is-uneditable——contenteditable 键入通道）。
7. **corner-cases**（modeless showModal 转模态语义）；**form-submission 亚像素坐标**；**modal-dialog-ancestor-is-inert / multiple-centered-dialogs / non-modal-dialog-layout / scroll-height / showModal / closedby-bounds-clicking**（既有 8 文件未建面）。
8. **inert hit-testing 穿透近似面**：本刀近似只屏蔽 inert 目标返回 null/兜底，未实现命中穿透到下层可交互元素。
9. **fresh parse 通道 modal 注入缺口**：static RwLock 通道覆盖 bridge 代际槽面；pipeline fresh-parse 已有 inject_modal_state，其余 fresh-parse 使用方未勘。
10. 既有在册：基建面 T=3（s102/s124）、probe8/9/10 转正决策；runner filter 口径——the-iframe-element 不在 fetch 盘点（FILTER 空集静默 exit 1，非本 runner 口径案件）。
11. 探针产物已删（zz_probe_s373/b gitignore 区）；ZZ_DBG_S373 探针环境变量勘根毕已删。
