# M2 s584 — **完全零位移轮（stable 集逐字节恒 s583＝NEW 0/GONE 0/flips 0）+ band 聚合形持位第 1 连兑现轮（band8 8 子测 Fail 双腿恒）+ 全桥 b583 零位移轮（keys 1731 fail=0 NEW=0 GONE=0＝b582/b583 连续两轮零噪声；b584≡b583 逐字节恒）+ 零换代轮（pre=mid=post ae5baec4＝必换代 ×12 连后首零换代，纯 docs 窗 Fresh 预判兑现；churn Compiling ×6 全同 md5）+ 跨 clone 收敛第六实例确认轮（窗内 9f73d59c1 对端 kb88 申报 ae5baec4 ×3 恒≡s583 mid=post）+ solo 发射形复现第 6 连轮（2/2 IDENT）+ 七哨持位轮（mpreload 第 9 连兑现）**：join **stable 8517（8305P/206F/5T/1U）disagree 0 + l1only 0 + l2only 0**，s584.stable.keys 与 s583 **cmp 逐字节恒**。

日期：2026-11-20（s583 收口同日接续轮）：前 session s583
收口推送后布防 s584 三件套（09:03 waiter 第二轮臂＋post 链
＋/tmp/s584 全套预装载），本 session 接管：开工 pull 已最新
（HEAD c910151aa）→ 开窗 **09:06:21 clean-soaked**（procs=0
load<3 3×45s，仅等 ~3 分钟）HEAD=c910151aa 钉扎起腿 →
leg1 09:06:21–09:08:48（2:27，rc=2 惯例）→ leg2
09:08:49–09:10:40（1:51，rc=2 惯例）→ 桥测 24.46s rcb=0 →
analyze → solo1 09:11:16–09:11:29＋solo2 09:11:29–09:11:41
（rc=2 ×2，13s+12s）→ 收口。

## 窗底账（s584 窗＝4923a4398→c910151aa）

- 窗＝**2 commits 全 docs、零 code**＝9f73d59c1 对端
  rendering R5847（**kb88 56代 申报 ae5baec4 ×3 恒＝跨
  clone 收敛第六实例对端侧落账确认**：≡s583 mid=post
  值；序列 s561 69967b2e／s579 f37f570e／s580 0b5c2914／
  s581 95926afc／s582 a528acd8／s583 ae5baec4 六例）＋
  c910151aa 本流 s583 收口。
- 零 code＝**runner Fresh 预判兑现**（零换代；leg1
  Compiling ×6＝pull --rebase 全树 mtime churn 重编，
  产物 md5 全同 ae5baec4＝非世代更替；leg2 Compiling 0
  /Fresh 0）。

## 双腿 join（HEAD c910151aa 树）

- raw canon：leg1 **8519**／leg2 **8519**（uniq 8517 双
  腿＝双发伪影 ×2 各腿恒）＝双腿逐键全同第 2 连。
- join：**stable 8517（8305P/206F/5T/1U）+ disagree 0 +
  l1only 0 + l2only 0**——vs s583 stable（8517）：
  **NEW 0 + GONE 0 + flips 0＝完全零位移轮**；
  **s584.stable.keys 与 s583.stable.keys cmp 逐字节恒**
  （聚合形首战后首轮即集级逐字节冻结，最强持位表述）。
  状态拆分 8305P/206F/5T/1U 全恒。

## 族带第 17 轮：聚合形持位第 1 连兑现

- band8（8 子测键）双腿各 8 hits 全 Fail，逐键同态＝
  **聚合形持位第 1 连**（s583 首现后首轮冻结）；页级 T
  零再现（detailed 页持续秒级完成）。
- c503a585e 热点修复归因侧证＋1：非带 5 页级 T 键与
  全局计时零漂移（5T 逐键恒），聚合形稳定非瞬态。

## 全桥 b583 对账：零位移（连续第二轮零噪声）

- keys **1731 fail=0 NEW=0 GONE=0**（rcb=0，1731 passed
  24.46s；2501 filtered）＝b583 恒形；**b584≡b583 逐字
  节恒**（与 b582≡b583 同值链）。d63 双钉后连续两轮桥
  测键面零响应定谳。

## runner 与编译档

- pre=mid=post **ae5baec4/128,917,856**＝零换代轮（必换
  代 ×12 连后首零换代；纯 docs 窗预判兑现）。收敛第六
  实例：s583 ae5baec4 ≡ 对端 kb88（9f73d59c1 申报 ×3
  恒）＝本侧生成值与对端重建值同 md5 确认。
- leg1 Compiling ×6（churn）／Fresh ×0；leg2 全 0。
  rc leg1=2/leg2=2/bridge=0 惯例；爆栈 0/0/0 惯例。

## 七哨 + 伪影 + solo

- dialog r18：2P 双腿（17→18 连）。
- select-event r23a：279 键 8F，F-set ≡ se.577 双腿 IDENT。
- password r23b：225 键 0F 双腿。
- **mpreload r46：15/15 P 双腿＝持位第 9 连兑现**。
- waapi r105：picker P／checkmark F 恒形双腿。
- focus-dynamic r64：121/121 P 双腿。
- onblur r16：10 键 8P/2F，F-set ≡ ob.s559 双腿 IDENT。
- 双发伪影 2 键双腿各 ×2 恒（dup(2) 计数行核）。
- solo ×2（FILTER=select-home-end-pagedown-pageup.optional
  定规）：2/2 发射形 IDENT＝**第 6 连**（4 子测 Fail 恒
  形；13s+12s）；第 5 形未现。

## s585 预判与布防

- 对账基线 s584 stable 8517（＝s583 集逐字节同值，基线
  文件内容零变）；全桥基线 **b584**（≡b583）；/tmp/s585
  已预装载全套（stable keys/canon＋b584＋band8＋七哨
  keys＋脚本 s585 化；analyze2 band 注记 r18/8 keys 已修）。
- runner 预判按窗码面定（纯 docs＝Fresh/churn 同 md5；
  engine/下游码面＝必换代）；窗带 js_dom_bridge 新钉则
  全桥 NEW/GONE 对账（b584 基线）。
- 族带 r18＝**聚合形持位第 2 连观察**（8 子测 Fail 双腿
  恒＝聚合持位；页级 T 再现或新形＝谱内摆动重开）；solo
  发射形第 7 连观察；mpreload r47＝第 10 连观察。
- 三件套照规：开工 pull 捕获全列核实（窗＝c910151aa→
  s585 legs HEAD）；waiter 超时按 s580 预案重臂，kill 旧
  实例防 double-fire。
