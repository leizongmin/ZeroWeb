# g33：progress() 三实参数学函数全链——values face 对 g32 提交基线净 +74 零回归

日期：2026-11-14。语料桶：`css/css-values/progress-{serialize,invalid,computed}.html`
三页（CSS Values 5 §progress，tentative）。提交基线（g32 态全量 JSON
`.acceptance/wptfyi-css/results-g32-syscolor-full/css-values/`）：serialize
13P/55F、computed **0P**/36F、invalid 33P/2F——progress 全链为本批工作树
新增（HEAD 无 CalcExpr::Progress）。

## 批内修法（工作树，13 码文件修改 +525/-35 + 新增单测 progress_function_tests.rs 262 行）

核心实现：`CalcExpr::Progress` 变体 + `try_parse_progress`（三实参顶层逗号 +
`no-clamp` 关键字 ident 边界 + 三实参 `calc_expr_math_type` 同族门 + 逐实参
跨度顶层角度单位家族一致性门）+ `eval_progress` 三段（全 eval → % 族
`parent_length=100` 符号基准重试 → **V 不可求值时 S==E 部分求值臂**：clamped
恒 0 不依赖 V、no-clamp 按 V 比较或 None、NaN 毒化部分臂同生效）+ specified
串化（可求值塌缩 `calc(<number>)`、不可求值 canonical 保形含 no-clamp/±∞）。

写面/门禁五修：

1. **A css-parser `serialize_math.rs`**：`span_has_top_level_angle_unit` TRIG
   屏蔽表补 `progress(`（角度叶 parse 期弧度化后型别 provenance 须文本级扫描；
   实参族一致性 parse 期自裁，扫描器不得把合法角度实参当混族）。
2. **B css-parser `types.rs`**：eval 三段重构（见上）。
3. **C style-system `apply.rs`**：`strip_trig_argument_spans` TRIG 表补
   `progress`（opacity/scale 槽 number 面拒绝侧同源泄漏）。
4. **D engine `part11-color.js`**：`_zwCOpacNormalize` calc 包裹 progress 形
   保形透传——`_zwCJoinRaw` id→quantity 无空格粘连（`no-clamp100px`）存文
   畸形交宿主 `fold_decl_specified` 正规管道；无 progress 的 calc 形维持 JS
   fold 口径。
5. **E style-system `apply.rs`**：`math_value_paren_unbalanced` 字节级括号
   深度扫描入通用 math 门 None 臂（single-spanning 豁免前）——未闭合形
   `progress(5, 0%, sign(10px)` 外层 `(` 永不配平（末尾 `)` 属 sign），
   ends_with(')') 启发式 + 双豁免误放行封堵。

配套臂：rotate3d 四臂（`parse_transform.rs`，R5192 口径对齐）——x/y/z 分量
`f64::parse`→`parse_transform_number`（<number> 文法位合法数学函数）+ 角度
槽 `parse_angle`→`parse_angle_or_math` + 其内 `eval_calc`→
`eval_calc_with_default_font`（em/rem 基准）+ **分词器 depth-0 修复**
（`split_transform_value_args` 逗号/空白盲拆把函数实参表撕成垃圾 token——
`rotate: progress(21em, 1rem, 11em) …` 四元组数学分量位整体无效；改写为
括号深度感知扫描，分隔符只认括号外逗号/空白，逗号段 trim 后判空语义与
旧实现一致（`1,,2`/`,1`/尾逗号/纯空白段拒收），individual
translate/rotate/scale 共享面 transform 族 211/211 全绿）；% 折叠屏蔽
（`calc_mix.rs` `expr_has_percentage_leaf_outside_progress`：progress 实参
% 函数内比值消解不外泄，`progress(1%, 0%, 100%)` → `calc(0.01)`）+
serialize `fold_dimension_calc` 换用 + `mask_tree_counting`/`expr_contains_mix`
Progress 递归臂；`computed.rs`/`converter/mod.rs` 树级百分比/长度谓词
Progress 臂；engine `part01.js` gCS opacity 抢答门 regex 补 progress。

