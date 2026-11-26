# 批四十九门重探四演判读＝CAPKILL_BALLOON 门不开（四连）

**2026-11-26 06:43Z（14:43 本地）判读**。arm（05:25Z）树头 b07f9a77f
（刀㉛＋刀㉜＋k140 面），build 实取 live HEAD bdc8bcbff（本流 docs 笔
恰在 build 前落树＝docs-only 恒等形，码面同 b07f9a77f）。

## 判定

- test262-builtins 探针 rc=124，guard 击杀 `单进程内存超限
  (6317064 KB > 6291456 KB)`，soak 结束（05:26:42Z）后 ~73min
  击杀（06:39:37Z），json 0 字节＝INCOMPLETE。
- workers 腿（head 面二进制）rc=1＝套件含非 Pass 常态退出：175 files
  696P **SET-IDENTICAL 四连**（json 106,832 B 与 b46 基线字节同尺寸；
  RSS 4,442,744 KB＝已知 4.5GB 档），06:43:59Z 完跑。
- SUMMARY 原文：
  `HEAD_SHA=bdc8bcbff VERDICT=CAPKILL_BALLOON RC=124 PEAK_KB=6317064
  JSON=INCOMPLETE / WORKERS_RC=1 COMPARE_TAIL=VERDICT: SET-IDENTICAL`。

## 六源同带（气球确定性）

| 源 | face | peak KB | 击杀点 |
|---|---|---|---|
| CI batch48 | 8485db7a76 | 6,309,664 | — |
| 本地探针 A | p8 面 | 6,338,488 | 67min |
| 五臂 | adf767f5b（s10k28 批） | 6,297,236 | ~70min |
| 二演 | a842bf953（返修三g） | 6,370,364 | ~72.3min |
| 三演 | 2ea852d1c（返修三g＋刀㉚） | 6,344,380 | ~73.6min |
| 四演 | b07f9a77f/bdc8bcbff（＋刀㉛㉜＋k140） | 6,317,064 | ~72min |

六源带 6.30-6.43GB（<1% 离散）、击杀点 ~67-74min 漂移＝中途存活读数
不作数，唯完跑/cap-kill 可判。**排除清单扩至**：刀㉗/㉘/㉘-B（TT/CSP
嫌疑域）、刀㉙（TT eval 门惰性装置）、s10k25返修~返修三g、刀㉛/㉜
（engine reactions 簇）、k140（bind journal）——全部未解＝气球仍归
对端 TT/CSP 链，culprit 目录 `test/built-ins`（bisect 定谳），机制
画像＝6GB 在渲染子进程侧。本流不硬修（master 卡点边界不变）。

## face 后漂移甄别（冻结判形）

bdc8bcbff 之后落树：k141 c734bfa4f（`engine/js_dom_bridge/
computed_style_cache.rs`＝css 语义刀，getComputedStyle 代际缓存——
test262 built-ins 纯 JS 不触 computed style＝判读洗清形，下笔重探
自然携带）＋docs-only 五笔（f92f1ba44/d04d9496c/0dd7262e3/881f86dd4/
f5592a654）。**无 TT/CSP/engine 行为域新笔＝不触发重演**；CAPKILL
下 gate 本就不开，漂移不影响任何判定。

## 下一步（后续轮次承接）

守对端 TT/CSP 下笔 → 落树后重跑门链（HEAD 重取＝冻结判形与 drift
域甄别不变；判据 CLEAN rc=0 peak<1GiB json COMPLETE＋workers
SET-IDENTICAL）：

```
nohup /tmp/batch48/b49-gate-chain-v3.sh \
  > /tmp/batch48/b49-gate-chain-v3-nohup.log 2>&1 &
echo $! > /tmp/batch48/b49-gate-chain-v3.pid
```

判读：查 `/tmp/batch48/B49_GATE_DONE` → 读 `B49_GATE_V3_SUMMARY`；
双 CLEAN → `/tmp/batch49/dispatch49.sh`（pull→dispatch
`-f run_wptfyi_coverage=true`→RUNID.txt→waiter/autodl/failarch/
shas-gated/finalarch 收账族；bash -n 已核）。

工件：`/tmp/batch48/{B49_GATE_V3_SUMMARY,B49_GATE_DONE}.run3`（本轮
留档）、`b49-gate-chain-v3.log`（全五演链日志）、
`t262-head-v3-time.txt`（guard 击杀消息）、`workers-retest-v3-*`。
