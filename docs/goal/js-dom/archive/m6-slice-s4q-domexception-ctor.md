# M6 S4q 完整化 — DOMException 构造器 + instanceof 面（R73）

**日期**: 2026-08-16
**Commit**: `be431968`
**前置**: R72（Event/CustomEvent 构造器，`f663477b` rebase 后）
**证据**: [evidence/2026-08-16-r73-quickjs-s4q-domexception-ctor.json](../evidence/2026-08-16-r73-quickjs-s4q-domexception-ctor.json)

## 背景

R66 的 `throw_dom_exception` 抛带 name 的 plain Error 对象（e.name 可观测但 instanceof 不可达）。V8 侧 R6 的 DOMException identity 三重根因（prototype.constructor / 幂等注册 / wrong-global）教训在案——QuickJS 经单一全局构造器天然避免 wrong-global 问题。

## 实现

1. **JS 胶水构造器**（R71/R72 模式第三次复用）：`new DOMException(message?, name?)`——缺省 `''`/`'Error'`（R31 V8 parity）、legacy code 映射（name→code spec 表）、stack、prototype 链挂 `Error.prototype`、toString；21 个 `*_ERR` 常量；`new.target` 守卫（无 new 抛 TypeError）。
2. **throw_dom_exception 升级**：经全局构造器 `Constructor::construct_args` new 实例后 `Ctx::throw`——全部 R66 错误路径（appendChild cycle / createElement 非法 tag / ce.define 重复 / classList token 校验 / dataset 等）现抛真构造器实例；install 前早期路径回落 plain 对象。

## rquickjs API 注记

`Constructor::construct` 的 `IntoArgs` 对 tuple + 泛型 R 组合易碰 trait 界——`Args::new` + `push_arg` + `construct_args` 是稳定形态。

## 验证

- PoC 断言三组：instanceof 双面（DOMException + Error）+ name/message/code/toString、缺省参 + legacy 常量、native 错误路径 instanceof 可观测（`createElement('<bad>')` catch 得 `e instanceof DOMException === true`）
- engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵零警告；fmt 无 diff
- pre-commit-guard PASS

## M6 剩余

S0q 续 weak/finalizer（V8 R3133 对等物）→ whenDefined 真 pending。
