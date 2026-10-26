# M2 切片 54（s54）——details name-attribute 3F 分诊轮：R5260 toggle 任务重排队 + R5261 template content 元素族，2F 收口 / 1F 底座缺口挂账

日期：2026-10-24。name-attribute.html 18 子测试 15P/3F → **17P/1F**（GAIN 2），余 1F 定性 parse-world 底座缺口、单独立项。

## 三案分诊

### 1. mutation event and toggle event order（R5260 收口）

R4009 toggle 事件任务队列旧实现 in-place 去重（复用已排 entry），使 e0 解析期排程的合并事件停留在队列首位；spec「queue a details toggle event task」合并语义为**移除已排 task 并重排新 task 到队尾**（tracker 非 null 时 oldState 取首排程 pre-state，newState 取最新调用）。

修复（part03 `_zwToggleQueueNotify4009`）：换新 entry 对象 + `old` 承接 `prevEntry.old`——旧 timer 经 fire 闭包身份守卫（`_zwDetailsToggleQ4009[key] !== entry` 即 return）自动作废，无需 clearTimeout，兼容 part81 `__zw_setTimeout` 记录式 stub 环境。

spec 推演对齐测试期望 FIFO [e3,e2,e1,e0]：e3 创建排 toggle → 插入让位合并重排队尾；e2 detached open → 插入让位合并；e1.open=true 排队 → close-others 关 e0 合并重排。

### 2. attachment scenario connected-in-template（R5261 收口）

plain wrapper 模板（innerHTML-into-sel 解析产物，`sel=undefined`、ctor=HTMLTemplateElement）的 content 视图走 part03 `_zwContentView`（模板 childNodes 内联存储视图），该视图缺 ParentNode 元素族 getter——`content.firstElementChild` 恒 undefined，测试 setup 断言 `container instanceof HTMLDivElement` 直接失败。part04 `_tplContent`（sel/handle 形态视图，R151）有全套元素族 getter，`template-in-disconnected` 场景（createElement 路径）因此本来就过。

修复（part03 `_zwContentView`）：补 firstChild/lastChild/children/firstElementChild/lastElementChild/childElementCount 六 getter（R151 同款形状，spec [dom-parentnode-children](https://dom.spec.whatwg.org/#dom-parentnode-children)）。

探针全链验证：`inner instanceof HTMLDivElement` ✓、`inner.innerHTML` 解析互斥 [1,0] ✓、qSA ✓——wrapper world 元素 IDL 与 R4100 enforcement 在该面全通。

### 3. attachment scenario connected-in-xhr-response（底座缺口，不收口）

`xhr.response.body` 是 `_zwParseEl` 快照元素（探针 protoChain `_zwParseEl > Object > null`，无接口原型接线）。场景链需三件套，探针逐步实证断点：

1. **instanceof HTMLBodyElement**——R130 的 setPrototypeOf 方案对 `_zwParseEl` 会把原型链上全部方法（qSA/mutTree 桥/attr 面）剪出链外；需接口原型与 `_zwParseEl.prototype` 双链保留方案（per-instance 中间链或 Symbol.hasInstance）。
2. **innerHTML 写解析**——ctor own data property（`this.innerHTML = _zwInnerFromOuter(...)`），赋值仅覆写字符串不触发解析；R3019 mutTree 桥 `_ensureMutTree` 只做读 rewire，无 setter 进解析。
3. **open IDL**——`_zwParseEl.prototype.querySelectorAll` 经 `_zwWrapQ` 返回 `_zwParseEl` 包装，`e.open` 为 undefined（读写皆缺）。

定性：parse-world 写面切片，波及 DOMParser/XHR/responseXML 全语料（快照架构语义），单独立项排期，不本轮弯就。

## 验证

- 探针页 zzz-probe-s54.html（template 链 / xhr identity / xhr 全链 stepwise / impl-doc 参照 / toggle order 五子测试）：修复后 template 链与 toggle order Pass，xhr 两案按分诊预期 Fail；**已删**
- name-attribute.html 单案 **17P/1F**（15P/3F → GAIN 2）
- 全域 interactive-elements 98 files **435P/48F/0T**（s53 基线 433P/50F/0T，GAIN 2 / REG 0 / Timeout 0）
- zero-engine lib **3448P/0F**（part60 R4100 契约 T1-T6 per-element 形态不断言跨元素序，重排队零回归；part81 R5254 stub 环境同验）
- fmt/clippy：本轮 diff 仅 JS shim（.rs 零变化），不适用

## 未解决项

- connected-in-xhr-response 1F 挂账（parse-world 写面三件套切片）
- 其余挂账序不变：B 族 pointer-capture 2F、C 族渲染流 20F、declarative shadow 联合切片待用户点名、parser form pointer 4F 用户决策缓行
