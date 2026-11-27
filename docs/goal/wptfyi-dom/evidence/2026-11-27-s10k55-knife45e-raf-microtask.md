# s10k55 · 刀㊺e：TT block-text ×2 收敛（runner rAF OFF 路径改微任务派发）

日期：2026-11-27。刀号：㊺e（㊺ 后缀序列续：㊺b nav／㊺c CSP location／㊺d S4 双键／㊺e block-text）。

靶面：TT 池残 3F 中两键——`block-text-node-insertion-into-script-element.html` 与
`-svg-script-element.html` 的 module 臂（`Test that default policy applies to module
script.`／`… svg:script`）。classic 臂已绿。

## 一、根因考古（probe A 单刀定谳）

靶页事件链（`resources/block-text-node-insertion.js`）：page 写入 module script（body
含 default policy 转换逻辑）→ fn() 触发 module 臂 → 断言 **count 消息先于 done 消息**
→ 收尾 `requestAnimationFrame(_ => requestAnimationFrame(_ => postMessage("done",
"*")))` 双层 rAF。

wpt-runner 不设 `__ZW_RAF_FRAME_DRIVEN`（renderer worker 默认 ON、runner 走 OFF 路径），
OFF 路径原实现为**同步 stub**（`fn(performance.now())` 立即执行）。

zz-s10k55-module-order-probe 探针页事件序实测：

- 修复前：`msg[done] → msg[Acount]`——**倒序**。module 脚本求值走刀㊼（s250）的
  `Promise.resolve().then` 微任务车道，排在同步 rAF 链之后；module 体内
  `postMessage('count')`（part06 实现＝setTimeout(0) 宏任务投递，s336 R3 车道）入队时，
  rAF 链已同步跑完并先行投递了 "done" 宏任务 → 监听器先见 done（count=0）→ 断言 F。
  classic 臂同步求值，count 先入队，故恒绿。
- 修复后（刀㊺e）：`msg[Acount] → msg[done]`——微任务 FIFO 中先排队的 module eval
  微任务先执行，其 count 投递先入宏任务队。

探针页自断言（`A:final expected true got false`）因监听器记录形与断言匹配串不一致的
探针自身 bug 恒 F，判读以事件序日志（判据本体）为准。

## 二、修复刀面（part01.js，唯一代码改动）

`__zw_begin_script` 每脚本执行重置 `_rafBudget = 512`；rAF OFF 路径（约 part01.js
15898 行）同步 stub 改微任务派发：

```js
_rafBudget--;
Promise.resolve().then(function () {
  if (typeof __zw_mut_hover_sync_settle === 'function') { try { __zw_mut_hover_sync_settle(); } catch (_eMhs8) {} }
  var _r3254ts = (typeof __zw_performance_now === 'function') ? __zw_performance_now() : 0;
  try { fn(_r3254ts); } catch (_e) {}
});
```

回调排**当前任务尾的 microtask 检查点**执行（V8 auto-drain 保证 sandbox 生命周期内
必然执行——`v8_runtime.rs` 748-768 `MicrotasksExplicitGuard`：默认 kAuto 顶层
script.run 栈空自动排空；仅 execute_classic catch 车道临时 kExplicit 后恢复）。预算
512 防同脚本失控 rAF 循环；帧驱动路径（`__ZW_RAF_FRAME_DRIVEN`）分支不动。

## 三、安全性论证（爆炸半径逐面核对）

1. **reftest double-rAF setup**：重排后 double-rAF 两回调仍在 harness 单渲染捕获前
   落地——execute() 内 V8 kAuto 自动排空使 setup 变异（微任务）在渲染快照前完成。
   ref12 全量电池臂终判（见第五节）。
2. **renderer worker**：帧驱动默认 ON，不走 OFF 分支，字节不变。
3. **corpus testharness.js**：语料 testharness 本体零 rAF 调用；editing 语料仅 6 文件
   用 rAF（editor-test-utils.js:572 double-rAF await 形态与微任务派发兼容）。
4. **shim 内部自用**：全 shim 仅 part02.js:9278 anchor scroll 一处自用 rAF（安全）。
5. **跨套件漂移**：dom12 电池臂 F/T 逐键 comm 对账（见第五节）。

## 四、附带考古产出（SVGScript-internal-slot 五探针链 → 刀② 挂账）

同轮对 TT 池第三键 `SVGScriptElement-internal-slot.html` 考古（zz-s10k55-slot-probe
～probe6 五探针）。页形：document.write 流式写未闭合 `<svg><script>…first…` →
manipulator 找 script[1] appendChild `/*byapi*/` → write2 闭合＋第二 script；期望
first 被 TT 阻断、second（经 createHTML sink）执行。

定谳：**write-face RunFace（part06.js 9540-9860）直 eval 绕 TT 执行门**＋解析投递
不盖章＋未闭合 script 物化提前执行：

- SVG 形：write1 时 svg script（干净文本）被 RunFace 直 eval → `msg[first]`
  ARRIVED（should-be-blocked）在 manipulator 变异**前**执行＝F 根因。
