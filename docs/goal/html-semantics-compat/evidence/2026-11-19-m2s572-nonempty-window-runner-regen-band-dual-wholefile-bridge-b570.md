# M2 s572 — **非空窗巡查轮（空窗后首个新窗 pull 捕获 ×9 全列核实 code ×5/docs ×4）+ runner 必换代兑现轮（Compiling ×9 全链 pre f0ebaf5b→mid=post 86389795 −8,368 bytes）+ 全桥 b570 定谳轮（1728P/0F rc=0＝b569+NEW 1 atp 钉+GONE 0 预判精确兑现、基线换 b570）+ 族带分裂形消解＝双整文件键 Timeout 并存形（stable 面新形）+ 七哨全持位轮**：join **stable 8511（8305P/198F/7T/1U）disagree 0 + l1only 0 + l2only 0（raw 双腿 8513/8513）**；vs s571 stable（8510）**NEW 1 + GONE 0 + flips 0**——NEW 恰＝族带 optional 整文件键（双腿回带），非带非哨键逐键零差。

日期：2026-11-19（s571 收口同日第四轮；
本 session 11:06 滚入开工：pull
--rebase 捕获 ×9 全列核实（4b513af17
→a38a020f6，对端 wptfyi-css 流批）：
**code ×5**＝abe0d38d3 fix(css-parser
,engine) @property 全链恢复道六 fix（
解析门+注册表分表+规则通道接线）＋
aee9915cb fix(engine) WAAPI 均匀 add
影族引擎合成+animation-composition
apply 面＋f3b00b59a fix(engine)
transition 简写读面 duration 缺省位强
制+五 longhand max-zip＋363dc0aa7
feat(style-system,engine) mask 简写
<mask-layer># 全文法 CSSOM 双车道＋
7010aef08 style(engine) animation.rs
doc 注释缩进修复；**docs ×4**＝
wptfyi-css 三轮＋rendering R5821。
上轮遗留：s571 阶段布置的 s572 全链
（10:50 sed 布置＋waiter 10:51 启动）
在本 session 接管时 waiter 已在飞
（cmdline 绝对路径自排除在位），接管
后先做开跑前校验再放行：analyze.sh
基线引用面两侧核对（comm -13/-23 均
s571.stable 异源对＝sed 基线误引坑未
复现）＋grep 旧轮路径残留为零＋chmod
+x 六脚本全在位。11:15:32 v3.2 判净
（load<3 procs=0 3×45s，近窗峰值系对
端 kb62/R5821 与 d57 系列占用）HEAD=
a38a020f6 钉扎起腿。

## 窗底账（s572 窗＝4b513af17→a38a020f6 非空 ×9）

- **pull 捕获 ×9＝空窗后首个新窗**：
  code ×5（上列）＋docs ×4。
- runner 预判按 code 面重估：css-parser
  /style-system/engine 三 crate 码面全
  在 runner 链接链 → **必换代**（非
  s570/s571 的 docs-only 同值重建形）
  ——实测兑现：leg1 Compiling ×9 全链
  product-version→css-parser→net→
  style-system→layout-engine→engine→
  page-runtime→webview→runner，pre
  f0ebaf5b/128,508,616 → mid=post
  **86389795/128,500,248（−8,368
  bytes）**；leg2 Fresh ×0（Finished
  0.26s）。
- **窗带 js_dom_bridge 新钉**：
  part145_atp_recovery_pins.rs 新建
  （abe0d38d3），窗内桥测 diff 全列＝
  +1 钉 test_atp_note_stylesheets_
  registry_pins / −0 → 全桥 NEW/GONE
  对账预案触发，b570 预判＝b569（1727
  ）+NEW 1+GONE 0＝1728P/0F。

## 双腿 join（HEAD a38a020f6 树）

- 11:15:32 钉扎起腿；leg1
  11:15:32–11:18:47（3:15）rc=2 惯例
  →leg2 11:18:48–11:21:28（2:40）
  rc=2 惯例；stack overflow 0/0。
- raw canon：双腿各 **8513** 行（双发
  伪影 2 键双腿各 ×2 恒 Pass 形：
  wrapping-transformation 窗键＋
  execution-timing/149 键）。
- join：**stable 8511（8305P/198F/7T
  /1U）+ disagree 0 + l1only 0 +
  l2only 0**——分歧形全消解；vs s571
  stable（8510）：P/F/U 三桶逐项恒，
  T 桶 6→7 恰＝NEW 族带键 Timeout；
  **NEW 1 + GONE 0 + flips 0**。

