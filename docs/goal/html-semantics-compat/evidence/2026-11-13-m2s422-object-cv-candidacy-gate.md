# M2 s422 收口账——constraint validation candidate 门（object/fieldset/output 非 candidate，object-constraint-validation 1F 清账）+ A1 drained 态勘根定谳闭账

- 日期: 2026-11-13；基线: HEAD effc987e5（s421 收口 rebase 后），驱动: s421 挂账
  「A1 drained 态与 s284 联合勘根」承前 + 红池分诊 object-constraint-validation
  1F（s403 起在册承袭红，无语料面施工记录）。

## A1 drained 态 × s284 联合勘根（定谳 no-op 闭账，零施工）

勘根三步定谳「drained 态裸 handle 几何」无语料面，刻意不施工：

1. **机制考古**: `_layoutRect`（part05.js:14973）与 gBCR **同源**——注释明示「经
   `__zw_getBoundingClientRect`（与 getBoundingClientRect 同源）」，即 img IDL
   rendered 臂（naturalWidth 族）直通宿主 gBCR 回调；s284 雷区（clone-img 真
   rect 泄入 attr 期望面，density 巧合通过面 30 行 REGRESSED）读点在 settle 后
   （drained 时序）——**pending 臂（s421）对该雷区天然免疫**（attr 期望面读到
   的读点全在 drained 态，pending 窗口内不点亮），drained 臂则无此时序护盾，
   任何全局点亮即复燃。
2. **红池全量归因核查**: s421 收口 204F 红池 90 文件逐族核对——customizable
   select/combobox（簇 D 用户门）、dialog ×8（渲染流 top-layer/inert）、img
   density ×3（渲染流 sizing）、embed ×2（渲染流 intrinsic sizing）、tabular
   ×3（渲染流亚像素）、video ×2（渲染流）、其余 canvas/object/form/misc 单件
   ——**零个 drained 裸 handle 几何断言面**（form-nameditem/object-constraint-
   validation 抽查 gBCR/offsetWidth 读数恒 0——object-constraint-validation
   随本切片施工转绿，见下）。
3. **既有基建核对**: forward map 基建完备（webview `merge_handle_selectors`
   按 apply merge 发布 handle→selector 正置表，selector 按文档位次生成，id-less
   handle 亦入表；`computed_style_by_handle` forward 臂已在用），drained 臂
   成本低——但无语料需求 + s284 暴露面结构性（`_layoutRect` 同源直通），按
   简洁优先（无驱动面不施工）定谳 **no-op 闭账**；s395 残差「forward map 空返」
   同期闭账（生产路径 webview 已发布，空返仅 fresh 测试上下文，无语料面）。

## 勘根（object-constraint-validation 1F）

- 驱动案: `<object id="element"></object>` 静态元素，`check()` 断言
  willValidate=false / checkValidity=true / reportValidity=true，`setCustomValidity`
  后**复验同三断言**。失败点 = `assert_true(element.checkValidity())`（set 后轮）。
- 根因: `willValidate` 臂已有 OBJECT→false（R57 非 submittable 面），但
  `checkValidity/reportValidity` 通用臂无 **candidacy 门**——object 掉进
  `_validityState` 读 valid，setCustomValidity 后 customError → valid=false →
  返 false + 派 invalid。spec §4.10.5.2.2
  #candidate-for-constraint-validation 仅 input/select/textarea/form-associated
  custom elements；object/fieldset/output 是 form-associated 但非 candidate →
  #dom-cva-checkvalidity/#reportvalidity 步 1 恒 true 不派（validity IDL 是
  被动状态，customError 照常记录翻转——两 API 门在 candidacy 不在 validity）。

## 施工（shim 1 文件 + rust 测试 1 文件）

- **part04.js CV 臂 candidacy 门**: FORM 分支后插入 `_cvTag ∈ {OBJECT, FIELDSET,
  OUTPUT}` → return true（三 tag 同一 spec 条款共享面；FORM 级遍历臂控件选择器
  `input,select,textarea,button` 天然不含非 candidate，零触碰）。setCustomValidity/
  validity/validationMessage 零改动（被动状态语义原样）。
- **part112 单测**（rust 1 文件 4 断言面）: ①object 驱动案镜像（双轮三断言
  false/true/true + validity.customError=true/.valid=false 被动翻转）②output/
  fieldset 同门钉 ③input candidate 不越界（set 后 false、清后复 true——门不改变
  candidate 行为）④FORM 级遍历臂保持（invalid 控件 → form.checkValidity false）。

## 验证

- 拼接 shim（js_dom_bridge.rs 25 part 规范序）node --check **SYNTAX-OK**。
- unit part112: **1P**（test-guard cargo test --lib）。
- engine lib 全量（终码）: **3913P/7F**，失败名单对 s420 在册慢性红腿
  7F 逐名 **IDENTICAL**（img/svg settle ×2、fetch_filtered、iframe_sw_messages、
  response_body_r2967、websocket ×2）。
- solo object-constraint-validation: **1F→1P**。
- solo the-object-element 目录: **46P/0F**（修复面清账 + 邻域零扰动——object
  共享 CV 臂的 events/attributes 面零位移）。
- sweep ×2 join 对 s421 双基线（\x1f pairs、(key,state) 语义、stable 集）:
  - my1 vs s421-b1: **stable=8454 state-diff=1 new=0 gone=0**——state-diff 恰
    driving F→P（唯一位移）。
  - my2 vs s421-b2: **stable=8454 state-diff=1 new=0 gone=2**——同恰 driving
    F→P；gone=2 = sizes-auto 子测键（超时上报闪变家族，在册）。
  - my2 vs my1: **stable=8454 state-diff=1 new=0**——簇 D detailed.optional
    文件键 F→T（在册 T↔F 闪变，红集恒定）。
- make test 全量: **154 腿 ok，唯一 FAILED 腿 = engine lib 3913P/7F**，失败
  名单对 s420 在册慢性红腿逐名 IDENTICAL（failures: 块归一 diff 干净）；fmt
  --check 与 quickjs clippy 门禁腿通过；exit 101 即该慢性腿传播（与在册口径
  同源非新增）。单跑执行，启动前 ps 核对无同族残留（s421 轮双跑事故后新纪
  律）；ZeroWeb-2 并行流 make test 经 /proc cwd 核对系独立 clone 各自 target。

## 口径

- 在册红 s421 收口 raw 204F+14T（归一簇 D/sizes-auto 闪变面后稳定 −5F 面）→
  本轮终态 raw **203F+14T**（轮1）/ **202F+15T**（轮2，簇 D optional 文件键
  F→T +1T；sizes-auto 本轮全上报 −1F 系闪变回摆）——归一闪变面后稳定红面
  **−1F**（object-constraint-validation 1F 清账），new 红 0。
- 下一步: 红池分诊续（video 帧像素表面 FIXME 跨域大件待立项 / canvas toBlob
  jpeg 编码面勘根候选 / remaining 单件红逐一分诊）；挂账池承 渲染流 6 面 +
  embed intrinsic sizing ×2（渲染流）/ sizes-auto F5 用户门 / 簇 D select
  立项征询（用户门）/ form-nameditem override builtins（fix#19c 冲突，用户
  门 s137 定谳维持）。
