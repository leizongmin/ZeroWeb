# M2 s311 — textarea 多行显示语义接线：子文本 defaultValue 走控件路径 + IFC 面抑制 + CR 显示归一 + 顶对齐（multiline 三页 ✗1.15% → ✓0.00%）+ 语料 placeholder.css 补件挂账

日期：2026-11-06。切片号 s311（承 s310 下一步候选①小簇排队首簇，勘根→施工型切片）。

## 勘根/边界确认

- **簇基线**：solo reftest `FILTER=the-textarea-element` 7 案 4F/3✓——
  multiline-placeholder{,-cr,-crlf}.html 均 ✗1.15%（5508/480000 px，同一差值
  提示同根因）、placeholder-white-space.tentative.html ✗0.00%（mismatch 型：
  0 px 差，期望 >0.01%）；绿锚 textarea-newline-bidi 0.01%、wrap-reflect-1a/1b
  0.00%。日志 [logs/s311-solo-textarea.log](logs/s311-solo-textarea.log)。
- **语料缺口（三红页叠加面）**：4 页（3 test + ref）引用
  `support/placeholder.css`，语料整目录无 support/——上游原件
  （188B，md5 07368e84cb705a96ff176ec45652a4db）对称归一两侧着色
  （`textarea.placeholder, textarea::placeholder { color: GrayText; opacity: 1 }`）。
  缺席时任何引擎都失衡（test 侧占位灰 / ref 侧正文另一色）。
- **像素取证（REFTEST_DUMP）**：test 侧（placeholder attr）控件路径按 \n 拆行
  4 行（含空行）居中灰字；ref 侧（子文本）被通用 IFC 面绘制——white-space
  normal 塌行折行成 2 行顶排深字（"this is a multiline"/"placeholder"），
  \n 行结构丢失。textarea-newline-bidi 绿页为 test/ref 双 textarea 对称页
  （同路径恒等，从不检验换行语义，绿不可作换行正确性证据）。
- **GrayText 假设证伪**：勘根中段曾疑系统色未支持——实为
  `crates/css-parser/src/values/named_colors.rs:181` 系统色族齐备
  （`"graytext"|"greytext" => rgba(128,128,128)`，CSS Color 4 §system-colors +
  deprecated 映射表），渲染流移交面不成立。
- **引擎显示语义缺陷（主因）**：`paint_input_value` textarea 臂已有
  `doc.text_content` 种子（R2996 模型），但无 harness 值时非空种子臂
  `return` → 子文本漏给通用 IFC 面塌行绘制；`paint_text` 的 textarea 抑制门
  仅在 harness 值在场时闭合，双绘风险并存。CR 面：`&#xd;` 字符引用绕过
  HTML 输入流预处理进入 placeholder/raw value，显示面仅按 \n 拆行，CR 残留
  行内。垂直对齐：textarea 多行块沿用 content box 垂直居中（R3254-M11 单行
  input 口径），与 Chromium UA 顶对齐语义不符。
- **white-space 页独立面**：与 notref 仅差 `white-space:nowrap +
  text-overflow:ellipsis` 两行；本引擎控件路径只按 \n 拆行、无宽度软折行
  （textarea UA pre-wrap 语义未实现）亦无控件内 ellipsis
  （paint_input_value 注释「本 slice 不做 ellipsis」R1660 既有记录）→
  两页同构渲染 0 px 差，mismatch 恒败。

## 施工

**语料补件（gitignored 本地数据 + 脚本挂账）**：

1. `support/placeholder.css` 上游原件（pin efcaa5bb87，gh api contents 取件）
   补入语料 4 页依赖位；`scripts/update-wpt-data.sh` 头部补件挂账段追加条目
   （升级新 ref 后需重新补，s288/s304 同款惯例）。

**引擎绘制接线（crates/engine/src/paint/painter/text.rs，4 点）**：

2. textarea 臂非 return 化：无 harness 值时非空种子（子文本）同样走控件路径
   `（current_value, false）`，空值回落 placeholder attr 不变；显示语义锚
   HTML textarea（显示值 = raw value 按 \n 拆行）。
3. `paint_text` textarea 抑制门恒闭（去 harness 值前提）——子树文本恒由控件
   路径单份绘制，IFC 面整盒跳过（双绘风险同时消除）。
4. 显示换行归一：textarea label 在拆行前 CR LF/CR → LF（仅显示面；
   boundary_value 保持原值，选区/边界 UTF-16 偏移含 CR 计数不失配）。
5. textarea 多行块顶对齐（`block_top = content_y`），input 单行维持垂直居中
   （R3254-M11 原口径不变）。

**单测（crates/engine/src/tests/pipeline.rs，3 案）**：

