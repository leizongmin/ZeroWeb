# M2 s585 — **完全零位移第 2 连轮（stable 集逐字节恒 s584＝NEW 0/GONE 0/flips 0）+ band 聚合形持位第 2 连兑现轮（band8 hits=8×2 全 Fail）+ 全桥 b584 零位移轮（keys 1731 fail=0＝b582/b583/b584 连续三轮零噪声；b585≡b584 逐字节恒）+ 零换代第 2 连轮（pre=mid=post ae5baec4；churn Compiling ×6 全同 md5）+ 空窗轮（窗＝c4d86bd8e→c4d86bd8e＝0 commits，legs HEAD＝本流 s584 收口笔）+ solo 发射形复现第 7 连轮（2/2 IDENT）+ 七哨持位轮（mpreload 第 10 连兑现）**：join **stable 8517（8305P/206F/5T/1U）disagree 0 + l1only 0 + l2only 0**，s585.stable.keys 与 s584 **cmp 逐字节恒**。

日期：2026-11-20（s584 收口同日接续轮）：前 session s584
收口推送后布防 s585 三件套（09:3x 臂；对端构建 load 12.8
压窗 ~50 分钟），本 session 接管续盯：开窗 **09:57:23
clean-soaked** HEAD=**c4d86bd8e** 钉扎起腿 → leg1
09:57:23–09:59:5x → leg2 →0:01:5x → 桥测 19.58s rcb=0 →
analyze → solo1 10:02:52–10:03:03＋solo2 10:03:03–10:03:14
（rc=2 ×2）→ 收口。

## 窗底账（s585 窗＝c4d86bd8e→c4d86bd8e）

- **空窗＝0 commits**（legs HEAD＝本流 s584 收口笔
  c4d86bd8e；窗内零远端进笔＝对端与零code窗持续）。
- 零 code＝**runner Fresh/churn 预判兑现**（零换代第 2
  连；leg1 Compiling ×6＝mtime churn 重编产物全同
  ae5baec4；leg2 Compiling 0）。

## 双腿 join（HEAD c4d86bd8e 树）

- raw canon：leg1 **8519**／leg2 **8519**（uniq 8517 双
  腿＝双发伪影 ×2 各腿恒）＝双腿逐键全同第 3 连。
- join：**stable 8517（8305P/206F/5T/1U）+ disagree 0 +
  l1only 0 + l2only 0**——vs s584 stable（8517）：
  **NEW 0 + GONE 0 + flips 0＝完全零位移第 2 连**；
  **s585.stable.keys cmp 逐字节恒 s584**（聚合形集级冻
  结第 2 连）。

## 族带第 18 轮：聚合形持位第 2 连兑现

- band8 双腿各 hits=8 全 Fail 逐键同态；页级 T 零再现。
  聚合形（s583 首现）持位 2 连，c503a585e 归因侧证＋2。

## 全桥 + runner + 惯例

- 全桥 **keys 1731 fail=0 NEW=0 GONE=0**（rcb=0，19.58s）
  ＝**b582/b583/b584 连续三轮零噪声**；b585≡b584 逐字节
  （md5 66995b33 链）。
- runner pre=mid=post **ae5baec4/128,917,856**＝零换代
  第 2 连（收敛第六实例值持位）。
- rc 2/2/0；爆栈 0/0/0 惯例。

## 七哨 + 伪影 + solo

- dialog r19：2P 双腿（18→19 连）。
- select-event r24a：279 键 8F，F-set ≡ se.577 双腿 IDENT。
- password r24b：225 键 0F 双腿。
- **mpreload r47：15/15 P 双腿＝持位第 10 连兑现**。
- waapi r106：picker P／checkmark F 恒形双腿。
- focus-dynamic r65：121/121 P 双腿。
- onblur r17：10 键 8P/2F，F-set ≡ ob.s559 双腿 IDENT。
- 双发伪影 2 键双腿各 ×2 恒；band r18 hits=8×2。
- solo ×2：2/2 发射形 IDENT＝**第 7 连**（4 子测 Fail 恒
  形；11s+11s）；第 5 形未现。

## s586 预判与布防

- 对账基线 s585 stable 8517（集内容三连逐字节同值）；
  全桥基线 **b585**（≡b584）；/tmp/s586 已预装载全套
  （脚本 s586 化，哨 r 注记已 bump）。
- runner 预判按窗码面定（纯 docs＝Fresh/churn 同 md5；
  engine/下游码面＝必换代）；窗带 js_dom_bridge 新钉则
  全桥 NEW/GONE 对账（b585 基线）。
- 族带 r19＝**聚合形持位第 3 连观察**；solo 发射形第 8
  连观察；mpreload r48＝第 11 连观察。
- 三件套照规：开工 pull 捕获全列核实（窗＝c4d86bd8e→
  s586 legs HEAD）；waiter 超时按 s580 预案重臂，kill 旧
  实例防 double-fire。
