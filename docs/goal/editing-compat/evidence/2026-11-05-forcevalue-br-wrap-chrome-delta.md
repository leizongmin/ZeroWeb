# forceValue invisible-br Chrome delta——color 家族 + fontname 包裹 / fontsize 裸（fix7）

日期：2026-11-05。上一轮残差②（wrap br 插入面）主刀。修复 `crates/engine/src/js_dom_shim/part01d.js` `_zwVaForceValue` invisible 早退面：选区内 extraneous br（`<p>[foo<p><br><p>bar]` 中块 `<p><br></p>` 形）不再 invisible 早退，改落尾部外裹臂独立值包裹；`fontsize` 命令门除外（语料期望 br 裸）。判决五语料对 fix6：**F2P 8 / P2F 0 / newF 0，59F→51F**；multitest 10,092 行全分母：**F2P 0 / P2F 0 / newF 31**（31 全落 ？9001-last 尾片 base 截断点后首执行带，fd+insertText 色壳族已知先例）。

## 根因与 Chrome delta 规则

vendored force-the-value 对 invisible 节点早退（implementation.js :2753），孤立 extraneous br 因此永不入包裹臂。Chrome 语料期望：color 家族（forecolor/backcolor/hilitecolor）+ fontname 对选区内 br 仍落独立值包裹——forecolor compare 期望 `<p><span style="color:rgb(0, 0, 255)"><br></span></p>` 形、fontname 同族；**fontsize 期望 br 裸**（`<p><br></p>` 恒无 font/span 包裹，fontsize 语料 2 变体实证）。保护面核查：五语料 + multitest 全部 br compare fixture 改前零通过（fix6 15/16 multitest br fixture + 8 五语料行全红）——bypass 无既有绿面可伤。

## 修复（part01d.js 一处）

invisible 早退条件补 br 旁路 + fontsize 门：

```js
if (_zwJusIsInvisible(node)
    && !(node.nodeType === 1 && _zwJusIsHtmlElement(node, 'br')
         && command !== 'fontsize')) {
  return;
}
```

上游 wrap 合并臂不受影响：可见 br 本就不入本臂，`foo<br>bar` 形走合并；孤立 extraneous br 落尾部外裹臂。等值早退（已获同值继承的 br）仍零变更。

## 门禁时间线事故（二进制陈旧伪象）

v2 批 fontsize 腿（04:00:44 批首）跑了无门旧二进制——fontsize 门 04:03:08 才落盘（晚于批首 make），批内 backcolor 腿 04:03:47 重编译才带上门。表现为 fontsize 对 fix6 P2F 2（br 被包裹 `<p><span style="font-size:large"><br></span></p>` vs 期望 `<p><br></p>`）。定谳手法：mtime 时间线 + 二进制 grep 门标记（`command !== 'fontsize'` 04:03:47 二进制 1 处命中，无门旧二进制 0 处）。v3 补跑 fontsize 腿（带门二进制）：**P2409/F13 对 fix6 恒等**（F2P 0 / P2F 0 / newF 0）。

另：v2 批被 harness 后台时限杀死（trap 已恢复语料，MT-RESTORED + 10 meta 核对）；v3 续批补齐 mt 4001-last 六片 + 守卫腿。

## 判决（judge 元组口径：键 = 断言文本后段）

| 语料 | fix6 | fix7 | Δ |
|---|---|---|---|
| fontsize | 2,409P/13F | 2,409P/13F | 恒等（fontsize 门排除） |
| fontname | 2,219P/6F | 2,221P/4F | F2P 2（br fixture ×2 flag 变体） |
| forecolor | 2,304P/23F | 2,306P/21F | F2P 2 |
| backcolor | 1,020P/6F | 1,022P/4F | F2P 2 |
| hilitecolor | 1,207P/11F | 1,209P/9F | F2P 2 |
| 合计 | 59F | **51F** | **F2P 8 / P2F 0 / newF 0** |

multitest（10 片合并 10,092 行 = 9×1,000 + 尾片 1,092，尾片行数与 spin2d solo 完走基线恰等）：base 9,404P/407F → post 9,654P/438F，**F2P 0 / P2F 0 / newF 31**——31 全落 ？9001-last 尾片 base 截断点（813 行 + T）后从未执行的 forwarddelete+insertText 色壳 caret 族 compare innerHTML 行（gate28 截断带 flake 族先例、fix6 轮 3 newF 同族扩大面，非行为回归）。

守卫腿：inserttext **3,009 行 / 227F**（2,782P/227F 基线恰等）；typing-text **腿缺失**——本地合成页 `wpt-data/editing/run/typing-text.html`（gitignored 产物、非上游文件——pin 315976933 raw 404 实证）跨 clone 全失且无行日志副本，16 断言守卫本轮缺口，页面重建挂账（fetch 通道 api.github.com 限流 403 期间无法补拉上游依赖）；探针 zz-probe-observe **13/13 绿**、zz-probe-vawrap2 **8P/2F**（tF push-down / tI 多属性 peel = 残差①已知面）、zz-probe-xel dump 型通道 2/2（qCV=rgb(0, 0, 255) 值正确，树形 `<span style="color:#0000ff">o<br>bar</span>` = br 包裹生效直接实证）。

## 执行体证据

target/release/zero-wpt-runner（04:03:47 构建）grep `command !== 'fontsize'` 1 处命中；mt 全片 + fontsize 补跑 + 守卫腿均该二进制（源码 04:03:08 后零改动，v3 批 make 零重编译）。

## 残差与后续

① peel carry-form 族 ~13F（嵌套同标签外层拆分、跨元素同色 end 侧值丢失→got rgb(0,0,0) expected rgb(165,42,42)、全选区 carry 形、`<font size="undefined" style="font-size:2em">` = `_zwVaCssSizeToLegacy` 对 2em 不可映射 bug）——下轮主刀；② 读数面族 ~17F（qV 位置型 computed style 读 expected=起后字符形 ~10F、qS stylewithcss-before Chrome quirk ×5、fontsize qI-after br ×2——qI/qV formattable 过滤补 extraneous-br 可见性门候选）；③ typing-text 合成页重建（行日志无副本，需按 g19 TypingStyle 家族重造 16 断言）；④ multitest ?9001-last 尾片 31 行 fd 色壳族（截断带首执行新账，随尾片分母扩大归骨架族后续）。
