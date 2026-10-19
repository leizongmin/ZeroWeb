---
date: 2026-10-19
modules: engine
---

# 基建缺口修复会暴露 vacuous-pass 断言，需同轮把被钉行为真值化

## 问题

R4001 给 `_liveQueryCollection`（`document.forms` 等）补 HTMLCollection
named property 访问后，全量九域 corpus 出现一例 Pass→Fail：
`the-fieldset-element/HTMLFieldSetElement.html` 的 form owner 断言。
隔离复跑确定性失败，不是环境抖动。

## 根因

该测试 `fm1 = document.forms.fm1` 旧返 undefined（named access 缺失），
而断言 `assert_equals(fs_outer.form, fm1)` 里 `fs_outer.form`（fieldset
form owner，另一处未实现）也返 undefined——**两个 undefined 相等**，
测试 vacuous-pass 了多个切片轮次。named access 修复后左侧变成真元素，
右侧仍是 undefined， vacuous 断言现形。

## 解决方案

`part03.js` 的 `.form` getter（R2841）本就实现 spec form-owner 算法
（form 属性 → getElementById，否则最近祖先 form），只是 tag 表没收
FIELDSET。把 `FIELDSET` 加进分支条件即翻绿，同切片内回归归零。

## 避免方式

- 修复「上游取值」类缺口（集合访问/反射解析）后，凡是此前依赖该取值
  的断言都可能从 vacuous 变 real——全量 corpus diff 里 Pass→Fail 先隔离
  复跑定性：确定性失败 = vacuous 暴露，不是新回归。
- 判 vacuous 的标志：断言两侧都是「未实现的同一 fallback 值」
  （undefined/null/''）。处置是**把被钉行为真值化**（同轮补齐最小实现），
  不是回退基建修复、也不是放宽账本纪律。
