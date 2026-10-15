---
date: 2026-10-15
modules: engine/js-dom-shim
---

# 切出 setter 子分支时误删兄弟路径首行副作用——textarea value 缓存写

## 问题

R3901 给 shim value setter 切出 INPUT 专属分支（`else if (_realTag(...) === 'INPUT')`）
时，把原 else 首行 `_inputValues[key] = String(value); _inputValuesSet[key] = true;`
当作 INPUT 专属死代码一并删除。全域跑出现 13 个 textarea/select 回归
（reset-form Precondition 17、setRangeText textarea、validity 系列）。

## 根因

那一行是 **TEXTAREA 的唯一缓存写**。textarea 的 value getter 靠
`_inputValues` 读回（`__zw_set_form_value` 在 applied view 是 no-op，
SetFormValue mutation 不回读）；没有这行，`ta.value = "123"` 后 getter
lazy-init 直读 child text，返回 stale 旧值 "abc"。首行副作用服务于整个
else 分支的所有元素，不是它下方 INPUT 特判（`if (_realTag === 'INPUT')
_captureInputDefault(...)`）的附属品——两处 INPUT 痕迹里只有一处真是死代码。

## 修复

恢复该两行到 else 首部，注释标明「textarea 无专属缓存面，getter 靠它读回」。
13 回归清零。

## 避免方式

- 从共享 setter/getter 分支切子分支前，先对该分支做**按元素类型的读写
  对账**：列出每个落点元素（INPUT/TEXTAREA/其余），各自「写路径落哪、
  读路径从哪取回」，再判断哪几行可移可删。读回路径（getter）依赖的写
  （setter）删一行就断一路。
- 「死代码」判定必须以分支的**全部落点元素**为准，不以新增分支的元素为准；
  拿不准时保留并在注释标注归属。
- shim 里 `_inputValues`/`_inputValuesSet` 的写入点要当契约看：grep 全部
  写入点（`grep -n "_inputValues\[" part0*.js`）再动 setter 结构。
