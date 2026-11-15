# M2 s461 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（主文件 4 子测 F 双腿一致入 stable，detailed 文件级腿间 T↔F 振荡出 stable，P 集恒等 8306 八轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s460 下一步「守成巡查（他流入树归因轮或轻覆
盖）」。本轮开工 `git pull --rebase` 携入渲染流两 docs 提交
（af293f8b0 R5611 轮末补验、cbd7048e8 R5612 三源空转核查，均
docs-only）→ 零源码零语料净变更，HEAD = d37cb3a99（代码树与 s460
双腿树 6babd9aac 全同）→ 无归因对象，按预案执行纯轻覆盖轮。

## 双腿 join（d37cb3a99 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**，leg2 **8516 键（8306P/204F/
  5T/1U）**；腿键集完全一致（leg-only 0/0），单腿缺席 0。
- disagree = 1：detailed 文件级整文件键（leg1 Timeout ↔ leg2
  Fail）——select-home-end 族相位腿间振荡延续。
- **stable 8515（8306P/203F/5T/1U，norm 配方口径，distinct 8513）**：
  族主文件 4 子测 Fail 双腿一致入 stable；detailed 文件级键因腿间
  T↔F 振荡出 stable；族整文件 Timeout 键本轮双腿缺席。
- **vs s460 stable（distinct 8510）恰 1 GONE（detailed 文件级键）+
  4 NEW（主文件 4 子测），全数族内**；8509 共享键零状态翻转，非族
  键逐键零差，零真位移。
- **P 集账本口径 8306 八轮连续恒等**。
- onblur 哨键组 10 键双腿一致 Pass 在 stable（哨解除后第七轮）；
  红池 onblur 2 F 键照常在册。
- stable T 键集 5 键全非族（style-error-01 / audio-loading-lazy-
  source-deferred / sizes-auto / update-media /
  video-loading-lazy-source-poster-deferred），与 s460 stable 同集。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 15.45s；基线 s460 1643P，+0——零入树
刀，无新钉预期）。

## 过程事故与入口备注

- 本轮一次 `pkill -f "test-guard"` 清理尝试因模式匹配到自身命令行
  杀死自身 shell（exit 144），同扫可能中断 ZeroWeb-3 并行 clone 在
  飞的 `cargo build --release -p zero-wpt-runner`（对方 rally 循环
  下轮重跑构建即自愈，无文件面损伤）。教训：进程清理按 PID 圈定，
  不用共享名模式。
- 首次 `make guarded-test` 变参误用实为全量 `make test` systemd
  scope 入口，由本会话 10 分钟墙钟主动终止；核查无 systemd scope
  残留、本 clone 无残留进程。桥套件收口改用 s460 同款 test-guard
  直调（见上）。

## 结论

s461 收口（守成巡查轻覆盖轮，零源码改动）：零入树零刀同码树基带复
现确认——基带形态 **8306P/203~204F/5~6T/1U，全部变动限于
select-home-end 族相位**（主文件 4 子测 F 双腿一致入 stable +
detailed 文件级腿间 T↔F 振荡出 stable），P 集口径 8306 八轮连续维
持。挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137
（s445 已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿零
hang）。

工件：/tmp/s461/{s461-leg1,s461-leg2}.log、s461-{l1,l2}.norm、
s461-stable.norm、s461-stable.keys(.norm)、s461-bridge.log、
s461-bridge.rc。