- HTML 形（对照 `HTMLScriptElement-internal-slot.html` test1「假绿」定性）：未闭合
  script 物化文本带尾垃圾（len=64 期望 57，`SyntaxError: Unexpected token '<'`）→
  eval 静默跳过＝「阻断」是意外非 TT 门功效。

刀② 设计（挂账）：RunFace 集合时盖章＋eval 前过 `__zwTtScriptExecGate`＋变异失效
盖章（appendChild/textContent 触带写槽标记 script 时删章）＋未闭合 script 延迟到闭合
（解析器加未闭合标记）。

face B 挂账（`HTMLScriptElement-internal-slot.html` tests 2-8 页级 Timeout）：test2
空 script 插入 → prepare → 空体＋createHTML-only default policy → 刀㊷ rescue 臂 →
`_zwTtSinkValue` TypeError → 门返 null → BLOCK 臂置 run-once 章（already-started）→
后续 `.textContent = policy.createScript(...)` 永不执行 → promise_test pending →
页 Timeout。修复方向：门空体臂区分「静默臂（无转换发生，不置章）」vs「rescue 返空臂
（置章）」＋sink setter 对已 prepare-空 script 触发再执行。

## 五、电池终判

**tt12**（bin `6d340ff3c009`，guard 包装 --per-proc-mem 4 --total-mem 8 --time-limit
1800，11:13-11:28Z，与兄弟 ZeroWeb-2 reftest-upstream 腿经 wait_idle 互斥）＝
**2280P / 14F / 20T** vs tt11 锚 2278P/9F/19T（`/tmp/s10k54/ver-tt11.log`，同提取法
LC_ALL=C awk ＋ comm 对账）：

- P 2278 → 2280 ＝ +2 恰两靶页 module 臂翻面。
- F anchor-only 恰两键：`Test that default policy applies to module script.`／
  `Test that default policy applies with module script. svg:script`。
- F new-in-tt12 恰 7 条＝本轮 zz-s10k55 探针（probe A/B＋probe2-6）不入库噪声。
- T anchor-only **空**＝19 T 键零漂移；T new 恰 1 条＝zz-s10k55-slot-probe 探针。
- 其余遗留键集逐键恒等（SVGScript-internal-slot 1F、HTMLScript-internal-slot T、
  005/006.sub/csp-headers T、navigation 8×T、ro-probe3d/e/f 等探针 F）。

**＝TT 真实池 3F → 1F**（残 SVGScript-internal-slot＝刀②）。

**dom12**（同 bin，guard 包装 --time-limit 2700，11:28-11:45Z，完跑 rc=2 常规形）＝
**57,449P / 361F / 12T** vs dom10 锚 57,505P/361F/12T（`/tmp/s10k54/ver-dom10.log`）：

- F/T 四象限 comm **全空**＝361F＋12T 键集逐键恒等零漂移。
- 页集 348 = 348 恒等；P −56（57,505 → 57,449）恰两页
  `Range-mutations-dataChange` −40＋`Range-mutations-replaceData` −16＝巨页 census
  截断抖动噪声族（上轮 dom10 对 s10k53 锚 +42 同两页同族落位漂移；F 集零差已排除
  回归，判读基线见 dc1 逐文件对账惯例）。

**ref12**（make reftest 全量，guard 包装，11:45-11:48Z）＝**725/725 全绿**（Layout
521/521＋Text 204/204，与 css 流 2026-11-21 证据文历史形逐字一致）。reftest 语料
（css21 等）零 `requestAnimationFrame` 使用（grep 实证）＝本臂为 shim 载入兜面而非
rAF 路径触发面；Duration 0.39s 经 main.rs:3898 `start.elapsed()` 核为全跑真值（8
job 并发＋css21 简单页 CPU 管线）。

**editing 定向 A/B**（rAF 改动真实触发面保险臂）：`testharness-editing-other
inserttext-last-mutation`（editor-test-utils double-rAF await 形，part01.js 46-47）
新旧 bin 对跑——基线 bin `93a4a1fbec02`（stash 刀体后重建，31.19s fresh）＝48F/16P，
刀体 bin `6d340ff3c009`（pop 后重建，31.30s）＝48F/16P，**F 键集 comm 双空逐键恒等**
＝该页红为既有失败非本刀引入（editing 语料属 05 html 流域资产，红面记对端参考不
修）。stash 过境 patch 备份 `/tmp/s10k55/part01-knife45e.patch` md5 `dbacfebbd57c`
（push/pop 前后 md5 复核一致）。

rAF 微任务路径的真实触发面（corpus 731 文件含 rAF）由 tt12（block-text 双页 rAF 链
臂 F→P）与 dom12 F/T 零漂移覆盖。

## 六、遗留与挂账

- TT 池残 1F＝SVGScript-internal-slot（刀② write-face TT 集成，设计见第四节）。
- face B＝HTMLScriptElement-internal-slot tests 2-8 Timeout（空体 prepare 静默/置章
  区分＋sink-set 再执行）。
- zz-s10k55-* 探针页 8 件留语料目录（gitignored 不入库），终判清场时删。
