# s430：engine act:option 臂施工（双刀合龙，seltouch 稳定翻绿）

日期：2026-11-14 ｜ 车道：engine/dom ｜ 前序：s428 carve-out（渲染面首刀）+ s425 定型设计

## 施工（part03.js 单文件两处）

1. `_zwHasOwnActivationBehavior` OPTION 停靠位（R5530）：`if (tag === 'OPTION')
   return true` + 注释枚举五类→六类。nearest 止步语义双调用位同惠
   （`_zwFindClickActivation` 返回 act:option；pre-click 回滚走查同函数止步——
   option 内容模型不含 INPUT，无穿透回归面）。closed 形态 option 无布局盒
   （s428 抑制谓词）天然不入本判定，激活面与 carve-out listbox 判据精确对齐。
2. dispatch post 段 act:option 分支（gate `event._zwUaDispatch === true`，镜像
   act:button R4400 先例）：`_zwOptNearestSelect` 无归属（datalist option）不
   激活；own disabled 属性门（FIXME: optgroup/fieldset 继承链未查，语料无钉面）；
   toggle 经 option.selected IDL setter（R3908b selectedness+dirtiness）——
   R3909 ask-for-reset 自带形态分（非 multiple 互斥清兄弟 / multiple 不清他人，
   本语料钉面零新增逻辑）；实际翻转才派 input+change 到所属 SELECT（镜像
   `__zw_select_key_action` 派发形 bubbles/composed/cancelable 精确一致）。

**spec 锚**：HTML 渲染规范 §the-select-element-2 listbox 形态；pick 激活 spec
无正式定义（Chromium listbox 语义近似，语料 select-listbox-touch.optional 上游
同标 .optional）——R5530 注释已锚。`.click()`/合成 dispatch 通道不接线（UA
通道专属，镜像 act:button 门语义；语料仅 test_driver touch）。

## 验证链

| 门 | 结果 |
|---|---|
| 25 part 拼装 node --check | OK（153,401 行） |
| make 构建（runner release 重建） | exit 0（36.55s；改动面纯 .js，无 Rust 面——check/fmt/clippy 不适用） |
| solo: select-listbox-touch.optional | **1P（翻绿）**——6 断言全过（初态双不选 / touch option1 选中 / touch option2 双选不清他人 / 再 touch option2 翻回不干 option1） |
| solo: select-option-display-inherit.tentative | 3/3 Pass——s387 回归清（含 multiple-select style hook + closed gBCR 钉面） |
| solo: select-multiple-resize | 1F 同形（断言② `expected >106 got 106`，③④ 不可达——如预判维持） |
| the-select-element 目录回归 | 336 键 vs s429 r1 同域 330：(key,state) 级差异 12 行全部落两处——seltouch F→P（目标）+ home-end 簇 D 家族 file 级挂起表示→8 子测级 Fail（在册表示漂移族，红位不变）；零其他位移 |
| 全域 sweep ×2 + stable-set join vs s429 双基线 | **stable-F 202→200（−2）/ stable-T 13→14（+1）/ stable-P 8239→8242（+3）/ 沾染 cur=0**——seltouch F→P 稳定双轮；detailed.optional file 级 F→T 桶内重排（簇 D 在册家族，红位不变）；checkable-active-onblur following 2 子测自愈回 stable-P |

## 判读

**双刀合龙达成：seltouch 双轮稳定 Pass，稳定红面 215→214（−1F）——本车道
自 s424 施工以来首个真实解挂键**（簇 D 关联面：seltouch 是簇 D select 家族
成员之一，其解挂不关闭 GB-20261031——该门覆盖 selmany/selfocus base-select
门等其余成员）。join 三证自洽：①checkable 沾染（s429 挂账）双轮自愈回
stable-P，反证 s429 时序归因（非代码位移）正确；②detailed.optional F→T 桶内
重排与 s424/s427/s428 历史翻转同族（表示层跨轮漂移，红位恒定）；③new/gone
集合无第四种位移。multiple-resize 维持 1F（断言② scroll overflow 记账独立缺
失面 + ③④ drag-resize 浏览器 chrome 级交互面，s388 判例维持挂账）。

## 残留面与下一步

- **簇 D 余池**：selmany/selfocus base-select 门（GB-20261031 用户门维持）；
  home-end 家族表示漂移（挂起 vs 子测上报，红位不变，不施工）。
- scroll overflow 记账（可选优化面，multiple-resize ②，零账面影响）维持挂账。
- 用户门禁待决不变：GB-20261114-VIDEO、GB-20261031（簇 D）、s137。
- 下一步候选：dialog R-s31 簇 / img density / embed sizing face 到货复验；
  门决策后 video P1+P2 或簇 D A2 分诊续。
