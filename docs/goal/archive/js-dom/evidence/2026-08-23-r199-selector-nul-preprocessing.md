# R199 Evidence — 选择器输入预处理 NUL→U+FFFD（M4）

**日期**: 2026-08-23
**切片**: M4 轻量——ParentNode-querySelector-escapes 的 NUL 族 2F（`#\u{0}` / `#ab\u{0}c`）转绿，全量 9765P/104F → 9767P/102F（净 +2P/-2F 零新增）
**改动面**: `crates/dom/src/query.rs`（`preprocess_selector` + 三个解析入口接线 + 单测）+ `part05.js`（`_parseSelectorListOf` 归一 + R181 工厂匹配器 ident 域放宽）

## 一、spec 依据与三层根因

CSS Syntax §5.3.3 "input preprocessing"：输入中的每个 U+0000 NULL 在词法层之前
替换为 U+FFFD。WPT `testMatched("\u{fffd}", "#\u{0}")`：选择器里的裸 NUL 等价于
`#<FFFD>`，命中 id U+FFFD。旧版三层各自吞 NUL，逐层修复均由探针实证：

| 层 | 旧行为 | 修复 |
|----|--------|------|
| host 词法（zero_dom） | 裸 NUL 落 `selector_lexically_valid` 的 ident 首字符判定（NUL 非 ident 字符）→ 整串判非法抛 "not a valid selector" | `preprocess_selector`（NUL→FFFD）前置到 `selector_is_valid` / `parse_selector_chain` / `parse_simple_selector` 三入口——document/shadow/dom_bridge/factories 全消费方继承 |
| shim handle 查询路径（part05 `_parseSelectorListOf`） | `_readCompoundToken` 把裸 NUL 当普通 ident 字符读出（value=NUL），`_matchCompoundOf` 拿 NUL token 比对 FFFD id 恒 false | `_splitSelectorListOf` 输入前 `replace(/\x00/g, '\u{FFFD}')`（与 host 同步） |
| shim 工厂元素匹配器（part05 R181） | ident 正则 `[a-zA-Z_][\w-]*` 把 FFFD（非 ASCII）拒之门外 → 复杂形态回落空 | 正则放宽到 CSS ident 域 `[^\s.#:>\[+,]+`（结构字符仍排除）+ 同款 NUL 归一 |

**探针方法论**：`zz_r199_escapes_probe`（land 前删除）逐步验证
`fffdres:child`（FFFD 直查命中）vs `nulres:null`（NUL 查询 miss）——排除「id 侧
序列化丢字符」假设，锁定选择器侧三层各自吞 NUL；逐层修复后 `nulres:child`。

## 二、深项记录（本轮不做，记档）

1. **escapes 剩 2F（lone-surrogate never-match）**：id 含孤立代理项
   （`\ud83dsurrogateFirst`）在 JS→序列化→re-parse 往返中塌缩为 U+FFFD——与
   `#\d83d` 转义反解出的 FFFD 相同 → 误命中（expected null got object）。真浏览器
   内部 UTF-16 字符串保得住孤立代理；本管线 HTML 序列化（UTF-8）天然不能。需
   surrogate-preserving wire（超出 HTML 序列化域）——**深结构记档**。
2. **ParentNode-querySelector-scope 2F**：`:scope` 的 query-root 语义
   （`div.querySelectorAll(':scope > p')` 的 scope=div）需把 scope NodeId 贯穿
   matches 链——`lang_dir.rs:is_scope_element` 注释已明记 follow-up。中等结构
   改动（host 查询 walker 签名族），留独立切片。

## 三、A/B 与全量

- 全量 polyfill **9767P/102F** / native **9767P/102F**（fail 清单 diff IDENTICAL）
- vs R198 基线：**零新增**，fixed 2（escapes NUL 双 subtest）
- `make test` **全绿**（含上轮 SW flake 本轮亦绿）；fmt/clippy（v8 + quickjs 矩阵）干净
- 单测：`zz_r199_nul_preprocessing`（合法性 + 命中 + 裸 `#<FFFD>` 形态 + 无 NUL
  回归守卫）

## 四、commit

`991e17f5d`
