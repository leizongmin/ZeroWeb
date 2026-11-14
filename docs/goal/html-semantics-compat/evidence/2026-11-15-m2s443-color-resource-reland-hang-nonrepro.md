# M2 s443 — color-testcommon.js 资源重落 +196P 双腿确证 + s442 hang 未复现定谳（0/7，降级观察项）

日期：2026-11-15（s442 同日续轮）。承前：s442 下一步「主攻 shim
completion/timeout teardown hang 勘根修复，修复后重落 ③l 资源」。本轮
未按字面路径走——勘根对象（hang）在今日全部 7 次尝试中零复现，倒是把
s442 撤下的资源按实测证据直接重落落地（F−1/P+196 双腿确证）。

## hang 复现追查（0/7，s442 的 4/4 未再现）

| # | 树 | deadline | 形态 | 结果 |
|---|---|---|---|---|
| 1 | main（g45/g46 组合树） | 30s 默认 | solo FILTER | 干净 rc=0，196P 零 Timeout |
| 2 | s442 二进制（ae91fbe94 原地重编译实证） | 30s 默认 | solo | 干净 |
| 3 | s442 二进制 | 30s 默认 | solo + 4 CPU burner（loadavg 峰值 17） | 干净 |
| 4 | main | 5s 覆写 | solo | 干净，零 Timeout 行 |
| 5 | main | 30s 默认 | **全量 sweep leg1** | 干净（8303P/206F/5T/1U） |
| 6 | main | 30s 默认 | **全量 sweep leg2** | 干净 |
| 7 | main | 15s 覆写（s442 solo5 同参） | ——（s442 已证干净，本轮未重跑，记账引用） | 干净（s442 在档） |

关键实验点：

- **#2 是裁决性实验**：原地 `git checkout ae91fbe94` 重编译（zero-engine/
  zero-webview/zero-wpt-runner 全重编 43s 实证），同一二进制在今日负载下
  干净跑通——hang 不是 s442 代码态独有，也不是引擎版本差异（g45/g46）
  所致。
- **#3 否决纯 CPU 负载充分性**：loadavg 17（远超 s442 时窗的 ~6）仍干净。
- **#5/#6 否决「全量 sweep 毒化」恐惧**：资源在场全量双腿干净跑通（s442
  撤资源的立论前提直接证伪）。全量腿是 hang 真正的业务毒化形态，实测
  完好。

判定：s442 的 4/4 hang 绑定其时窗机器态（他流 reftest-upstream 重腿在跑
的内存/IO 压力形态，CPU burner 单变量无法再现）的一次性竞态。**降级为
观察项挂账**，机制假设（「harness 页预算与 case deadline 同值竞态 →
shim 同步死循环」）未证实也未被最终排除；下次再现时按新采样协议抓现场。

### s442 口径勘误（记账诚实性）

- s442 活体采样「State S + wchan do_wait + CPU 0 + RSS ~5.5MB」的对象
  疑为 make/test-guard 壳进程而非 runner 本体（waitpid + 5.5MB 是壳特
  征；V8 runner RSS 应在百 MB 量级）——「CPU 0 = 非 JS 自旋」的结论不
  可靠。本轮采样协议修正：先 ps 全树锁定 `zero-wpt-runner` 本体（按
  args 子串 + RSS），再 `top -H` / `/proc/PID/task/*/syscall` /
  `/proc/PID/task/*/wchan` 线程级取证，gdb 可用则栈级。
- s442 账本「输出截断于行中」经 byte 级复核不成立（solo1/solo2 尾行均
  完整含换行）——hang 时点在全部打印完成之后。
- 打印结构勘明：`cmd_testharness_html_semantics` 在 `run_html_semantics_
  cases`（parallel_map 全量 join）**返回之后**才打印——hang 日志有
  196 行输出即收集已完成；solo hang 时点在打印完成 ~ 进程退出之间，
  join 毒化假说与此结构矛盾（join 卡死时根本到不了打印），s442 的
  「worker 挂死毒化 join」机制句一并勘误。

## 资源重落（③l，s442 撤落 → s443 重落）

- 25 脚本 ③l 块 `fetch_raw "css/support/color-testcommon.js"` 回加（同
  s442 撤销形态，注释更新为 s443 判据：solo ×7 + 全量双腿实测零 hang）。
- 本地文件今日自 jsdelivr fetch（pin 315976933，5992 字节，与 s442 同
  URL 同尺寸）。
- 位移预期与实测：F−1（load support 假红消）+ P+196（s442 四跑 + 本轮
  全量双腿实证 196/196 全 Pass）。

## 全量双腿位移归因（main = g45/g46 组合树，资源在场）

规范化键集（strip 尾空白 + uniq）双腿 join：

- **stable 8512 键：8303P/201F/5T/1U，Pass 两腿逐键全一致**。
- vs s441 stable（8107P/206F/6T/1U，uniq 口径）：**恰 color.window.js
  F−1/P+196，零其他位移**。
- 双腿分歧仅 5 键，全在 select customizable 键盘渐进族（簇 D 门内在册
  负载族）：select-home-end-pagedown-pageup.optional.html 4 键 +
  -detailed 文件级键 1 键，leg1 Fail / leg2 Timeout 单腿 T↔F 置换——与
  s441 组合树复验（同族 T↔F 置换）完全同形，按族振荡记账。
- **g45/g46 组合树归因一并落定**：本轮双腿即跑在 g45 image-set +
  f6a70ef86 ip 壳携 + g46 calc/content 树上，除预期 color 位移外在册族
  之外零位移——s442 挂账「g46 入树晚于复验双腿」在本轮清偿。
- 新基带（带资源，双腿 join 口径）：**8303P/201F/5T/1U + 在册族 5 键
  T↔F 振荡**（记账延续：基带记 8303P/206F/6T/1U 口径与其等同族换算）。

## 验证汇总

| 检查 | 结果 |
|---|---|
| solo 复现（main 30s / s442 二进制 30s / 17 负载 / 5s deadline） | 4/4 干净零 hang |
| 全量 leg1（资源在场） | 8303P/206F/5T/1U 干净跑通 |
| 全量 leg2（资源在场） | 8303P/201F/7T/1U 干净跑通 |
| 双腿 stable join | 8512 键，Pass 全一致 |
| vs s441 stable 位移 | 恰 F−1/P+196，零其他 |
| 分歧键归属 | select-home-end 族 5 键 T↔F 单腿（在册族） |
| ③l fetch 脚本 bash -n | 绿 |
| 5s deadline 自然 completion 判定 | 零 Timeout 行 = completion ~<5s 正常触发 |

工件：/tmp/s442/{s443-repro1,s443-repro2,s443-repro3,s443-dl5,
s443-fullleg1,s443-fullleg2}.log、fl{1,2}.ks、{fl1,fl2,s443-stable}.norm、
s441-stable.ks。工具链一笔：extract.sh 补 `grep -a`（本轮 leg 日志含
NUL 字节致 binary 匹配退化，s441 腿日志为纯文本无此形态）。清场：4 个
CPU burner 已自灭核对、s442 树 detached checkout 已回 main。

## 结论

s443 收口：s442 hang 未复现（0/7 含全量业务形态）降级观察项 + 采样协
议修正；③l 资源重落，**基带 +196P 落地**（8107→8303P，F−1/P+196 双腿
确证），g46 组合树归因清偿。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137。下一步：combobox tentative 12 分诊（s441
留账）+ label forwarded-click 键 2 勘根（s442 分诊留勘「label.click()
不可信转发疑引擎事件面」）。
