# M2-P1 — named property 写入/定义语义（Proxy trap 面）

**日期**: 2026-10-11
**改动**: `crates/engine/src/js_dom_shim/part02.js` `_createStorage()` Proxy trap 组
**A/B**: 1187/1251 = 94.9%（M1 基线）→ **1221/1251 = 97.6%**（+34P，Fail 64→30）

## 修齐语义（WPT set/defineProperty/symbol-props/storage_builtins 语料口径）

Storage 为无 `[LegacyOverrideBuiltIns]` 的 legacy platform object：

1. **成员遮蔽**：`[[Get]]`/`[[GetOwnProperty]]` 中接口成员（Storage.prototype，含动态
   新增）优先于 named property——同名时 `storage[k]` 读成员、`gOPD(storage, k)` 报
   `undefined`（原实现顺序相反，R4875 FIXME 记录的已知偏差，本轮消除）。
2. **named setter 拦截全部 string 键写入**：`storage[k] = v` 与
   `Object.defineProperty(storage, k, {value})` 一律走 setItem 语义（值 String 化入
   存储项），成员不被覆盖（builtins 不可遮蔽）、原型 setter 不被调用（accessor on
   prototype 的 unreached_func 断言）。defineProperty 的 descriptor 标志不参与
  （named setter 覆盖语义，两次 define 皆可）。
3. **Symbol 键走 ordinary own property**：不属 named property 面——get/set/has/
   delete/defineProperty/gOPD/ownKeys 全 trap 按字符串判定分流；`configurable:false`
   尊重（delete 返回 false）；修复原 get trap 对 Symbol own prop 恒返 undefined
   触发的 Proxy 不变式异常（基线日志 "proxy did not return its actual value"）。
4. **ownKeys** = 存储项键 + target ordinary 键（`__zwItems` 排除）；非成员非项 string
   键 get 走 `Reflect.get` 原型链兜底（toString 等 Object.prototype 面）。

## A/B 分案

| 案 | M1 | P1 后 |
|---|---|---|
| defineProperty.window.js | 0/12 | **12/12** |
| set.window.js | 8/20 | **20/20** |
| symbol-props.window.js | 6/14 | **14/14** |
| storage_builtins.window.js | 0/2 | **2/2** |

余 Fail 30：missing_arguments 10（P2）、storage_key 8（P2）、quota 4（P3）、
StorageEvent 构造/init 8（M3）。

## 回归验证

- `cargo test -p zero-engine --lib`（test-guard 包裹）**2977 passed / 0 failed**
  （`--skip test_pipeline_deeply_nested_html`——该案在无本改动的干净 main 上同样
  stack overflow，预存问题与本轮无关，A/B 经 git stash 隔离确认）；其中
  js_dom_bridge 模块 1094 案全绿（R4875 Storage 接口/Named property 语义对位在内）。
- `make testharness-webstorage` 全量复跑（test-guard 包裹）。
