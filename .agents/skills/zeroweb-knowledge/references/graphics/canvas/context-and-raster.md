# 路径、状态、像素与 OffscreenCanvas

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 `CanvasContext`、`Path2D`、填充/描边光栅、状态栈、像素读写和 Rust `OffscreenCanvas`。页面脚本代理、资源导出与 CSS 上屏看同级 DOM 桥主题。这里的 Rust 图元流和像素缓冲是两个输出，不应混为一个。

## Gotchas
- 当前路径的点在追加时应用 CTM，独立 Path2D 在绘制时变换；把 Path2D 提前变换再交给绘制会双重 CTM。 (`crates/canvas/src/path.rs`, `crates/canvas/src/context/context_impl.rs`)
- `Path2D::transformed` 对半径用 determinant 的平方根近似缩放，非均匀缩放加旋转不保证精确椭圆；不能由 API 存在推断任意仿射曲线等价。 (`crates/canvas/src/path.rs`)
- `round_rect` 明确产生新子路径和 ClosePath；缺闭合标记会影响描边 wrap-around join/cap，不能只检查 fill 像素。 (`crates/canvas/src/path.rs`, `crates/canvas/src/context/tests/raster.rs`, `git:c218f196911bcbb5c0b96d01055cf3d39581717f`)
- clip 是已有裁剪区域的交集，空路径 clip 让后续绘制全裁；当前路径本身不被清空，save/restore 还须保存 clip_paths 列表。 (`crates/canvas/src/context/context_impl.rs`)
- save/restore 保存绘图状态而不保存 bitmap 或当前 path，空状态栈 restore 无动作；不能把 restore 用作撤销已绘制像素。 (`crates/canvas/src/context/context_impl.rs`)
- resize 用 new 重置状态、路径和像素，但保留 color space 与 float16 模式；共享字体 loader 被重建丢弃，DOM host 的 resizeContext 会重新注入。 (`crates/canvas/src/context/context_impl.rs`, `crates/engine/src/js_dom_bridge/canvas.rs`)
- `get_image_data` 支持负宽高和画布外透明黑，读取不应用 CTM；`put_image_data` 直接按像素行写，不执行绘图 transform/clip/alpha 合成。 (`crates/canvas/src/context/context_impl.rs`)

## Architecture
- flatten 曲线使用自适应细分而非固定八段，描边边界还考虑像素方形与带状几何相交；粗线巨坐标故障不能只提高中心点命中阈值。 (`crates/canvas/src/context/raster.rs`, `crates/canvas/src/path.rs`, `crates/canvas/src/context/tests/raster.rs`, `git:e332878f4d43f2cf7e10a7267b38bcdb319af533`)
- 填充/命中有 NonZero 和 EvenOdd 分支，rule-aware 命中拒绝非有限坐标且边界算内部；旧 `is_point_in_path` 的射线奇偶实现不能代替默认 NonZero host 语义。 (`crates/canvas/src/context/context_impl.rs`, `crates/canvas/src/context/raster.rs`, `crates/canvas/src/context/offscreen.rs`)
- stroke 命中将查询和顶点逆变换到用户空间，dash 按用户空间长度累计；直接用设备距离比较 line_width 会破坏 scaled dashes。 (`crates/canvas/src/context/context_impl.rs`)
- OffscreenCanvas 持有同一 context，transfer 返回 ImageData 快照并只清 bitmap、保留状态；Rust 尚无独立 ImageBitmap 类型，不能把这个返回值等同完整 JS 对象生命周期。 (`crates/canvas/src/context/offscreen.rs`)

## Patterns
- shadow blur 的计算半径封顶 8192，避免极大 blur 值导致宽高算术和分配失控；维度封顶并不证明所有像素操作都有总内存预算。 (`crates/canvas/src/context/raster.rs`, `crates/engine/src/js_dom_bridge/canvas.rs`)
- float16 模式实际另存 f32 buffer，`clear_bitmap` 当前只清 u8 pixel_buffer；float16 transfer/clear 的完整清理语义需额外核验，不能根据方法注释宣称已覆盖。 (`crates/canvas/src/context/context_impl.rs`, `crates/canvas/src/context/offscreen.rs`)

## Testing Strategy
- raster 常驻判据包括自适应曲线的边界像素、零长段剪除和闭合 roundRect 粗描边中心覆盖，path_tests 验证路径命令；本轮仅阅读，未实跑。 (`crates/canvas/src/context/tests/raster.rs`, `crates/canvas/src/path_tests.rs`)
