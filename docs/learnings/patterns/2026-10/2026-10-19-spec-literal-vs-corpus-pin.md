---
date: 2026-10-19
modules: engine
---

# spec 字面与 WPT 语料分歧时：先查真实引擎结果，语料是验收标尺

## 问题

实现 selected autocomplete value 算法（HTML autofill
processing-model）时，spec 字面只对 `section-*` token 做「converted to
ASCII lowercase」，`field`/credential token 保留原大小写拼接 IDL 值。
但 WPT `form-autocomplete.html` 的 keywords 面（57 测）断言
`setAttribute('autocomplete', ' NAME')` 后 `input.autocomplete === 'name'`
——按 spec 字面实现必挂 55 测。

## 根因与核实路径

spec 文本演进后留下与浏览器实际行为的分歧；Chrome 上游当日 run
67/67 全过（wpt.fyi summary JSON，`c: [67, 67]`）证明真实引擎对
field token 也做 ASCII 小写归一，WPT 语料钉的是浏览器共识行为。

## 处置方式

1. 按 spec 算法搭骨架（category 表、token 消费序、default 分支严格照抄）。
2. 与语料断言冲突的归一点，**以语料为准**并就地注释钉明：
   「spec 字面仅 section 显式小写，corpus keywords 面 + Chrome 67/67
   钉死单/多 token field 皆小写」。
3. 另一隐蔽点：spec 的 credential 子步会**重赋 category**（`Set
   category to the result of...`），后续 contact 块以重赋值为门——漏读
   这句会让 `section-login shipping work tel webauthn` 形态在 contact
   消费前中断返 ''。算法步进类实现要把每步对既有变量的重赋值圈出来。

通用化：WPT 驱动目标里 spec 是骨架来源、corpus 是验收标尺；两者
冲突不是「测试错了」，先用 wpt.fyi 引擎结果裁决再落码。
