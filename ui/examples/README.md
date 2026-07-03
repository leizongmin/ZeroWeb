# zero-ui-examples

UI SDK 的可复用示例应用。证明 SDK 可被外部程序独立复用（不依赖任何浏览器 crate）。所有示例均经 `WinitDriver` 驱动 retained 闭环。

## 示例

### counter

最简单的 SDK 应用，验证通用 UI SDK 可被外部程序复用：

- `CounterApp`（AppState + reducer + `build_spec`）
- 自定义 `Label` 控件（paint 文本）
- 复用 `ui/widgets::Button`
- 无窗口 headless 测试：点击→emit→reducer→重建→Scene 文案随状态更新
- 稳定 WidgetId 跨重建复用 epoch 断言
- **DC-1 机械验证**：`cargo tree` 零浏览器 crate 依赖（仅 core/render/runtime/widgets）

### form

受控文本输入示例，证明焦点/键盘/校验/IME 组合闭环：

- `TextField`（受控输入：聚焦时键盘→`form.change`→reducer→props.text 回灌）
- `FormApp`（reducer + 校验：非空→"Hello, X!" / 空→"Error: name is required"）
- Tab 聚焦遍历 + Enter 提交 + 点击按钮提交
- ime_rect 查询（caret 绝对坐标）

## 依赖

- `zero-ui-core` / `zero-ui-render` / `zero-ui-runtime` / `zero-ui-widgets` / `zero-ui-adapter-winit`
- 零浏览器业务 crate 依赖（DC-1）

## 测试

- `cargo test -p zero-ui-examples` — 5 集成测（counter 2 + form 3）+ 所有示例经 WinitDriver 驱动
- 实跑：Count 0→1→2→3→2；输入 'Ada' → submit → "Hello, Ada!"

## 运行

```bash
# counter 示例
cargo run -p zero-ui-examples --example counter

# form 示例
cargo run -p zero-ui-examples --example form
```
