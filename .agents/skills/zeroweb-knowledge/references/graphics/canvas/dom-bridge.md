# DOM Canvas、文字、图像桥与上屏

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 shim → `__zw_canvas_op` → `CanvasRegistry` → context bitmap → painter `ImagePrimitive` 与资源缓存。排查 JS 操作成功但 canvas 空白、动态创建未进入布局、resize 后文字消失或图像 wire 失真时读这里。通用脚本事件循环归 scripting。

## Gotchas
- DOM proxy 的 getContext 仅接受 2d，缺必参抛 TypeError，其他类型或未注册 host 返回 null；重复 getContext 复用 per-element context，不重新应用新配置。 (`crates/engine/src/js_dom_shim/part04.js`)
- DOM 与动态创建 Canvas 是两套对象入口：standalone `_zwMakeCanvas` 创建时同步 host handle，才能让 append mutation 进入布局；有 JS Canvas 对象但没有 host 元素不保证上屏。 (`crates/engine/src/js_dom_shim/part05.js`, `crates/engine/src/js_dom_shim/part04.js`)
- JS width/height 的反射值与 host bitmap 维度独立，getContext2d/resizeContext 把 bitmap 每维钳到 16384；属性巨大不代表实际分配同尺寸，封顶也不等于像素操作的总预算。 (`crates/engine/src/js_dom_bridge/canvas.rs`)
- context、gradient、Path2D 各从 1 分配独立 ID，数字重合不是同类对象；CanvasRegistry 的 grad_refs 在 addColorStop 后重放引用样式，使渐变对象具有 live 更新语义。 (`crates/engine/src/js_dom_bridge/canvas.rs`)
- resizeContext 重建 CanvasContext 后重新注入 registry 的共享 FontLoader；漏此步骤会使 resize 后 fillText 丢失字体，像素清空与文字资源保活必须分别处理。 (`crates/engine/src/js_dom_bridge/canvas.rs`)
- Rust `snapshot_rgba` 只要 u8 像素有任一非零才导出；纯透明黑 canvas 不产图元，只有 recorded primitives 不一定已有可导出的 bitmap。 (`crates/canvas/src/context/context_impl.rs`, `crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/pipeline/mod.rs`)

## Architecture
- DOM getContext 把内部 ctx ID 写到 `data-zw-canvas-ctx`，painter 从 registry 找快照，产 image_key=ctx_id 和 canvas_images；调用方仍须注入 ImageCache，单有 ImagePrimitive 无像素。 (`crates/engine/src/js_dom_shim/part04.js`, `crates/engine/src/paint/painter/mod.rs`, `crates/webview/src/webview.rs`)
- Canvas 上屏以 context 位图宽高为固有尺寸，内容原点 round 到整数像素，再应用 object-fit/object-position 并 clip 回容器；CSS box 与 bitmap 分辨率不能互相替代。 (`crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/paint/painter/text_image.rs`)
- host 创建 context 注入共享字体 loader，measure/fillText 与 render-foundation 的真实字体栈衔接；下载字体成功仍须核对 registry loader 是否同步，而不是只看页面 resolver。 (`crates/engine/src/js_dom_bridge/canvas.rs`, `crates/canvas/src/context/context_impl.rs`)
- drawImage 普通/scaled 桥将源 color space 转成 context 空间，sliced 分支当前不调用同一 convert_buffer；不能默认三种 overload 的色彩处理等价。 (`crates/engine/src/js_dom_bridge/canvas.rs`)
- host toDataURL 只编码 PNG，再通过逗号十进制串交给 shim 转 base64；编码失败返回空串并回落 `data:,`，不是 native JPEG/WebP encoder。 (`crates/engine/src/js_dom_bridge/canvas.rs`, `crates/engine/src/js_dom_shim/part04.js`)

## Security
- 当前 CanvasRegistry/context 像素回读接口不携 source origin 或 origin-clean 标志，所读 getImageData/toDataURL host 分支也未做跨源污点 gate；不能将 crossOrigin 属性反射或 SVG 污点机制当作 Canvas 导出已安全。 (`crates/engine/src/js_dom_bridge/canvas.rs`, `crates/canvas/src/context/types.rs`, `crates/engine/src/js_dom_shim/part04.js`)
- drawImage wire 传像素和尺寸而非可信来源身份，色彩空间转换不能证明同源校验；完整页面跨源传播与 SecurityError 还需独立追踪和运行证据。 (`crates/engine/src/js_dom_bridge/canvas.rs`)

## Testing Strategy
- pipeline 的 canvas 常驻测试预填 registry 并断言图元 key、尺寸和红像素，空 canvas 测试断言无图元；这证明局部桥接判据，不覆盖真实 JS、网络 taint 或生产 GPU 呈现。 (`crates/engine/src/pipeline/mod.rs`)
- `canvas_render` 中名为 WebView renders page with canvas 的测试实际 HTML 只有 div 且仅断言 timing，`e2e_canvas_dom` 多个 Canvas 测试只调用 API；不可把测试名称作为端到端像素证据。 (`tests/integration/src/canvas_render.rs`, `tests/integration/src/e2e_canvas_dom.rs`)
