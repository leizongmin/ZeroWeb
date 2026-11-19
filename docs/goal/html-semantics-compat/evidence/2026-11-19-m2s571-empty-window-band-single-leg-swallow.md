# M2 s571 — **空窗巡查轮（pull 零捕获 0 commit）+ 同值重建判代新规首用申报=实测轮（Compiling ×6 字节恒 f0ebaf5b）+ 族带单腿吞形归因轮（GONE 4 全落带内、solo ×2 回位定谳）+ 六哨持位轮**：join **stable 8510（8305P/198F/6T/1U）disagree 0 + l1only 1 + l2only 4（raw 双腿 8513/8516）**；vs s570 stable（8514）**GONE 4（全族带 optional 变体子测键）+ NEW 0 + flips 0，非带键逐键零差**；runner **pre=mid=post 全恒 f0ebaf5b/128,508,616**——leg1 Compiling ×6 同值重建（s570 新规「docs-only/空窗＝同值重建形、判代以 mid md5 为准」首用即兑现＝申报=实测）；归因三证＝同二进制（字节恒）＋窗 code ×0（空窗）＋solo ×2 全数回位（4 子测键 Fail 红池形＝s553 在册形），**判族带时序谱内 leg1 单腿 harness-no-report 相位（pending:4 零报告形）非码面非新极形**。

日期：2026-11-19（s570 收口同日第三轮；本
session 10:32 滚入开工：pull 零捕获
（4b513af17 已是远端最新，**s571 窗＝
4b513af17 钉扎空窗 0 commit——本流推
送后对端未再落窗的首例空窗**）→
10:33 布置 s571 全链——**sed 基线误引
坑第二次复现被开跑前校验拦下**（sed
把 comm/join 两侧基线名一并改写致左右
同源，直接写净版修正；s569「analyze.sh
基线误引」教训同类，grep 核对
s570/s571.stable 引用面后放行）＋grep
旧轮路径残留为零校验＋chmod +x 核验
→ 10:33 waiter 绝对路径启动 → v3.2
判净 10:35:16（load<3 procs=0 3×45s）
HEAD=4b513af17 钉扎起腿 → leg1
10:35:16–10:38:23（3:07，Compiling ×6
同值重建，rc=2 惯例）→ leg2
10:38:23–10:40:43（2:20，Fresh ×0，
rc=2 惯例）→ analyze 10:40 自治 →
solo ×2 回位定谳 → 哨复核。

## 窗底账（s571 窗＝4b513af17 钉扎空窗）

- **pull 零捕获＝0 commit 空窗**（本流
  s570 收口 4b513af17 推送后对端未再
  落窗；HEAD 与 s570 腿 d120efc18 相差
  两 docs commit＝f441bf1b2 R5820 rebase
  沉底＋本流收口）。
- runner 预判按 s570 新规重估：**HEAD
  位移（两 docs commit）→ product-version
  链 build.rs 重跑 → Compiling ×N 预期
  ＋md5 恒（BUILD_COMMIT 常量死代码剥
  离不入字节）＝同值重建形**——实测
  leg1 Compiling ×6（product-version→
  net→page-runtime→webview→runner）+
  pre=mid=post 全恒 f0ebaf5b
  /128,508,616＝**新规首用申报=实测**；
  leg2 Fresh ×0。
- 全桥：空窗零码面零新钉 → 桥清单持
  位 b569 基线 1727P/0F（s570 收养定
  谳值），本轮无桥测安排，下轮若窗带
  js_dom_bridge 新钉则 NEW/GONE 对账。

## 双腿 join（HEAD 4b513af17 树）

- v3.2 判净 10:35:16 HEAD=4b513af17
  钉扎起腿；rc=2/2 惯例，stack overflow
  0/0。
- raw canon：leg1 **8513** 行（−3＝
  optional 4 子测键未发射＋optional 整
  文件键 1 行发射）/leg2 8516 行（与
  s570 同形恒）；双发伪影 2 键双腿各
  ×2 恒 Pass 形。