单测：`progress_function_tests.rs` 9 钉（parse valid/invalid 矩阵、型别表、
clamp/S==E 简并/NaN、部分求值臂、嵌套、% 族符号解析、rotate3d transform
槽、specified 串化）+ apply.rs 文法槽族门断言集更新（progress 入文法断言集
后单参残缺形 `calc(progress(50%) * 1px)` 由未知函数豁免转拒收）。

**rotate3d 槽钉面对齐规范 clamp（本批终定谳）**：`progress(21em, 1rem,
11em)` 的 ratio = (21e−r)/(11e−r)，分子−分母 = 10e > 0 对任意正字号基准恒
成立 → ratio 恒 >1 → 默认 clamp [0,1] → 分量 1（css-values-5 "if omitted,
the return value is clamped to the range [0, 1]"，Issue 11825 澄清）。语料
progress-computed line 54 期望 `rotate3d(2, 2, 2, 2deg)`（未 clamp）与同页
line 35 `progress(1000em, 10em, 110em)` → '1'（clamp）及 serialize 页
`calc(0.5 * progress(200px, 0px, 100px))` → matrix(0.5)（clamp）自相矛盾，
系 11825 之前陈旧期望（上游 master 现行仍如此，逐字节 diff 证实非本地语料
过期）——**挂账改判：上游陈旧期望，不追，不改 clamp 语义**（解 clamp 会
破同页 clamp 钉面并违反规范）。测试断言由初稿的 2.0 对齐为
`Rotate3d(1,1,1,2deg)`（角度槽 ratio=1 → 2deg 不受 clamp 影响仍对）。

## 探针定谳（zz-g33-probe.html，已清出）

- 写面 7 断言全绿（rad attached/detached→`calc(0)`、unclosed→拒收、
  calcwrap-noclamp→`calc(0.5)`、setDim）。
- computed 面：nested progress→1、sign 嵌套→0、cqw S==E 数学折叠生效
  （`50%` 输出证 S==E 臂工作，used 解析差异归 converter 挂账）。
- rotate3d 析因 8 断言全绿（写面 serialize + computed 全形 T/U/V/W/X/Y）——
  真因 = transform 文法位裸 `f64::parse`/纯 `parse_angle`/`eval_calc` 无字体
  基准三洞，三臂修复后全通。

## 终账（values 全量逐页 diff vs g32 提交基线，256 页口径）

| 页 | g32 基线 | 当前树 | 变化 |
|---|---|---|---|
| progress-serialize | 13P/55F | 54P/13F | +41：rad 族、calc 包裹 no-clamp、% 族塌缩 |
| progress-computed | 0P/36F | 27P/9F | +27：全链从零到 nested/sign/rotate3d 面 |
| progress-invalid | 33P/2F | 35P/0F | +2 全页清零 |
| clamp-integer-invalid | 12P/2F | 14P/0F | +2（E 门 collateral） |
| hypot-pow-sqrt-invalid | 45P/4F | 47P/2F | +2（E 门 collateral） |
| zz-g33-probe（探针页） | — | 12P | +12 不入账（页已删） |

**净 +74（除探针页）/250 页逐页零回归**（python 逐页 diff：REGRESSED 0 页）。
首次全量跑二进制建于 08:17:45，早于 08:18-08:23 精修轮（rotate3d 三臂/%
折叠屏蔽）——终值以终跑为准（见验证账）。

