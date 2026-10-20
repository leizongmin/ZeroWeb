# M4 片 e——ARIA Element 反射特性片 + 尾簇扫尾（2026-10-07）

## 结果

**全通道 61216/61304 = 99.86%**（片 d 收尾 61168/61304 = 99.78%，**+48 passes**），
M1 基线逐案 **0 回归**（对片 d 收口全量 per-subtest diff）。

## 修齐簇

| 簇 | 修法 |
|---|------|
| aria-element-reflection 23F → 3F（+20） | **ARIA Element 反射特性落地**：`_ZW_ARIA_EL_ATTRS` 8 属性（ariaActiveDescendantElement 单元素 + Controls/DescribedBy/Details/FlowTo/LabelledBy/Owns/ErrorMessage Elements FrozenArray；**ariaErrorMessageElement 单数不入表**——spec 不存在）；树链有效性（`_zwAriaTreeChain`——shadow root 经 `_shadowHandleMeta` 跳 host、ownerDocument 身份区分分离文档；有效 ⟺ T_E ∈ C_A）；内容属性 token **自身树内 DFS 解析**（proxy childNodes 同步视图 + getAttribute latest-wins——host getElementById 对同 turn id 变更 stale）；显式引用存储（IDL set 存代理 + attr 写空串；removeAttribute 解除、setAttribute 覆盖——WPT errormessage 面）；FrozenArray 身份缓存（签名含结果集/attr 串/树链——scope 变更即时失配）；TypeError 面（单元素收非 Element、数组收非数组/非 Element 项）；**sel 子挂 handle 容器同步父记录**（`_zwAriaSyncParent`——appendChild 写点 + R334 sel 父接手清理，shadow 移动同 turn 可见） |
| aria-element-reflection-disconnected 2F → 0 | 同上（分离树内 id 解析 + 同分离树引用仍有效——树链成员判定非「已连接」判定） |
| aria-attribute-reflection 21F → 0（+21） | role/aria nullable **getter 面**补全（片 d 修了 setter）：null-set 清 `_reflectedAttrs` 缓存 + `_zwAriaExplicit` set-then-remove 标记 → getter 返 null（testNullable 全族；初始 unset → ''） |
| historical 2F（applets/cssFloat） | document.applets 恒空集合；computed style float/cssFloat 初始 'none'（host 未覆盖返 '' 曾空） |
| fragment-parse-form-in-template 1F → 0 | input.form 祖先链 **TEMPLATE 门**（template content 内 form element pointer 为 null——WHATWG#12257） |

## 残差（88F）

- aria-element-reflection 2F：同 turn 内 sel 元素移树后 **host 视图 stale**（引擎
  async-apply 架构缝——test 自身 setup 的 getElementById 也受影响）+
  implementation.createHTMLDocument plain-object 元素树身份。均非 ARIA 反射面本身。
- render-blocking 行为 ~13F（blocking IDL/tokenlist 面已落，「Rendering is blocked」
  需渲染管线集成——缺口清单 P-render-blocking 挂账）。
- document.all exotic 对象 1F（[[IsHTMLDDA]] 特性级，未立项）。
- M2 残差：math-parse 5F、unclosed-svg-script 3F、ambiguous-ampersand 2F、
  html5lib 3F（document.write 管线面）。

## 门禁

- make test（TIME_LIMIT=1800）68 suites 全绿；fmt 干净；clippy（guarded，v8）干净；
  reftest 704/704 零不一致。
- 全通道 61216/61304 = 99.86%；M1 基线逐案 0 回归。
