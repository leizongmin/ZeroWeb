# R96 — get trap 表查原型链污染：Object.prototype 方法名被 _REFLECTED_UINT 吞 + NamedNodeMap 语义三修

**日期**: 2026-08-17
**里程碑**: M3（真实 lit 库 e2e 前置——R95 诊断的「第二层缺口」收口）
**commit**: 见 master.md 本轮记录

## 诊断过程（R95 遗留 bisect 收口）

R95 报告：lit e2e 的 `enableUpdating`（ctor 内 Promise executor 赋值的实例属性）读回原型 noop；expando-first 尝试暴露 `this.hasOwnProperty is not a function`——hasOwnProperty 在 get trap 长链不可达；bisect 显示 part03 标记全达、part04 计数器未达。

本轮探针（probe_r96，land 前移除）+ 逐段中点标记二分（mid1/mid2/midA/midB/mid3/fallback/pchain）：

```
dataset:8, gBCR:8, mid1:8, mid2:8, midA:8, lastProp@midA:enableUpdating,
midB:2, lastProp@midB:enableUpdating, mid3:2
```

midA=8（`_REFLECTED_UINT[prop]` 查表前）但 midB=2——5 个 Object.prototype 方法名 miss
（hasOwnProperty/propertyIsEnumerable/valueOf/toLocaleString/isPrototypeOf）在 2733-2744 区间消失。

## 根因（三重）

1. **`_REFLECTED_UINT[prop]` 裸下标查表**（part04 get trap）：`hasOwnProperty` 等名经
   Object.prototype 原型链返回 truthy 函数 → `if (_ruEntry)` 误入 → `parseInt(entry.a=undefined)`=NaN
   → `return entry.d = undefined` 提前吞掉。**任何元素读 `el.valueOf`/`el.hasOwnProperty` 全返
   undefined**（lit 的 hasOwnProperty 探测、Object.prototype 方法以元素为 receiver 的调用中断）。
   `constructor` 侥幸因 R95 顶部短路未炸；`toString` 有 part03:3672 显式分支。
2. **`_attributesProxy` get trap 无原型链回落**（part03）：miss 名（含 hasOwnProperty）恒返
   undefined——WPT attributes.html `getEnumerableOwnProps1` 的 for-in own 过滤抛 TypeError 整 subtest 崩。
3. **`_zwMEl`（detached createElement 元素）`.attributes` 是裸数组**：`Object.getOwnPropertyNames`
   返回方法名；named getter/ownKeys 语义全无；`setAttributeNS` 方法缺失。

## 修复

1. part04：`_ruEntry` 查表改 `Object.prototype.hasOwnProperty.call(_REFLECTED_UINT, prop)` own-property 判定。
2. part03 `_attributesProxy`：miss 名回落 Object.prototype（`constructor` 排除）；named 属性
   descriptor `enumerable:false`（spec named properties 平台对象枚举语义——for-in 只见数值索引，
   WPT attributes.html 仲裁；getOwnPropertyNames 顺序不依赖 enumerability，R44 用例 3/3 保持）。
3. part03 `_attributesProxy` 新增 `supportedNames()`：**HTML 文档 + HTML ns 元素**的 named keys
   仅含全小写 qualified name（`_nsHandles` 无条目 = createElement 隐式 HTML ns；ownerDocument.
   contentType 判 HTML-ness）；索引语义（length/item/索引读）仍走全量 `readNames()`——期望数组
   `["0".."5","g:h","j"]` 索引 0-5 全可达 + named 只有小写名。
4. part03 `_zwMEl`：`.attributes` 改 lazy accessor 返 NamedNodeMap 视图 Proxy（length/item/
   getNamedItem/setNamedItem/removeNamedItem/索引读/named getter/ownKeys/gOPD/has——has trap
   防 Array 泛型方法 slice 把索引当 hole）；补 `setAttributeNS`（忽略 ns 按限定名，与 proxy NS
   族既有近似一致）。

## 验证（A/B：stash 重建二进制对照）

| 面 | 结果 |
|---|---|
| WPT dom/nodes | 6654P/1574F → **6658P/1570F（净 +4，per-subtest 零回归）**——attributes.html 3 + attributes-namednodemap.html 1 解锁 |
| WPT dom/events / collections / traversal | per-subtest 与修复前**零差异**（236P/194F、48P/1F、1593P/11F） |
| passive-by-default A/B | pre/post 逐 subtest 零差异（43/100 维持——R95 已解锁面不受扰） |
| zero-engine v8 | **2192** 全绿（+1 R96 单测：六名 typeof function + hasOwnProperty 调用语义 + expando + colSpan 缺省 1 不受扰；R44 断言③按 WPT 期望纠正 for-in 只见索引） |
| zero-engine quickjs | **1427** 全绿 |
| zero-integration-tests | **777** 全绿（含 e2e_lit_library / e2e_web_components 资产） |
| fmt / clippy | 双矩阵零警告、无 diff |

## 教训

1. **Proxy get trap 内查表必须 own-property 判定**——裸 `TABLE[prop]` 对 `hasOwnProperty`/`valueOf`
   等 Object.prototype 继承名恒 truthy，且症状（返 undefined）远离病因（查表），bisect 中点标记
   是定位长 trap 链提前 return 的最快手段。
2. **枚举语义分层**：索引可见性（length/item/索引读）与 supported property names（named keys）
   是两个独立观察面——HTML 文档的 lowercase 规则只作用 named 层，动 readNames 会连锁破坏索引语义
   （首轮过滤 readNames 致 `["0","1","g:h","j"]` 索引丢失，A/B 捕获后拆双层）。
3. 旧实现自洽的单测断言可能固化错误语义（R44 断言③「for-in 枚举到全部」）——WPT 期望表是最终
   仲裁者（R83 先例第三次实证）。
