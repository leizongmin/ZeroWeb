# M3 迁移前基线落账 + 组成归因（老代码同语料重跑实验）

日期：2026-11-08。命令：`make testharness-net-api`（test-guard 包裹
`--per-proc-mem 4 --total-mem 8 --time-limit 900`，systemd-run MemoryMax=12G）。
两条全量日志（本地工件不入库）：基线
`/tmp/m3-baseline-164121.log`（16:41–16:49）+ 归因重跑
`/tmp/m4code-reanchor-171102.log`（17:12–17:18）。

## 一、基线账（M3 A/B 锚，m3-baseline-164121）

- 树 = 411e862bb 代码态 + 3f2d80d27 机械 rustfmt（五文件，无行为面）；
  二进制 run 内新建（36.18s Finished）；语料 = 当前 wpt-data 工作区。
- **全通道 13,582P / 1,337F / 8T（14,927 subtest）**；其中 wss 变体池
  **360P / 101 键全 Pass**（与 M2 档逐键同口径），基础面（非 wss）
  **13,222P / 1,337F / 8T = 14,567 subtest**。
- 域分布：fetch 2,349/183/1 · xhr 1,006/240/2 · websockets 381+360/0/0 ·
  streams 918/96/5 · url 7,196/216/0 · mimesniff 1,296/602/0 ·
  eventsource 76/0/0。
- 稳定域逐一对账既有锚：websockets 基础 381P/0F/0T = M2 A/B 锚全等；
  eventsource 76P = 43 号 M4 终态全等；mimesniff = M0 档全等；
  streams 918/96/5 = M0 档 ±1P（5T 同集，含 4×readable-byte-streams
  既有挂账——非新回归）。

## 二、组成归因（老代码同语料重跑实验）

对照三态：**M4 终账**（c80e3dd16，2026-11-07 当日语料）13,275P/1,412F/9T
= 14,696；**老代码×今日语料**（c80e3dd16 重跑）13,150P/1,409F/8T = 14,567；
**新代码×今日语料**（基线）基础面 13,222P/1,337F/8T = 14,567。

1. **分母 −129 = 语料漂移，非代码**——老代码在今日语料上重跑分母
   14,567 与新代码逐 subtest 同分母。43 号 M4 当日语料态已不可复现
   （wpt-data 为 gitignored 滚动工作区，M4 窗口后 fetch/api/redirect
   16 个 `.any.js` 等再拉入），跨窗总数对比按 v1 M5-S6 判例不作账。
2. **代码差 = fetch 域 +72P/−72F，零 P→F 零新 T**——老×新代码同语料
   逐域比对，七域中仅 fetch 变化（P 2,277→2,349 / F 255→183），其余
   六域逐 subtest 全等。归因 = M4 窗口后落树的引擎/net 提交：
   http-cache M1-b（cache.py 端点 + 案级缓存包裹，C 簇 48 绿）+
   CacheMode M1-a（reload/force-cache 注入映射修正）——净效应 72 个
   F→P，基线头条 13,582P 含此增量。
3. **wss +360 = M2 接线**（101 键变体枚举追加式，M2 档已落账）。

## 三、口径回填

- 入口文档/master「零回归复验面 2」的「net-api 全通道 13,275P 面
  （43 号 M4 终账）」自本档起锚定为**基线 13,582P/1,337F/8T**
  （m3-baseline-164121）；M3 迁移 B 面逐 subtest A/B 对照本基线日志，
  changed 集为空即零回归（wss 池按 M2 档口径显式声明内含）。
- 头条不回落（DC-4）：13,582P ≥ 13,275P ✓（含语料漂移 + 代码增量
  + wss 池三项，本档逐项归因）。
- 实验方法注记：老代码重跑前 5 分钟 stdout 块缓冲零输出，非挂死；
  test-guard 900s 墙钟未触发（6 分钟完成）。老代码无法 ptrace 取栈
  （yama scope），结论以终账数字为准。

## 四、对 M3 的影响

- 基线有效性与 provenance 三重核实（树出处 / run 内新建二进制 /
  域级对账既有锚）完成，迁移开工无前置阻塞。
- fetch 域 ±72 为 http-cache 目标行为改善（47 号域），迁移 A/B 判读
  以「与基线 changed 空集」为准——迁移不触碰任何行为面，B 面应
  逐字节复现本基线（含这 72 个 P 与 1,337 个 F 的既有分布）。
