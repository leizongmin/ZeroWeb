# DC-2 · 批十一：html/syntax/parsing/template 子树定向腿

2026-11-22。批十一准备（a62d221df：扫描面 8 叶子目录 + fetch 清单）的定向验证
落地：`testharness-html-syntax template/` 过滤腿，批十后 release 二进制，
test-guard 包裹。19 键 / 329 子测试全执行，**Pass 42** / Fail 287 / Timeout 0，
**+42 Pass** 入账，覆盖分母键 +19。

## 对账口径（summary-chrome @4b26db8978）

分母 19 键 / 329 子测试（chrome 329 全过）：**19/19 键计数精确对平**（逐键
runner 子测试数 == chrome c[1]），无 pin 缺失、无记账残差。批十一准备提交的
「22 键 ~323 子测试」预估值修正为 19 键 / 329 子测试（22 键为预扫描期宽估，
chrome 分母实测 19 键）。

## 结果

| 面 | 值 |
|---|---:|
| 键 | 19 |
| 子测试 | 329（分母 329，计数精确） |
| Pass | **42** |
| Fail | 287 |
| Timeout | 0 |
| 全绿键 | 9 |

- 全绿面：additions-to-the-in-body-insertion-mode 的 ignore-* 4 键
  （body/frameset/head/html token 忽略语义 25 子测试）+ start-tag-body/
  template-end-tag-without-start-one(in-body)/template-is-not-a-foster-parent
  等——template 插入模式解析主链已通。
- 最大单项挂账：template-owner-document.html **223 F**——template content
  owner document 语义（engine template 面）。
- clearing-stack-back-to-a-given-context 3 键 42 F——解析栈回退语义。
- 小面 22 F：generating-of-implied-end-tags 4+4、
  template-end-tag-without-start-one(head) 5+1、template-child-nodes 4 等。

逐键明细（含 chrome 分母对照）见 [逐文件 JSON](2026-11-22-dc2-batch11-syntax-template-leg.json)。

## 门禁

语法语料零 `pipe=sub`/`.sub.js`/`{{GET` 使用（grep 实证）——批十注入刀对
syntax 套件零爆炸半径，template 腿与 preflight syntax 腿（字节级批八全同）
互证；套件全量重跑免去。

[批十证据](2026-11-22-dc2-batch10-query-encoding-getsub.md)
