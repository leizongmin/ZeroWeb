# M2 s570 — **s569 全桥收养定谳轮（b569 预判精确兑现）+ 守成巡查轮 + 基线恒等第 2 连（NEW 0/GONE 0/flips 0）+ 同值重建判代新证轮（Compiling ×6 字节恒）+ 七哨全持位轮**：s569 桥测在飞工件 10:07 落地收养——**b569＝1727P/0F rc=0（filtered 2487）＝预判 b568（1726）+NEW 1（exec 门钉 1383ac371）+GONE 0 精确兑现，刀 G 归因不触发**，全桥基线换 b569 实测值；s570 双腿 join **stable 8514（8305P/202F/6T/1U）disagree 0 + l1only 0 + l2only 0（raw 双腿 8516/8516）**；vs s569 stable（8514）**NEW 0 + GONE 0 + flips 0＝基线恒等第 2 连**；runner **pre=mid=post 全恒 f0ebaf5b/128,508,616**——leg1 Compiling ×6（product-version→net→page-runtime→webview→runner）但**字节恒＝同值重建非换代**（本流首例，与 rendering 流 kb52「Compiling ×6 字节恒等」跨流对偶互证）。

日期：2026-11-19（s569 收口同日续轮；本
session 10:05 滚入开工：pull 零捕获
（已经是最新的）→ 10:05-10:07 收养判定
→ 10:11 布置 s570 全链（grep 旧轮路径
残留为零校验＋chmod +x 核验双过）→
10:12 waiter 绝对路径启动 → v3.2 判净
10:14:24 HEAD=d120efc18 钉扎起腿 →
leg1 10:14:24–10:17:19（2:55，Compiling
×6 同值重建，rc=2 惯例）→ leg2
10:17:19–10:19（Fresh ×0，rc=2 惯例）→
analyze 10:19 自治（全零 split）→ 哨
复核全持位。桥测收养时序：wait-bridge4h
08:54 布防（deadline 12:54）→ 让位窗
72 min（对端 ZW-3 d56 A/B 系列＋peer
corpus 长跑）→ 10:06:39 判净（load<3
procs=0 3×45s）→ 10:06-10:07 桥测
16.82s 完成 → bridge.done/rc=0 双落。

## 收养判定（s569 全桥 4h waiter 工件）

- bridge.judge 10:06:39 clean-soaked；
  s569-bridge.rc=**0**；
  **1727 passed; 0 failed**（16.82s，
  filtered 2487）。
- 预判对账：b568（1726）+NEW 1
  （exec 门钉）+GONE 0 → 实测 **1727P
  /0F 恰合＝「申报=实测」**；意外偏离
  无，**刀 G 归因不触发**。+1 键落窗归
  因＝1383ac371 刀 exec2/exec3
  execCommand 双门（s569 窗 code ×7 之
  一，merge-base --is-ancestor 核实在
  窗）。
- bridge.runner.md5＝f0ebaf5b＝s569 腿
  post＝**同二进制**；**全桥基线换
  b569 实测值完成**（s570 窗 code ×0
  → 桥清单持位 1727 预期，下轮对账
  基线）。

## 窗底账（s570 窗＝5ca52641c→d120efc18，
15 commit 全列核实＝本流 s568 补账
6380462dd＋s569 收口 d120efc18＋对端
rendering R5811-R5819 九轮＋d56 三轮
（35e1dcd7d/b4d20851b/7b803b182）＋
learnings 0b449b27f）

- **code ×0＋merge ×0＝纯 docs 窗**（
  本轮开工 pull 零捕获，全列 git log
  核实 15 行）。
- runner 预判教训入账：开工按 code ×0
  预判「不换代不重编」——**实测 leg1
  Compiling ×6**，预判表述作废，根因
  ＝build-support/product_version.rs
  `rerun-if-changed=.git/HEAD|.git/
  logs/HEAD`（HEAD 任何位移含 docs-only
  → build.rs 重跑 → 全链重编）。

## 判代新证（同值重建＝Compiling ≠ 换代）

- runner **pre=mid=post 三点全恒
  f0ebaf5b/128,508,616**；leg1 Compiling
  ×6 字节恒、leg2 Fresh ×0。
