# M6 S5q 深化 — QuickJS 完整 ctor 执行 + attributeChangedCallback（R65）

**日期**: 2026-08-16
**commit**: `7a486499`
**里程碑**: M6 QuickJS 原生绑定移植（js-dom goal DC-7）第十切片
**证据**: [evidence/2026-08-16-r65-quickjs-s5q-ctor-attrchanged.json](../evidence/2026-08-16-r65-quickjs-s5q-ctor-attrchanged.json)

## 目标

S5q 深化：完整 ctor 执行（custom class 字段初始化真正生效，替代 R64 的纯原型
挂载）+ attributeChangedCallback 首批。

## 核心发现：construct 语义 vs this 绑定

`JS_CallConstructor2`（`Args::construct` 底层）**即使传 this 也遵循 construct
语义——新建 this 对象**：ctor body 的 `this.count = 41` 落在临时对象上，native
元素读回 NaN（实证定位）。**正确路径**：this 绑定**普通调用**（`Args::this` +
`apply`）——等价 V8 侧 super() 注入链的 Rust 侧目标形态，字段初始化直接落在
native 元素。

## 实现

- 完整 ctor 执行：registry 命中的 create_element 在原型挂载后，以 native 元素
  为 this 普通调用 ctor。字段初始化生效 + 状态保持（`++this.count` 42/43 断言）。
  ctor 抛异常静默（退回纯原型挂载，方法面仍可用）；箭头函数等非 constructor
  形态同样走普通调用（this 绑定不依赖 constructor 标志）。
- attributeChangedCallback：setAttribute 路径对 registry 命中元素派发
  `(name, oldValue=null, newValue)`——oldValue 写前捕获重排 + observedAttributes
  过滤延后（注记在案）。

## PoC 断言

ctor `this.count=41` + `greet()` 返回 `hi-MY-EL2:42/43`（字段+状态）；R64 无
字段路径回归；`setAttribute('data-k','v1')` → `attr:data-k:v1`；conn/attr/disc
ceLog 序贯集成。

## 验证

engine quickjs **1419** / v8 **2153** 零回归；webview quickjs wiring 绿；
clippy quickjs 矩阵零警告；fmt 无 diff。

## M6 剩余（深度补齐清单）

- S4q 完整化：capture/bubble 祖先链虚站（镜像 V8 R40）+ Event 构造器族 +
  DOMException 基建（补 appendChild/createElement 错误路径）
- observedAttributes 过滤 + oldValue 写前捕获
- S0q 续 weak/finalizer 生命周期
- S1q 复合对象 attributes/classList（二级身份缓存）
