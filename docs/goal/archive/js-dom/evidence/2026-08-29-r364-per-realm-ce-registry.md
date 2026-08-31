# R364 — CE registry 专项首片：per-realm registry 实例（define 冲突分离）

**日期**: 2026-08-29
**切片**: CE registry 专项首片（realm 族 5 的共同前置——define 冲突分离）
**改动面**: `part03.js`（registry 状态工厂化 + define/whenDefined 参数化）+ `part05.js`
（R181 转发点改独立实例）+ `part24.rs`（+1 单测）

## 1. 改动

1. **`_zwMakeCERegistry()` 工厂**（part03）：子 realm registry 实例——own 三态
   （registry/byCtor/pending）+ 校验/waiter 逻辑经**参数化 helper** 共享：
   - `_ceDefine(reg, byCtor, pending, name, ctor, options, opts)`：valid-name 校验 +
     name/ctor 双冲突检测 + 注册 + R98 observedAttributes Get + R149 主文档 upgrade 子树
     （子实例 `noUpgrade`——iframe 文档升级路由为后续片）+ waiter resolve；
   - `_ceWhenDefined(reg, pending, name)`：valid-name reject / 已定义 resolve / 挂起。
   主实例（`globalThis.customElements` 字面量）读既有三变量（`_ce_registry/_ce_byCtor/
   _ce_pending`——20 处既有读点零改动），define/whenDefined 委托同一 helper（主文档语义
   单点保持）。
2. **iframe win 接线**（part05 R181 转发点）：`win.customElements = _zwMakeCERegistry()`——
   旧转发主实例（单 registry 近似）使 `inner.define("x")` 注册进主表、主 `define("x")`
   误碰「already used」。

## 2. 过程回归（当轮抓回当轮修）

首版 `_ceDefine` 遗漏 **waiter resolve**（原 literal define 尾段的 whenDefined 挂起者
resolve 段）——`test_custom_elements_r2813`（「define 触发挂起的 whenDefined resolve」）
当场红。补入 `_ceDefine`（`pending` 参数 + resolve 循环），r2813 转绿。教训：参数化
重构的「逻辑搬家」必须逐段核对原体尾部——吞段即静默丢语义。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，333 文件） | **55485P/18F/16T——真实 Fail 集合 13=13 恒等（两目标件仍在集合内但 subtest 级收敛），Pass +1 净正** |
| create-element-realm-after-adoption | 0P/1F（page-throw 整文件崩）→ **1P/4F**（define 冲突消除，subtests 解锁运行；余 4F = parse-time 创建域路由——后续片） |
| node-realm-mixed-across-adoption | 3P/1F（余 1F = node-document-realm 路由断言——后续片） |
| engine 单测 | v8 2491（+1 `test_per_realm_ce_registry_r364`：内外 realm 同名 define 互不冲突/各自 get/getName/whenDefined 独立/per-realm 冲突检测保持）/ quickjs 1472 全绿 |
| r2813 whenDefined 哨兵 | 转绿（waiter resolve 补入后） |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

## 4. 后续片

- **创建路由**（realm 族核心）：parse/create 时按 owner-doc realm 查表升级——
  create-element-realm-after-adoption 的 4F 与 node-realm-mixed 的 1F 的直接解；
- iframe 文档内 createElement 的 registry 查表（工厂 create 现无 upgrade——pre-existing）；
- native CE hooks（`__zw_native_ce_*`）的 per-realm 查表（native 路径）。
