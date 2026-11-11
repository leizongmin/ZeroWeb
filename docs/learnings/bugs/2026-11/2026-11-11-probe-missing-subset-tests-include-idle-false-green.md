---
date: 2026-11-11
modules: tests/wpt-runner, editing-compat
---

# 探针页缺 /common/subset-tests.js 会整页空转假绿——P>0 门防不住，须恰集计数门

## 问题

outdent 刀探针页（zz-probe-outdent.html）v1.1 门禁 L0 红 `P=0 F=1
PICKS-FOUND`；修复 pick 失配后首跑变 **P=1 F=0**——但语料条目实际一条都没执行。

## 根因

corpus 的 `runConformanceTest`（tests.js :5556）内部经 `subsetTest(test, fn,
name)`（/common/subset-tests.js）注册子测试。探针页 include 链少了
`<script src="/common/subset-tests.js"></script>`，首个条目调 `subsetTest`
即 **ReferenceError 杀断整个 IIFE**——16 个语料条目从未注册执行，页面上只剩
PICKS-FOUND 一个 test()。

危险在于：若 pick 失配恰好已修，`PICKS-FOUND` 单独通过即 P=1 F=0，
**P>0 形门整页空转放行**——刀体行为完全没被测到。

## 识别签名

- 失败日志只有 1 行（PICKS-FOUND 本体），没有任何语料条目行；
- P 数量级与预期条目数严重不符（1 vs 16+）。

## 修法与通用规则

```html
<script src=/resources/testharness.js></script>
<script src="/common/subset-tests.js"></script>
<script src=/resources/testharnessreport.js></script>
```

- **探针页复用 corpus 断言面（runConformanceTest）时，include 链必须与语料页逐项对齐**（testharness + subset-tests + testharnessreport），少一个即空转；
- **探针门禁禁用 P>0 形**：门值用 preflight 实测定谳的精确计数 + 失败键恰集清单（本例 P=215 F=7 恰集），空转（P=1）与真绿（P=215）才可区分；
- 恰集门还顺带防「探针条目静默漂移」：语料条目改名/删除会立即使 pick 失配红出，而非假绿。
