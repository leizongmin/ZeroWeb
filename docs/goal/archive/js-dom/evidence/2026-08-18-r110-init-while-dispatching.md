# R110 — init-while-dispatching 语义 + legacy initXxxEvent 族补齐（events +10P 双路径）

**日期**: 2026-08-18
**里程碑**: M4（WPT dom 上游基线扩展）
**Driving 用例**: `dom/events/Event-init-while-dispatching.html`（连带 `Event-initEvent.html` / `CustomEvent.html` 两簇）
**基线（R109 后）**: init-while-dispatching 0P/5F · Event-initEvent 9P/3F · CustomEvent 1P/2F
**结果**: **init-while-dispatching 5P/0F · Event-initEvent 12P/0F · CustomEvent 3P/0F（三簇全 100%）**

## 根因

1. **initUIEvent / initMouseEvent / initKeyboardEvent 方法缺失**（`is not a function`）——shim 只实现过 initEvent/initCustomEvent。
2. **派发中 init 不 no-op**：spec 各 init 方法步骤 1「dispatch flag set → return」，R106 已建 `_zwDispatching` 计数但 initEvent/initCustomEvent 没接守卫。
3. **init 首参 mandatory 缺失**：`e.initEvent()` 应抛 TypeError（WebIDL 位置参数 non-optional）。
4. **CustomEvent detail 缺省 undefined 而非 null**：`_makeEvent` 落 `options.detail`（undefined，Event 语义），spec CustomEventInit `detail: any = null`。

## 修复（part05.js 单文件四处）

- initEvent / initCustomEvent 头部加 `_zwDispatching` 守卫 + `arguments.length < 1` TypeError。
- CustomEvent 构造路径 detail undefined → null；initCustomEvent detail 缺省 null。
- 新增 `UIEvent.prototype.initUIEvent` / `MouseEvent.prototype.initMouseEvent` / `KeyboardEvent.prototype.initKeyboardEvent`（legacy 位置签名；基类字段经原型链 initEvent 复用；派发中守卫同款）。

## A/B 结果（WPT testharness 双路径）

| 路径 | R109 | R110 |
|---|---|---|
| polyfill dom/events | 363P/64F | **373P/54F（+10 净）** |
| native dom/events | 354P/73F | **364P/63F（+10 净同步）** |
| dom/nodes / collections / traversal | 6656P / 48P / 1595P | 不变（零回归） |

簇明细：init-while-dispatching 0P/5F→**5P/0F**；Event-initEvent 9P/3F→**12P/0F**；CustomEvent 1P/2F→**3P/0F**。

## 单测

- `test_init_while_dispatching_and_legacy_init_family_r110`（engine part03.rs）：五 init 派发中 short-circuit（listener 内改值派发后不变）+ 方法存在性 + 首参 TypeError + detail null（构造/init 双路径）+ 非派发态 initMouseEvent 全字段生效——5 断言组。

## 验证

- `make test` 65 套件全绿（v8 + quickjs 双矩阵）
- `cargo fmt --all -- --check` 无 diff；v8 clippy + quickjs clippy（engine）零警告

## 教训

1. legacy initXxxEvent 族是「一个语义 × 五个入口」——守卫（dispatch flag / mandatory 首参）做单一模式后**每个入口都要接线**，与 R106 dispatchGuard 教训同构。
2. detail 语义按接口分：Event 读 undefined / CustomEvent 缺省 null——`_makeEvent` 共享底座上子类构造路径须补自己的 init-dict 缺省。