- join：**stable 8510（8305P/198F/6T
  /1U）+ disagree 0 + l1only 1 + l2only
  4**——P/T/U 三桶与 s570 逐项恒，
  F 桶 202→198 恰=4 GONE 族键；
  l1only＝optional 整文件键（leg1 单腿
  发射）。
- **vs s570 stable（8514）：GONE 4 +
  NEW 0 + flips 0**——GONE 全落族带
  optional 变体（End/Home/PageDown/
  PageUp 4 子测键），非带非哨键逐键
  零差。

## 族带单腿吞形归因（s476/s480 先例流程）

- leg1 发射形＝**harness-no-report 超
  时形**：optional+detailed 双变体整文
  件键 Timeout（"testharness completed
  without reporting registered tests
  (state={"loaded":false,"pending":4,
  "phase":4,"tests":4}"）＝s554 在册谱
  形（90s 0-declared→harness-no-report
  →完整/部分发射四形谱）；leg2 optional
  4 子测完整发射（l2only 4）。
- **归因三证**：①同二进制——runner
  pre=mid=post 字节恒 f0ebaf5b（与
  s570 腿同 md5，本窗零码面）；②空窗
  ——code ×0 pull 零捕获；③solo ×2
  全数回位——FILTER=s
  elect-home-end-pagedown-pageup.optional
  双跑均 4 子测键发射且同相 Fail（
  assert_true expected true got false
  ＝s553 在册红池形）。
- **判决：族带时序谱内 leg1 单腿相位
  （harness-no-report 形再现），非码面
  非新极形**——s530 起族带摆动在册，
  升级路径（单页探针＋对端 pre-knife
  反事实 A/B 前置申请）不触发（solo
  回位已定谳）；带外无任何位移自证。

## 观察哨复核（六哨持位＋族带分裂形）

- dialog 哨：**第 5 轮 key1/key2 双 P
  持位**。
- password 哨：**第 10 轮持位兑现**
  （select-event 279 键 8 非 P 形与
  s569/s570 逐项同口径全同；password
  键 0F；整文件键 0）。
- onblur 哨组 10 键（常规连轮第 3 轮
  ）：**8P/2F 恒**——F 集 ≡ s559 逐
  字节 IDENT；following-label 双键 P/P。
- modulepreload 哨：15 键 stable 全 P
  ——第 33 轮零翻红。
- waapi 哨 2 键：picker-icon 双 P（第
  92 轮）/checkmark F/F 恒形。
- focus-dynamic 121 键 stable 全 Pass：
  第 51 轮零再现。
- 族带 home-end：**分裂形第 1 轮**——
  detailed 整文件键 Timeout 持位
  stable；optional 4 子测键 stable 外
  （leg1 吞形×leg2 发射不入 stable）＝
  GONE 4 全落带内；回潮判定挂 s572（
  双腿完整发射则回潮形）。

## 下一步（s572）

- 守成巡查续轮：**对账基线换 s571
  stable 8510**；开工 pull 捕获批全列
  核实（空窗后首个新窗，对端落窗即
  非空）；runner 预判按 code 面＋同值
  重建规重估；若窗带 js_dom_bridge 新
  钉则全桥 NEW/GONE 对账（基线 b569
  1727P/0F）；dialog 第 6 轮；password
  第 11 轮；mpreload 第 34 轮；waapi
  第 93 轮；focus-dynamic 第 52 轮；
  onblur 组第 4 轮；**族带第 5 轮观察
  ＝分裂形回潮判定**（optional 4 子测
  回 stable＝回潮形；solo 红池形 Fail
  4 键回 stable 亦属带内）；双发伪影
  恒；爆栈 0/0 惯例；起腿三件套照规
  （浸泡门＋残留零校验＋chmod +x；
  sed 复制后 grep 基线引用面两侧核对
  开跑前执行）。
- 候门决项等待用户拍板维持（video
  P1+P2 立项 / 簇 D A2 立项 / s137；
  对端 ZRG-2026-11-18-01 归用户）。

账本=evidence/2026-11-19-m2s571-empty-window-band-single-leg-swallow.md
