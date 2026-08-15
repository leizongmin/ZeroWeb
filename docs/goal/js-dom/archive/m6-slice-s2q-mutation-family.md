# M6 S2q — QuickJS 写入族：textContent setter + mutation 起步（R60/R60b）

**日期**: 2026-08-16
**commits**: `be577323`（textContent setter）/ `b807e5e9`（create/append/remove）
**里程碑**: M6 QuickJS 原生绑定移植（js-dom goal DC-7）第四/五切片
**证据**: [evidence/2026-08-16-r60-quickjs-s2q-textcontent-mutation.json](../evidence/2026-08-16-r60-quickjs-s2q-textcontent-mutation.json)

## R60：textContent setter（`be577323`）

镜像 V8 `native_text_content_setter`：ToString 值清空全部子节点（先收集 NodeId
防边遍历边改）+ 非空追加单 Text 节点。**LegacyNull 语义**：setter 收 raw Value
手动判 null/undefined → 空串——`Coerced<String>` 的 JS ToString 对 null 产出
"null" 字面量（不可区分），这是本轮关键 API 发现。

## R60b：mutation 族起步（`b807e5e9`）

- `__zw_native_create_element(tag)` 全局工厂（与 V8 同名同 wire——A/B 对照门双
  引擎复用）。detached 元素入 arena；`get_or_build_node_value` 统一身份缓存入口
  （element_for_id 与 create_element 两工厂共享，消除重复）。
- `appendChild(child)`：child 经 `node_id_from_value`（隐藏 `__zwNodeFfi` 标记
  判 native 族）；spec 移动语义（zero_dom append_child 内建 reparent）；返回
  child（spec）；DomError → null（**DOMException 构造器基建延 S4q**——V8 侧
  R4 的 DomError→DOMException 映射随异常基建对齐，注记在案）。
- `removeChild(child)`：返回被移除 child；失配 → null。

## PoC 全链路断言

create（detached nodeType=1/tagName=P）→ append（父 textContent 反映新子树）
→ remove（父 textContent 空）。textContent set/null-clear 闭环同测。

## 验证

engine quickjs **1419** / v8 **2153** 零回归；webview quickjs wiring 绿；
clippy quickjs 矩阵零警告；fmt 无 diff。

## API 发现（沉淀）

1. LegacyNull setter 须收 raw Value 手动判 null（Coerced ToString 吞 null 语义）。
2. `Value::as_object()` 返 `&Object`——`let obj: &Object = v.as_object()?` 是
   clippy needless_borrow 与类型对齐的平衡写法。
3. `with_dom_mut` 包 `append_child` 返 `Option<Result<(), DomError>>` 双层，
   match `Some(Ok(()))`。

## 下一步

S2q 续（insertBefore/replaceChild + childNodes/parentNode 读回）→ S3q 查询 →
S4q EventTarget + DOMException 基建（补 appendChild 错误路径对齐）。
