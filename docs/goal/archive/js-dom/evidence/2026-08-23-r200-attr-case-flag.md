# R200 Evidence — 属性选择器大小写标志 `[attr=value i]`（M4）

**日期**: 2026-08-23
**切片**: M4 轻量——ParentNode-querySelector-case-insensitive 双 subtest 转绿，全量 9767P/102F → 9767P/100F（-2F 零新增，Pass 计数 +2 归入既有文件）
**改动面**: `crates/dom/src/query.rs`（`AttributeSelector.ci` 字段 + `strip_attr_case_flag` + 六运算符 ci 比较 + 单测）

## 一、spec 依据

CSS Selectors L4 §attribute-selectors：属性值后可跟空白 + 单个 `i`/`I`（比较大小写
不敏感）或 `s`/`S`（显式敏感——HTML 文档属性比较本就敏感，恒等）。WPT
`input[name*=user i]` 命中 `name="User"`。

## 二、实现

| 件 | 内容 |
|----|------|
| **ci 字段** | `AttributeSelector` 增 `ci: bool`（Exists 无值比较不受影响，恒 false） |
| **flag 剥离** | `strip_attr_case_flag`：去引号 + unescape **之后**剥值尾标志（值尾 trim → 末字符 i/I/s/S 且其前有分隔空白——裸 `i` 值（`[a=i]`）无分隔不算标志） |
| **六运算符 ci 比较** | Exact/Includes/Prefix/Suffix/Substring/DashMatch 在 ci 时双侧 `to_ascii_lowercase()`（Exact 用 `eq_ignore_ascii_case`——clippy 建议同轮采纳）；无标志路径与旧版逐字节等价（构造性零回归） |

**过程坑（全量跑当场抓回）**：首版 `&trimmed[..len-1]` 字节切分在多字节值
（`é` 等）上 panic（"end byte index 1 is not a char boundary"）——改
`chars().next_back()` + `len_utf8()` char 边界安全剥尾。

## 三、本轮评估并搁置的簇（深项记档）

1. **MutationObserver-document 3F**：parser insertion mutations——观察 document 的
   subtree+childList 时 HTML parser 自身的插入（`<p id=n00>`、`<script id=s002>`）
   须发 record。polyfill document 由解析快照构建（无 live parse stream），parser
   时 childList record 架构上不可发——**html-compat / live-parser 深域**。
2. **MutationObserver-inner-outer 2F**：record 的 addedNodes 与
   `getElementById().firstChild` 期望值是**两个不同 wrapper 对象**（identity
   分歧）——L2 identity 统一域家族（R171 等已记档同族）；另 1 Timeout 为 harness
   层 async 完成问题。

## 四、A/B 与全量

- 全量 polyfill **9767P/100F** / native **9768P/100F**（fail 清单 diff IDENTICAL；
  ±1P 为已知边缘 Timeout 互换）
- vs R199 基线：**零新增**，fixed 2（case-insensitive 双 subtest）
- `make test` 全绿；fmt/clippy（v8 + quickjs 矩阵）干净
- 单测：`zz_r200_attr_case_flag`（六运算符全覆盖 + 裸 i 非标志 + s 标志 + 无标志
  敏感守卫）

## 五、commit

`7f524865c`