## 全桥 b570 定谳（s567 wait-bridge 先例 sed 换装）

- 对端 d57 系列（input-events→
  uievents 连跑）占窗 → wait-bridge
  （v3.2 comm 版 sed 换装
  /tmp/s572，grep 残留零校验后放行）
  让位 29 min，12:02:32 判净起跑。
- b570＝**1728 passed; 0 failed; rc=0
  （filtered 2493）18.65s**＝预判
  精确兑现（b569 1727+NEW 1+GONE 0）
  ；bridge.runner md5 86389795＝腿
  mid/post 同二进制；桥库 leg1 已建
  Fresh 形。
- NEW/GONE 键级对账：**NEW 1＝
  test_atp_note_stylesheets_registry
  _pins（abe0d38d3 @property 恢复道
  ）+ GONE 0＝申报=实测零意外，
  刀 G 归因不触发**；**全桥基线换
  b570 实测值**（b569 自洽预检在册：
  b568→b569＝+exec 门钉 GONE 0）。

## 族带第 5 轮：分裂形消解＝双整文件键 Timeout 并存形

- s571 分裂形（leg1 吞形 l1only 1 ×
  leg2 子测发射 l2only 4）消解：
  s572 双腿同相——optional 整文件键
  **双腿 Timeout 在带**（NEW 1），
  detailed 整文件键 Timeout 持位，
  optional 4 子测键缺席（band566
  键级比对在册）。
- **判定＝双整文件键 Timeout 并存形
  ＝stable 面新形**：plan 预判两形（
  optional 4 子测回 stable＝回潮形/
  solo 红池形 Fail 4 键回 stable）
  均未兑现；s570/s571 stable 该键
  grep 0/0（s570 形＝子测 4F 在带+
  整文件键不在；s571 形＝子测 GONE+
  整文件键不在）——本形首次入 stable
  ；谱内形（whole-file Timeout 谱＝
  harness-no-report 家族，s571 leg1
  曾单腿发射同一键）非新极形。
- **非码面**：窗 code ×5 全在 CSS 面
  （@property/mask/transition/WAAPI
  /doc lint），零 select/keyboard
  码面；disagree 0/l1only/l2only 0
  ＝双腿收敛自证；升级路径不触发。

## 观察哨复核（七哨全持位）

- dialog 哨：**第 6 轮 key1/key2 双 P
  持位**。
- password 哨：**第 11 轮持位兑现**
  （select-event 279 键 8 非 P 形与
  s571 逐字节 IDENT；password 键 6
  全 P 0F；整文件键 0）。
- onblur 哨组 10 键（常规连轮第 4 轮
  ）：**8P/2F 恒**——F 集 ≡ s559
  逐键 IDENT（checkbox/radio 双键）。
- modulepreload 哨：15 键 stable 全 P
  ——第 34 轮零翻红。
- waapi 哨 2 键：picker-icon 双 P（
  第 93 轮）/checkmark F/F 恒形。
- focus-dynamic 121 键 stable 全 Pass
  ：第 52 轮零再现。
- 双发伪影 2 键双腿各 ×2 恒；rc=2/2
  惯例 stack overflow 0/0。

## 下一步（s573）

- 守成巡查续轮：**对账基线换 s572
  stable 8511**；开工 pull 捕获批全列
  核实；runner 预判按 code 面＋同值
  重建规重估；**全桥基线 b570（1728P
  /0F）持位**——下轮若窗带
  js_dom_bridge 新钉则 NEW/GONE 对账
  （对账键基线 b570.keys 在
  /tmp/s572 可留档提纯）；dialog 第
  7 轮；password 第 12 轮；mpreload
  第 35 轮；waapi 第 94 轮；
  focus-dynamic 第 53 轮；onblur 组
  第 5 轮；**族带第 6 轮观察＝双整文
  件键 Timeout 并存形持位判定**（双
  腿恒＝新形确立；optional 子测键回
  带＝谱内回摆；单腿发射＝分裂形再
  现）；双发伪影恒；爆栈 0/0 惯例；
  起腿三件套照规（浸泡门＋残留零校验
  ＋chmod +x；sed 复制后 grep 基线引
  用面两侧核对开跑前执行）。
- 候门决项等待用户拍板维持（video
  P1+P2 立项 / 簇 D A2 立项 / s137；
  对端 ZRG-2026-11-18-01 归用户）。

账本=evidence/2026-11-19-m2s572-nonempty-window-runner-regen-band-dual-wholefile-bridge-b570.md
