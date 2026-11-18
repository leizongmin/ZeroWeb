# s431：form target=_blank aux 上下文施工（6T 全解挂，稳定红 214→207）

日期：2026-11-14 ｜ 车道：engine/dom + runner py_routes ｜ 前序：s430 act:option 臂

## 勘根定性（6T 面分解）

s430 移交的 stable-T 14 键中 6 键同属 forms/form-submission-target 目录
（rel-button/form/input/base-target 四文件 + form-target-blank-useractivation +
form-target-request-header，全部 file 级 Timeout 挂起表示）。定性拆解：

- **5 个 engine 面**：rel 矩阵四文件（reltester.js 7 变体 × formUsesTargetBlank
  链）+ useractivation（3 触发臂断言提交后 isActive=false）。共同根因：S4J
  ⑤b target 链无 `_blank` 臂——提交落入 ⑥ navigate 到当前文档，endpoint 不
  存在、BroadcastChannel 永不回投、harness 等 completion 超时。
- **1 个 runner 资产面**：request-header（POST helper.py 断言 content-type 送达
  + BroadcastChannel 回投 OK）。勘根发现 runner 根本不执行 .py——py_routes
  `pred_method_gate_non_get` 兜底门拒绝一切非 GET/OPTIONS。本轮定性为 runner
  端点仿真缺位（非 engine 缺陷）。
- **定性链支撑**：①BroadcastChannel 跨文档可达——iframe 子文档脚本经 R206
  merged single Function 在同一 JS VM 执行，part05.js per-name 实例注册表天然
  跨 entry，sender-not-self 不拦回投；②子文档脚本在 `_zwFinishIframeEntry`
  内同步执行——脚本期读 `window.opener`/`document.referrer` 先于调用方拿到
  entry，预置只能走 flags 通道；③iframe skip 规则（`source.contains("<iframe")`
  等，testharness_html_semantics.rs:65）使含 iframe 字面的自建探针静默空集
  exit 1（s341 记忆陷阱变体，本轮两探针烧此坑）；④form-target-iframe 与
  multi-globals 两文件本就不在 runner 语料（同 skip 规则），目录恰集 6 键。

## 施工（四文件五刀，零共享字面触碰）

1. **part04.js S4J ⑤b base target 回落**：form 无 target 属性时读
   `document.querySelector('base').getAttribute('target')`——spec
   form-submission-algorithm 的 target 缺省步（rel-base-target 钉面）。
   无 base 文档零变化。
2. **part04.js S4J ⑤b `_blank` 臂**：rel 语义矩阵（spec window opening
   steps 的 _blank noopener 缺省强制）——noopener 强制 = 显式
   noopener/noreferrer 或无显式 opener；referrer 送达 = 非 noreferrer。
   合成 aux entry 走 `_zwLoadIframeEntry('__zwAuxTarget:<seq>', _abs, flags)`；
   POST 附 method/body/headersWire（multipart boundary 保真）；提交后消费
   瞬态激活（`_zwTransientActive = false`）并 return。
3. **part01.js finish referrer flags 预置**：`entry.flags.referrer != null`
   时落 `_zwReferrer`（noreferrer → ''），缺省父 href 不变（零面外位移）。
4. **part01.js finish aux opener/close 预置**：opener 非空 = 顶层 win 引用
   （WindowProxy 近似），null = noopener 强制；`close` 无操作（消 endpoint
   `window.close()` 的 TypeError 逸出——脚本期吞错 + `_zwReportListenerError`
   事件通道双层上报面）。均先于子文档脚本执行落位。
5. **py_routes `route_form_target_request_header`**（注册于非 GET 兜底门之前，
   Exact 匹配 helper.py）：镜像 upstream helper 语义——content-type 头扫描
   门（`n.eq_ignore_ascii_case("content-type")`）→ OK/FAIL → channelname 取
   `req.body.split('=').nth(1)` → BroadcastChannel 脚本回投 + window.close。
   + **testharness.rs 单测**
   `form_target_request_header_helper_py_channel_roundtrip`（有 content-type
   → BroadcastChannel('tok_123') + OK；无 → FAIL，两臂）。

**spec 锚**：HTML 规范 form-submission-algorithm（target 缺省步 + navigable
选择）；window opening steps（_blank noopener/noreferrer 缺省强制 + 瞬态激活
消费）。rel 反射走 IDL setter expando + attr 双通道读取（无 FORM rel 反射器，
reltester 自身以 form.rel 赋值钉面）。

## 验证链

