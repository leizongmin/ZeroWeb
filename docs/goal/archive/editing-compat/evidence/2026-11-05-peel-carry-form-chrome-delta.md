# peel carry-form 族 Chrome delta——七刀合收（fix8/fix9）

日期：2026-11-05。⑧ 轮残差①（peel carry-form 族 ~13F）主刀轮。`crates/engine/src/js_dom_shim/part01d.js` 七刀：① `_zwVaNormalizeStyleAttrCss`（peel 克隆/祖先 style attr CSSOM 归一，color/background-color 两属性 keyword 色→rgb 形）② push-down children 快照先于 clearValue（vendored :2652 原文时序，修 fs4a/font6 载体零执行——简单可改性祖先被摘壳后 childNodes 恒空，快照后置 = 携带臂零执行）③ `_zwVaForceValue` carrySrc 参数（fontsize push-down 载体 raw attr 保留，font size="6" ≠ 归一 "7"）④ fontsize 关键字成员门（vendored :2860 第二条件 newValue ∈ 7 关键字，xx-small/2em 不可映射落 span+style 臂，消 `<font size="undefined">` 错形）⑤ backcolor/hilitecolor peel 门（仅 fcS===fcE 才 peel——首个 specified 冲突祖先两侧相同、元素整体包住选区形；跨界/嵌套载体形 Chrome 恒整壳保留、新包裹收进内容侧）⑥ `_zwVaRematerializeCarries`（peel 载体嵌套 flag-form 重物化：font 属性载体在嵌套 specified 祖先域转 span style 形——嵌套 font1-in-font4 fs4 flag=true 期望 span 载体、flat 形恒保 font attr）⑦ `_zwVaCanonicalizeAncestorStyleAttrs`（setSelectionValue 入口两边界祖先链 style attr 规范形直出——harness 归一器命中面受命令后 sel/查询态影响不稳定，引擎直出无条件对齐录制面）。判决五语料对 fix7：**F2P 16 / P2F 0 / newF 0，51F→35F**；multitest 全分母 10,092 行**逐位恒等（F2P 0 / P2F 0 / newF 0）**。

## 修复面与语料实证

- **fontsize 5 F2P**：嵌套 `font4>font1` 外层拆分载体（fs4a 形，⑥ 重物化）、`font6>span 2em` flag 双变体 + `p style 2em`/`p style xx-small`（④ 关键字门 + ③ raw attr 保留；改前 `<font size="undefined">` 错形）。
- **forecolor 3 F2P**：跨元素同色（`font color=brown` + `span style=brown`、`span rgb>span brown`）克隆/祖先归一（brown→rgb(165, 42, 42) 形，①/⑦）。
- **backcolor 4 + hilitecolor 4 F2P**：`div aqua>p tan` / `span aqua>span tan` / `p aqua>span tan` 嵌套 bg 形（⑤ fcS 门 + ① 归一；**backcolor 全扫 0F rc=0 收官**）。

## 门禁

五语料（07:08-07:11 solo，07:07:36 二进制 grep `_zwVaRematerializeCarries` 3 命中执行体证据）：

| 语料 | fix7 | fix9 | Δ |
|---|---|---|---|
| fontsize | 2,409P/13F | 2,414P/8F | F2P 5 |
| fontname | 2,221P/4F | 2,221P/4F | 恒等 |
| forecolor | 2,306P/21F | 2,309P/18F | F2P 3 |
| backcolor | 1,022P/4F | **1,026P/0F rc=0** | F2P 4 |
| hilitecolor | 1,209P/9F | 1,213P/5F | F2P 4 |
| 合计 | 51F | **35F** | **F2P 16 / P2F 0 / newF 0** |

fix8 中间代（05:28:40 二进制）五语料行集与 fix9 逐位恒等（fs/fc/hc diff 空）——fix9 ⑤ 门为语料中性收尾刀。

multitest 10 片（07:12-07:33，07:07:36 二进制全程单一，trap 恢复 pristine 10 meta 核对）：10,092 行 = 9×1,000 + 尾片 1,092（与 spin2d solo 完走基线恰等），**9,654P/438F 对 fix7 基线逐位恒等 F2P 0 / P2F 0 / newF 0 / lost 0**（judge-fix9-mt.py 行名元组口径）。

守卫腿：inserttext 3,009 行 / 227F 恒等基线；typing-text-in-direct-child-of-editing-host 实名页 6P/10F 恒等基线；探针 zz-probe-vawrap2 10 行 2F 恒等（tF push-down 残差族 + tI teal 已知面）、zz-probe-xel 2F 恒等（dump 型设计）、zz-probe-nstyle 7F（dump 型设计 `assert_true(false, dump)` 无条件输出通道，fix7 基线无此探针非位移）；**zz-probe-observe P5-bc-t/f Pass→Fail 为期望旧形非引擎回归**——⑦ 规范形直出把保留 tan 壳改写 `rgb(210, 180, 140)`，语料权威（backcolor.js :233 kept-half 期望 `background-color:rgb(210, 180, 140)`）与引擎一致，探针期望系规范形工作前手写 keyword 旧形，已更新 p5Exp 对齐语料（复跑 **13/13 绿**）。

engine lib 终核（make guarded-test）：**3,727P/5F**——5 红名（s249 CSSOM 族 4：r2809/r2810/r3276/p99s + p56 adopted snapshot）对 fix7 轮逐名恒一零新增，无其他 suite 失败、clippy 零 error（RC=2 系已知 5 红唯一来源）。

## 残差 35F 构成与后续

全部落 ② 读数面族 + 少量 compare：qV/qI/qS before/after 读（fontsize 6、fontname 3、forecolor 13、hilitecolor 1）+ compare innerHTML（forecolor 5 嵌套/跨元素色壳、fontsize 2 = ③ teal `32px+background-color` 族、hilitecolor 2 = `<font size=6>[foo]</font>` 全选区载体形）。③ teal end 侧 prop-strip 与全选区载体形挂下轮；② 读数面族（qV 位置基 computed-style、qS quirk、qI-after br、pre/listing formattable 排除）为下一主刀。

## 上轮中断补账

上轮（429 限流 05:39 中断）已跑 r9（05:16 旧码，F2P 11）+ fix8（05:31，F2P 16）两批但判决未落；r9-mt 批（05:22-05:41）横跨 05:28 重编译为混合二进制批次不可作账（judge 脚本未执行）。本轮以当前树重建（07:07:36）重跑五语料 + multitest 干净收口，r9/fix8 判决数字事后补算（judge-r9.py / judge-fix8.py）仅作轨迹参考。
