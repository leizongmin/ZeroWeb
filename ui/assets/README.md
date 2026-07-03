# zero-ui-assets

通用 UI SDK 的资源管理。提供资源标识、变体解析（theme×density）、内存存储和检索能力。浏览器无关。

## 核心类型

| 类型 | 说明 |
|------|------|
| `AssetId` | 资源标识符（`compact_str`） |
| `AssetVariant` | 变体选择器：`VariantTheme {Any, Light, Dark}` × density（1x/2x/3x） |
| `AssetProvider` trait | 资源提供者接口：`load(id, variant) → Result<AssetData, AssetError>` |
| `InMemoryAssets` | 内存资源存储（`.insert(id, data, variant)` + `.load()` fallback 解析） |
| `AssetData` | 资源数据：`Svg(Bytes)` / `Image(Bytes, Format)` / `Json(Value)` / `Raw(Bytes)` |
| `AssetError` | 资源错误：`NotFound` / `VariantNotFound` / `LoadError` |

## Fallback 解析顺序

`InMemoryAssets::load(id, variant)` 按优先级检索：

1. 精确匹配（同 theme × 同 density）
2. 同 density × Any theme
3. 同 theme × 1x density
4. Any theme × 1x density
5. `NotFound`

带 candidate 去重。

## DSL 集成

`ui/dsl` 的 `asset_bridge` 模块将 DSL `{asset: <id>}` 对象解析为 `AssetId`，在 loader 作为 prop 保留。

## 依赖

- `zero-ui-core` / `compact_str` / `hashbrown` / `thiserror`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-assets` — 6 测
- 覆盖：insert/load 精确匹配 + fallback 全链 + 候选去重 / NotFound / variant 构造器
- Coverage 95.24%