| 门 | 结果 |
|---|---|
| 25 part 拼装 node --check | OK（153,473 行） |
| runner release 重建 | exit 0（shim 1m11s + Rust 面 16.27s） |
| solo 恰集：rel×4 | 各 7P/0 红（28 变体全绿：opener/noreferrer noopener 强制 + opener 显式传 + referrer 空/父 href 双态） |
| solo 恰集：blank-useractivation | **3/3 P**（submit/requestSubmit/test_driver.click 三臂全绿，含 isActive=false 断言） |
| solo 恰集：request-header | **1P**（POST content-type 送达 + BroadcastChannel OK 回投） |
| cargo fmt -p zero-wpt-runner | clean（两处换行修正后 FMT-CLEAN） |
| cargo clippy -p zero-wpt-runner --all-targets | exit 0（429 打断后终稿树重跑补验） |
| 单测 form_target_request_header_helper_py_channel_roundtrip | exit 0，1 passed / 0 failed（终稿树重跑） |
| 全域 sweep ×2（8484 键/轮）+ join vs s430 双基线 | **stable-F 200=200 / stable-P 8242→8274（+32 零消失）/ stable-T 14→7（−7）/ 沾染 0→1** |
| **组合态整验（他流到货后终稿树重跑，2026-11-14 06:4x）** | pull --rebase 合入 editing v3.2（壳尾重开 v3.2）/ css bolder-lighter / css attr() 三笔（均触 part01.js 共享面）后：25 part 拼装 OK（153,527 行）+ release 重建 exit 0 + **solo 恰集 ×6 组合态逐键保真（7×4+3+1=32P 零红）** + ut2/clippy 终稿树重跑 exit 0 + 全域 sweep ×2 join 对 s431 双基线——**stable-F 200=200 / stable-P 8274=8274 逐键零漂移**；stable-T 7→8 全归因在册 flake 族：select-event + focus-dynamic-type-change-on-blur 双轮 file 级 T 翻入——**solo ×2 双文件全绿定谳负载闪变**（s292 判据；s387/s405/s407 同族在档），home-end.optional T→沾染桶（簇 D 时序闪变家族），沾染 1→2。到货零账面位移 |

+32 全部落 form-submission-target：rel 四文件 7×4=28 子测 + useractivation 3
触发臂 + request-header 1。−7 = 6 个目标文件级键解挂 + detailed.optional
（select-home-end-pagedown-pageup-detailed，簇 D 在册家族）双轮态分裂翻入
沾染桶（sweep1 T / sweep2 非 T——时序闪变，与施工面零交集：form 提交/aux
上下文不触 select 键盘导航路径）。

## 判读

**6T 全解挂：稳定红 214→207（F 200 持平，T 14→7）**。join 三证自洽：
①new 集合 32 键无一越出目标目录；②gone 集合 = 6 目标键 + 1 沾染键，零
既有键消失；③stable-F 零漂移反证 aux/flags/base 三刀对面外零扰动。另证：
join 首跑全零漂移假象系 /tmp/s431/join.sh cur 行误指 s430 pairs（base≡cur
恒零 diff）——sweep1 headline T=8 与 pairs 原始内容矛盾揭破，修输入重跑后
三桶与恰集证据精确闭合。教训：join 脚本 sed 再生必须逐行核对输入路径，
headline 三数（P/F/T）与 join 输出先做数量级 sanity 对账再进判读。

request-header 判 runner 端点仿真面（非 engine）：.py 不执行是 runner
net-infra 既有事实，emulated arm 按 upstream helper 行为镜像，POST 通道
content-type 断言语义保真。

## 组合态整验补记（429 打断恢复轮，终稿树重跑）

429 在 clippy 链排队点打断（ut2 无 done、clippy 未启动、sweep/solo 证据
为同步前树）。恢复轮：git stash -u → pull --rebase（合入他流三笔——
editing v3.2 壳尾重开 / css bolder-lighter JS plain 臂 / css attr() 写面
豁免，均触 part01.js 共享面，stash pop 自动合龙零冲突）→ stash pop →
终稿树全链重跑（/tmp/s431 resync-chain.sh：ut2 → clippy → 拼装 →
release 重建 → solo ×6 → sweep ×2 → join2 对 s431 双基线）。结果：
s431 面逐键保真（32P 零红、F/P 双桶零漂移），T 桶 7→8 与沾染 1→2 全落
在册 flake 族（select-event / focus-dynamic 负载闪变 solo ×2 定谳 +
home-end.optional 簇 D 翻转），**到货零账面位移**。教训：证据生成与
commit 之间被长打断时，恢复后必须终稿树重验整链，不可直接采信旧树
sweep 数字（本轮 s431 数字恰好保真，但这是验证结果而非前提）。

## 残留面与下一步

- **不在语料清单（记账）**：form-target-iframe、multi-globals 因 iframe/
  多全局 skip 规则不入 runner 语料——named-iframe 路径的现网回归覆盖由
  form-submission-0 语料文件维持，零账面影响。
- **POST-to-static 兜底**：runner 对静态资产的非 GET 请求兜底门属 net-infra
  面（fetch 层），本轮不动。
- **window.open 真弹窗**：aux 上下文仅覆盖 form target=_blank 提交臂，
  window.open 弹窗链（含rel 特性串）defer。
- **簇 D 沾染键**：detailed.optional 维持在册（时序闪变，不施工）。
- 用户门禁待决不变：GB-20261114-VIDEO、GB-20261031（簇 D）、s137。
- **基线移交**：后续施工轮 join 改用 /tmp/s431/pairs-{1,2}.txt。
- 下一步候选：dialog R-s31 簇 / img density / embed sizing face 到货复验；
  门决策后 video P1+P2 或簇 D A2 分诊续。
