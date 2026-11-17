---
date: 2026-11-18
modules: style-system, engine
---

# CSS 校验门挂在 cascade 面、WPT 页走 CSSOM 写面导致"实现完成零位移"

## 问题

刀 N 一期给 `font` 简写挂了完整文法门（`expand_font` family 位 + apply.rs
supports oracle 臂），实现正确、单测全绿，但主靶 WPT 页
`test_font_family_parsing.html` 608 Fail 一条不动，css-fonts corpus
3914→3914 零位移。

## 根因

WPT 的 CSSOM parsing 页面不走 cascade：`sheet.insertRule(".x{...}")` 与
`el.style.fontFamily = v` 走 shim 的 CSSOM 写面（`_parseDeclarations` /
style Proxy setProp），声明原文落库回显——回显值就是断言面。cascade 面
的 `apply.rs` 门在声明被拒/收之后才执行，CSSOM 写面根本不经过。校验门
挂错通道 = 实现完成但零位移。

同轮第二例同构：`validate_decl_value` 白名单接入 `font-family` 后，通用
random 门对 valid 语料 `16px 納豆嫌い` panic——白名单表接入会**激活**通用
前置门对该属性值的首次执行，暴露门内潜伏假设（`&lower[i..i+7]` str 切片
窗口终点落多字节字符中间；起点 ASCII 守卫不够，改 `&bytes[i..i+7] !=
b"random("` byte 切片比较）。

## 解决与避免

- 判定位：先追 WPT 页断言读什么（`cssRules[0].cssText` / getter 回显）
  → 反推执行通道 → 把门挂到该通道的既有分派枢纽。shim 已有三个先例：
  `_zwNormalizeDeclValue`（rule 声明/规则样式 setter 枢纽）、
  `__zw_validate_decl_value`（inline setter 白名单）、
  `__zw_style_shorthand_expand`（简写展开回调）。新属性优先白名单接入
  + 分派臂，不新开通道。
- 白名单表接入新属性 = 该属性值**首次**进入 validate_decl_value 全部
  前置通用门（sibling-index/random/math/css-wide）——接入前先用语料中
  最"野"的合法值（非 ASCII、转义、calc）跑一次全通道探针，不要只测
  新写的分派臂。
- Rust byte 扫描 ASCII needle 一律用 `[u8]` 切片比较（无 char boundary
  约束），不用 `&str[i..j]` 窗口；UTF-8 语料（CJK 家族名等）是 corpus
  常态输入。

## 关联

- evidence：`docs/goal/wptfyi-css/evidence/2026-11-18-knife-n2-font-family-grammar.md`
