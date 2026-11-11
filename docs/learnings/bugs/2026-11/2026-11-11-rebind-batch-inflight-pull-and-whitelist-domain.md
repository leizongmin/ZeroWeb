---
date: 2026-11-11
modules: tests/wpt-runner, .acceptance/dps
---

# 复绑批三类终局：在飞 pull 混合二进制、白名单漏刀键静默死亡、锚表盲区

## 问题描述

tq 刀收口后的复绑链（batch-tq-rebind v3→v7，2026-11-11 晨）在一小时内连续暴露三类批链终局，各 kills 一个批次：

1. **v4 白名单断言 False 后 heredoc `|| exit 1` 无标记静默退出**：judge 基础度量（P2F/lost/rows）全 OK，批在 mt 发射前无声消失，日志无 ABORT 字样。
2. **v5 launch 锚捕获入站漂移自弃**：发射后 9 分钟他流入站 5 提交，重射批的 part01c md5 断言直接 ABORT。
3. **v6 在飞中途 pull 毒化**：rollover 轮开工 `git pull --rebase` 并入他流提交（js_dom_bridge.rs 在锚表内 md5 漂移 + 两个锚表外新文件），k11 腿 `make` 触发 rebuild → 后续腿全部跑在混合二进制上，launch 锚断言不再复核，若无干预会跑完全程产出绑定幻影树的假 ALL-DONE。

## 根因分析

1. **白名单域失配**：judge F2P 白名单按「伪影键」集合设计，但基线 log 录制于刀体落地**前**（rbv2-fb 为 tq 刀前基线），刀修复键 vs 该基线必现 F2P——漏计 4 个刀键使 F2P 实测 13 ≠ 期望 9。规格错而非行为漂移，但静默退出使其看起来像批挂死。
2. **锚表盲区**：md5 锚表是手工清单，只覆盖上一轮已知入站面；新入站提交触碰清单外文件（尤其**新增文件** callbacks.rs / position_area.rs）不被断言捕获，judge 域与提交树静默脱钩。
3. **开工 pull 与在飞批互斥未成文**：批脚本所有锚断言（HEAD/md5/特征串）只在 launch 执行一次；批在飞数小时窗口内任何 `pull --rebase` 改动 crates/ 面，都会让下一个 `make` 腿 rebuild 出与锚不一致的二进制。rollover 轮按规程「开工先 pull」执行，恰好踩进 v6 的在飞窗。

## 解决方案

1. heredoc/内联 python 判读的失败路径**必须带显式 ABORT 标记**（`echo "ABORT ..." ; exit 1`），让批死因可 grep；白名单期望 = 伪影键集 ∪ 「刀键 vs 刀前基线」必现 F2P 集。
2. 发射复绑批前先 `git log --stat` 入站 range 的 crates/ 面**逐文件**比对锚表，新增文件也补锚（v5 五点扩锚、v7 又补 2 点——两度实证盲区不可枚举预防，只能逐波对账）。
3. rollover/接手轮开工顺序改为：**先查在飞批（flock 持有、pid 活性、日志终态标记）再 pull**；在飞即停批（杀序：批脚本→腿进程树→fd 穷举验证锁释放，`exec 9>` 锁 fd 被子链继承）或延后 pull。混合二进制批判读不跨批传递，停批全腿重跑。