- 根因定谳：product-version 的
  ZERO_BUILD_COMMIT/ZERO_BUILD_VERSION
  常量**不被 runner 链接图任何代码引用
  （grep 全树仅 product-version lib +
  browser/webdriver app 消费）→ 链接期
  死代码剥离**——二进制内嵌串搜
  6380462/d120efc 均 0 命中实证；
  重编只是 build.rs 指纹搅动传导
  （rustc 重跑输出元数据变、链接产物
  不变）。
- **判代规则精化入规：runner 换代
  （md5 变）预判只看窗内 code 面是否
  触 runner 链接图；docs-only 窗预期
  ＝同值重建（Compiling ×N＋md5 恒）；
  判代以 mid md5 为准，Compiling 行数
  ≠换代**。本流首例，与 rendering 流
  kb52 对偶观察（R5811「Compiling ×6
  字节恒等＝同值重建非换代」）跨流互
  证；rendering 流「product-version 搅
  动面常态注记」同根。

## 双腿 join（HEAD d120efc18 树）

- v3.2 判净 10:14:24（load<3 procs=0
  3×45s）HEAD=d120efc18 钉扎起腿。
- raw canon：leg1 8516 行/leg2 8516 行
  （vs s569 同形恒：双发伪影 2 键双腿
  各 ×2 恒 Pass 形；log 尾与 s569 同位
  ＝corpus 末尾 the-time-element 完整
  跑完）。
- join：**stable 8514 + disagree 0 +
  l1only 0 + l2only 0**（stable canon
  8305P/202F/6T/1U——与 s569 四档逐项
  恒）。
- **vs s569 stable（8514）：NEW 0 +
  GONE 0 + flips 0＝基线恒等第 2 连
  （s569 第 1 连）**——纯 docs 窗行为
  面零位移预期兑现，非族非哨键零漂移。
- rc=2/2 惯例（make Error 1＝runner
  退码非零红在册），stack overflow 计
  数 0/0。

## 观察哨复核（七哨全持位）

- dialog 哨（结案转常规连轮）：**第 4
  轮 key1/key2 双 P 持位**。
- password 哨：**第 9 轮持位兑现**
  （select-event 279 键 8 非 P 形与
  s569 逐项同口径全同；password 键
  0F；整文件键 0）。
- onblur 哨组 10 键（结案后常规连轮
  **新计第 2 轮**）：**8P/2F 恒**——F
  集（parent-label 双键）stable F/F 恒
  且 ≡ s559 逐字节 IDENT（按哨键集口
  径 diff 全等）；following-label 双键
  P/P 持位。
- modulepreload 哨：15 键 stable 全
  P——常规连轮计数第 32 轮零翻红。
- waapi 哨 2 键：picker-icon 双 P
  （s478 刀增益第 91 轮）/checkmark
  F/F 恒形。
- focus-dynamic 121 键 stable 全 Pass：
  第 50 轮零再现。
- 族带 home-end：**全 5 键齐现 stable
  回潮形第 3 轮**（detailed T/T＋End/
  Home/PageDown/PageUp F×4 持位；全局
  flips 0 自证族键零位移）。
- 双发伪影 2 键（exact len/for='
  WINdow）双腿各 ×2 恒 Pass 形。

## 下一步（s571）

- 守成巡查续轮：**对账基线换 s570
  stable 8514**；开工 pull 捕获批全列
  核实；runner 预判按 code 面重估（
  docs-only 窗＝同值重建形；code 刀窗
  ＝必换代）＋若带 js_dom_bridge 新钉
  则全桥 NEW/GONE 对账（基线 b569
  1727P/0F）；dialog 第 5 轮；password
  第 10 轮；mpreload 第 33 轮；waapi
  第 92 轮；focus-dynamic 第 51 轮；
  onblur 组第 3 轮；族带第 4 轮观察；
  双发伪影恒；爆栈 0/0 惯例；起腿一律
  v3.1 浸泡门＋grep 旧轮路径残留为零
  校验＋chmod +x 核验。
- 候门决项等待用户拍板维持（video
  P1+P2 立项 / 簇 D A2 立项 / s137；
  对端 ZRG-2026-11-18-01 归用户）。

账本=evidence/2026-11-19-m2s570-bridge-adopted-baseline-identical-samevalue-rebuild.md
