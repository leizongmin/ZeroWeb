# M6 S2q — QuickJS 属性方法族（R59）

**日期**: 2026-08-16
**commit**: `fe0a23d5`
**里程碑**: M6 QuickJS 原生绑定移植（js-dom goal DC-7）第三切片
**证据**: [evidence/2026-08-16-r59-quickjs-s2q-attribute-methods.json](../evidence/2026-08-16-r59-quickjs-s2q-attribute-methods.json)

## 目标

QuickJS native 从「Accessor 属性」扩展到「方法」形态：get/set/remove/hasAttribute
四件套（框架最高频的 element API），经 `Function::new` 挂元素对象 prop。

## 实现

- 具名 fn + `This<Object>` 模式延续（S0q 经验第三次零摩擦复用）。
- `getAttribute` missing → **JS null**（区别空串值；V8 同语义，String 返回形态
  做不到 null → Value 返回形态 + ctx 构造，同 namespaceURI 模式）。
- `removeAttribute` **真移除**（`Document::remove_attribute`）——布尔属性 unset
  语义（`disabled`/`checked` 须移除才 unset），区别 set 空串（镜像 V8 RemoveAttr
  OnHandle 修正的历史教训）。
- Function prop **非 enumerable**：Object.keys 面保持
  `id,className,title,lang,accessKey` 不变（与 V8 ObjectTemplate 方法注册一致）。

## 验证

- engine quickjs **1419** 全绿（PoC 扩展全闭环断言）
- clippy quickjs 矩阵零警告；fmt 无 diff
- webview quickjs wiring 测试绿（生产路径经同一 build_element_object 注册）

## M6 累计状态

S0q 骨架（R57）+ S1q 反射属性 8 项（R58/R58b）+ S2q 方法族 4 项（R59）。
元素对象现有：3 构造 getter（nodeType/tagName/nodeName）+ 8 反射属性（5 带
setter）+ namespaceURI/localName/textContent + 4 方法 + 工厂 + 身份缓存。

## 下一步

- S2q 续：textContent setter / 子树 mutation 族（appendChild 等，需
  create_element 工厂配套）
- S1q 复合对象 attributes/classList（二级身份缓存）
- S0q 续 weak/finalizer
