# R366 — CE registry 专项 slice 2b：ShadowRoot.prototype innerHTML（node-document 路由；12→11 create-element-realm-after-adoption 整文件转绿）

**日期**: 2026-08-29
**切片**: CE registry 专项 slice 2b（R365 创建路由的 shadow 域对偶——原型 accessor + 工厂委托 + realm 转发）
**改动面**: `part03.js`（ShadowRoot.prototype innerHTML accessor + shadow QSA 原型面）+
`part05.js`（工厂 shadow own setter 委托化 + host 印记 + iframe win ShadowRoot 转发）+
`part24.rs`（+1 单测）

## 1. 根因（WPT create-element-realm-after-adoption 4F 三层）

1. **子 realm prototype accessor 缺失**——用例经
   `Object.getOwnPropertyDescriptor(inner.ShadowRoot.prototype, 'innerHTML').set`
   直调；R364 后 iframe win 构造器多为主 realm 转发（R295 只 per-realm 化
   Text/Comment），`inner.ShadowRoot` 无转发 → `undefined.prototype` 崩（subtest 3/5）。
2. **工厂 shadow setter 无路由**——R181 时代的 own innerHTML setter（`_zwMBuildBodyTree`
   + parentNode 重指）按创建域文档落 ownerDocument，无 node-document realm 查表、
   无 hyphen tag 升级 → adopted host 的解析子走 inner realm（subtest 2/4 的
   `<p>`/`<x-baz>` instanceof 主 realm 断言 false）。
3. **part03 R194 占位 shadow 无 setter**——`shadow.innerHTML = …` 赋值静默 no-op。

## 2. 修复

1. **part03**：`ShadowRoot.prototype` 上挂 innerHTML accessor（幂等 guard）——getter
   序列化子树；setter 复用 R365 三法则（adopt 印记优先 + parentNode 爬链 + 兜底）+ 主/子
   per-realm registry（`doc._zwCERegistry` 槽）+ hyphen tag `_ceRunCtor` 升级；解析子
   ownerDocument 盖 node-document 印记。
2. **part03**：ShadowRoot.prototype 的 querySelector/querySelectorAll（元素 own QSA
   R181/R359 同构本树 walk：简单形态 regex + `_zwParseCompoundSel` compound 判定）——
   light-shadow（R194 占位 / R365 消费域）此前查询恒空。
3. **part05 工厂 shadow**：own setter 改**委托原型**（`Object.getPrototypeOf(this)`
   的 set `.call(this)`——R356 教训：遮蔽分支消费 `this` 而非闭包 shadow）；host 印记
   `_zwCreatorDoc` 同步（路由源）；getter 遮蔽保留（R365 二跑记录的既有形态风险不扩）。
4. **part05**：iframe win 补 `ShadowRoot: globalThis.ShadowRoot` 转发（R179
   Range/DOMImplementation 同模式）——子 realm prototype accessor 直调的前提。

## 3. 过程回归（当轮抓回当轮修）

- **单测 ④ 揭示兜底序缺陷**：非 adopted 的 inner-doc host（ownerDocument ===
  creatorDoc、body 无 parentNode）首版爬链失败后兜底**主文档** → 解析子误升主
  registry。修正兜底序 = **创建域印记优先兜底**（`host._zwCreatorDoc` 为 nodeType 9
  时用之；adopt 印记/爬链命中才覆盖；plain host 无印记回落主文档）。
- **R143 教训第八次实证**：Rust 字符串续行 `\` 后的 JS `//` 行注释把下一物理行
  语句吞进注释 → "Unexpected end of input"（test JS 载荷禁用行注释，已从串内移除）。

## 4. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55489P/13F/17T——Fail 文件集合 12→11（create-element-realm-after-adoption 整文件转绿 5P/0F），零新增零回归**（Timeout ±2 为已知并发噪声轮转族） |
| create-element-realm-after-adoption | 1P/4F → **5P/0F** |
| attach-shadow-realm-after-adoption | 3P/2F → **5P/0F**（连带转绿——同域二阶收益） |
| node-realm-mixed / preserved / preserved-frameless / node-creation-realm | 4P/5P/4P/13P 全保持 0F |
| native 路径（ZW_NATIVE_DOM=1） | 目标件 5P/0F + attach-shadow 5P/0F + node-realm-mixed 4P/0F——polyfill/native 行为一致 |
| engine 单测 | v8 **2492**（+1 `test_shadowroot_prototype_innerhtml_node_document_r366`：子 realm accessor 可达 / adopted host 走主 registry / 工厂 own setter 委托 / inner doc 走 inner registry / 原型 getter 可读）/ quickjs 1472 全绿 |
| webview / integration / Vue-lit-WC e2e | 658P / 781P / Vue 3P 全绿（框架消费面无回归） |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |
| make test | 唯一失败 XOpenDisplayFailed 环境既存项（R342/R355/R356 多轮记录 clean HEAD 同败） |

## 5. 剩余（realm 族 11 文件构成）

- **Node-isConnected iframe 专项**（R360 转档——iframe 子文档 connected 缺三层独立链路）
- **node-realm-adoption-after-frame-removal 1F**（creation realm 记录——「Node reached
  through the adopting document」变体；其余 2 subtest Pass）
- **MutationObserver-cross-realm**（工厂 body 可观察 id——R302 归因深项）
- **MutationObserver-document 3F**（parse-time record 基建——深项）
- **remove-and-adopt-thcrash / querySelector-mixed-case / remove-next-sibling**（window.open
  无 popup 通道 / R220 identity 双源域）
- **events 2F**（event-global-onerror 跨 realm / click-on-absolute-pseudo Chromium 专有）
- **ranges dataChange/replaceData 2F**（文件级 Timeout 尾批——R351 后注册表 GC 压力域）

## 6. 后续

- **native CE hooks per-realm 查表**（`__zw_native_ce_lookup` 等按 realm 查
  registry——native 路径的对应收口；当前读主实例，default-on 前对齐项）
- **node-realm-adoption-after-frame-removal 1F**（creation realm 印记在解析子上的
  独立记录——`<p id>` 深后代经 document.querySelector 访问路径）
