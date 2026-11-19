# M2 s573 — **同 session 第二轮（s572 收口即布置接管）+ runner 必换代 ×2 连兑现轮（Compiling ×6 86389795→e0015ad4）+ 族带谱内回摆兑现轮（optional 4 子测 Fail 双腿回带＝红池形回归 NEW 4、双整文件键 GONE、detailed 整文件键首现 disagree）+ 七哨全持位轮**：join **stable 8513（8305P/202F/5T/1U）disagree 1 + l1only 0 + l2only 0（raw 双腿 8516/8516）**；vs s572 stable（8511）**NEW 4 + GONE 2 + flips 0——位移全落族带 home-end 族，非带非哨键逐键零差**。

日期：2026-11-19（s572 收口同日第二轮
；同 session 连轮：s572 push 阶段补账
落定后即布置 s573 全链——**sed 基线
误引坑第三次复现于布置阶段被开跑前
校验拦下**：sed 基线换装规则 s572\.→
s573\. 把刚换好的 s572.stable 基线名
再次改写致 comm/join 左右同源，净版
直写 analyze.sh 修正（s571 同类教训
第三次兑现），grep 残留零＋基线两侧
异源核对＋chmod +x 后放行）→ 12:21:51
v3.2 判净（对端 d57 系列收尾让位）
HEAD=c61abb750 钉扎起腿 → leg1
12:21:51–12:24:52（3:01，rc=2 惯例）
→ leg2 12:24:52–12:27:26（2:34，
Fresh ×0，rc=2 惯例）→ analyze 12:27
自治 → 哨复核。

## 窗底账（s573 窗＝a38a020f6→c61abb750）

- pull 捕获 ×6（925a921c4 fix(engine)
  getAnimations 浮出面＋transition 管
  辖归并/取消收窄＋float/position 离散
  接入＝**engine 码面在链接链**＋
  js_dom_shim part04/05 运行面；docs
  ×5）＋本流 s572 两 docs commit。
- runner 预判＝必换代（engine 码面）
  ——兑现：leg1 Compiling ×6（
  product-version→net→engine→
  page-runtime→webview→runner，
  css-parser/style-system 未动＝与
  925a921c4 单 crate 码面一致）pre
  86389795 → mid=post **e0015ad4**；
  leg2 Fresh ×0（Finished 0.16s）。
- **零桥测钉**（925a921c4 桥测 diff
  ×0）→ 全桥对账不触发，**b570
  （1728P/0F）基线持位**；shim 行为
  面位移由本轮 join 对账捕获（实测：
  非带键零位移，见下）。

## 双腿 join（HEAD c61abb750 树）

- raw canon：双腿各 **8516** 行；双发
  伪影 2 键双腿各 ×2 恒 Pass 形。
- join：**stable 8513（8305P/202F/5T
  /1U）+ disagree 1 + l1only 0 +
  l2only 0**——vs s572 stable（8511）
  ：P/U 桶逐项恒，F 桶 198→202（+4
  ＝NEW 子测键），T 桶 7→5（−2＝双
  整文件键 GONE）。
- **NEW 4 + GONE 2 + flips 0＝位移全
  落族带**：NEW 4＝optional 变体
  「Behavior of End/Home/PageDown/
  PageUp」4 子测键（band566 在册面）
  双腿 **Fail** 回带；GONE 2＝detailed
  整文件键＋optional 整文件键（
  optional 本轮双腿零整文件发射）；
  multiple:/single: 2 子测键（同文件
  非 band566 追踪面）跨轮持位恒 Fail。

## 族带第 6 轮：谱内回摆兑现＋detailed 首现 disagree

- s572 下一步预判三形中「optional 子
  测键回带＝谱内回摆」精确兑现：4 子
  测键双腿 Fail 回带＝**红池形回归
  （s553/s570 在册形）**。
- **新扭＝detailed 整文件键首现
  disagree**：leg1 Timeout × leg2
  Fail（多轮 stable Timeout 持位后首
  次异相）——s574 起挂 solo 定相观察
  （先例 s476/s480：持续两轮异相则
  solo 双跑定相）。
- **非码面判定**：窗 engine 码面全在
  transition/getAnimations/shim JS 面
  ，零 select/keyboard 码面；非带键
  逐键零差＋NEW/GONE 全落 band566
  族内自证——族带时序谱内相位摆动
  续轮，升级路径不触发。

## 观察哨复核（七哨全持位）

- dialog 哨：**第 7 轮 key1/key2 双 P
  持位**。
- password 哨：**第 12 轮持位兑现**
  （select-event 279 键 8 非 P 形与
  s572 逐项同口径；password 键 0F）。
- onblur 哨组 10 键（常规连轮第 5 轮
  ）：**8P/2F 恒**——F 集 ≡ s559
  逐键 IDENT。
- modulepreload 哨：15 键 stable 全 P
  ——第 35 轮零翻红。
- waapi 哨 2 键：picker-icon 双 P（
  第 94 轮）/checkmark F/F 恒形——
  **getAnimations 浮出面窗零位移**
  （waapi 哨是本轮窗内码面最近邻观察
  点，恒形＝shim 行为面未及此二键）。
- focus-dynamic 121 键 stable 全 Pass
  ：第 53 轮零再现。
- 双发伪影 2 键双腿各 ×2 恒；rc=2/2
  惯例 stack overflow 0/0。

## 下一步（s574）

- 守成巡查续轮：**对账基线换 s573
  stable 8513**；开工 pull 捕获批全列
  核实；runner 预判按 code 面＋同值
  重建规重估；全桥 b570（1728P/0F）
  持位——窗带 js_dom_bridge 新钉则
  NEW/GONE 对账（键基线 b570.keys 在
  /tmp/s572）；dialog 第 8 轮；
  password 第 13 轮；mpreload 第 36
  轮；waapi 第 95 轮；focus-dynamic
  第 54 轮；onblur 组第 6 轮；**族带
  第 7 轮观察＝回摆后形态判定**（4
  子测 Fail 双腿恒＝红池形持位确立
  ；整文件键回带＝谱内再摆；**detailed
  整文件键第二轮观察＝异相是否持续
  ，持续则 solo 双跑定相（s476/s480
  先例）**）；双发伪影恒；爆栈 0/0
  惯例；起腿三件套照规（浸泡门＋残留
  零校验＋chmod +x；sed 复制后 grep
  基线引用面两侧核对开跑前执行——
  基线误引坑三现，净版直写为首选）。
- 候门决项等待用户拍板维持（video
  P1+P2 立项 / 簇 D A2 立项 / s137；
  对端 ZRG-2026-11-18-01 归用户）。

账本=evidence/2026-11-19-m2s573-band-subtest-swing-back-detailed-first-disagree.md