progress-computed 残 9F 挂账：lh 字体度量族 6F
（`sign(1001em - 10lh * progress(…))` 需 rex/lh 混单位 ctx）、margin %·px
converter 1F（CB 宽转换期不可得）、cqw 宽 used 解析 1F（converter 期）、
rotate3d 案 1F——**已定谳为上游陈旧期望不追**（上文终定谳节：期望 2 与
同页 clamp 钉面自相矛盾，系 Issue 11825 之前的未 clamp 产物；我方
`rotate3d(1,1,1,2deg)` 为规范正确值，该案由 none 推进到仅值差 1 字符）。
serialize 残 13F 全为 S1 transform computed scale→matrix 序列化缺口（浏览器
gCS transform 恒 matrix，波及 css-transforms 1,238 案消费点，独立批次）。
flaky 观察 1 案：`calc(1px * progress(no-clamp abs(10%), …))` computed 轮间
±1，非修法面。

## 验证账

| 项 | 结果 |
|---|---|
| values 全量（首轮 08:17:45 二进制，257 案含探针页） | **4,021P**——对 g32 提交基线 3,935 净 +86，除探针页 +12 净 +74；python 逐页 diff REGRESSED=0（基线 JSON `.acceptance/wptfyi-css/results-g32-syscolor-full/css-values/`）。该二进制早于 08:18-08:23 精修轮 |
| values 全量（终跑，含全部精修 + rotate3d 四臂 + 分词器 depth-0 修复 + 探针页已删 256 案） | **4,009P**（fail 1,577 + timeout 7）对 g32 基线 3,935 净 +74 零回归：python 逐页 diff REGRESSED=0、GAINED 恰 5 页（serialize +41/computed +27/invalid +2/clamp +2/hypot +2）、256 页集与基线 1:1 无缺页无新增（transcript /tmp/css-css-values-corpus.log 08:46 二进制）。rotate3d 双侧案 1F 维持（定谳上游陈旧期望，见终定谳节——预期 +75 差 1 即此案） |
| css-color 全量 | 首轮 10,999 持平（早于分词器修复二进制）；终值以补套件电池为准 |
| css-transforms 全量 | 首轮 229 持平（早于分词器修复二进制，individual translate/rotate/scale 共享面直接消费者）；终值以补套件电池为准 |
| css-motion 全量 | 首轮 105/391 持平（早于分词器修复二进制）；终值以补套件电池为准 |
| css-typed-om 全量 359 页 | 首轮 9,872 持平（早于分词器修复二进制）；终值以补套件电池为准 |
| cssom 全量 177 页 | 首轮 1,968 持平（早于分词器修复二进制）；终值以补套件电池为准 |
| 补套件电池（终二进制：color/typed-om/cssom/transforms/motion/transitions/animations/easing 八套件串行） | g33 电池（/tmp/zw2-g33-final-suites-0850.log）跑至 transitions 中断（驱动链被杀）；由 g34 链全量重跑覆盖（见下行），无损失 |
| rustfmt | clean（--all --check EXIT=0，分词器 depth-0 修复后复验通过） |
| make test 全量（含 css-parser/style-system 单测） | **3,935P/7F**（/tmp/zw2-g33-final-mktest.log，`test result: FAILED. 3935 passed; 7 failed`）——7F 全在册组（js_dom_bridge r2967、t8k websocket ×2 等）零新增；Error 101 系在册 7F 致退出码非零 |
| clippy | 零告警（/tmp/zw2-g33-final-clippy.log，`Finished dev profile in 28.89s`，done10 标记） |
| g34 终账链（PPID=1 独立链，当前树二进制：values 全量 + 八套件电池） | 跑动中（/tmp/zw2-g34-values.log、/tmp/zw2-g34-suites.log，done-all 标记收口；终值待填）。重跑缘由：values 终跑二进制 08:44:29 编译窗（08:43:37 起 52s）横跨 08:43:56 分词器编辑，fingerprint 链仲裁 + progress-computed 单页行为复验（9F 同构）判定数字仍有效，但二进制 provenance 不清洁 → 静树重建（g34 build 29.31s，css-parser fingerprint fresh 即 post-edit 产物）后 values+八套件全量重跑定谳；终跑 transcript 已存 `.acceptance/wptfyi-css/values-final4-stale-binary-transcript.log` |
