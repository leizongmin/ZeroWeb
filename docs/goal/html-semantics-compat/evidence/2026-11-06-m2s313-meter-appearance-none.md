# M2 s313 — meter `appearance: none` 原生外观关闭：value 条 appearance 门 + track chrome 抑制（the-meter-element 簇 4 mismatch 红 ✗0.00% → 全簇 5/5 全绿收口）

日期：2026-11-06。切片号 s313（承 s312 下一步候选①小簇排队首位，勘根→施工型切片）。

## 勘根/边界确认

- **簇红页形态**：the-meter-element 5 reftest 页 = 4 mismatch 红（全 ✗0.00%，0 px 差）
  + 1 match 绿（meter-min-rendering ✓0.00%）。mismatch 四页同构：test = meter 带
  `style="appearance: none"`，ref = 同属性裸 meter——mismatch 通过只需两页渲染可分
  （>0.01% 像素差）。
- **根因三层**：①value 填充条画笔（controls.rs `paint_progress_meter_value`）完全
  无视 appearance（`_style` 未消费），带值页（value=0.5/0.9 三页）两侧同绘条；②UA
  track chrome（R1670 UA 表 bg #efefef + 1px border）为普通计算值，两侧同绘；③引擎
  既有 appearance 全链（css-parser `parse_appearance` / ComputedStyle.appearance /
  R5304 `effective_appearance` used 值解析 + paint_appearance widget 臂）对
  Meter/ProgressBar/Listbox/Menulist 变体按 R4249 兼容别名早退，meter 无独立消费面。
- **勘根修正（中途回归闭环）**：首版设计含 style-system UA 表 meter/progress 补
  `appearance: auto` 声明（button/checkbox 先例同构）；单测红灯暴露设计前提错误——
  `ComputedStyle::default()` 的 appearance 初值实为 **Auto**（default_impl.rs:336，
  types.rs 枚举注释「None（默认值）」为陈旧错误描述），裸 meter 本就计算 Auto，UA
  声明零行为增量。回退该刀，最终修复全落 engine paint 单侧（渲染流域 crate 零触碰）。
- **语料盘点**：corpus 全树 meter/progress + `appearance:none` 消费面仅此 4 页
  （fieldset-appearance-none-manual 为手动页；select customizable-select 的
  `appearance: base-select` 非本门语义）；the-progress-element 无 reftest 页
  （0 红 0 绿，回归面为空）。

## 施工（crates/engine/src/paint/painter/{controls.rs,mod.rs}，engine 单侧）

1. **value 条 appearance 门**（controls.rs）：`paint_progress_meter_value` 于 tag
   判定后经 `effective_appearance` 判 None 即早退（原生映射恒非 None，effective 判定
   与原始 computed 一致）；`_style` 参数转正。
2. **track chrome 抑制**（mod.rs 新 helper `widget_track_chrome_suppressed`）：
   computed appearance None + tag ∈ {meter, progress} → 主路径（paint_node_inner）
   与脏矩形增量路径（paint_node_in_rect）的背景/边框装饰条件各加 `&& !track_chrome_off`
   （4 处条件门）。`none` = 素盒（css-ui-4 §appearance-switching：原生外观 = track
   盒与 value 条整体）。
3. 单测 2 面：content_objectfit.rs `paint_meter_appearance_none_no_value_bar`（作者
   关闭形显式置 None，value 条不绘；R1671 既有 4 测试保持 bare default=Auto 基线）；
   pipeline.rs `test_s313_meter_appearance_none_chrome_suppressed`（裸 meter ≥2 fill
   全链 vs none 侧全图元桶 0——UA 声明→计算→抑制全链验证）。

## 语义边界（记录）

- R4249 注记维持：Meter/ProgressBar 变体在 `paint_appearance` 仍早退（其 widget
  消费面在 controls.rs 专属画笔，非通用指示器臂）；作者对非 meter/progress 元素设
  `appearance: meter` 不产生 meter 外观（同现状）。
