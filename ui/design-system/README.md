# zero-ui-design-system

通用 UI SDK 的设计系统。定义样式 token（密度/圆角/动效/字号层级）和风格包（StylePack）。首个风格包 `zero_default()` 已交付。浏览器无关。

## 核心类型

| 类型 | 说明 |
|------|------|
| `MotionTokens` | 动效 token（duration / easing 预设） |
| `SpacingTokens` | 间距 token（`xs` / `sm` / `md` / `lg` / `xl` / `xxl`，密度缩放系数 `scaled(density)`） |
| `RadiusTokens` | 圆角 token（`none` / `sm` / `md` / `lg` / `full`） |
| `TypographyScale` | 字号层级（`body` / `body_small` / `caption` / `h1`~`h6` / `label`，可 `scaled(text_scale)` 缩放） |
| `ComponentVariant` | 组件变体描述（预设风格包内的组件变体） |
| `StylePack` | **风格包**聚合（name + motion + spacing + radius + typography，Serde 可序列化） |
| `Density` | 布局密度枚举（Material 风格：compact / comfortable） |

## 风格包

| 函数 | 说明 |
|------|------|
| `zero_default()` | Zero 默认风格包（DC-15 首个风格包） |
| `zero_compact()` | Zero 紧凑风格包（更小间距） |

风格包只负责几何/动效/字号密度 token，颜色由 `ui/core::SemanticTokens` 提供（DC-5 边界）。

## 依赖

- `zero-ui-core` + `serde`
- dev-dep：`serde_json`（序列化 roundtrip 测试）
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-design-system` — 7 测
- 覆盖：zero_default / zero_compact / SpacingTokens scaled / StylePack serde roundtrip / TypographyScale scaled
- Coverage 89.78%
