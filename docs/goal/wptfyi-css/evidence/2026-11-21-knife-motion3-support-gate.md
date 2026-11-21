# 刀 MOTION-3 Slice 1 · CSS.supports calc-size/interpolate-size 前置门（2026-11-21）

## 缘起与认领

MOTION-3（认领刀 CS-INTERP，commit `7ad13f46a` 入队）首靶：interpolation
testcommon 页 support 断言桶 ~1500F。机制侦察（NOTES-knife-motion3-support-claim
预注册分带）：

- `interpolation-testcommon.js` 每 test 以 `CSS.supports(property, from/to)`
  前置门（:396/:463），假阴性即整桶 assert_fail（support 断言与值断言分离）；
- `interpolate-size` 在 `decl_supported` 注册表外 → contains 恒假 →
  interpolate-size-interpolation 页 84 断言全红（页面基线 0 过）；
- calc-size 不定形（basis keyword / sum 含 `size` 词元）used size 需
  layout 期求值，apply oracle fail-closed 恒拒 → support 假阴性。

支持性是解析层判定，与求值可达性分离（css-values-5 §8 calc-size()、
§interpolate-size）。

## 机制

face 3 文件 117 行：

1. `js_dom_bridge.rs` decl_supported 双臂：`interpolate-size` 属性走写面
   `validate_decl_value` 同一解析单点；值含 `calc-size(` 走
   `calc_size_decl_supports_verdict` 文法门。`None`（var()/env()/attr()/
   random() 替换形、非尺寸槽域）回落既有通用通道，零 true→false 漂移面。
2. `apply.rs` 公开门 `calc_size_decl_supports_verdict`（薄封装既有 private
   `calc_size_decl_gate`，写面/supports 同源单点）。
3. `part02.rs` 钉测 `test_css_supports_calc_size_motion3`：不定形真组 /
   definite 真不漂移＋invalid 拒＋非尺寸槽拒组 / interpolate-size 文法域组。
4. `MOTION3_FACE_MARKER`（`#[used]` rodata 代码字面 ASCII，判别只认字面）。

## 电池（unit zw-knife-motion3-support-0908）

09:07:42 发射，09:19:53 GATE-DONE rc=0（wall ~12 分钟，六腿三语料双臂）。
编译门过（三 crate check）；b0 = HEAD＋ride 剥 face，b1 = HEAD＋ride＋face；
M2 双 bin md5 异；FM b0=0/b1=1；b1 md5 `80cda00a9f7b61519e12bbeaa9702007`。
RIDE 7 文件（SKILL.md/css-parser values 两文件/es_module.rs/wpt-runner
三文件＝兄弟流在途 WIP），S1/S2/S3 三窗 md5 零扰动；TRAP-NOOP state=done
（face 回树＝发射态）。

产物：`.acceptance/wptfyi-css/results-knife-motion3-support/`（bin/b0/b1 +
ride-files.txt + s{1,2,3}-md5.txt；日志 knife-motion3-support-battery-0908.log）。

## 结果（case 键级全对账，reconcile 现算）

| 页 | b0 | b1 | Δ |
|---|---|---|---|
| calc-size-height-interpolation | 10/1266 | 386/1266 | +376 |
| calc-size-width-interpolation | 10/1150 | 240/1150 | +230 |
| interpolate-size-interpolation | 0/84 | 84/84 | +84 |

净 **+690（F2P 690 / P2F 0 / new 0 / gone 0 / ASYM 0）**；motion/typed-om
双语料哨兵零漂移。

band 归因（预注册 5 带 vs 实测）：

| 带 | 预注册 | 实测 | 备注 |
|---|---|---|---|
| A（no-interp × transitions 双通道） | +308 高置信 | **308** | 精确命中 |
| B-AD（allow-discrete 双通道） | 不确定 | 164 | 翻面 |
| B-anim（animations/WAAPI） | 0（值管线不对称） | 124 | expected-target 同管线对称串相等：expected 端与 actual 端同走 `apply_single_property`→`parse_px` 0px 落槽 → 串等判 pass；无真值风险（零 P2F） |
| C（插值真值断言） | +0（Slice 2 域） | 10 | 端点恒等案（at(0)/at(1) 端点与 expected-target 串等） |
| D-isz（interpolate-size 页） | 随 A | 84 | 页全翻 |

reconcile 的 `unmatched 858` 为脚本标签缺陷（无状态门的格式不匹配计数，
非增益）；带内 F2P 合计 308+164+124+10+84=690 与 case 级总账精确一致。

DETERM 哨 10 案 calc-size-parsing Fail→Pass＝兄弟流 CALC-SIZE-CANON 骑乘
（b0 ≢ results-knife-calcsize/b1 逐案恒等，方向全为 Fail→Pass）——RIDE
S1/S2/S3 md5 零扰动证双臂同载，该页本 A/B 零 delta（不在 F2P/P2F 清单），
归兄弟流账不入本刀。

## 提交

face 提交 `9f9bc0f2`（rebase 后 hash；origin/main `fdca05c39` 之上），
显式 pathspec 3 文件；lei-pre-commit-guard PASS（0 发现 0 提醒）；
push 后 behind=0/ahead=0。钉测单测形态（V8 sandbox `__zw_css_supports`
回调）未入 runner bin，判别依赖 FACE-MARKER＋M2。

## 下一步

Slice 1.5a/b 收割面（`apply_single_property` definite calc-size 解析臂＋
interpolate-size 写值臂——animations 车道现 `parse_px` 0px 落槽，B 带
+288 串等 pass 的真值化）；或 Slice 2 同 basis sum lerp（插值主池 ~3.3K）。