- `appearance` 其余变体（checkbox/radio/button 族）既有臂不涉；渲染流 R5359
  `appearance: base-select`（parse_appearance 无该变体，声明回落）为渲染流域
  active workfront，本切片不触碰。
- types.rs 枚举注释「None（默认值）」与 default_impl.rs 实际初值 Auto 矛盾——
  渲染流域 crate 既有注释失准，只提及不代改（rule 4）。
- perf-gate 豁免：改动为 meter/progress 元素专属分支（UA 声明无、共享热路径仅 2 个
  谓词赋值 + 4 个布尔条件），无 bench 面；承 s311/s312 先例。

## 验证

- 簇 solo reftest：**the-meter-element 5/5 全绿**——4 mismatch 页 ✗0.00% → **✓0.31%**
  （test 侧素盒 vs ref 侧 track+条，像素差 >0.01% 阈值），meter-min-rendering
  ✓0.00% 保持绿（match 型两侧同绘零扰动）。日志
  [logs/s313-meter-after.log](logs/s313-meter-after.log)（修前基线
  logs/s313-meter-initial.log：4✗0.00%+1✓）。
- 簇外回归：select 保持页 appearance-base-select-other-elements ✓0.00%；
  the-progress-element 无 reftest 面（0 failed）。
- 单测（test-guard 包裹）：zero-engine `--lib` 全量 **3778P/7F**——较 s312 基线
  3776P +2P（本切片新 2 案），7F 恰为已知集（CSSOM cssText 5 键 p56/p99s/r2809/
  r2810/r3276 + 跨流 websocket 2 键 t8k×2，s312 stash 归因 ba03fd7ff 维持），
  零新增失败。日志 [logs/s313-engine-lib.log](logs/s313-engine-lib.log)。
- 产品 smoke（表单/UA 面）：make product-smoke-legacy 42 fixture
  **struct-check failures: 0**；make product-smoke **PASS**。
- clippy：`cargo clippy -p zero-engine --lib -- -D warnings` 零告警（首版 doc 注释
  `+ value` 行首误判列表项已改写）。
- make test（guarded-test，后台全量）：**80 腿 ok + engine lib 3778P/7F**（7F 同单测
  已知集），guarded-test 退出 101（唯一失败腿即上已知集），其余 80 腿零回归。
  日志 [logs/s313-make-test.log](logs/s313-make-test.log)。

## 证据

日志（docs/goal/html-semantics-compat/evidence/logs/）：
- s313-meter-initial.log（修前 4✗0.00% 基线）
- s313-meter-after.log（簇 5/5 全绿终态）
- s313-progress.log / s313-select.log（簇外回归）
- s313-engine-lib.log（3778P/7F 已知集）
- s313-legacy-smoke.log / s313-smoke.log（产品 smoke 双绿）
- s313-make-test.log（后台全量落盘）

代码锚：crates/engine/src/paint/painter/controls.rs（value 条 appearance 门）；
crates/engine/src/paint/painter/mod.rs（widget_track_chrome_suppressed + 双路径
4 条件门）；crates/engine/src/paint/tests/content_objectfit.rs（1 新案 + 前提修正
注记）；crates/engine/src/tests/pipeline.rs（s313 1 案）。

注记：本切片 diff +100/−3 全落 engine 4 文件；style-system 中途回归后零触碰。

## 下一步候选

1. 小簇排队其余（input 3/link 3/embed 3/object 2/option 1/iframe 1/wbr 1）逐簇
   勘根；br 1 勘根前先 solo 复核 editing-compat br 刀（fe760dc57）外溢现势。
2. shaped 值行 ellipsis 接线（FIXME 承接，待 corpus 真实案驱动）。
3. （挂账承前）生产动画重绘驱动 / 隔离通道帧表 IPC 扩展 / paint IPC duplicate
   动画携带——渲染流 + 协议面协同候选，非本流域内施工。
