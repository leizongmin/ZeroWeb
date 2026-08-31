# M6 S1q — QuickJS 只读属性族（R58）

**日期**: 2026-08-16
**commit**: `84c76b25`
**里程碑**: M6 QuickJS 原生绑定移植（js-dom goal DC-7）第二切片
**证据**: [evidence/2026-08-16-r58-quickjs-s1q-readonly-family.json](../evidence/2026-08-16-r58-quickjs-s1q-readonly-family.json)

## 目标

S0q 骨架上扩只读属性族（镜像 V8 dom_bindings S1 既有面），按「高频先行」分片：
本轮 4 项——className(+setter)/namespaceURI/localName/textContent。

## 实现

全部具名 fn + `This<Object>` 模式（S0q 踩坑经验直接复用，本轮零 API 阻抗）：

| API | 语义 | V8 对应 |
|-----|------|---------|
| className get/set | class 反射，缺省 ""；setter ToString 写 class | native_class_name_getter/setter（read/write_reflected_attr） |
| namespaceURI get | 空 ns/非元素 → **JS null**（非 undefined） | native_namespace_uri_getter 的 Some(None)→null 分支 |
| localName get | parser ns 感知大小写透传 | element.rs localName 面 |
| textContent get | 子树文本拼接（Document::text_content） | native_text_content_getter（setter 属 S2q） |

- 注册面：id/className enumerable（反射属性可见），其余非 enumerable——与 V8
  ObjectTemplate accessor 默认非枚举一致；Object.keys 断言 `id,className`。
- `reflected_attr_string_of` / `element_ns_of` 两个共用 helper 沉淀（后续反射族
  扩展直接复用）。

## 验证

| 矩阵 | 结果 |
|------|------|
| zero-engine quickjs | **1419 passed**（PoC 扩展 + 新 namespace_uri_variants：SVG vs XHTML ns） |
| zero-webview quickjs | **552 passed**（wiring 测试扩展 S1q 断言面，与 V8 同断言） |
| zero-engine v8 | 2153（零回归） |
| zero-webview v8 | 599（零回归） |
| clippy | engine/webview × v8/quickjs 四矩阵零警告 |
| fmt | 无 diff |

## S1q 剩余（对 V8 S1 面的差距，分片推进）

- 字符串反射族其余项：title/lang/accessKey（V8 经 native_string_reflected_*
  泛化——QuickJS 侧可共用 helper 泛化为参属性名版本）
- 枚举/boolean 反射：dir/hidden/tabIndex/contentEditable/spellcheck
- 复合对象：attributes（NamedNodeMap）/ classList（DOMTokenList）/ dataset
  ——需二级身份缓存（owner ffi → 复合对象），结构上是 S1q 最重的一片
- 方法族：getAttribute/hasAttribute/setAttribute/removeAttribute（S2q 写入）

## 经验

- S0q 的具名 fn + This<Object> 模式在本轮零摩擦复用——**API 踩坑成本一次性**，
  后续 S1q–S5q 是纯语义映射（「翻译」而非「设计」，与入口文档 §后续执行建议 4 一致）。
- namespaceURI 的 null 语义（区别 undefined）需 Value 返回形态 + ctx 构造
  String——String 返回形态的 getter 无此能力，混合返回形态（String vs Value）
  是 QuickJS 绑定的常规形态，V8 侧统一 ReturnValue 无此区分。

---

## R58b 补充（同切片追加，commit `b9136b38`）

字符串反射族收口：title/lang/accessKey getter/setter。V8 侧 `native_string_reflected_*`
按 accessor name 动态分发（name_to_content_attr 小写化）；QuickJS Accessor 无 name
回调 → per-property 具名 fn + 共享 helper（`reflected_attr_string_of`/新增
`set_reflected_attr`）+ 静态 IDL→content 映射（accessKey→accesskey）。语义等价：
缺省 ""、ToString setter、LegacyNullToEmptyString 由 `Coerced<String>` 覆盖。

PoC 断言扩展：三属性读写闭环 + Object.keys enumerable 面
`id,className,title,lang,accessKey`。engine quickjs 1419 全绿。
