# M2 s444 — label 2 键伪红第三例清偿（③m pointerevent_support.js，F−2/P+2 双腿
确证）+ combobox tentative 12 分诊定谳（全簇 D 门内）+ 全域资产盘点地图

日期：2026-11-15（s443 同日续轮）。承前：s443 下一步「combobox tentative
12 分诊 + label forwarded-click 键 2 勘根」。

## label 2 键勘根：资产伪红第三例（s442 双判据翻案）

solo `label-forwarded-click-pointer-properties.html` 实况：两键均为
`promise_test: Unhandled rejection with value: object "ReferenceError:
getEvent is not defined"`——该案外链
`/pointerevents/pointerevent_support.js`（getEvent 事件助手），资产不在
语料。资源就位（jsdelivr 上游 pin 315976933 实测 19111 字节
self-contained）后 **solo 2/2 全 Pass**：

- 键 1 `Click forwarded from label to control preserves pointer
  metadata`：testdriver.click 真输入管道 + 转发 click 的
  pointerType/pointerId/isPrimary/detail 保持——**s442 判「testdriver
  真输入管道能力门」翻案**：runner testdriver 为内建 stub
  （testharness.rs prepare_harness_html 内建替换 testdriver.js/vendor/
  actions 三引用），能力面健在。
- 键 2 `Click forwarded from label.click() is not trusted`：
  `label.click()` 合成链 isTrusted=false 双事件保持——**s442 判「疑引
  擎事件面」翻案**：shim label 激活行为转发面健全（含 untrusted 传播）。

与 s441 green-2x2.png（46 键）、s442 color-testcommon.js（196 键）同型
第三例。施工：25 脚本 ③m 块 `fetch_raw
"pointerevents/pointerevent_support.js"`（语料内唯一消费者即本案）。

## combobox tentative 12 分诊定谳：全簇 D 门内（非能力门）

12 键失败消息三分（s443 全量腿日志逐键取证）：

| 机制 | 键数 | 消息形态 |
|---|---|---|
| datalist-popover 基建缺口 | 9 | `Datalist should open after focusing/clicking input` ×5、`:popover-open after focusing` ×1、`beforetoggle event listener` ×1、初始过滤形态（`option should not be display:none`）×2 |
| `:active-option` 选择器未实现 | 3 | `SyntaxError: ':active-option' is not a valid selector` |
| （`:filtered` 伪类并入上两族判据） | — | 同初始过滤形态消息 |

判据：combobox 案资源面完整（customizable-combobox-styles.css 在档 +
testdriver stub + harness），**非 runner 能力门**；三机制均为
customizable select/combobox tentative 规格基建组成件（datalist 以
popover 形态展示 + :popover-open/beforetoggle + :active-option/:filtered
伪类）——与 GB-20261031 簇 D（customizable select 立项）同一基建面。
**分诊定谳：12 键并入簇 D 门记账**，s441「疑能力门族留续分诊」挂账关
闭。本域不在门下自行开工。

## 全域资产盘点（2636 案外链全解析，伪红散账模式收口）

python 逐案解析 script src/link href（按引用文件目录解析相对路径，滤
data:/模板串/mailto:/故意 404 判据）→ 真缺失分诊：

| 资产 | 消费引用 | 判定 |
|---|---|---|
| `/resources/testdriver.js|vendor|actions`（×243/243/122） | runner 内建 stub 替换（prepare_harness_html 实证） | **伪缺失**，非资产面 |
| `pointerevents/pointerevent_support.js` ×1 | label 案，+2P 实证 | **本轮 ③m 落地** |
| `customizable-select/resources/customizable-select-styles.css` ×47 | customizable-select 族（簇 D 门） | **簇 D 门开时随批落**（门下不施工） |
| `html/resources/common.js` ×19 | template-element XHTML 族——当前零稳定红 | 无位移不落（等案红再落） |
| `resources/chromium/enable-hyperlink-auditing.js` ×18 | links/downloading-resources——该目录零键（不在枚举面） | 无位移不落 |
| 其余单例（py 端点/子资源判据/`.sub.js` 等） | 多为 runner py-routes 仿真或故意 404 判据 | 等对应案红面出现逐案勘 |

价值：s441 起「资源遮蔽伪红」三例均靠个案 solo 偶然揭出；全域地图落账
后，后续任何案红先查本表资产列再入代码勘根，散账模式收口。

## 全量双腿位移（main 树，③m 在场）

- leg1/leg2 规范化键集 join：**stable 8514 键（8305P/203F/5T/1U），
  Pass 两腿逐键全一致**。
- vs s443 stable（8303P/201F/5T/1U）：**恰 label 2 键 F−2/P+2，零其他
  位移**。stable Fail 差额（201→203）= select-home-end 族 4 键本轮双腿
  同 Fail 相位入 stable（族内相位轮换：s443 两腿 5F/5T 全振荡出、s444
  4F 入 + detailed 1 键 T↔F 分歧），在册负载族记账口径不变。
- 分歧仅 detailed 1 键 T↔F（在册族）。
- **新基带：8305P（stable 口径）+ select 族 5 键振荡记账**；g47 组合树
  归因随本轮双腿清偿（legs 即跑在 g47 树上，除预期位移零其他）。

## 验证汇总

| 检查 | 结果 |
|---|---|
| label solo（资源缺席） | 2×ReferenceError 伪红实况 |
| label solo（资源就位） | 2/2 全 Pass |
| ③m fetch 脚本 bash -n | 绿 |
| 全量 leg1 | 8305P/203F/6T/1U 干净跑通 |
| 全量 leg2 | 8305P/204F/5T/1U 干净跑通 |
| 双腿 stable join | 8514 键，Pass 全一致 |
| vs s443 stable 位移 | 恰 F−2/P+2，零其他 |
| 全桥套件绿窗收口跑 | 1625P/0F rc=0；基线 1596P，+29 = 组合树他流入树新钉（g43–g48 系列 engine 桥测），零红 |

工件：/tmp/s442/{s444-label,s444-label2,s444-leg1,s444-leg2,
s444-bridge}.log、l{1,2}.ks、{l1,l2,s444-stable}.norm、s444-{gone,new}.
txt、s444-script-refs.txt/s444-css-refs.txt（裸清单存档）。清场：CPU
burner 零残留、语料新增两资产由 ③k/③l/③m 通道管养（gitignore 不入库
政策不变）。

## 结论

s444 收口：label 2 键伪红第三例清偿（③m 落地 F−2/P+2 双腿确证）+
combobox tentative 12 分诊定谳全簇 D 门内 + 全域资产盘点地图落账。
**in-domain 自由面实质清零**：稳定红池余量全数归门（select 147 +
combobox 12 = 簇 D；video 4 = GB-20261114-VIDEO）、跨域（dialog 16 /
tabular 4 / img 13 = 渲染域与 decoder 耦合面）、在册族（select-home-end
5 / onblur 2）与渲染邻接单例。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D，combobox 12 并入）+ s137。
