# zero-ui-i18n

通用 UI SDK 的国际化支持层。提供 locale 管理、文案 catalog、参数替换、plural 规则和 RTL 方向判定等基础设施。浏览器无关，不内置任何浏览器领域文案。

## 架构位置

```
ui/i18n ←── ui/runtime（I18nRuntime）
       ←── ui/dsl（i18n 桥接：DSL `i18n:` 对象→LocalizedText）
       ←── browser-ui/chrome/i18n（浏览器具体文案 catalog）
```

## 核心模块

| 模块 | 职责 |
|------|------|
| `catalog` | `MessageCatalog` + `CatalogStore`：多 locale catalog 存储、resolve 链（精确→fallback→根→MissingKey diagnostic） |
| `locale` | `LocaleId`：locale 标识符、parent 派生、fallback_chain 构建 |
| `message` | `LocalizedText`：Literal / Plural / Message(MessageRef) 三种文案引用方式 |
| `formatter` | 参数替换（`{name}` → 运行时值）、plural 变体选择、diagnostic 产出 |
| `plural` | CLDR cardinal 规则手写子集（未选 ICU4X/Fluent，零新依赖）：英语/阿拉伯语/俄语/乌克兰语/白俄语/波兰语 |
| `direction` | 文本方向判定：`TextDirection::Ltr` / `Rtl`（支持 ISO 639-1 + ckb/dv/nqo 三字母 RTL 语言） |
| `fallback` | Fallback chain 构建与解析 |
| `diagnostics` | `I18nDiagnostic`：MissingKey / FallbackUsed / PluralFallback |

## plural 覆盖规则

| 语种 | 变体 | 说明 |
|------|------|------|
| 英语 / root | one, other | 根规则 |
| 阿拉伯语 | zero, one, two, few, many, other | 完整 6 类 |
| 俄语/乌克兰/白俄 | one, few, many | 整数无 other |
| 波兰语 | one, few, many | 集合代数证明整数无 other → `else=>Many` |

未覆盖语种回落英语规则。

## 依赖

- `serde` / `compact_str` / `hashbrown` / `thiserror`
- 零浏览器业务 crate 依赖（DC-1）

## 测试

- `cargo test -p zero-ui-i18n` — 23 测
- 覆盖：plural 多语种 / RTL 方向（含 ckb/dv/nqo） / catalog resolve+fallback+diagnostic / formatter 参数替换 / locale parent / fallback_chain

## 深度审查

2026-07-03 全 crate 审查，逐字句核对 CLDR cardinal 规则正确。修复 direction.rs RTL 集合补 ckb/dv/nqo。1 处文档自洽修正。
