# M2 s446 — 守成巡查轮：g49 GCS 视口真值刀入树零位移清偿（P/F 集恒等，T −1 族相位）+ 全桥绿窗 1637P/0F

日期：2026-11-15。承前：s445 下一步「守成巡查（轻覆盖或他流入树归因轮）+
候门决项等待用户拍板」。

## 归因面：s445 push 后入树一刀

bb3eff6cf（渲染流 g49 GCS 视口真值对齐——gcs_viewport_dims 单源 +
渲染 media 门接真 viewport）：触 engine 共享面五文件
（computed_style.rs / computed_style_cache.rs / pipeline/mod.rs /
pipeline_budget.rs / tests/pipeline.rs）。行为面与本语料潜在交叠点：
`getComputedStyle` viewport 相对值 + `<style media>` 门求值由恒
1280×800 改为真 800×600——document-metadata render-blocking/media 门
族（link-stylesheet-with-non-match-media 等）为候选位移面。

## 双腿 join（bb3eff6cf 树，make testharness-html-semantics ×2）

- **stable 8510（8305P/199F/5T/1U）**。
- **vs s445 stable（8511 = 8305P/199F/6T/1U）：恰 1 差集** =
  select-home-end 主文件文件级 Timeout 键相位出 stable（在册负载族，
  leg1 该文件整文件相位轮换、leg2 回 4 子测 Fail 相位——族内形态轮换
  口径不变）；**P 集逐键恒等（8305 = 8305）、F 集恒等（199 = 199）**。
- 唯一 unstable 键 = detailed 变体文件级（同族）。
- **g49 零位移定谳**：候选位移面（media 门/render-blocking 族）实测
  零键移动——本语料无依赖 1280×800 伪真值的断言；runner 视口 800×600
  语义键面与 GCS/media 门真值化不交叠。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1637P/0F rc=0**（test 阶段 26.52s；基线 s445 1637P，
+0——g49 未新增 bridge 面钉测）。

## 结论

s446 收口（守成巡查轮，零源码改动）：**in-domain 自由面维持实质清零**，
g49 刀零位移清偿，基带 **8305P/199F/5T/1U**（T −1 = 族相位，下轮双腿
可能摆回 6T；P/F 恒等为主口径）+ select-home-end 族振荡记账不变。
挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已
提示用户，本轮不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s445/{s446-leg1,s446-leg2}.log、s446-{l1,l2}.norm、
s446-bridge.log、s446-bridge.rc、s445-stable.norm（上轮基带固化）。
