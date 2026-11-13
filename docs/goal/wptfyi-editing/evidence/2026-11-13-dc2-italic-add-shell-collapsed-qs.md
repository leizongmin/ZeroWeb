# DC-2 · italic ADD 壳内嵌套 + collapsed state override 记账（d9 批，2026-11-13）

P2 主循环 ⑥ 两条臂收口：face 1（六 format 文件 + 45 探针，含新增
zz-probe-d9diag 10 case）断言级 **F2P=5 / P2F=0 / goneF=0**（italic
6F→3F：oblique ADD 双 flag ×2 + collapsed qS ×1；bold 13F→12F：
collapsed qS ×1；d8diag I3c oblique 残红代表转绿）。face 2 数字见下
文。

## 修复面

- **⑥-a italic ADD 冲突 decl 壳保壳内裹**（peel 链 italic 镜像门）：
  `_zwVaSplitConflictingInline` 链收集条件加
  `!(command === 'italic' && newValue === 'italic')`（bold ② 同门镜像，
  仅 ADD 面挂门，toggle-off 切分面不受此门）。oblique 壳部分入选不再
  边界抽取合并（GOT `fo<span italic>ob</span><span oblique>ar</span>`
  → 录形 `fo<span italic>o</span><span oblique><span italic>b</span>
  ar</span>`，italic.js:640-647 双 flag；css-false 壳内裹 i）。I3a/I3b
  toggle-off 面与 C1-C3 cite/var 关值面不受影响（方向不同源：起边界
  qS=false → ADD；qS=true → off，peel/③ 面分属两臂）。
- **⑥-b collapsed exec 六命令 state override 记账接通**：part01d 新
  导出 `__zw_ce_state_override_arm`（记账值 = !!on；sub/sup 互斥 unset；
  value override 不记——Chrome qV(bold) 恒 '' 口径，vendored
  setSelectionValue 记账面会被 `__zw_ce_value_override_value` 读出
  'normal' 化 delta），part06 collapsed typing 臂单点调用，on =
  !`_zwQueryFormatStateDom(cmd)`（与 typing toggle 同源 DOM-only 读——
  gate14 双执行语义保真：override-aware 读会把首记 override 反馈进第
  二次 collapsed toggle 判定，Chrome 第二次执行不关账）。此前
  collapsed 臂零记账 → qS 落回 caret 处 effective state：`<b>foo[]bar
  </b>`/`<i>foo[]bar</i>` toggle-off qS after=false 双红（bold.js:48-52、
  italic.js:53-57 dict 第 5 位）；`foo[]bar` ADD qS after=true 语料绿
  实为跨 case 残留态假绿（隔离红，见方法）。

## 方法

同树前后双二进制对跑（base = face-b6.log@d8 收口态；after =
d9c-runner@工作树，systemd --user scope MemoryMax=24G 守卫构建，
`ZW_SCRIPT_TIMEOUT_MS=300000`，face 脚本从 repo root 跑），断言级
前后 diff。face 1 = 六 format 文件 + 44 既有探针 + zz-probe-d9diag
10 case。face 2 = multitest + misc 哨兵。

**isolation 探针分诊（bold 13F 归因）**：scratch zz-probe-d9qs 7+1
case（验后删）——700/900 partial、跨块 b>p、bold-span qS、100|200
邻接共 12 断言隔离全绿（单 case 期望形状引擎全中），唯 collapsed
`<b>foo[]bar</b>` qS after 隔离红（got true，dict false）→ bold 13F
= 1 真缺陷（本批 ⑥-b 修）+ 12 跨 case 状态面残留。italic 对称确证：
I5 哨兵旧断言 qS=true 系照抄引擎行为（dict false），本批随 ⑥-b 转
绿并改断言入册。**qS(stylewithcss) before ×2 红**（bold-span css-false
twin、100|200 css-false twin）= case-entry 状态面：Chrome dict 期望
before=true（css-true twin 执行后 flag 跨 case 保留），引擎返回
false——引擎 flag 生命周期与 Chrome 相反向缺口，跨 case 专项批（d10）
挂账。raw 记账判别（scratch X1 case）：collapsed bold ADD 后
`__zwTypingTags=['b']` 落地、state override 先置 false（首版接线以
state 直传 on 的方向反置）后修正为 `!state`——arm 布尔语义
（override = 命令后激活态）以探针定谳。

## 数字

face 1：base 15,420P/100F → after **15,435P/95F**（d9diag 10 新 case
全绿入册；d8diag I5 断言随 ⑥-b 更新入册）：

| 簇 | base F | after F | 修复臂 |
|---|---:|---:|---|
| italic oblique ADD 双 flag | 2 | 0 | ⑥-a italic 镜像门 |
| collapsed qS off（bold+italic） | 2 | 0 | ⑥-b 记账接通 |
| d8diag I3c oblique 残红代表 | 1 | 0 | ⑥-a |
| d8diag I5 旧断言（qS=true 误断言） | 1 | 0 | 改断言入册 |
| bold 700/900 partial + 跨块 + qS 杂项 | 13 | 12 | 未动（跨 case 状态面） |
| italic 跨块 i>p ×2 + I3a qS 读面 | 3 | 3 | 未动 |

- 断言级：**F2P=5 / P2F=0 / newF=0 / goneF=0**（face-d9c2 终跑）。
- italic 6F→3F；bold 13F→12F；六文件残余 65F→61F。

face 2（multitest + misc 哨兵）：base 10,413P/24F → after 10,173P/24F
——**F2P=0 / P2F=0 / newF=0 / goneF=0，fail 名集 24=24 逐名恒等**；
P 差 -240 与名漂 4 断言全落 `?9001-last` 慢页预算窗口（300s script
预算中途击杀切片点非确定，承 d7/d8 批同口径），misc 423 断言逐名恒
等。⑥-b 的 mt 状态命令族仲裁面零变化。

定点引擎钉测（test-guard 包裹，`cargo test -p zero-engine --lib
execcommand`）：**7 passed / 0 failed / 0 ignored**（nested_cross_b
已解钉恒态承 d8）。本批仅改 .js shim 与探针，Rust 面零变化。

## 残余与下一步

- bold 12F / italic 3F / u/s 20F / sub·sup 26F 未动（master ② 清单）。
- **d10 专项候选**（isolation 分诊产出）：跨 case 状态面 12+3F
  （cssFlag 跨 case 保留语义 + state override 跨 case 残留边界）+
  qS(stylewithcss) before ×2 的 flag 生命周期缺口。
