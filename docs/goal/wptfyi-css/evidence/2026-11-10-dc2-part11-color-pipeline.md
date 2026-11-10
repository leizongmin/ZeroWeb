# DC-2 次批：css-color declared/computed 通路（part11 色值管线）

**2026-11-10**。css-color 簇 62 页全量重跑：**Pass 1,293 → 7,229（净增 +5,936）**，
Fail 3,780。终轮产物 `.acceptance/wptfyi-css/results-part11-final/css-color.json`
（守卫包裹，`make testharness-css-color`）。

## 账面与逐页不降核对

- 基线（DC-1，逐位一致复现快照）：1,293P。
- 终轮：7,229P / 3,780F；33 页净增、28 页持平、1 页回归。
- 唯一回归：`color-valid-relative-color.html` 597 → 583（**-14**）——基线靠旧
  part11-computed-color.js 的 relative computed 支持（含 currentColor 面），新管线
  尚未建模该面；次批后半移植时恢复。已记账，属逐页不降契约的显式豁免。
- 首轮中间账 7,167P（results-part11）后修复 color-layers 白名单漏项（-144 回归
  恢复至基线 160P/10F）与 var 透传，终轮 +62 入账。

## 改动面

| 文件 | 内容 |
|---|---|
| `js_dom_shim/part11-color.js`（新增，接线） | css-color-4/5 声明值管线：`__zwColorNormalizeDecl` 委托入口（!important 剥离回拼、css-wide 关键词小写化、系统色表补 threedshadow、opacity 独立归一、值内注释剥核合法保原串）；`_zwCAlpha8` 8-bit round-trip 最短十进制 alpha 序列化（css-color-4 §16.1.1）；透传白名单 color-mix/relative 形/var/color-layers 返原串非 null |
| `js_dom_shim/part11-computed-color.js`（留盘未接线） | computed 数学管线基座，次批后半启用 |
| `js_dom_shim/part06.js` | 委托块改 **null 即拒**，不回退旧分词器——旧面未知 ident 透传（'yelow' typo 被接受，color-invalid-named 183F）；透传形返原串非 null 不受影响。invalid 全家翻绿（named +183、hsl +23、hwb +6、lab +18、rgb +30、general +10 等） |
| `js_dom_shim/part01.js` | getComputedStyle proxy 补 `has` 陷阱（computed-testcommon 首断言 `property in cs`）；宿主 partial-parse 抢答覆盖（color/background-color 宿主错误计算值时落 plain 重读）；opacity 加入 plain 回落四属性 |
| `js_dom_shim/part01c.js` | `_zwPlainColorSpec` 注释剥离（`/**/transparent` 语料实证）；opacity computed 臂（clamp [0,1]，无声明返初始值 '1'） |
| `js_dom_bridge.rs` | include part11-color（+14 行挂载） |
| `js_dom_bridge_tests/part06.rs` | T48 五个分歧形状钉**翻正**为 Blink 对齐（hsl 保形/现代形保形/越界不 clamp/LISTITEMTEXT/system 依赖）——corpus WPT 证据背书，钉与注释同步翻正 |
| `js_dom_bridge_tests/part03.rs` | T49/T49fix pin 恢复（modernSpaceVerbatim/cssTextImportant） |

## 验证链

- `cargo fmt --all` 无 diff；`cargo clippy -p zero-engine --all-targets -- -D warnings` 零告警（守卫包裹）。
- js_dom 全库（终树，守卫包裹）：**3,827 通过 / 7 失败**。7 失败经 HEAD
  worktree 重编定性为 `0d1ef9229` 既有（fetch filtered matrix、websocket×2、
  readable stream、iframe SW dispatch、img/svg settle），逐位复现，非本批引入。
- 4 个 cssom 修复测试全绿：t46（hex8 alpha）、t48（context/翻正钉）、
  t49（decl value）、t49fix（cssText quoted semicolon）。

## 残余桶（3,780F 归类）

| 桶 | 量 | 定性 |
|---|---|---|
| color-mix/relative **computed** 数学管线缺失 | ~3,108 | computed-relative-color 1,169、computed-color-mix-function 948 整页 + valid 两页内 computed 断言 991（次批后半主目标；含 -14 currentColor 恢复） |
| color-mix/relative/layers/contrast **invalid** 声明真校验缺失 | 403 | invalid-relative 160、invalid-mix 141、invalid-color-layers 93、invalid-contrast 9（透传形需真解析后才可拒） |
| 杂项 computed 面 | ~269 | alpha-color-computed 29、system-color compute/consistency/support 53、computed-rgb 残 21、opacity-computed 20、out-of-gamut 45、light-dark-image 31 等 |

## 下一步

1. **次批后半**：color-mix()/relative color/color() computed 真色彩数学管线
   （part11-computed-color.js 移植接线，~3.1k 断言 + currentColor -14 恢复）。
2. **DC-2 再批**：未 fetch 目录语料扩面（~8.1k）；`.any.js` 变体通道（588）。
3. **DC-3**：全量对账（验收线 182,392）。
