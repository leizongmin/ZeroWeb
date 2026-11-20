---
date: 2026-11-20
modules: wpt-runner
---

# 电池中断后用冻结二进制补腿，不重跑整批

## 问题

A/B 电池（如 knife-wa-battery.sh 的 strip→b0 build→apply→b1 build→12 腿→reconcile 流程）跑到一半被外部停止（systemctl stop / session 死亡 / rm 事故）时，直觉做法是把脚本整个重跑。但重跑会：①覆盖已有部分结果，若二次中断则连原账都丢；②重新 strip/replay + 重建二进制，若树上有他流在飞编辑（如 include_str! 的 shim 文件），两臂会烙入不同代际的 ride 文件，产生新的 S3-MISMATCH 归因负担；③多花一遍 build + 全部腿的墙钟。

## 根因分析

电池的两个组成部分生命周期不同：**二进制（b0/b1-runner）在 bake 完成后即冻结**，腿只是「用冻结二进制读语料写 JSON」的纯执行，不依赖树态。因此只要 ①两臂二进制文件完好且 md5 与账面一致，②已完成的腿 JSON 非空完整，③缺失的腿数可控，中断电池就可以在不重建、不剥树的前提下补完，且补完的账与原电池是同一个二进制对，归因口径不变。

## 解决方案

补全脚本三要素（实例 `.acceptance/wptfyi-css/knife-wa-r3-complete.sh`）：

1. **二进制 md5 门**：补跑前核 `b0/b1-runner` md5 与账面恒等，目录被挪或换装即 ABORT——防止在已被覆盖/改装的结果目录上写错账。
2. **只补缺失腿**：复用电池脚本 leg() 的同形命令（test-guard 包裹、同 `--per-proc-mem/--total-mem/--time-limit`、同 `CASE_WALL_S`），以 `>` 覆盖 0 字节残腿、以补齐缺腿；输出目录与原电池一致，补完即完整 12 腿账。
3. **completion 走 systemd transient 链**（等前置 unit 退场→补腿→reconcile→落完成标记），不依赖发射 session 存活。

配套纪律：

- 补跑前先 `cp -r` 快照既有部分结果（防再遭 rm——本目标曾一夜三删）。
- 发射前查全量活跃 unit + `.acceptance` NOTES 尾部声明，另一 fork 先手同案时让位（双写毒化先例），补腿与在跑电池并发会互抢 CPU 造成窗噪，reconcile 甄别。
- 中断根因要先用 `journalctl --user -u <unit>` 定谳（`Stopping` 条目＋`RuntimeMaxUSec=infinity`＋`Result=success`＝外部显式 stop；`Result=timeout` 才是超时），外部干扰与自身失败分开记账。

## 效果

knife WA 电池（2026-11-20）：r3 跑到 b1 第 2 腿被外部 stop，补全链只跑 5 条缺失腿（~13 分钟）即得与原电池同一二进制对（b0 `5cc27980`/b1 `764b081d`）的完整账，避开重建窗内 typed-om 流第 3 代编辑会造成的 ride 漂移。
