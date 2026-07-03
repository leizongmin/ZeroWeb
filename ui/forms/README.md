# zero-ui-forms

通用 UI SDK 的表单能力。提供字段校验、脏状态跟踪、触摸跟踪和提交生命周期管理。浏览器无关。

## 核心类型

| 类型 | 说明 |
|------|------|
| `Validator` trait | 校验器接口（`validate(value:String)→Result<(),ValidationError>`） |
| `ValidationError` | 校验错误（`message: String`） |
| `FieldState` | 字段状态（`value` / `error` / `dirty` / `touched` / `valid`；`touch()` / `validate()`） |
| `FormState` | 表单状态（builder：`.field(id, initial, validator)` + `.submit()` 全量校验 + 错误收集） |
| `Required` | 必填校验器（非空检查，trim + chars） |
| `MinLength` | 最小长度校验器（chars count） |
| `MaxLength` | 最大长度校验器 |
| `All` | 组合校验器（多个校验器全部执行后合并错误） |
| `SubmitResult` | 提交结果：`Ok(HashMap<Id, String>)` / `Err(HashMap<Id, ValidationError>)` |

## 使用场景

DC-13 §8.4.1B 设置页、偏好编辑、搜索表单等。

## 依赖

- `zero-ui-core` + `compact_str`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-forms` — 7 测
- 覆盖：Required/MinLength/MaxLength/All 校验器 / FieldState dirty+touched+error+validate / FormState submit lifecycle + reset
- Coverage 91.91%
