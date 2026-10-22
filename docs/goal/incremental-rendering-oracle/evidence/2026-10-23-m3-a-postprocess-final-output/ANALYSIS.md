# M3-a 拆面：后处理敏感路径冷温最终输出一致 + 溢出面入快照（DC-4 布局面）

**运行身份**：基线 HEAD `1d5624454`（上轮 M2 收口），本轮改动 `crates/layout-engine/src/engine/tests/layout_incremental_oracle.rs`（快照结构扩张 + 3 后处理序列 + 有效性自检）。**纯测试切片，零生产代码变更**。
**环境/守卫**：`./target/test-guard --compile-first … cargo test -p zero-layout-engine`（oracle 9/9，crate 全量 2080/2080）；fmt、clippy、`make test`（见提交说明）。

## 切片定位

slice14 撤用增量臂的根因是"复用 taffy 缓存、不跑全量后处理族"（pipeline/mod.rs:2046 注记）。本切片在后处理最敏感的路径上钉住**冷温全量一致**：未来任何"全量里跳过/重复某后处理 pass"的改动都会被检出。折叠/避让本身的规范正确性属 rendering-compat，本切片只断言双路径一致性。

## 最终输出快照扩张（M1 几何快照 → 完整可观察面）

`GeoSnapshot`：x/y（相对父内容区）+ 边框盒/内容盒宽高 + **overflow_x/overflow_y 枚举**（DC-4"溢出符合预期"的布局面可观察量）。`scroll_x/scroll_y` 布局期恒 0 不入快照（滚动范围由盒树几何蕴含）。浮点容差契约不变：f32 精确相等无 epsilon。

## 序列与结果（oracle 9/9）

| 序列 | 步骤 | 期望 | 结果 |
|---|---|---|---|
| layout_margin_collapse_postprocess_consistent | 相邻兄弟折叠 max(30,20)→max(15,20)，changed=#a | Equal | PASS |
| layout_float_avoidance_postprocess_consistent | float 加宽 100→250、改高 40→120（BFC 块几何随动），changed=#a | Equal | PASS |
| layout_overflow_scroll_extent_consistent | 内容增高 200→300、overflow scroll→visible（溢出枚举变更），changed=#b/#a | Equal | PASS |
| layout_postprocess_fixtures_are_layout_effective | 三对 CSS 变更必须改变全量输出 | 检出变更 | PASS |

**有效性自检的必要性**：对照器对"双路径一致但都陈旧"同样无感（变更未到达输出时 Equal 平凡真）——三个后处理 fixture 各自独立证明 layout-effective（BASE vs 变更 CSS 的全量输出逐盒不等），Equal 结果非空转。同 style oracle 首轮 fixture 笔误的教训（检出器对期望错误同样灵敏）。

## 生产路径运行证据（DC-4/DC-5 复用生产 parity）

`make product-smoke`（welcome.html vs chromium Oracle，DC-13 门禁，exit 0）：全生产管线（style→layout→paint）在产品 fixture 上对齐外部 oracle——本轮及上轮 oracle 切片零生产变更的前提下，该运行同时是 DC-5"关闭检查模式运行生产路径"的实证：生产默认路径在本 goal 引入的全部 instrumentation 仅 `:has()` 回退分支一行计数（实例字段，慢分支），基准 PASS（m2-s2 bench-gate）。

## 边界与未覆盖

- 命中测试几何（DC-3 余项）：hit-test 走 engine/js_dom_bridge（并行流工作面），布局侧已保证盒树几何冷温一致——命中消费的输入是稳定的，命中逻辑本身的对照归跨流协调。
- 半透明帧复用、多次导航/缓存释放（DC-4 主体）：engine pipeline 面，维持跨流协调待办（控制面）。
- 像素容差契约：待 engine paint 面切片（跨流协调）后按输出契约定。
