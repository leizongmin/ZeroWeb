---
date: 2026-11-13
modules: engine,tests
---

# wpt editing 语料红账判读：切片伪失败与期望形真值源

## 问题

wptfyi-editing 目标按语料 Fail 数收敛，两类"红"不是引擎缺陷，按红数
排修复优先级会烧批次：

1. **切片割裂伪失败**。run 页 meta variant（如 `?2001-3000`）把语料
   条目拦腰割断：`runConformanceTest` 的注册期副作用（setupDiv、选区、
   before 查询）对全部条目无条件执行，但 execCommand 闭包只在窗口内
   创建的 test 里执行（subsetTest 跳过 test 创建，而 vendored
   testharness.js 的 `test()` 同步立即执行）。被割条目的窗口内断言跑在
   未执行命令的 DOM 上 → got=输入原样。Chrome 在 wpt.fyi 上同红，
   harness 固有伪失败。
2. **语料期望形与 spec 算法相悖**。sub/sup 对 inline `vertical-align`
   decl 壳：vendored implementation.js（spec 草案直译）产出壳内嵌套
   （`<span va:sub><sub>bar</sub></span>`），语料期望摘壳换 tag
   （`foo<sub>bar</sub>baz`）——语料 data 文件按 Chrome 录音，两者在此
   形上分歧。按 vendored 算法对齐实现会反向翻红。

## 根因

- 切片机制：`common/subset-tests.js` 的窗口过滤作用于 test 创建层，
  而条目注册与命令执行分属两个时机；`?A-B` 边界落在条目中途即产生
  割裂条目。边界条目号可由断言数公式预测（每条目 = ncmd + 2 +
  6×查询键数），再用自定义 meta variant 的探针页 ground-truth 实证。
- 期望形来源：editing 语料 data/*.js 的 expected 是浏览器行为录音，
  不是 spec 算法的推演结果；spec 草案本身在 sub/sup 等角落与 Chrome
  分歧。vendored implementation.js 只能当"spec 参照系"，不能当期望
  真值源。

## 解决

- 红账先按条目切片完整性分诊：fail 行反查条目号 → 对照 variant 边界
  → 命中割裂条目记伪失败，不进引擎修复队列。探针页可用
  `browserTests.slice(...).forEach(runConformanceTest)` 做窗口 bisect。
- 兼容性修复的真值仲裁顺序：语料 expected（Chrome 录音）> vendored
  spec 算法推演。两者冲突时，若 vendored 复现的恰是我们的 got，说明
  引擎走了 spec 路径而 Chrome 走了另一条，修复目标按语料建模。
- 伪失败与真差异混合的 face 账目按断言级对拍口径写：F2P 只会落在
  真差异行，伪失败行对引擎改动恒等（可作零回归护栏）。

## 避免方式

- 拿到语料红账先问"这个断言的命令真的在这个窗口里执行了吗"，再问
  "期望形来自哪份录音"。两问都过才进引擎根因链。
- 涉及 spec 草案与浏览器行为分歧的角落（sub/sup、vertical-align、
  styleWithCSS 边界），先造 vendored 算法对照探针再定修复方向。