6. 子文本控件路径多行（\n 行结构 2 基线 + 字符单份不双绘）、`&#xd;` 实体
   placeholder 换行（2 基线）、rows=1/5 单行首基线顶对齐一致。

## 语义边界（记录）

- textarea 宽度软折行（UA pre-wrap）+ 控件内 ellipsis 未实现——
  placeholder-white-space.tentative.html（mismatch 型）维持红，csswg#6669
  tentative 面；实域影响：textarea 长行溢出不折行。挂账候选（施工量中等：
  控件路径宽度折行循环 + 折行与 boundary/selection 映射交互）。
- select/option 子树在 converter 无特例（与 textarea 修复前同构泄漏面）——
  本切片不动（无簇内测试驱动，只记录观察）。
- harness 值路径（form_control_values/compositions）行为不变：抑制门原已
  闭合、臂原已（current_value, false），零行为面外移。

## 验证

- 簇 solo reftest（补件 + 接线后）：`FILTER=the-textarea-element` 7 案
  **6✓/1✗**——multiline 三页 ✓**0.00%** 完美匹配（✗1.15% → ✓0.00%）、
  textarea-newline-bidi ✓0.01% / wrap-reflect ✓0.00% 保持绿（零回归）、
  placeholder-white-space.tentative ✗0.00%（mismatch 面，挂账）。
  连续两轮同形（静态页确定性）。日志 [logs/s311-solo-textarea-postcss.log](logs/s311-solo-textarea-postcss.log)。
- 簇外回归（语料全树 66 页含 textarea，reftest 型仅 2 处）：select 页
  appearance-base-select-other-elements.html ✓0.00% 保持绿。
- 单测（test-guard 包裹）：zero-engine `--lib` 全量 **3773P/5F**——5F 与
  s310 基线已知集逐键相同（CSSOM cssText 面 4 键 + p56/p99s 族，r5296 本轮
  通过），**零新增失败**；s311 新 3 案全过。日志
  [logs/s311-engine-lib.log](logs/s311-engine-lib.log)。
- clippy：`cargo clippy -p zero-engine --lib` 零告警。
- make test（guarded-test，后台全量）：**80 腿 test result: ok 全绿 + engine
  lib 腿 3776P/5F**（5F = 已知 CSSOM cssText 5 键逐键相同，零新增失败；
  3776P 含 s311 新 3 案；make 止于该腿 exit 101，与 s310 已知形态同）。
  日志 [logs/s311-make-test.log](logs/s311-make-test.log)。
- fmt 共享门禁顺带修复：make test fmt 前置腿暴露并行流 net-api 已落库
  testharness.rs 13 处格式违规阻断全腿（该流 1607ab946/2d7f3ef3f 落库面），
  `cargo fmt --all` 语义保持修复（rustfmt token 不变、行合并重排；仅 2 文件
  触达：testharness.rs + 本切片 pipeline.rs 追加块）。

## 证据

日志（docs/goal/html-semantics-compat/evidence/logs/）：
- s311-solo-textarea.log（勘根基线：4F/3✓ + 差值形态）
- s311-solo-textarea-postcss.log（补件后 4F→1F 过渡态 + 接线后终态）
- s311-engine-lib.log（guard 单测全量）
- s311-make-test.log（guarded make test 全腿，后台落盘）

代码锚：crates/engine/src/paint/painter/text.rs（textarea 臂/paint_text 门/
CR 归一/顶对齐 4 点）；crates/engine/src/tests/pipeline.rs（s311 3 案）；
scripts/update-wpt-data.sh（补件挂账条目）；
tests/wpt-runner/wpt-data/html/semantics/forms/the-textarea-element/support/
placeholder.css（语料本地补件，gitignore 面）。

注记：text.rs（6255 行）/pipeline.rs（2860 行）均早已超 2000 行准则线
（既有面，本切片增量 ~80 行不触拆分，只提及）。

## 下一步候选

1. textarea 宽度软折行（UA pre-wrap）+ 控件 ellipsis——收口
   placeholder-white-space.tentative.html（本簇尾红页）+ 实域 textarea
   长行折行；与 boundary/selection UTF-16 映射交互需单测锚定。
2. 小簇排队其余（meter 4/input 3/link 3/embed 3/object 2/option 1/iframe 1/
   wbr 1）逐簇勘根；br 1 勘根前先 solo 复核 editing-compat br 刀
   （fe760dc57）外溢现势。
3. （挂账承前）生产动画重绘驱动 / 隔离通道帧表 IPC 扩展 / paint IPC
   duplicate 动画携带——渲染流 + 协议面协同候选，非本流域内施工。
