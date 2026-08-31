# R365 — CE registry 专项第二片：创建路由（node-document realm；13→12 node-realm-mixed 转绿）

**日期**: 2026-08-29
**切片**: CE registry 专项第二片（realm 族路由核心——创建/解析时按 node-document realm 查表升级）
**改动面**: `part05.js`（工厂 innerHTML setter node-document 路由）+ `part01.js`/`part04.js`
（iframe doc `_zwCERegistry` 槽两处接线）

## 1. 改动

**工厂元素 innerHTML setter（part05 R181 setter）的解析路由升级**：
1. **node document 解析**（spec create-an-element-for-a-token 步骤 1：解析子属 node
   document 非创建域文档）——① adopt 印记优先（主 body appendChild 的 R191
   ownerDocument getter 已重指 node document；pending 未 apply 时 parentNode 链断，印记
   是唯一可靠信号——首版仅爬链探针实证失效）；② parentNode 爬链（直接挂载形态）；
   兜底创建 doc。
2. **解析子 ownerDocument = 当前文档**（adopt 后=主文档；未 adopt=创建 doc——两形态
   spec 都归 node document）。
3. **hyphen tag 升级**——当前文档 realm registry（主文档 → `globalThis.customElements`；
   iframe doc → doc._zwCERegistry 槽）`get(tag)` 命中即 `_ceRunCtor`（R94 ctor 体）。
   factory 解析子此前从不升级（instanceof 双 registry 全 false 的根因）。

**`_zwCERegistry` 槽两处接线**：part01 `_zwFinishIframeEntry`（加载路径）+ part04
no-src fallback 路径——`doc._zwCERegistry = win.customElements`（R364 per-realm 实例）。

## 2. 教训

1. **印记先于链**：pending mutation 未 apply 时 parentNode 链断——adopt 印记
   （ownerDocument getter）是唯一可靠信号。首版仅爬链探针实证 instanceofA=true
   （升级走错 registry），印记优先修正后 instanceofMain=true。
2. **升级路由与所有法规律**：node-document realm（非创建域、非运行域）——WPT 用例的
   断言方向（instanceof main registry）直接给出路由键。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，333 文件） | **55487P/17F/15T——真实 Fail 集合 13→12（node-realm-mixed-across-adoption 整文件转绿），零新增零回归**（Pass +2、Timeout -1） |
| 目标件 | node-realm-mixed 3P/1F→**4P/0F** |
| 探针四形态 | adopted 容器解析→主 registry 升级 ✓ / inner-doc 解析→inner registry 升级 ✓ / ownerDocument=主 ✓ / A 类 instanceof 正确负 ✓ |
| create-element-realm-after-adoption | 1P/4F 维持（ShadowRoot.prototype innerHTML setter 跨域调用 + per-realm element creation——slice 2b 独立片） |
| engine 单测 | v8 2491 / quickjs 1472 全绿 |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

## 4. 后续

- **slice 2b**：ShadowRoot.prototype innerHTML setter（主/子 realm）+ factory attachShadow
  产物接 prototype setter（create-element-realm-after-adoption 4F）；
- **native CE hooks per-realm**（`__zw_native_ce_*` 查表按 realm——native 路径）；
- realm 族余 4：Node-isConnected（专项）、node-realm-adoption-after-frame-removal
  （creation realm 记录）、MutationObserver-cross-realm（工厂 body 可观察 id）、
  create-element-realm（slice 2b）。
