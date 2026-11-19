---
date: 2026-11-20
modules: engine,tests/wpt-runner
---

# typed-om 组件 toString 参与 WPT 子测试命名——序列化变更伪装成全语料大面积回归

## 问题描述

刀 TD（typed-om transform 组件槽位校验）首轮 A/B 电池给出
`gained=133 lost=1 regressed=655` 的 FAIL 面：`the-stylepropertymap/properties/`
下 ~335 页每页 2 条「Setting 'X' to a transform: … throws TypeError」签名回归，
另有 transform.html 1 条真回归。按签名差口径该刀应整体否决。

## 根因分析

WPT css-typed-om testsuite 的 `<transform>` 示例不是字符串字面量，而是
`CSSTransformValue([new CSSTranslate(50%,50%)])` 等**组件对象**；子测试名由
`` `Can set '${prop}' to ${desc}: ${example.input}` `` 模板生成，`${example.input}`
走组件 `toString()`。刀 TD 把 `CSSTranslate` 的 z 槽从裸数 `0` 归一为
`CSSUnitValue(0,'px')` 后，toString 仍用「z 是否 number(0)」（`_zwOmIsZeroNum`）
决定是否输出第三参，于是 2 参构造的序列化从 `translate(50%, 50%)` 漂移为
`translate(50%, 50%, 0px)`——**示例串变更 = 全语料子测试改名**。逐页
gained=lost 完美配对（accent-color 2/2、logical 92/92），Pass→Fail 实测为 0；
唯一的真回归（transform.html 1 条）是改名后的 3 参示例串进了引擎 set/get
通道才暴露的解析缺口。规范修法：toString 按 css-typed-om-1 §serialization 以
`_is2D` 槽定形，修复后 v2 电池 `gained=134 lost=0 regressed=0` 全绿。

## 解决方案与避免方式

- 签名差 A/B 判读遇大面积 REG 时，先做**逐页 gained/lost 配对分解**再定谳：
  配对完美且 Pass→Fail 计数为 0 的是改名面，不是行为回归。
- 改动 typed-om 组件的存储归一（裸数→CSSUnitValue）时，同步检查该组件
  `toString` 的分支判据是否依赖旧表示；序列化判据应锚定规范内部槽
  （如 is2D）而非值的字面形态。
- testhelper 由组件对象生成子测试名（examples 含 percents/relative values/
  all-components 三组）是 css-typed-om 语料的普遍机制，color/length 组件刀
  同样受影响；收账 evidence 应记录「改名对」与真回归的分解结论。
