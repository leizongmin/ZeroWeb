# M2 s312 — textarea 宽度软折行（UA pre-wrap 显示语义）+ 控件 ellipsis：white-space 型分派 + 空格断点视觉行 + text-overflow 截断（placeholder-white-space.tentative ✗0.00% → ✓0.26%，the-textarea-element 簇 7/7 全绿收口）

日期：2026-11-06。切片号 s312（承 s311 下一步候选①，勘根→施工型切片）。

## 勘根/边界确认

- **簇尾红页形态**：placeholder-white-space.tentative.html mismatch 型 ✗0.00%
  （0 px 差）——test 页（`white-space: nowrap; text-overflow: ellipsis;
  max-width: 100px`）与 notref（同内容仅缺前两行）同构渲染。s311 后控件路径
  只按 \n 拆行：无宽度折行（textarea UA pre-wrap 未实现）亦无控件 ellipsis
  （R1660 既有注释「本 slice 不做 ellipsis」）→ 两侧单行溢出恒等。
- **layout 面核验（像素测量）**：dump 实测 test 页 textarea 盒 x≈8..113
  （≈100px max-width + 边框）——layout 对表单控件 max-width 已尊重
  （rendering 流移交面不成立）；字形溢出至 x≈345（无截断面）。
- **fixture 内容**：placeholder = "This is a really long string that needs to
  be truncated"（含空格 → 空格断点折行可用；mismatch 通过只需两页渲染可分）。
- **样式字段**：ComputedStyle `white_space: WhiteSpaceValue`（全变体齐备）+
  `text_overflow: TextOverflowValue` 均为既有解析面；style-system UA 表
  textarea 无 white-space 规则（上游 Chromium UA 为 pre-wrap）——UA 缺省
  折行语义由控件路径自持（s311 UA 缺省色 #757575 同款先例），不改
  style-system（渲染流域不越界）。

## 施工（crates/engine/src/paint/painter/text.rs，paint_input_value）

1. **视觉行预拆**：逻辑行（\n 拆分 + CR 归一）后按 wrap 型分派——
   `wrap_enabled = is_textarea && !matches!(white_space, Pre|Nowrap)`
   （Normal/PreWrap/PreLine/BreakSpaces 折行；input 恒单行；Pre/Nowrap 不折）。
   溢出行在空格断点拆视觉子行（断点空格留前行尾），无空格可断时按字符硬断
   （overflow-wrap:normal 近似，控件路径无断词词典；硬断至少推进一个 char
   防死循环）。视觉行携带值内 UTF-16 起点。
2. **循环改视觉行驱动**：行几何（line_top/block_height/caret）随视觉行；
   行首 boundary 与 shaped 簇边界 utf16 基准随视觉行起点（折行续行行首有
   boundary，caret 映射更准）；R5052 整行选区判定与 boundary_lines 配对仍按
   逻辑行（line_ranges/boundary_value 原值口径不变）；孤儿累加
   （line_utf16_start 行尾推进）删除（视觉行起点取代）。
3. **ellipsis 截断面**：非 shaped 行（占位符臂恒非 shaped）`text-overflow:
   ellipsis` + 行溢出（total_w > content 宽）时预留 `…` 实字形宽截断并以
   U+2026 收尾；短行零影响。shaped 值行 ellipsis 未接线（FIXME 挂账——
   shaped 簇边界与截断交互未涉及 corpus 案）。
4. 单测 2 案（pipeline.rs）：`test_s312_textarea_width_soft_wrap`（wrap 型
   ≥2 基线且不溢出盒右缘 / nowrap 单行）、`test_s312_textarea_ellipsis_clip`
   （省略号收尾 + 全字形不越盒右缘）。

## 语义边界（记录）

- shaped 值行（textarea 非 placeholder LTR 值文本）ellipsis 未接线——
  FIXME 挂账（无 corpus 驱动案；截断与簇边界映射交互待真实案驱动）。
- PreLine 未做空格折叠（保留空白近似）；行尾空格悬挂（pre-wrap hang）未实现
  （断点空格留前行尾、零墨迹，像素面无差）。
- 跨流信号（rule 9/10 归因，本流不硬解）：engine lib 全量新现 2 键
  `test_t8k_websocket_error_nonterminal` /
  `test_t8k_websocket_event_flow_open_message_close` 确定性失败——stash 至
  干净 fa4696ebc 复测同败，归因并行流 ba03fd7ff（net-api M3 ws interfaces）
  落库即败面，与 s312 绘制变更无交集；该流账本 +45P 零回归口径与此树实测
  不符，属共享面碰头信号，记控制面待该流核认。

## 验证

- 簇 solo reftest：**the-textarea-element 7/7 全绿**——
  placeholder-white-space.tentative ✗0.00% → **✓0.26%**（mismatch 型：
  test 单行 ellipsis vs notref 折行多行，像素差 >0.01% 阈值 ✓），multiline
  三页 ✓0.00%、textarea-newline-bidi ✓0.01%、wrap-reflect ✓0.00% 保持绿
  （零回归）。日志 [logs/s312-solo-textarea.log](logs/s312-solo-textarea.log)。
- 簇外回归：select 页 appearance-base-select-other-elements ✓0.00% 保持绿
  （语料全树 66 textarea 页 reftest 型仅 2 处均覆盖）。
- 单测（test-guard 包裹）：zero-engine `--lib` 全量 **3776P/7F**——5F 为
  已知 CSSOM cssText 集（s310 基线逐键相同）+ 2F 为跨流 websocket 键（见上，
  stash 归因非本切片）；s312 新 2 案过 + s311 3 案过（3776P 含全量新案）。
  日志 [logs/s312-engine-lib.log](logs/s312-engine-lib.log)。
- 产品 smoke（表单/legacy 面）：make product-smoke-legacy 42 fixture
  **struct-check failures: 0**（无结构性回归）。
- make test（guarded-test，后台全量）：**80 腿 test result: ok 全绿 + engine
  lib 腿 3776P/7F**（5F 已知 CSSOM 集 + 2F 跨流 websocket 键，见上 stash
  归因；零新增失败；make 止于该腿 exit 101 已知形态）。
  日志 [logs/s312-make-test.log](logs/s312-make-test.log)。
- clippy：`cargo clippy -p zero-engine --lib` 零告警。

## 证据

日志（docs/goal/html-semantics-compat/evidence/logs/）：
- s312-solo-textarea.log（簇 7/7 全绿终态）
- s312-engine-lib.log（guard 单测全量 + 跨流 2 键失败态）
- s312-make-test.log（guarded make test 全腿，后台落盘）

代码锚：crates/engine/src/paint/painter/text.rs（视觉行预拆 + 循环视觉驱动 +
ellipsis 截断面）；crates/engine/src/tests/pipeline.rs（s312 2 案）。

注记：text.rs（6327 行）/pipeline.rs（2936 行）均早已超 2000 行准则线
（既有面，本切片增量 ~90 行不触拆分，只提及）。

## 下一步候选

1. 小簇排队其余（meter 4/input 3/link 3/embed 3/object 2/option 1/iframe 1/
   wbr 1）逐簇勘根；br 1 勘根前先 solo 复核 editing-compat br 刀
   （fe760dc57）外溢现势。
2. shaped 值行 ellipsis 接线（FIXME 承接，待 corpus 真实案驱动）。
3. （挂账承前）生产动画重绘驱动 / 隔离通道帧表 IPC 扩展 / paint IPC
   duplicate 动画携带——渲染流 + 协议面协同候选，非本流域内施工。
